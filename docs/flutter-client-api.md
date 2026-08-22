# SimBridge Flutter Client API Documentation

## Overview

This document describes the backend API contract for the SimBridge Flutter mobile client. The SimBridge server is a Rust-based application that provides REST and WebSocket endpoints for controlling iOS Simulators and Android Emulators.

## Base URLs

**Development:**
- HTTP: `http://localhost:8080`
- WebSocket: `ws://localhost:8080/ws`

**Production:**
- HTTP: `https://your-server.com`
- WebSocket: `wss://your-server.com/ws`

## Authentication

The current implementation does not require authentication (development mode). However, the protocol supports:

1. **Pairing Flow:**
   - Client sends `PairRequest` with device information
   - Server responds with `PairResponse` containing pairing code
   - Client authenticates using pairing code

2. **Authentication Flow:**
   - Client sends `AuthRequest` with device ID and token
   - Server responds with `AuthResponse` containing session token

**Note:** Current implementation is in development mode without enforced authentication.

## REST API Endpoints

### Health Check

**GET /health**

Check server health and version.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### Simulator List

**GET /api/v1/simulators**

Retrieve list of available simulators/emulators.

**Response:**
```json
{
  "simulators": [
    {
      "id": "android-emu-1",
      "name": "Pixel 7",
      "platform": "android",
      "status": "offline"
    },
    {
      "id": "ios-sim-1",
      "name": "iPhone 15 Pro",
      "platform": "ios",
      "status": "offline"
    }
  ]
}
```

**Simulator Status Values:**
- `available` - Simulator is ready for connection
- `busy` - Simulator is in use
- `offline` - Simulator is not running
- `error` - Simulator has an error

### Session Management

**GET /api/v1/sessions**

List active sessions.

**Response:**
```json
["session-id-1", "session-id-2"]
```

**POST /api/v1/sessions**

Create a new session.

**Request:**
```json
{
  "simulator_id": "android-emu-1",
  "device_id": "client-device-id"
}
```

**Response:**
```json
{
  "session_id": "uuid-string",
  "simulator_id": "android-emu-1",
  "status": "active"
}
```

**DELETE /api/v1/sessions/:id**

Delete a session.

**Response:**
```json
{
  "status": "deleted",
  "session_id": "session-id"
}
```

## WebSocket Endpoint

**Connection: `ws://localhost:8080/ws`**

The WebSocket endpoint uses a binary protocol with JSON-serialized messages defined in the shared protocol library.

### Message Structure

All WebSocket messages follow this structure:

```json
{
  "message_type": "MessageType",
  "version": 1,
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "optional-uuid",
  "payload": { /* message-specific data */ }
}
```

### Client → Server Messages

#### PairRequest

Initiate device pairing.

**Payload:**
```json
{
  "device_id": "client-device-id",
  "device_name": "My Phone",
  "device_type": "android",
  "public_key": "base64-encoded-key"
}
```

**Device Types:** `android`, `ios`, `desktop`

#### AuthRequest

Authenticate with the server.

**Payload:**
```json
{
  "device_id": "client-device-id",
  "token": "auth-token",
  "challenge_response": "optional-response"
}
```

#### SimulatorList

Request list of available simulators.

**Payload:** Empty object `{}`

#### ConnectSimulator

Connect to a specific simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "stream_config": {
    "quality": "high",
    "fps": 30,
    "audio_enabled": false
  }
}
```

**Stream Quality:** `low`, `medium`, `high`, `ultra`

#### DisconnectSimulator

Disconnect from current simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1"
}
```

#### TouchEvent

Send touch events to simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "touches": [
    {
      "id": 1,
      "x": 100.5,
      "y": 200.3,
      "phase": "began",
      "force": 0.5,
      "major_radius": 20.0
    }
  ]
}
```

**Touch Phases:** `began`, `moved`, `ended`, `cancelled`

#### Gesture

Send gesture events to simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "gesture_type": "swipe",
  "data": {
    "direction": "up",
    "distance": 500.0
  }
}
```

**Gesture Types:** `swipe`, `pinch`, `rotation`, `long_press`, `double_tap`

**Swipe Directions:** `up`, `down`, `left`, `right`

#### GpsUpdate

