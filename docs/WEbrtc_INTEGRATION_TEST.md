# SimBridge WebRTC Integration - Testing Guide

**Version:** 0.3.0  
**Status:** ✅ Ready for Browser Testing

---

## Overview

This guide explains how to test the WebRTC signaling implementation in SimBridge. The implementation enables real-time video streaming from iOS Simulators and Android Emulators to companion devices with <200ms latency.

---

## Quick Start (Browser Testing)

### 1. Start the Server

```bash
cd server
cargo run
```

Expected output:
```
Starting SimBridge Server v0.1.0
Database initialized at simbridge.db
Discovering Android devices...
Found 0 Android device(s)
Discovering iOS devices...
Found 0 iOS device(s)
Server listening on 0.0.0.0:8080
```

### 2. Open Test Client in Browser

Navigate to:
```
file:///path/to/server/test-webrtc.html
```

**OR** serve the HTML file locally:
```bash
cd server
python3 -m http.server 9000
# Then open http://localhost:9000/test-webrtc.html
```

### 3. Test WebSocket Connection

1. Enter server URL in test client (default: `ws://localhost:8080/ws`)
2. Click **"🔌 Connect to Server"**
3. Verify status shows "Status: Connected" with green background

**Expected Log:**
```
[time] ✅ Connected to SimBridge server via WebSocket
```

### 4. Create WebRTC Offer

1. Click **"📤 Create Offer"** button
2. Wait for offer to be generated (SDP will appear in text box)
3. Click **"✅ Send Offer to Server"**

**Expected Log:**
```
[time] Creating SDP offer...
[time] ICE Candidate: candidate:1 1 UDP...
[time] All ICE candidates gathered
[time] Created offer. SDP length: 2500+
[time] ✅ Offer sent to server. Waiting for answer...
```

### 5. Receive Answer from Server

Server will respond with SDP answer. The client will show:
- "Received SDP answer from server" in logs
- Answer text in the SDP Answer field

Click **"⬅️ Receive Answer from Server"** to set remote description.

**Expected Log:**
```
[time] Received SDP answer from server
[time] ✅ Remote description set successfully
[time] WebRTC connection state: connecting...
[time] 🎉 Connection established! Video should start flowing...
```

### 6. Verify Connection State

Watch the "Connection State" card - it should progress through:
1. `connecting` → `connected` (success) or `failed`/`closed` (failure)

**Success Indicators:**
- ✅ WebSocket status: Connected (green)
- ✅ ICE Candidates relayed: >0
- ✅ Connection state: `connected`
- ✅ No errors in event log

---

## Testing WebRTC Signaling Flow

### Complete Signaling Sequence

```
┌─────────────┐       WebSocket       ┌──────────────┐
│ Browser     │  (Offer)              │ SimBridge    │
│ Client      │ ─────────────────────► │ Server      │
└─────────────┘                       └──────┬───────┘
                                             │
                                              (Generate Answer)
                                             ▼
                                            ┌───────┐
                                            │ WebRTC │
                                            │ Frame  │
                                            │ Delivery│
                                            │ System │
                                            └───────┘
```

### Message Types Tested

| Direction | Message Type | Payload Fields | Description |
|-----------|--------------|----------------|-------------|
| Client → Server | `webrtcOffer` | `sdp`, `session_id`, `stream_id` | Browser offers connection |
| Server → Client | `webrtcAnswer` | `sdp`, `session_id`, `stream_id` | Server responds with answer |
| Both ↔️ Server | `webrtcIceCandidate` | `candidate`, `sdp_mid`, `sdp_mline_index`, `session_id`, `stream_id` | ICE candidate exchange |

---

## REST API Testing

### Create WebRTC Session

```bash
curl -X POST http://localhost:8080/api/v1/webrtc/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "simulator_id": "ios-sim-1",
    "device_id": "companion-device-1",
    "stream_id": "screen-stream-1"
  }'
```

