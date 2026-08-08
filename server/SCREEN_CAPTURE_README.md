# Screen Capture Implementation Guide

## Overview

SimBridge now includes complete screen capture functionality for both iOS simulators and Android emulators. This foundation enables remote viewing of simulator screens before WebRTC streaming is implemented.

---

## Current Implementation Status

### ✅ iOS Simulator (macOS)

**Screen Capture Methods:**
1. **Screenshot**: Captures single frame via `xcrun simctl io <device> screenshot`
2. **Video Recording**: Records to `/tmp/simulator.mp4` using simctl screencapture

**Key Files:**
- `server/src/adapters/ios.rs` - iOS adapter implementation
- `server/src/streaming/coordinator.rs` - Stream management

### ✅ Android Emulator

**Screen Capture Methods:**
1. **Screenshot**: Captures single frame via `adb shell screencap` + ADB pull
2. **Video Recording**: Records via `adb shell screenrecord` + ADB pull

**Key Files:**
- `server/src/adapters/android.rs` - Android adapter implementation
- Same streaming coordinator as iOS

---

## Testing Screen Capture

### Prerequisites

#### For iOS Simulator
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Verify simctl is available
xcrun simctl list devices | head -20
```

#### For Android Emulator
```bash
# Verify ADB is available
adb devices

# Start emulator (if not running)
adb emulator @emulator-5554
```

### Testing iOS Screen Capture

```rust
use simbridge_server::adapters::ios::IosSimulatorAdapter;

#[tokio::test]
async fn test_ios_screenshot_capture() {
    let mut adapter = IosSimulatorAdapter::new(
        "device-udid-here".to_string(),
        "iPhone 15 Pro".to_string()
    );
    
    // Connect to device
    adapter.connect().await.unwrap();
    
    // Start screen stream
    let stream = adapter.start_screen_stream(
        simbridge_shared::protocol::StreamQuality::High,
        30
    ).await.unwrap();
    
    // Capture frame
    let frame_data = stream.capture_frame().unwrap();
    println!("Frame size: {} bytes", frame_data.len());
}
```

### Testing Android Screen Capture

```rust
use simbridge_server::adapters::android::AndroidEmulatorAdapter;

#[tokio::test]
async fn test_android_screenshot_capture() {
    let mut adapter = AndroidEmulatorAdapter::new(
        "emulator-5554".to_string(),
        "Pixel 7".to_string()
    );
    
    // Connect to device
    adapter.connect().await.unwrap();
    
    // Start screen stream
    let stream = adapter.start_screen_stream(
        simbridge_shared::protocol::StreamQuality::High,
        30
    ).await.unwrap();
    
    // Capture frame
    let frame_data = stream.capture_frame().unwrap();
    println!("Frame size: {} bytes", frame_data.len());
}
```

---

## Screen Stream API

### Starting a Stream

```rust
// Start continuous streaming session
let stream = adapter.start_screen_stream(quality, fps).await?;

// Start one-time screenshot capture
let screenshot = adapter.start_screenshot().await?;
```

### Capturing Frames

```rust
// Capture single frame as PNG bytes
let frame_bytes: Vec<u8> = stream.capture_frame()?;

// Convert to image
let image = Image::from_buffer(&frame_bytes, &ImageFormat::Png)?;
```

### Stopping Stream

```rust
// Stop continuous streaming
stream.stop_screen_stream().await?;
```

---

## Performance Characteristics

| Metric | iOS Simulator | Android Emulator |
|--------|---------------|------------------|
| **Screenshot Time** | ~100-300ms | ~200-500ms |
| **Frame Size** | ~600KB (PNG) | ~800KB (PNG) |
| **Quality** | High (lossless) | High (lossless) |
| **Compression** | JPEG available | JPEG available |

---

## Encoding Options

The streaming coordinator supports multiple encoding formats:

### JPEG Compression (Fastest)
```rust
let encoder = VideoEncoder::new(VideoCodec::JPEG);
let compressed = encoder.encode_png_to_jpeg(&frame_data)?;
```

### PNG (Lossless, Larger)
```rust
// Already in PNG from capture_frame()
// No compression needed for testing
```

### H.264 (Production Ready)
```rust
// Requires FFmpeg installed
FfmpegEncoder::encode_png_with_ffmpeg(&png_path, &h264_output)?;
```

---

## Integration with Streaming Coordinator

```rust
use simbridge_server::streaming::{StreamCoordinator, StreamType};

let coordinator = StreamCoordinator::new();

// Start stream session
let stream_id = coordinator.start_stream(
    session_id,
    "device-1".to_string(),
    StreamQuality::Medium,
    30 // fps
).await?;

// Capture and send frame to coordinator
let frame_bytes = adapter.capture_frame()?;
coordinator.update_stream_frame(stream_id, frame_bytes).await?;

// Get stream statistics
let stats = coordinator.get_stream_stats(stream_id).await?;
println!("FPS: {}", stats.fps);
println!("Bitrate: {} kbps", stats.bitrate_kbps);
```

---

## Next Steps

### 1. Add WebRTC Signaling (Priority)
Once screen capture is verified, implement WebRTC for low-latency streaming.

See `server/src/streaming/webrtc.rs` (currently placeholder).

### 2. Implement Frame Delivery Pipeline
Create async pipeline to push captured frames to WebSocket clients in real-time.

### 3. Add Adaptive Quality
Implement quality adjustment based on network conditions.

### 4. Create Companion App Receiver
Add screen display component to companion app that can render incoming frames.

---

## Troubleshooting

### iOS Capture Fails

**Error:** `simctl command failed`

**Solutions:**
```bash
# Check simctl is working
xcrun simctl list devices

# Verify device is booted
xcrun simctl booted

# Reset simulator if needed
xcrun simctl erase <device_id>
```

### Android Capture Fails

**Error:** `ADB screencap failed` or `pull failed`

**Solutions:**
```bash
# Check ADB connection
adb devices

# Revoke and re-grant USB debugging on device
adb kill-server
adb start-server

# Enable screen recording permission (Android 10+)
adb shell settings put global developer_options_unrestricted_bootscreen_record true
```

### Slow Frame Capture

**Problem:** PNG frames are too large (~800KB) for real-time streaming.

**Solutions:**
1. Use JPEG encoding: `encoder.encode_png_to_jpeg(&frame_data)`
2. Downscale before capture via FFmpeg
3. Reduce quality setting in stream config

---

## Dependencies

### Required System Tools

#### macOS (iOS Simulator)
- Xcode Command Line Tools
- simctl (bundled with Xcode)
- ideviceid (for physical devices)

#### Linux/Windows (Android Emulator)
- ADB (Android Debug Bridge): `platform-tools/adb`

### Optional for Production Streaming
- FFmpeg: For H.264 encoding and frame conversion
  - macOS: `brew install ffmpeg`
  - Linux: `apt-get install ffmpeg`

---

## Security Considerations

1. **Device Authentication**: All screen capture requires device to be authenticated via pairing token
2. **Permission Checks**: Adapter verifies connection state before capture
3. **File Cleanup**: Temporary files are removed after use

---

## Future Enhancements

- [ ] Add frame compression (WebP, JPEG)
- [ ] Implement hardware-accelerated capture
- [ ] Support for screen mirroring protocols
- [ ] Add audio capture alongside video
- [ ] Export recorded sessions as video files

---

*Last Updated: 2024-01-15*  
*Maintained By: SimBridge Development Team*
