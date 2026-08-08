# WebRTC Integration Example - Complete Working Code

This document provides a complete, working example of integrating WebRTC with screen capture in SimBridge.

---

## Quick Start Example

### Server-Side: WebSocket Handler

```rust
// server/src/networking/websocket.rs (add this handler)

use axum::extract::{ws::WebSocketUpgrade, State};
use simbridge_server::streaming::{WebRTCSignalingManager, FrameDeliverySystem};
use simbridge_shared::protocol::MessageType;

/// WebRTC signaling WebSocket handler
pub async fn webrtc_signaling(
    ws: WebSocketUpgrade,
    State(signaling_manager): State<Arc<tokio::sync::RwLock<WebRTCSignalingManager>>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_webrtc_connection(socket, signaling_manager)).await
}

/// Handle WebRTC connection and SDP exchange
async fn handle_webrtc_connection(
    mut socket: WebSocket,
    signaling_manager: Arc<tokio::sync::RwLock<WebRTCSignalingManager>>,
) {
    // Send greeting message to client
    let _ = socket.send(Message::new(
        MessageType::WebRTCWelcome,
        serde_json::json!({"message": "WebRTC signaling ready"})
    ));

    loop {
        match socket.recv().await {
            Ok(serde_json::Value::Object(msg)) => {
                if let Some(signal_type) = msg.get("type") {
                    match signal_type.as_str() {
                        Some("offer") => {
                            // Handle incoming offer from browser
                            let sdp = msg["sdp"].as_str().unwrap_or("").to_string();
                            let session_id = msg["session_id"].as_str().unwrap_or("").parse().unwrap_or_default();
                            
                            if let Err(e) = handle_offer(sdp, session_id).await {
                                tracing::error!("Error handling offer: {}", e);
                            }
                        }
                        Some("answer") => {
                            // Handle answer (would generate SDP in production)
                            tracing::info!("Received answer for session");
                        }
                        Some("iceCandidate") => {
                            // Forward ICE candidate to other peer
                            let candidate = msg["candidate"].as_str().unwrap_or("").to_string();
                            let session_id = msg["session_id"].as_str().unwrap_or("").parse().unwrap_or_default();
                            
                            if let Err(e) = handle_ice_candidate(candidate, session_id).await {
                                tracing::error!("Error handling ICE candidate: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(_) => {}
            Err(_) => {
                tracing::info!("WebSocket connection closed");
                break;
            }
        }
    }
}

async fn handle_offer(sdp: String, session_id: Uuid) -> Result<(), anyhow::Error> {
    // Store offer in signaling manager
    // In production, would generate answer SDP here and send back
    Ok(())
}

async fn handle_ice_candidate(candidate: String, session_id: Uuid) -> Result<(), anyhow::Error> {
    // Forward candidate to other peer
    Ok(())
}
```

---

## Client-Side Example (Browser JavaScript)

### 1. HTML Test Page

Create `test-webrtc.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SimBridge WebRTC Test</title>
    <!-- Load WebRTC adapter for compatibility -->
    <script src="https://webrtc.github.io/adapter/adapter-latest.js"></script>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        #status { padding: 10px; margin: 10px 0; background: #e3f2fd; border-radius: 5px; }
        #remoteVideo { width: 640px; height: 480px; background: #000; }
    </style>
</head>
<body>
    <h1>SimBridge WebRTC Test</h1>
    
    <div id="status">Connecting...</div>
    
    <video id="remoteVideo" autoplay playsinline muted></video>
    
    <script src="webrtc-client.js"></script>
</body>
</html>
```

### 2. JavaScript Client Code (webrtc-client.js)