**Expected Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "simulator_id": "ios-sim-1",
  "status": "waiting_for_offer"
}
```

### Get Session Statistics

```bash
curl -X GET http://localhost:8080/api/v1/webrtc/sessions/{session-id}/stats
```

**Expected Response:**
```json
{
  "state": "connected",
  "duration_ms": 12345,
  "num_ice_candidates": 3,
  "connected_at": "2024-01-15T10:30:00.000Z",
  "created_at": "2024-01-15T10:29:45.000Z"
}
```

### List Active Sessions

```bash
curl -X GET http://localhost:8080/api/v1/sessions
```

---

## Troubleshooting

### Issue 1: WebSocket Connection Fails

**Symptoms:**
- Status shows "Connection Failed" or red error message
- Event log shows "WebSocket error"

**Solutions:**
1. Verify server is running on correct port (default: 8080)
2. Check firewall allows port 8080
3. Try different browser (Chrome, Firefox, Edge)
4. Verify no proxy interfering with WebSocket connections

### Issue 2: SDP Exchange Completes but Video Doesn't Stream

**Symptoms:**
- "Connection state: connected" shows
- Event log has no errors
- Video container stays empty/black

**Solutions:**
1. Screen capture not yet integrated - this is expected at this stage
2. Check if iOS/Android adapter actually captures frames
3. Verify frame delivery system is receiving data from adapters
4. Enable debug logging: `cargo run -- --log_level debug`

### Issue 3: ICE Candidates Not Exchanged

**Symptoms:**
- "ICE Candidate relay" count stays at 0
- Connection state stuck at "connecting"

**Solutions:**
1. Check browser supports WebRTC (most modern browsers do)
2. Verify STUN server is accessible (`stun.l.google.com:19302`)
3. Test with different network environment (try both local and internet)

### Issue 4: Session ID Parsing Errors

**Symptoms:**
- REST API returns "NotFound" for session stats
- Logs show UUID parsing errors

**Solutions:**
1. Ensure you're using valid UUID format in URLs
2. Check that sessions were actually created via REST API
3. Verify `webrtc_manager` is properly initialized in server state

---

## Integration Test Checklist

Use this checklist to verify WebRTC integration completeness:

### ✅ Browser Signaling Tests (Phase 1)
- [ ] WebSocket connection established successfully
- [ ] SDP offer generated without errors
- [ ] Offer sent to server via WebSocket
- [ ] Server responds with SDP answer
- [ ] Answer received and remote description set
- [ ] Connection state transitions to `connected`
- [ ] ICE candidates automatically exchanged
- [ ] Event log shows all expected messages

### ✅ REST API Tests (Phase 2)
- [ ] WebRTC session created via POST /api/v1/webrtc/sessions
- [ ] Session retrieved via GET /api/v1/webrtc/sessions/{id}
- [ ] Session stats accessible via /{id}/stats endpoint
- [ ] Sessions listed correctly in /api/v1/sessions

### ✅ Screen Capture Integration (Phase 3 - Future)
- [ ] iOS adapter captures frames successfully
- [ ] Android adapter captures frames successfully
- [ ] Frames sent to frame delivery channel
- [ ] Video stream visible in browser
- [ ] Latency <200ms measured

### ✅ Performance Tests (Phase 4 - Future)
- [ ] FPS stable at target value (30/60)
- [ ] Bitrate within acceptable range
- [ ] Memory usage reasonable
- [ ] No connection drops during extended testing

---

## Next Steps: Full Integration

Once browser signaling is verified, integrate with actual screen capture:

### Step 1: Wire iOS Adapter to Frame Delivery

File: `server/src/adapters/ios.rs`

```rust
impl IosSimulatorAdapter {
    pub async fn start_screen_stream_with_webrtc(
        &mut self,
        stream_id: String,
        webrtc_manager: Arc<WebRTCSignalingManager>,
        frame_delivery: Arc<FrameDeliverySystem>,
    ) -> Result<(), AdapterError> {
        
        // 1. Create WebRTC session
        let session_id = webrtc_manager.create_session(
            self.simulator_id().to_string(),
            "device-1".to_string(),
            stream_id.clone(),
        ).await?;

        // 2. Spawn async task to capture and deliver frames
        tokio::spawn(async move {
            loop {
                // Capture frame from iOS simulator
                match IosScreenStream::capture_frame(&stream_id) {
                    Ok(frame_data) => {
                        // Compress with JPEG encoder
                        let compressed = VideoEncoder::encode_png_to_jpeg(&frame_data);
                        
                        if let Ok(compressed) = compressed {
                            // Send through delivery channel
                            frame_delivery.broadcast_frame(stream_id.clone(), compressed)
                                .await.ok();
                        }
                    }
                    Err(e) => tracing::error!("Frame capture failed: {}", e),
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(33)).await; // ~30 FPS
            }
        });

        Ok(())
    }
}
```

### Step 2: Wire Android Adapter Similarly

File: `server/src/adapters/android.rs`

Similar implementation using ADB commands for screen capture.

### Step 3: Connect to WebSocket Signaling

In the main server loop, subscribe WebSocket connections to the signaling manager:

```rust
// In websocket_handler or new WebSocket connection handler
async fn handle_websocket_signaling(
    ws: WebSocketUpgrade,
    webrtc_manager: Arc<WebRTCSignalingManager>,
) -> impl IntoResponse {
    
    // Create broadcast channel for this connection
    let (send_tx, mut recv_rx) = tokio::sync::broadcast::channel::<WebRTCSignal>(100);
    
    ws.on_upgrade(move |socket| {
        handle_websocket_signaling_impl(socket, webrtc_manager, send_tx, recv_rx)
    })
}

