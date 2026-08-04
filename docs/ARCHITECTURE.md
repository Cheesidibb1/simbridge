# SimBridge Architecture Guide

## Overview

SimBridge is designed with a modular, plugin-based architecture that emphasizes clean separation of concerns, extensibility, and maintainability.

## System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                         Companion App                            │
│                        (Flutter Mobile)                          │
└───────────────────────────┬─────────────────────────────────────┘
                            │ WebSocket + REST
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│                      SimBridge Server                            │
│                          (Rust)                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Core Architecture Layer                    │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │   │
│  │  │  Session Mgr │  │   Auth Mgr   │  │  Plugin Mgr  │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Simulator Adapter Interface                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│                            │                                     │
│  ┌────────────────────────┼────────────────────────────────┐   │
│  │                         │                                │   │
│  ▼                         ▼                                ▼   │
│  ┌──────────────┐   ┌──────────────┐              ┌──────────┐│
│  │   iOS Adapter│   │ Android      │              │ Future   ││
│  │              │   │ Adapter      │              │ Adapters ││
│  └──────────────┘   └──────────────┘              └──────────┘│
└─────────────────────────────────────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│                    Shared Core Library                          │
│                    (Rust + TypeScript/JS)                       │
│  Protocol | Models | Networking | Auth | Utils | Logging       │
└─────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Companion Mobile App

**Technology**: Flutter (Dart)

**Responsibilities**:
- Device pairing and authentication
- Simulator discovery and selection
- Live screen rendering
- Touch event capture and transmission
- GPS/motion sensor streaming
- Notification display
- Clipboard management
- File transfer UI
- Settings and configuration

**Key Modules**:
- `networking/` - WebSocket client, REST API client
- `ui/` - Screens, widgets, navigation
- `services/` - GPS, sensors, clipboard, file transfer
- `protocol/` - Message serialization/deserialization
- `auth/` - Authentication and token management

### 2. SimBridge Server

**Technology**: Rust

**Responsibilities**:
- Connection management and session routing
- Authentication and authorization
- Plugin loading and management
- Simulator adapter coordination
- Screen streaming orchestration
- Notification forwarding
- Session recording
- Performance monitoring
- REST API and WebSocket endpoints

**Key Modules**:
- `core/` - Session management, authentication, plugin system
- `networking/` - REST API, WebSocket server
- `adapters/` - Simulator adapter interface and implementations
- `streaming/` - Screen streaming coordination
- `storage/` - SQLite database for persistence
- `recording/` - Session recording and replay
- `metrics/` - Performance monitoring

### 3. Simulator Adapters

**Technology**: Rust

Each adapter implements the `SimulatorAdapter` trait:

```rust
pub trait SimulatorAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn connect(&mut self) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;
    fn start_screen_stream(&mut self) -> Result<ScreenStream>;
    fn send_touch_event(&mut self, event: TouchEvent) -> Result<()>;
    fn send_gesture(&mut self, gesture: Gesture) -> Result<()>;
    fn set_location(&mut self, location: Location) -> Result<()>;
    fn press_button(&mut self, button: DeviceButton) -> Result<()>;
    fn install_app(&mut self, path: &Path) -> Result<()>;
    fn launch_app(&mut self, bundle_id: &str) -> Result<()>;
    fn get_notifications(&mut self) -> Result<Vec<Notification>>;
    fn get_clipboard(&mut self) -> Result<String>;
    fn set_clipboard(&mut self, content: &str) -> Result<()>;
    fn transfer_file(&mut self, direction: TransferDirection, path: &Path) -> Result<()>;
}
```

#### iOS Simulator Adapter
- Uses Xcode Simulator APIs via `simctl` command-line tool
- Screenshots via `simctl io`
- Location spoofing via `simctl location`
- Notification monitoring via private APIs or system logs

#### Android Emulator Adapter
- Uses ADB (Android Debug Bridge) for control
- Screen capture via `adb screenrecord` or `adb shell screencap`
- Location spoofing via `adb geo fix`
- Notification monitoring via `adb shell dumpsys notification`

### 4. Shared Core Library

**Technology**: Rust (shared with server) + TypeScript/JavaScript (for Flutter interop)

**Components**:

#### Protocol Definition
- Message types (requests, responses, events)
- Serialization format (JSON)
- Versioning strategy

#### Data Models
- Device models
- Session models
- Event models (touch, gesture, GPS, etc.)
- Configuration models

#### Networking
- WebSocket client/server utilities
- REST client utilities
- Connection pooling
- Retry logic

#### Authentication
- Device pairing protocol
- Token generation and validation
- Session key management
- TLS configuration

#### Utilities
- Logging framework
- Error handling
- Configuration management
- Performance monitoring

## Communication Protocol

### WebSocket Messages

All WebSocket messages follow this structure:

```json
{
  "type": "message_type",
  "version": 1,
  "timestamp": 1691234567890,
  "payload": { /* type-specific data */ }
}
```

### Message Types