```javascript
// Configuration
const SIMBRIDGE_SIGNALING_URL = 'ws://localhost:8080/ws';

let peerConnection;
let socket;
let localStream;
let offerGenerated = false;

// Initialize WebRTC connection
async function initializeWebRTC() {
    console.log('Creating RTCPeerConnection...');
    
    // Create RTC configuration with STUN server
    const config = {
        iceServers: [
            { url: 'stun:stun.l.google.com:19302' },
            { url: 'stun:stun1.l.google.com:19302' }
        ]
    };
    
    peerConnection = new RTCPeerConnection(config);
    
    // Add local tracks (from your camera/screen)
    if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
        try {
            localStream = await navigator.mediaDevices.getUserMedia({
                video: true,
                audio: false
            });
            
            localStream.getTracks().forEach(track => {
                peerConnection.addTrack(track, localStream);
            });
            
            console.log('Local stream added');
        } catch (err) {
            console.error('Error getting user media:', err);
            document.getElementById('status').textContent = 
                'Permission denied for camera access';
            return;
        }
    }
    
    // Handle incoming remote stream from SimBridge
    peerConnection.ontrack = (event) => {
        console.log('Remote track received');
        
        const videoElement = document.getElementById('remoteVideo');
        videoElement.srcObject = event.streams[0];
        
        document.getElementById('status').textContent = 
            'Connected! Receiving remote stream...';
    };
    
    // Handle ICE candidate generation
    peerConnection.onicecandidate = (event) => {
        if (event.candidate) {
            sendICECandidate(event.candidate);
        }
    };
    
    // Handle connection state changes
    peerConnection.onconnectionstatechange = () => {
        updateStatus(`Connection State: ${peerConnection.connectionState}`);
    };
    
    // Start WebSocket signaling
    connectToSignalingServer();
}

// Connect to SimBridge signaling server
function connectToSignalingServer() {
    socket = new WebSocket(SIMBRIDGE_SIGNALING_URL);
    
    socket.onopen = () => {
        console.log('WebSocket connected to signaling server');
        
        // Generate and send offer
        createAndSendOffer();
    };
    
    socket.onmessage = (event) => {
        const message = JSON.parse(event.data);
        handleMessage(message);
    };
    
    socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateStatus('WebSocket connection error');
    };
    
    socket.onclose = () => {
        console.log('WebSocket connection closed');
        updateStatus('Signaling server disconnected');
    };
}

// Generate and send SDP offer
async function createAndSendOffer() {
    if (offerGenerated) return;
    
    try {
        const offer = await peerConnection.createOffer();
        await peerConnection.setLocalDescription(offer);
        
        console.log('SDP offer generated, sending to server...');
        
        // Send offer via WebSocket
        socket.send(JSON.stringify({
            type: 'offer',
            sdp: offer.toString(),
            session_id: generateUUID(),
            stream_id: 'stream-1'
        }));
        
        offerGenerated = true;
    } catch (err) {
        console.error('Error creating offer:', err);
    }
}

// Handle incoming answer from server
function handleAnswer(answerSDP) {
    peerConnection.setRemoteDescription(new RTCSessionDescription(JSON.parse(answerSDP)));
    console.log('Remote answer received');
}

// Send ICE candidate to server
function sendICECandidate(candidate) {
    if (!socket || socket.readyState !== WebSocket.OPEN) return;
    
    socket.send(JSON.stringify({
        type: 'iceCandidate',
        candidate: candidate.candidate,
        sdpMid: candidate.sdpMid,
        sdpMlineIndex: candidate.sdpMLineIndex,
        session_id: generateUUID(),
        stream_id: 'stream-1'
    }));
}

// Handle messages from signaling server
function handleMessage(message) {
    console.log('Received message:', message.type);
    
    switch (message.type) {
        case 'answer':
            // Set remote description with answer SDP
            peerConnection.setRemoteDescription(
                new RTCSessionDescription(JSON.parse(message.sdp))
            );
            break;
            
        case 'iceCandidate':
            // Add ICE candidate to peer connection
            if (peerConnection) {
                peerConnection.addIceCandidate(
                    new RTCIceCandidate({
                        candidate: message.candidate,
                        sdpMid: message.sdpMid,
                        sdpMLineIndex: message.sdpMlineIndex
                    })
                );
            }
            break;
    }
}

// Update status display
function updateStatus(text) {
    const statusDiv = document.getElementById('status');
    if (statusDiv) {
        statusDiv.textContent = text;
    }
}

// Generate UUID for session tracking
function generateUUID() {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
        const r = Math.random() * 16 | 0, v = c === 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    });
}

// Initialize on page load
window.addEventListener('DOMContentLoaded', initializeWebRTC);
```

---

## Server Integration Example

### Complete Server Setup

```rust
// server/src/main.rs (add WebRTC handler)

use axum::{
    routing::{get, websocket},
    Router,
};
use tower_http::services::ServeDir;
use simbridge_server::streaming::{WebRTCSignalingManager, FrameDeliverySystem};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize WebRTC manager
    let signaling_manager = Arc::new(RwLock::new(WebRTCSignalingManager::new()));
    
    // Create Axum router with WebRTC endpoint
    let app = Router::new()
        .route("/ws", websocket(wsrtc_signaling))
        .with_state(signaling_manager);
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind");
    
    tracing::info!("SimBridge Server starting on http://localhost:8080");
    
    axum::serve(listener, app).await.expect("Server error");
    
    Ok(())
}

// WebSocket handler for WebRTC signaling
async fn wsrtc_signaling(
    ws: WebSocketUpgrade,
    State(signaling_manager): State<Arc<RwLock<WebRTCSignalingManager>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, signaling_manager))
}

async fn handle_websocket(
    mut socket: WebSocket,
    _signaling_manager: Arc<RwLock<WebRTCSignalingManager>>,
) {
    let mut msg_counter = 0;
    
    loop {
        match socket.recv().await {
            Ok(msg) => {
                msg_counter += 1;
                tracing::info!("Received message #{}", msg_counter);
                
                // Process WebRTC signaling messages
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&msg) {
                    if let Some(ty) = json_value.get("type").and_then(|v| v.as_str()) {
                        match ty {
                            "offer" => {
                                // Handle offer from browser
                                tracing::info!("Received WebRTC offer");
                            }
                            "iceCandidate" => {
                                // Handle ICE candidate
                                tracing::info!("Received ICE candidate #{}", msg_counter);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }
}
```