async fn handle_websocket_signaling_impl(
    socket: WebSocket,
    webrtc_manager: Arc<WebRTCSignalingManager>,
    send_tx: tokio::sync::broadcast::Sender<WebRTCSignal>,
    _recv_rx: tokio::sync::broadcast::Receiver<WebRTCSignal>, // Listen for server signals
) {
    
    // Track this WebSocket connection in the manager
    let ws_id = Uuid::new_v4();
    webrtc_manager.register_ws_connection(ws_id, send_tx).await;
    
    // Handle incoming messages from client
    while let Some(result) = socket.next().await {
        match result {
            Ok(Message::Text(text)) => {
                // Deserialize and route WebRTC messages
                // Create offer, answer, relay ICE candidates...
            }
            // ... handle other message types
        }
    }
}
```

---

## Production Considerations

### Security Hardening

Before production deployment:
1. [ ] Add authentication to WebSocket connections
2. [ ] Implement TLS/WSS for secure signaling
3. [ ] Rate limit session creation (prevent abuse)
4. [ ] Validate all incoming SDP data
5. [ ] Set up TURN server for NAT traversal in restricted networks

### Performance Optimization

For high-throughput scenarios:
1. [ ] Implement H.264 encoding instead of JPEG (better compression)
2. [ ] Add adaptive quality based on network conditions
3. [ ] Use frame skipping during low activity
4. [ ] Enable hardware acceleration if available
5. [ ] Implement connection pooling for multiple streams

### Monitoring & Observability

Add these metrics to production:
1. [ ] Active concurrent sessions count
2. [ ] Average session duration
3. [ ] Frame delivery latency (p95, p99)
4. [ ] ICE candidate success rate
5. [ ] Error rates by type
6. [ ] Memory and CPU usage per session

---

## Debugging Tips

### Enable Verbose Logging

```bash
RUST_LOG=debug cargo run --release
```

This will show:
- Detailed WebRTC negotiation steps
- Frame delivery statistics
- Adapter command execution details

### Capture Network Traffic

For troubleshooting ICE candidates and signaling:
```bash
# Use Wireshark or tcpdump to capture WebSocket traffic
tcpdump -i any port 8080 -w simbridge-signaling.pcap
```

### Test WebRTC in Browser Console

Open browser DevTools → Console and run:
```javascript
// Check if WebRTC is supported
console.log("WebRTC Supported:", !!(window.RTCPeerConnection));

// List all peer connections
let pc = new RTCPeerConnection({ iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] });
pc.onconnectionstatechange = () => console.log(pc.connectionState);
pc.createOffer()
  .then(offer => pc.setLocalDescription(offer))
  .then(() => console.log("Local description:", pc.localDescription));
```

---

## Success Criteria

The WebRTC integration is **complete and functional** when:

1. ✅ Browser client can connect to SimBridge server via WebSocket
2. ✅ SDP offer/answer exchange completes successfully
3. ✅ ICE candidates are exchanged (at least 1-3 candidates)
4. ✅ Connection state reaches `connected`
5. ✅ Video stream displays in browser (when integrated with capture)
6. ✅ Latency measured <200ms between frame capture and display

---

## Resources

- [WebRTC Server Guide](WEbrtc_SERVER_GUIDE.md) - Complete technical reference
- [WebRTC Integration Examples](WEbrtc_INTEGRATION_EXAMPLE.md) - Working code examples
- [Project Health Report](PROJECT_HEALTH_V2.md) - Overall project status

---

**Last Updated:** 2024-01-15  
**Author:** SimBridge Development Team
<EOF>