#### Client → Server
- `pair_request` - Device pairing request
- `auth_request` - Authentication request
- `simulator_list` - Request available simulators
- `connect_simulator` - Connect to a simulator
- `touch_event` - Touch event
- `gesture` - Gesture event
- `gps_update` - GPS location update
- `device_button` - Device button press
- `clipboard_sync` - Clipboard synchronization
- `file_transfer` - File transfer request
- `start_recording` - Start session recording
- `stop_recording` - Stop session recording

#### Server → Client
- `pair_response` - Pairing response
- `auth_response` - Authentication response
- `simulator_list` - Available simulators
- `screen_frame` - Screen frame data
- `notification` - Simulator notification
- `clipboard_sync` - Clipboard sync from simulator
- `file_transfer` - File transfer progress/data
- `recording_status` - Recording status update
- `error` - Error message

### REST API Endpoints

- `POST /api/v1/pair` - Device pairing
- `POST /api/v1/auth` - Authentication
- `GET /api/v1/simulators` - List available simulators
- `GET /api/v1/sessions` - List active sessions
- `POST /api/v1/sessions` - Create session
- `DELETE /api/v1/sessions/:id` - Terminate session
- `GET /api/v1/recordings` - List recordings
- `GET /api/v1/recordings/:id` - Download recording
- `GET /api/v1/plugins` - List plugins
- `POST /api/v1/plugins` - Install plugin

## Security Architecture

### Authentication Flow

1. **Pairing**:
   - QR code or manual entry
   - Exchange of public keys
   - Generation of shared secret
   - Issue of device token

2. **Session Authentication**:
   - Device token + session key
   - TLS encryption for all communication
   - Token rotation

3. **Authorization**:
   - Device-based permissions
   - Session-based permissions
   - Simulator access control

### Security Measures

- TLS 1.3 for all network communication
- Device fingerprinting
- Rate limiting
- Input validation
- Replay attack prevention (nonces + timestamps)
- Secure storage (keychain/keystore)
- Audit logging

## Plugin System

### Plugin Interface

Plugins can extend functionality by implementing:

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, context: PluginContext) -> Result<()>;
    fn on_message(&mut self, message: &Message) -> Result<Option<Message>>;
    fn shutdown(&mut self) -> Result<()>;
}
```

### Plugin Types

- **Simulator Adapters** - New simulator backends
- **Authentication Providers** - Custom auth methods
- **Streaming Codecs** - Custom screen encoding
- **Notification Providers** - Custom notification handling
- **Analytics** - Usage analytics
- **Automation** - Test automation and scripting

## Data Flow Examples

### Screen Streaming

```
Simulator → Adapter → Server (encode) → WebSocket → Companion App (decode) → Display
```

### Touch Event

```
Companion App (capture) → WebSocket → Server → Adapter → Simulator
```

### GPS Streaming

```
Phone GPS → Companion App → WebSocket → Server → Adapter → Simulator
```

### Notification Forwarding

```
Simulator → Adapter → Server → WebSocket → Companion App → Display
```

## Performance Considerations

### Screen Streaming
- Adaptive quality based on network conditions
- Frame rate throttling
- Region-based updates (only changed regions)
- Codec selection (H.264, VP8, etc.)

### Latency Optimization
- UDP for high-frequency events (optional)
- Message batching
- Priority queues
- Local caching

### Resource Management
- Connection pooling
- Memory limits per session
- Bandwidth throttling
- CPU usage monitoring

## Deployment Architecture

### Development
- Local server on developer machine
- Companion app on physical device
- Direct Wi-Fi connection

### Production
- Dedicated server or cloud deployment
- Load balancing for multiple users
- Reverse proxy with TLS termination
- Database replication for scale

## Extensibility

The architecture is designed to be easily extended:

1. **New Simulator Support**: Implement `SimulatorAdapter` trait
2. **New Companion Platforms**: Implement Flutter for new platform
3. **New Features**: Add message types to protocol
4. **New Plugins**: Implement `Plugin` trait
5. **New Auth Methods**: Implement auth provider plugin

## Technology Rationale

### Rust for Server
- Performance and memory safety
- Strong typing
- Excellent async support (Tokio)
- Cross-platform compilation
- Rich ecosystem

### Flutter for Mobile & Desktop
- Single codebase for multiple platforms
- Native performance
- Rich UI components
- Hot reload for development
- Strong community

### WebRTC for Streaming
- Low latency
- Adaptive quality
- Widely supported
- Built-in NAT traversal

### SQLite for Storage
- Zero configuration
- Embedded database
- ACID compliance
- Cross-platform

## Testing Strategy

- **Unit Tests**: Individual components and functions
- **Integration Tests**: Component interactions
- **End-to-End Tests**: Full user flows
- **Performance Tests**: Load and stress testing
- **Security Tests**: Penetration testing
- **Network Tests**: Various network conditions

## Monitoring and Observability

- Structured logging
- Metrics collection (Prometheus format)
- Health check endpoints
- Performance profiling
- Error tracking (Sentry integration)
