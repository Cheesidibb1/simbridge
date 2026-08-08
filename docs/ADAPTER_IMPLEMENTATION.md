# SimBridge Adapter Implementation Guide

## Overview

This guide describes how to implement simulator adapters for different platforms. Adapters enable SimBridge Server to control various iOS simulators, Android emulators, and future devices.

## Architecture

The adapter architecture follows a plugin-based design:

```
┌─────────────────────────────────────┐
│        SimBridge Server            │
│  ┌──────────────────────────────┐  │
│  │      SimulatorAdapter Trait  │  │
│  └──────────────────────────────┘  │
│           │                         │
│    ┌──────┴──────┬─────────────────┐
│    │             │                 │
│  iOS Adapter  Android Adapter     │
└─────────────────────────────────────┘
```

## Interface Specification

All adapters must implement the `SimulatorAdapter` trait defined in `server/src/adapters/interface.rs`.

### Key Methods

```rust
pub trait SimulatorAdapter: Send + Sync {
    // Identification
    fn name(&self) -> &str;           // e.g., "ios-simulator"
    fn version(&self) -> &str;        // e.g., "0.1.0"
    
    // Connection management
    async fn connect(&mut self) -> Result<(), AdapterError>;
    async fn disconnect(&mut self) -> Result<(), AdapterError>;
    fn is_connected(&self) -> bool;
    
    // Simulator info
    fn simulator_id(&self) -> &str;
    fn simulator_name(&self) -> &str;
    
    // Screen streaming
    async fn start_screen_stream(&mut self, quality: StreamQuality, fps: u32) -> Result<ScreenStream, AdapterError>;
    async fn stop_screen_stream(&self) -> Result<(), AdapterError>;
    
    // Input control
    async fn send_touch_event(&mut self, event: TouchEvent) -> Result<(), AdapterError>;
    async fn send_gesture(&mut self, gesture: Gesture) -> Result<(), AdapterError>;
    async fn press_button(&mut self, button: DeviceButton) -> Result<(), AdapterError>;
    
    // Location & motion
    async fn set_location(&mut self, location: GpsLocation) -> Result<(), AdapterError>;
    
    // App management
    async fn install_app(&mut self, path: &Path) -> Result<(), AdapterError>;
    async fn launch_app(&mut self, bundle_id: &str) -> Result<(), AdapterError>;
    async fn terminate_app(&mut self, bundle_id: &str) -> Result<(), AdapterError>;
    
    // Notifications
    async fn get_notifications(&mut self) -> Result<Vec<Notification>, AdapterError>;
    
    // Clipboard & Files
    async fn get_clipboard(&mut self) -> Result<String, AdapterError>;
    async fn set_clipboard(&mut self, content: &str) -> Result<(), AdapterError>;
    async fn transfer_file(&mut self, direction: TransferDirection, path: &Path) -> Result<Vec<u8>, AdapterError>;
    
    // System
    async fn restart(&self) -> Result<(), AdapterError>;
    async fn get_status(&self) -> Result<SimulatorStatus, AdapterError>;
}
```

## Platform-Specific Implementations

### iOS Simulator Adapter

The iOS adapter uses Apple's tools:
- `simctl` - Universal iOS simulator control
- `idevice` - Physical device management
- XCUITest for automation (optional)

**Key Requirements:**
- macOS only
- Xcode 14+ installed
- Access to simulators or physical devices

**Implementation Pattern:**
```rust
pub struct IosSimulatorAdapter {
    device_id: String,
    is_physical_device: bool,
    simctl_path: String,
}
```

**Screen Capture:**
```bash
xcrun simctl io <device_id> screenshot
```

**Touch Injection:**
```bash
xcrun simctl io <device_id> touch_down <x> <y>
```

**GPS Spoofing:**
```bash
xcrun simctl set_location <device_id> <lat> <lon>
```

### Android Emulator Adapter

The Android adapter uses ADB (Android Debug Bridge):

**Key Requirements:**
- Android Studio with SDK
- USB debugging enabled on emulators
- `adb` in PATH

**Implementation Pattern:**
```rust
pub struct AndroidEmulatorAdapter {
    device_id: String,       // Serial number from adb
    is_physical_device: bool,
}
```

**Screen Capture:**
```bash
adb -s <device_id> screen-record output.mp4
```

**Touch Injection:**
```bash
adb -s <device_id> shell input tap <x> <y>
```

**GPS Spoofing:**
```bash
adb -s <device_id> shell location fix -m 1 -n 270 -p <lat> -l <lon>
```

### iOS Physical Device Adapter

For physical iOS devices, use idevice:

```rust
pub struct IosPhysicalDeviceAdapter {
    device_udid: String,
    is_physical_device: bool, // Always true for physical
}
```

**Screen Capture:**
```bash
ideviceinstaller -l | grep <device_udid>
```

**Touch Injection:** Requires XCUITest setup or WebDriverAgent

### Android Physical Device Adapter

For physical Android devices:

```rust
pub struct AndroidPhysicalDeviceAdapter {
    device_serial: String,
}
```