Send GPS location to simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "location": {
    "latitude": 37.7749,
    "longitude": -122.4194,
    "altitude": 10.0,
    "accuracy": 5.0,
    "speed": 0.0,
    "heading": 0.0,
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### HeadingUpdate

Send compass heading to simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "heading": 45.5,
  "accuracy": 2.0,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### MotionUpdate

Send motion sensor data to simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "acceleration": [0.1, 0.2, 9.8],
  "gravity": [0.0, 0.0, 9.8],
  "rotation_rate": [0.01, 0.02, 0.03],
  "attitude": {
    "roll": 0.1,
    "pitch": 0.2,
    "yaw": 0.3
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### DeviceButton

Send device button press to simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "button": "home"
}
```

**Device Buttons:** `home`, `back`, `app_switcher`, `lock`, `unlock`, `volume_up`, `volume_down`, `mute`, `rotate_left`, `rotate_right`, `shake`, `screenshot`

#### ClipboardSync

Synchronize clipboard with simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "content": "text content",
  "content_type": "text"
}
```

**Content Types:** `text`, `image`, `url`

#### FileTransfer

Transfer file to/from simulator.

**Payload:**
```json
{
  "transfer_id": "uuid",
  "simulator_id": "android-emu-1",
  "direction": "upload",
  "file_name": "test.txt",
  "file_size": 1024,
  "chunk_data": "base64-encoded-chunk",
  "chunk_index": 0,
  "total_chunks": 10
}
```

**Directions:** `upload`, `download`

#### StartRecording

Start screen recording.

**Payload:** Empty object `{}`

#### StopRecording

Stop screen recording.

**Payload:** Empty object `{}`

#### GetRecordings

Get list of recordings.

**Payload:** Empty object `{}`

#### Ping

Ping server to keep connection alive.

**Payload:** Empty object `{}`

### Server → Client Messages

#### PairResponse

Response to pairing request.

**Payload:**
```json
{
  "success": true,
  "pairing_code": "123456",
  "message": "Pairing successful"
}
```

#### AuthResponse

Response to authentication request.

**Payload:**
```json
{
  "success": true,
  "session_token": "token-string",
  "expires_at": "2024-01-15T11:30:00Z",
  "message": "Authentication successful"
}
```

#### ScreenFrame

Video frame from simulator screen.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "frame_data": "base64-encoded-frame",
  "encoding": "jpeg",
  "width": 1080,
  "height": 1920,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Encodings:** `h264`, `vp8`, `jpeg`, `png`

#### Notification

Notification from simulator.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "notification": {
    "id": "notif-id",
    "app_name": "Weather",
    "title": "Severe Weather",
    "body": "Storm warning in your area",
    "timestamp": "2024-01-15T10:30:00Z",
    "icon": "optional-icon-url",
    "action": "optional-action"
  }
}
```

#### RecordingStatus

Status of screen recording.

**Payload:**
```json
{
  "recording_id": "uuid",
  "status": "recording",
  "duration_seconds": 60,
  "file_size_bytes": 1024000
}
```

**Recording Status:** `recording`, `paused`, `stopped`, `processing`, `completed`, `error`

#### Pong

Response to ping message.

**Payload:**
```json
{
  "status": "ok"
}
```

#### Error

Error message from server.

**Payload:**
```json
{
  "code": "AuthenticationFailed",
  "message": "Invalid credentials",
  "details": "Optional error details"
}
```

**Error Codes:** `AuthenticationFailed`, `InvalidRequest`, `SimulatorNotFound`, `SimulatorBusy`, `ConnectionError`, `StreamError`, `FileTransferError`, `RecordingError`, `InternalError`, `RateLimited`, `PermissionDenied`

### Bidirectional Messages

#### SettingsUpdate

Update settings.

**Payload:** Settings object (format TBD)

#### SessionInfo

Session information.

**Payload:**
```json
{
  "session_id": "uuid",
  "device_id": "client-device-id",
  "simulator_id": "android-emu-1",
  "connected_at": "2024-01-15T10:00:00Z",
  "status": "active"
}
```

**Session Status:** `active`, `paused`, `terminated`

#### MetricsUpdate

Performance metrics.

**Payload:**
```json
{
  "simulator_id": "android-emu-1",
  "cpu_usage": 45.5,
  "memory_usage": 1048576,
  "bandwidth": 1024.5,
  "fps": 30.0,
  "latency": 45.0
}
```

### WebRTC Messages

#### WebrtcOffer

WebRTC SDP offer (client → server).

**Payload:**
```json
{
  "sdp": "v=0\r\no=- ...",
  "session_id": "session-uuid",
  "stream_id": "screen-1"
}
```

#### WebrtcAnswer

WebRTC SDP answer (server → client).

**Payload:**
```json
{
  "sdp": "v=0\r\no=- ...",
  "session_id": "session-uuid",
  "stream_id": "screen-1"
}
```

#### WebrtcIceCandidate

WebRTC ICE candidate (bidirectional).

**Payload:**
```json
{
  "candidate": "candidate:1 1 UDP 2130706431 192.168.1.1 54421 typ host",
  "sdp_mid": "0",
  "sdp_mline_index": 0,
  "session_id": "session-uuid",
  "stream_id": "screen-1"
}
```

## Error Handling

All errors should be handled gracefully:

1. **HTTP Errors:**
   - 400: Bad Request
   - 401: Unauthorized
   - 404: Not Found
   - 500: Internal Server Error

2. **WebSocket Errors:**
   - Connection failures
   - Message parse errors
   - Invalid message types
   - Server-side errors via Error messages

3. **Reconnection Behavior:**
   - Implement exponential backoff
   - Start with 1 second, max 30 seconds
   - Re-authenticate on reconnection
   - Restore session state if possible

## Streaming Protocols

### Screen Streaming

The server supports multiple streaming protocols:

1. **WebSocket Binary Frames:** Base64-encoded JPEG/PNG frames
2. **WebRTC:** SDP-based peer-to-peer streaming (preferred for low latency)

**Recommended:** Use WebRTC for production applications due to lower latency and better performance.

### Frame Encoding

- **JPEG:** Good compression, widely supported
- **PNG:** Lossless, larger file size
- **H.264:** Best compression, requires decoder
- **VP8:** Alternative to H.264

**Current Implementation:** JPEG compression is implemented and recommended for Flutter clients.

## Data Models

### Simulator

```json
{
  "id": "string",
  "name": "string",
  "platform": "ios|android",
  "os_version": "string",
  "status": "available|busy|offline|error",
  "screen_size": {
    "width": 1080,
    "height": 1920,
    "scale": 2.0
  },
  "device_details": {
    "device_type": "string",
    "model": "string",
    "manufacturer": "string",
    "cpu_cores": 8,
    "memory_mb": 8192
  }
}
```

### Device

```json
{
  "id": "string",
  "name": "string",
  "device_type": "android|ios|desktop",
  "platform": "string",
  "os_version": "string",
  "paired_at": "2024-01-15T10:00:00Z",
  "last_seen": "2024-01-15T10:30:00Z",
  "is_trusted": true,
  "public_key": "string"
}
```

### Session

```json
{
  "id": "uuid",
  "device_id": "string",
  "simulator_id": "string",
  "status": "active|paused|terminated",
  "created_at": "2024-01-15T10:00:00Z",
  "connected_at": "2024-01-15T10:00:05Z",
  "disconnected_at": null,
  "last_activity": "2024-01-15T10:30:00Z",
  "stream_config": {
    "quality": "high",
    "fps": 30,
    "audio_enabled": false,
    "video_codec": "h264"
  }
}
```

## Required Headers

### HTTP Requests

```http
Content-Type: application/json
Accept: application/json
```

### WebSocket

WebSocket connections use the standard WebSocket protocol with binary message frames containing JSON-serialized protocol messages.

## Protocol Version

Current protocol version: `1`

Clients should check the version field in each message to ensure compatibility.

## Coordinate System

Touch coordinates are in **simulator screen space**:

- Origin (0,0) at top-left
- X increases to the right
- Y increases downward
- Coordinates should be scaled to simulator resolution

**Example:** If simulator is 1080x1920 and Flutter widget is 400x800:
- Flutter touch at (200, 400) → Simulator touch at (540, 960)

## Rate Limiting

The server may implement rate limiting for:

- Session creation (max N sessions per device)
- Touch events (max N events per second)
- File transfers (max N concurrent transfers)
- API requests (max N requests per minute)

## Security Considerations

**Current Status:** Development mode

**Production Requirements:**
- Enable TLS/WSS for all connections
- Implement JWT authentication
- Add rate limiting
- Use TURN server for WebRTC NAT traversal
- Validate all input data
- Sanitize SDP data in WebRTC messages

## Testing

### Manual Testing

```bash
# Health check
curl http://localhost:8080/health

# List simulators
curl http://localhost:8080/api/v1/simulators

# Create session
curl -X POST http://localhost:8080/api/v1/sessions \
  -H "Content-Type: application/json" \
  -d '{"simulator_id":"android-emu-1","device_id":"test-device"}'
```

### WebSocket Testing

Use the browser test client: `server/test-webrtc.html`

## Backend Limitations

The current backend implementation has these limitations:

1. **Authentication:** Not enforced in development mode
2. **WebRTC:** Signaling infrastructure exists but screen capture integration is pending
3. **Screen Capture:** Adapters exist but need wiring to frame delivery system
4. **Rate Limiting:** Not implemented
5. **TLS/WSS:** Not configured

## Future Enhancements

Planned backend features:

1. Full WebRTC screen capture integration
2. H.264 video encoding
3. TURN server support
4. JWT authentication enforcement
5. Rate limiting
6. TLS/WSS encryption
7. Enhanced error recovery
8. Multi-simulator support

## Contact

For API questions or issues, refer to the main SimBridge documentation or contact the development team.