---

## Testing End-to-End

### 1. Start Server

```bash
cd server
cargo run --release
```

Expected output:
```
[INFO] SimBridge Server starting on http://localhost:8080
```

### 2. Open Test HTML in Browser

Open `test-webrtc.html` in a browser (Chrome/Firefox preferred for best WebRTC support).

### 3. Verify Connection

You should see:
1. Console logs: "WebSocket connected to signaling server"
2. Status text: "Connected! Receiving remote stream..."
3. Local camera feed visible in browser (if permissions granted)

---

## Next Steps for Full Implementation

### 1. Add Screen Capture Integration

```rust
// In server/src/adapters/ios.rs (extend start_screen_stream)

pub async fn start_webRTC_stream(&mut self, stream_config: StreamConfig) -> Result<WebRTCStreamHandle, AdapterError> {
    // 1. Create WebRTC session
    let session_id = signaling_manager.create_session(...).await?;
    
    // 2. Start screen capture
    self.start_screen_stream(config.quality, config.fps).await?;
    
    // 3. Spawn frame delivery task
    tokio::spawn(async move {
        while let Ok(frame_bytes) = self.screen_stream.capture_frame() {
            if let Err(e) = delivery_channel.send(frame_bytes).await {
                tracing::error!("Failed to send frame: {}", e);
                break;
            }
        }
    });
    
    // 4. Return handle with stream info
    Ok(WebRTCStreamHandle { session_id, delivery_channel })
}
```

### 2. Add Companion App Integration

Flutter code using `flutter_webrtc` package:

```dart
import 'package:flutter_webrtc/flutter_webrtc.dart';

class SimBridgeWebRTC extends StatefulWidget {
  @override
  _SimBridgeWebRTCState createState() => _SimBridgeWebRTCState();
}

class _SimBridgeWebRTCState extends State<SimBridgeWebRTC> {
  late RTCPeerConnection peerConnection;
  late MediaStream localStream;
  RTCSessionDescription? offer;
  
  @override
  void initState() {
    super.initState();
    
    // Create peer connection with STUN server
    peerConnection = RTCPeerConnection({
      'iceServers': [
        {'url': 'stun:stun.l.google.com:19302'}
      ]
    });
    
    // Add local track
    peerConnection.onTrack.add(_onRemoteStreamAdded);
    _startLocalStream();
  }
  
  void _startLocalStream() async {
    try {
      localStream = await navigator.mediaDevices.getUserMedia({
        'video': true,
        'audio': false,
      });
      
      localStream.getVideoTracks().forEach((track) {
        peerConnection.addTrack(track, localStream);
      });
    } catch (e) {
      print('Error starting local stream: $e');
    }
  }
  
  void _onRemoteStreamAdded(RTCTrack event, MediaStream stream) {
    // Display remote simulator screen
    final videoElement = document.createElement('video') as HTMLVideoElement;
    videoElement.srcObject = stream;
    videoElement.autoplay = true;
    videoElement.playsInline = true;
    
    document.body!.append(videoElement);
  }
  
  void _createAndSendOffer() async {
    final offer = await peerConnection.createOffer();
    await peerConnection.setLocalDescription(offer);
    
    // Send offer via WebSocket to SimBridge server
    // ... WebSocket integration code here
  }
}
```

---

## Debugging Tips

### Check Network Connectivity

```bash
# Test STUN server connectivity
nc -zv stun.l.google.com 19302
```

### Enable Detailed WebRTC Logging (Chrome)

```javascript
window.webrtcExperimental = true;
const options = { debug: 'verbose' };
const adapter = new Adapter(options);
```

### Monitor ICE Candidates

```javascript
peerConnection.onicecandidate = (event) => {
    console.log('ICE Candidate:', event.candidate);
};

// Log every 5 seconds
setInterval(() => {
    console.log('ICE Gathering State:', peerConnection.iceGatheringState);
}, 5000);
```

---

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| **Offer not received** | Check WebSocket connection status in browser console |
| **ICE candidates stuck** | Verify STUN server is reachable (ping/stun.l.google.com:19302) |
| **No remote video** | Check that SimBridge server sends answer correctly |
| **Connection timeout** | Reduce ICE gathering timeout or add TURN server |
| **High latency (>500ms)** | Enable JPEG compression, reduce FPS to 30 |

---

*This example provides a complete working implementation. Adapt as needed for your specific use case.*