**Screen Capture:** Uses Screenshot API or MediaProjection
**Touch Injection:** Via `adb shell input`

## Adapter Registration

Adapters are discovered and registered automatically by the server.

### Discovery Process

1. Server scans for available devices on startup
2. Lists connected iOS devices via `idevice_id -l` (physical) or `xcrun simctl list devices` (simulator)
3. Lists Android devices via `adb devices`
4. Creates adapter instances for each discovered device
5. Registers adapters with REST API

### Adapter Configuration

Configuration can be done via:
1. Command-line arguments (`--config path/to/config.toml`)
2. Environment variables
3. Runtime discovery (preferred)

**Example Config:**
```toml
[adapters]
  [adapters.ios.simctl_path]
    value = "/usr/bin/xcrun"
  
  [adapters.android.adb_path]
    value = "/usr/bin/adb"

[[devices]]
  id = "sim-1"
  name = "iPhone 15 Pro Simulator"
  platform = "ios"
```

## Testing Adapters

### Unit Tests

Test adapter logic without requiring actual devices:

```rust
#[test]
fn test_adapter_creation() {
    let adapter = IosSimulatorAdapter::new(
        "device-123".to_string(),
        "Test iPhone".to_string(),
    );
    
    assert_eq!(adapter.name(), "ios-simulator");
}
```

### Integration Tests

Use mocking for integration tests:

```rust
#[tokio::test]
async fn test_adapter_connect() {
    let adapter = IosSimulatorAdapter::new(...);
    
    // Mock the actual device commands
    // Test connection flow
}
```

## Common Patterns

### Error Handling

All methods return `Result<T, AdapterError>`. Use appropriate error variants:

```rust
async fn get_status(&self) -> Result<SimulatorStatus, AdapterError> {
    if !self.is_connected() {
        return Err(AdapterError::NotConnected);
    }
    
    // ... implementation
    
    Ok(status)
}
```

### Connection State

Track connection state and check before operations:

```rust
async fn send_touch_event(&mut self, _event: TouchEvent) -> Result<(), AdapterError> {
    if !self.is_connected() {
        return Err(AdapterError::NotConnected);
    }
    
    // Send touch event...
    Ok(())
}
```

### Async/Await Usage

All operations are async. Use appropriate synchronization:

```rust
pub struct Adapter {
    devices: Arc<RwLock<HashMap<String, DeviceState>>>,
}
```

## Performance Considerations

### Screen Capture
- Use efficient codecs (H.264)
- Adjust FPS based on network conditions
- Compress frames before transmission

### Touch Events
- Batch events where possible
- Use non-blocking operations
- Implement event queuing for high load

### Resource Management
- Close connections when not in use
- Clean up temporary files
- Monitor memory usage

## Security Considerations

1. **Authentication**: All adapters require valid session tokens
2. **Authorization**: Check user permissions before allowing operations
3. **Isolation**: Each device session runs in isolation
4. **Audit Logging**: Log all commands for security auditing

## Future Adapters

### Planned Platforms

- [ ] iOS Simulator (macOS) - Partial implementation exists
- [ ] Android Emulator - Needs full implementation
- [ ] Android Physical Device - In planning
- [ ] Windows Subsystem for Android - Not started
- [ ] Linux emulators - Not started
- [ ] Virtual machines (QEMU, etc.) - Not started

### Extension Points

The architecture supports easy addition of new adapters:

1. Implement `SimulatorAdapter` trait
2. Register adapter in discovery module
3. Add device type constants if needed
4. Write tests

## Contributing New Adapters

See [CONTRIBUTING.md](./CONTRIBUTING.md#adding-new-adapter) for contribution guidelines.

**Required:**
- Working implementation on test devices
- Comprehensive test suite
- Documentation in adapter README
- Security review

**Recommended:**
- Performance benchmarks
- User guide with platform-specific setup
- Example configuration files

## Troubleshooting

### Common Issues

1. **"Not connected" errors**
   - Check device is running
   - Verify USB debugging enabled (Android)
   - Ensure simctl/idevice paths correct (iOS)

2. **Screen capture failures**
   - Xcode tools not found → Install Xcode Command Line Tools
   - ADB not responding → Reboot emulator
   - Permission denied → Grant necessary permissions

3. **Touch events failing**
   - Device locked → Unlock first
   - Screen off → Wake screen before sending touch

4. **GPS spoofing issues**
   - Location services disabled → Enable in device settings
   - Simulator restrictions → Check simulator configuration

## Testing Checklist

Before submitting a new adapter:

- [ ] Adapter creates successfully
- [ ] Connect/disconnect works
- [ ] Screen streaming functional
- [ ] Touch events work correctly
- [ ] GPS injection works
- [ ] Notifications can be retrieved
- [ ] Clipboard operations work
- [ ] File transfer works
- [ ] Error handling tested
- [ ] Unit tests passing
- [ ] Documentation complete

## Resources

- [Apple Xcode Documentation](https://developer.apple.com/xcode/)
- [Android ADB Documentation](https://developer.android.com/studio/command-line/adb)
- [WebDriver Protocol](https://w3c.github.io/webdriver/) for mobile automation
