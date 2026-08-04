# SimBridge API Reference

## REST API

### Base URL

```
http://localhost:8080/api/v1
```

### Authentication

Most endpoints require an authentication token in the Authorization header:

```
Authorization: Bearer <token>
```

### Endpoints

#### Health Check

```http
GET /health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

#### List Simulators

```http
GET /simulators
```

Response:
```json
{
  "simulators": [
    {
      "id": "ios-sim-1",
      "name": "iPhone 15 Pro",
      "platform": "ios",
      "status": "available"
    }
  ]
}
```

#### List Sessions

```http
GET /sessions
```

Response:
```json
[
  "session-id-1",
  "session-id-2"
]
```

#### Create Session

```http
POST /sessions
Content-Type: application/json

{
  "simulator_id": "ios-sim-1",
  "device_id": "device-123"
}
```

Response:
```json
{
  "session_id": "new-session-id",
  "simulator_id": "ios-sim-1",
  "status": "active"
}
```

#### Delete Session

```http
DELETE /sessions/:id
```

Response:
```json
{
  "status": "deleted",
  "session_id": "session-id"
}
```

## WebSocket API

### Connection URL

```
ws://localhost:8080/ws
```

### Message Format

All WebSocket messages follow this structure:

```json
{
  "message_type": "message_type",
  "version": 1,
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "optional-uuid",
  "payload": { /* type-specific data */ }
}
```

### Message Types

#### Client → Server

##### Pair Request

```json
{
  "message_type": "pair_request",
  "payload": {
    "device_id": "device-123",
    "device_name": "My Phone",
    "device_type": "android",
    "public_key": "base64-encoded-key"
  }
}
```

##### Auth Request

```json
{
  "message_type": "auth_request",
  "payload": {
    "device_id": "device-123",
    "token": "auth-token"
  }
}
```

##### Connect Simulator

```json
{
  "message_type": "connect_simulator",
  "payload": {
    "simulator_id": "ios-sim-1",
    "stream_config": {
      "quality": "medium",
      "fps": 30,
      "audio_enabled": false
    }
  }
}
```

##### Touch Event

```json
{
  "message_type": "touch_event",
  "payload": {
    "simulator_id": "ios-sim-1",
    "touches": [
      {
        "id": 1,
        "x": 100.0,
        "y": 200.0,
        "phase": "began",
        "force": 0.5
      }
    ]
  }
}
```

##### GPS Update

```json
{
  "message_type": "gps_update",
  "payload": {
    "simulator_id": "ios-sim-1",
    "location": {
      "latitude": 37.7749,
      "longitude": -122.4194,
      "altitude": 10.0,
      "accuracy": 5.0,
      "speed": 0.0,
      "heading": 0.0,
      "timestamp": "2024-01-01T00:00:00Z"
    }
  }
}
```

#### Server → Client

##### Pair Response

```json
{
  "message_type": "pair_response",
  "payload": {
    "success": true,
    "pairing_code": "ABCD-EFGH-IJKL",
    "message": "Pairing initiated"
  }
}
```

##### Screen Frame

```json
{
  "message_type": "screen_frame",
  "payload": {
    "simulator_id": "ios-sim-1",
    "frame_data": "base64-encoded-frame",
    "encoding": "h264",
    "width": 390,
    "height": 844,
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

##### Notification

```json
{
  "message_type": "notification",
  "payload": {
    "simulator_id": "ios-sim-1",
    "notification": {
      "id": "notif-123",
      "app_name": "Messages",
      "title": "New Message",
      "body": "Hello from SimBridge",
      "timestamp": "2024-01-01T00:00:00Z"
    }
  }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `authentication_failed` | Invalid or expired authentication token |
| `invalid_request` | Malformed request |
| `simulator_not_found` | Simulator does not exist |
| `simulator_busy` | Simulator is in use |
| `connection_error` | Network connection error |
| `stream_error` | Screen streaming error |
| `file_transfer_error` | File transfer failed |
| `recording_error` | Recording operation failed |
| `internal_error` | Server internal error |
| `rate_limited` | Too many requests |
| `permission_denied` | Insufficient permissions |

## Rate Limiting

- Default: 60 requests per minute per device
- Headers included: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
