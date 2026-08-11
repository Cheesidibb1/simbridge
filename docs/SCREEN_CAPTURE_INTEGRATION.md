# Screen Capture Integration with WebRTC

**SimBridge v0.3.1 - Screen Capture Wiring Guide**  
**Date:** 2024-01-16

---

## Overview

This guide walks you through integrating the screen capture adapters (iOS simctl and Android ADB) with the WebRTC frame delivery system to enable real-time video streaming.

### What You'll Build:

```
Screen Capture Manager
├── iOS Simulator Adapter → simctl screenshot
├── Android Emulator Adapter → ADB screencap
│   └── JPEG encoder compression
└── Frame Delivery → WebRTC Signaling Manager
```

---

## Architecture Overview

### Complete Data Flow:

```
┌─────────────────────────────────────────────────────────────────┐
│                    SimBridge Screen Capture Pipeline            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ iOS Simulator    │    │ Android      │    │ WebRTC       │  │
│  │ Adapter (simctl) │    │ Emulator     │    │ Signaling    │  │
│  │                  │    │ Adapter (ADB)│    │ Manager      │  │
│  └────────┬─────────┘    └──────┬───────┘    └──────┬───────┘  │
│           │                     │                    │          │
│           ▼                     ▼                    ▼          │
│  capture_frame()           capture_frame()        SDP exchange │
│  (simctl screenshot)       (adb screencap)         negotiation │
│           │                     │                    │          │
│           └─────────────────────┼────────────────────┘          │
│                                 │                               │
│                                  ▼                              │
│                      ┌──────────────────────┐                  │
│                      │ FrameDeliverySystem  │                  │
│                      │ (Async mpsc channels)│                  │
│                      │ - Broadcast frames   │                  │
│                      │ - Backpressure       │                  │
│                      └───────────┬──────────┘                  │
│                                  │                              │
│                                  ▼                              │
│                      ┌──────────────────────┐                  │
│                      │ VideoEncoder         │                  │
│                      │ PNG → JPEG (70%      │                  │
│                      │ compression)         │                  │
│                      └───────────┬──────────┘                  │
│                                  │                              │
│                                  ▼                              │
│                      ┌──────────────────────┐                  │
│                      │ WebRTC Peer          │                  │
│                      │ Browser/Flutter App  │                  │
│                      └──────────────────────┘                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Steps

### Step 1: Understand the Existing Components

#### A. WebRTC Frame Delivery System

Location: `server/src/streaming/webrtc.rs`

```rust
pub struct FrameDeliverySystem {
    active_senders: Arc<RwLock<HashMap<String, mpsc::Sender<Vec<u8>>>>>,
}

// Methods available:
pub async fn get_or_create_sender(stream_id: String, capacity: usize) -> Result<Sender, Error>
pub async fn broadcast_frame(stream_id: String, frame_data: Vec<u8>) -> Result<usize, Error>
pub async fn stop_stream(stream_id: String)
```

**Purpose:** Broadcast frames to all connected WebRTC peers

#### B. Screen Capture Manager (New!)

Location: `server/src/streaming/screen_capture_manager.rs`

```rust
pub struct ScreenCaptureManager {
    streams: Arc<RwLock<HashMap<String, CaptureStreamInfo>>>,
}

// Methods available:
pub async fn start_capture_stream(simulator_id, stream_id, quality) -> Result<(), Error>
pub async fn stop_capture_stream(simulator_id) -> Result<(), Error>
pub async fn get_active_streams() -> Vec<CaptureStreamInfo>
```

**Purpose:** Coordinate multiple concurrent capture streams

#### C. iOS Adapter (Existing)

Location: `server/src/adapters/ios.rs`

```rust
pub struct IosScreenStream {
    device_id: String,
    stream_id: String,
    width: u32,
    height: u32,
}

// Available method:
pub fn capture_frame(&self) -> Result<Vec<u8>, AdapterError>
```

**Purpose:** Capture iOS simulator screenshots via simctl

#### D. Android Adapter (Existing)

Location: `server/src/adapters/android.rs`

```rust
pub struct AndroidScreenStream {
    device_id: String,
    stream_id: String,
    width: u32,
    height: u32,
}

// Available method:
pub fn capture_frame(&self) -> Result<Vec<u8>, AdapterError>
```

**Purpose:** Capture Android emulator screenshots via ADB

---

### Step 2: Update iOS Adapter to Integrate with Frame Delivery

#### File: `server/src/adapters/ios.rs`

**Current Implementation:**
```rust
pub struct IosSimulatorAdapter {
    device_id: String,
    device_name: String,
    connected: bool,
    is_physical_device: bool,
}

impl IosSimulatorAdapter {
    pub fn start_screen_stream(&mut self, quality: StreamQuality, fps: u32) -> Result<ScreenStream, AdapterError> {
        // ... validation and dimension detection
        Ok(ScreenStream { id, width, height })
    }
}
```

**New Implementation with WebRTC Integration:**

Add a new method that starts the capture stream:

```rust
use crate::streaming::screen_capture_manager::ScreenCaptureManager;
use crate::streaming::webrtc::FrameDeliverySystem;

impl IosSimulatorAdapter {
    /// Start screen streaming with WebRTC integration
    pub async fn start_screen_stream_webrtc(
        &mut self,
        stream_id: String,
        simulator_id: String,
        quality: StreamQuality,
        fps: u32,
        capture_manager: Arc<ScreenCaptureManager>,
        webrtc_delivery: Arc<FrameDeliverySystem>,
    ) -> Result<(), AdapterError> {
        
        // Validate device is connected and running
        if !self.is_connected() {
            return Err(AdapterError::NotConnected);
        }

        // Get screen dimensions for this simulator
        let (width, height) = self.get_simulator_dimensions()?;
        
        // Create a dedicated sender for this stream
        let (stream_tx, mut stream_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
        
        // Start async capture task
        tokio::spawn(async move {
            let device_id = self.device_id.clone();
            
            loop {
                tokio::select! {
                    // Capture frame every 33ms (30 FPS)
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000 / fps)) => {
                        match Self::capture_single_frame(&device_id, width, height).await {
                            Ok(frame_bytes) => {
                                // Compress with JPEG encoder if quality allows
                                let compressed_frame = match quality {
                                    StreamQuality::Low | StreamQuality::Medium => {
                                        // JPEG compression for bandwidth efficiency
                                        if let Ok(jpeg) = 
                                            crate::streaming::encoder::VideoEncoder::encode_png_to_jpeg(&frame_bytes) {
                                            jpeg
                                        } else {
                                            frame_bytes.clone()
                                        }
                                    },
                                    StreamQuality::High | StreamQuality::Ultra => {
                                        // Keep PNG for high quality, TODO: H.264 later
                                        frame_bytes
                                    },
                                };
                                
                                // Send to delivery system
                                if stream_tx.send(compressed_frame).await.is_err() {
                                    info!("Stream receiver closed for {}", device_id);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to capture frame from {}: {}", device_id, e);
                                
                                // Brief retry before continuing
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                    
                    // Handle stream receiver close
                    _ = stream_rx.recv() => {
                        info!("Stream closed for {}", device_id);
                        break;
                    }
                }
            }
            
            info!("Capture task stopped for {}", device_id);
        });

        info!("Started WebRTC streaming from iOS simulator: {}", simulator_id);
        Ok(())
    }

    /// Capture a single frame (simplified version)
    async fn capture_single_frame(
        &self,
        device_id: &str,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, AdapterError> {
        // Use simctl to capture screenshot
        let output = Command::new("xcrun")
            .args([
                "simctl", 
                "io", 
                device_id, 
                "screenshot", 
                "/tmp/simulator.png"
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!(
                "simctl screenshot failed: {}", e
            )))?;

        if !output.status.success() {
            return Err(AdapterError::CommandFailed(format!(
                "Screenshot command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Read the PNG file
        std::fs::read("/tmp/simulator.png")
            .map_err(|e| AdapterError::FileNotFound(
                "/tmp/simulator.png".to_string()
            ))
    }
}
```

**Key Changes:**
1. New method `start_screen_stream_webrtc()` accepts manager references
2. Spawns async task for continuous frame capture
3. Captures at target FPS (configurable)
4. JPEG compression applied based on quality setting
5. Frames sent through delivery channel to WebRTC system

---

### Step 3: Update Android Adapter Similarly

#### File: `server/src/adapters/android.rs`

Similar implementation using ADB commands:

```rust
use crate::streaming::screen_capture_manager::ScreenCaptureManager;
use crate::streaming::webrtc::FrameDeliverySystem;

impl AndroidEmulatorAdapter {
    /// Start screen streaming with WebRTC integration
    pub async fn start_screen_stream_webrtc(
        &mut self,
        stream_id: String,
        simulator_id: String,
        quality: StreamQuality,
        fps: u32,
        capture_manager: Arc<ScreenCaptureManager>,
        webrtc_delivery: Arc<FrameDeliverySystem>,
    ) -> Result<(), AdapterError> {
        
        // Validate device is connected
        if !self.is_connected() {
            return Err(AdapterError::NotConnected);
        }

        // Get screen dimensions from ADB
        let (width, height) = self.get_device_resolution()?;
        
        // Create dedicated sender for this stream
        let (stream_tx, mut stream_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
        
        // Start async capture task
        tokio::spawn(async move {
            let device_id = self.device_id.clone();
            
            loop {
                tokio::select! {
                    // Capture frame every 33ms (30 FPS)
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000 / fps)) => {
                        match Self::capture_android_frame(&device_id, width, height).await {
                            Ok(frame_bytes) => {
                                // Compress with JPEG encoder
                                let compressed_frame = match quality {
                                    StreamQuality::Low | StreamQuality::Medium => {
                                        if let Ok(jpeg) = 
                                            crate::streaming::encoder::VideoEncoder::encode_png_to_jpeg(&frame_bytes) {
                                            jpeg
                                        } else {
                                            frame_bytes.clone()
                                        }
                                    },
                                    StreamQuality::High | StreamQuality::Ultra => {
                                        // Keep PNG for high quality
                                        frame_bytes
                                    },
                                };
                                
                                // Send to delivery system
                                if stream_tx.send(compressed_frame).await.is_err() {
                                    info!("Stream receiver closed for {}", device_id);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to capture Android frame: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                    
                    // Handle stream close
                    _ = stream_rx.recv() => {
                        break;
                    }
                }
            }
        });

        info!("Started WebRTC streaming from Android emulator: {}", simulator_id);
        Ok(())
    }

    /// Capture frame using ADB screencap
    async fn capture_android_frame(
        &self,
        device_id: &str,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, AdapterError> {
        // Use ADB to copy screenshot from device
        let output = Command::new("adb")
            .args([
                "-s", device_id,
                "shell", "screencap", "/sdcard/screen.png"
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!(
                "ADB screencap failed: {}", e
            )))?;

        if !output.status.success() {
            return Err(AdapterError::CommandFailed(
                "Screenshot capture command failed on device".to_string()
            ));
        }

        // Pull the screenshot file to local system
        let pull_output = Command::new("adb")
            .args([
                "-s", device_id,
                "pull", "/sdcard/screen.png", "/tmp/android_screen.png"
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!(
                "ADB pull failed: {}", e
            )))?;

        if !pull_output.status.success() {
            return Err(AdapterError::CommandFailed(
                "Failed to pull screenshot from device".to_string()
            ));
        }

        // Read the PNG file
        std::fs::read("/tmp/android_screen.png")
            .map_err(|e| AdapterError::FileNotFound("/tmp/android_screen.png".to_string()))
    }
}
```

---

### Step 4: Wire Everything Together in Server Main

#### File: `server/src/main.rs`

**Current State:**
```rust
let webrtc_manager = Arc::new(WebRTCSignalingManager::new());
let rest_state = RestServerState::with_webrtc_manager(webrtc_manager);
```

**Updated with Full Integration:**

```rust
use simbridge_server::{
    streaming::{webrtc::WebRTCSignalingManager, screen_capture_manager::ScreenCaptureManager},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    logging::init_logging(&args.log_level);
    info!("Starting SimBridge Server v{}", env!("CARGO_PKG_VERSION"));

    // Initialize database
    let database = Database::new(std::path::Path::new(&args.database)).await?;
    database.migrate().await?;
    info!("Database initialized at {}", args.database);

    // Initialize core managers
    let session_manager = Arc::new(SessionManager::new(10));
    let auth_manager = Arc::new(AuthManager::new(5, 300));
    
    // Initialize plugin manager
    let plugin_context = PluginContext {
        config_dir: std::path::PathBuf::from(".config"),
        data_dir: std::path::PathBuf::from(".data"),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let plugin_manager = Arc::new(RwLock::new(PluginManager::new(plugin_context)));

    // Initialize WebRTC signaling manager
    let webrtc_manager = Arc::new(WebRTCSignalingManager::new());
    
    // Initialize screen capture manager with target FPS (30)
    let capture_manager = Arc::new(ScreenCaptureManager::new(30));
    
    // Initialize device adapters
    let mut ios_adapter = IosSimulatorAdapter::new(
        "ios-sim-1".to_string(),
        "iPhone 15 Pro".to_string()
    );
    
    let mut android_adapter = AndroidEmulatorAdapter::new(
        "android-emu-1".to_string(),
        "Pixel 7".to_string()
    );

    // Initialize WebSocket server state
    let ws_state = WebSocketServerState::new();
    
    // Initialize REST server state with WebRTC manager and capture manager
    let rest_state = RestServerState {
        sessions: Arc::new(RwLock::new(Vec::new())),
        android_adapters: Arc::new(RwLock::new(android_adapter.device_id.clone())),
        ios_adapters: Arc::new(RwLock::new(ios_adapter.device_id.clone())),
        webrtc_manager: Some(webrtc_manager.clone()),
        capture_manager: Some(capture_manager.clone()), // NEW!
    };

    // Create Axum router with WebRTC endpoints
    let app = create_router(rest_state.clone())
        .route("/ws", axum::routing::get({
            let ws_state = ws_state.clone();
            move |ws| websocket_handler(ws, ws_state)
        }))
        .with_state(rest_state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port))
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}:{}", args.host, args.port);
    info!("Screen capture manager initialized (30 FPS target)");
    info!("WebRTC signaling ready - waiting for connections...");

    axum::serve(listener, app)
        .await
        .expect("Server error");

    Ok(())
}
```

---

### Step 5: Implement REST Endpoint to Start Streaming

#### File: `server/src/networking/rest.rs`

Add new endpoint to start capture:

```rust
/// Request to start screen streaming
#[derive(Debug, Deserialize)]
struct StartStreamRequest {
    simulator_id: String,
    device_id: String,
    stream_id: String,
    quality: StreamQuality,
    fps: u32,
}

async fn start_stream(
    State(state): State<RestServerState>,
    Json(req): Json<StartStreamRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Starting stream for simulator {} (quality: {:?}, fps: {})", 
           req.simulator_id, req.quality, req.fps);

    // Get capture manager from state
    let capture_manager = match &state.capture_manager {
        Some(manager) => manager.clone(),
        None => return Err(StatusCode::ServiceUnavailable),
    };

    // Start capture stream
    match capture_manager.start_capture_stream(
        req.simulator_id.clone(),
        req.stream_id.clone(),
        req.quality,
    ).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "stream_started",
            "simulator_id": req.simulator_id,
            "stream_id": req.stream_id,
            "quality": req.quality.to_string(),
            "fps": req.fps,
        }))),
        Err(e) => Err(StatusCode::InternalServerError),
    }
}

/// Request to stop screen streaming
async fn stop_stream(
    State(state): State<RestServerState>,
    Path(simulator_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Stopping stream for simulator {}", simulator_id);

    let capture_manager = match &state.capture_manager {
        Some(manager) => manager.clone(),
        None => return Err(StatusCode::ServiceUnavailable),
    };

    match capture_manager.stop_capture_stream(&simulator_id).await {
        Ok(_) => Ok(Json(serde_json::json!({ "status": "stream_stopped" }))),
        Err(e) => Err(StatusCode::InternalServerError),
    }
}
```

Add to router:
```rust
pub fn create_router(rest_state: RestServerState) -> Router {
    Router::new()
        // ... existing routes
        
        // Streaming endpoints
        .route("/api/v1/streams", post(start_stream))
        .route("/api/v1/streams/:simulator_id", delete(stop_stream))
        
        // WebRTC endpoints
        .route("/api/v1/webrtc/sessions", post(create_webrtc_session))
        .route("/api/v1/webrtc/sessions/:id", get(get_webrtc_session))
        .route("/api/v1/webrtc/sessions/:id", delete(delete_webrtc_session))
}
```

---

### Step 6: Update iOS and Android Adapters in lib.rs

#### File: `server/src/lib.rs`

Make new methods public:

```rust
pub use adapters::{
    ios::IosSimulatorAdapter, 
    android::AndroidEmulatorAdapter,
    // Add any adapter traits if needed
};
```

---

## Testing the Integration

### Test 1: Start Capture Stream via REST API

```bash
curl -X POST http://localhost:8080/api/v1/streams \
  -H "Content-Type: application/json" \
  -d '{
    "simulator_id": "ios-sim-1",
    "device_id": "test-device-1",
    "stream_id": "screen-stream-1",
    "quality": "medium",
    "fps": 30
  }'

# Expected Response:
{
  "status": "stream_started",
  "simulator_id": "ios-sim-1",
  "stream_id": "screen-stream-1",
  "quality": "Medium",
  "fps": 30
}
```

### Test 2: Monitor Active Streams

```bash
curl http://localhost:8080/api/v1/streams/active
```

### Test 3: Stop Stream

```bash
curl -X DELETE http://localhost:8080/api/v1/streams/ios-sim-1
```

### Test 4: Browser Integration

1. Open test client: `http://localhost:9000/test-webrtc.html`
2. Create WebRTC offer (as before)
3. **New:** Send offer with stream data including frame delivery info
4. Verify video frames appear in browser (when capture is running)

---

## Performance Considerations

### Frame Capture Overhead:

| Platform | Command | Typical Duration | Bandwidth (uncompressed) | Bandwidth (JPEG @ 70% compression) |
|----------|---------|------------------|--------------------------|-------------------------------------|
| iOS Simulator | `simctl io screenshot` | ~50ms | ~30KB PNG | ~9KB JPEG |
| Android Emulator | `adb screencap + pull` | ~150ms | ~30KB PNG | ~9KB JPEG |

### FPS Targets by Quality:

| Quality | FPS | Frame Interval | Bitrate (JPEG) | Target Use Case |
|---------|-----|----------------|----------------|-----------------|
| Low | 15 fps | 67ms | ~135 KB/s | Battery saving mode |
| Medium | 30 fps | 33ms | ~270 KB/s | Standard testing |
| High | 60 fps | 17ms | ~540 KB/s | Performance testing |
| Ultra | 60 fps (PNG) | 17ms | ~1.8 MB/s | Visual QA (future: H.264) |

### Memory Usage:

- **Per Stream:** ~5-10MB (frame buffer + channel buffer)
- **Max Concurrent Streams:** Recommended ≤ 10
- **Total RAM Impact:** Linear with stream count

---

## Known Limitations & Future Work

### Current Limitations:

1. **JPEG Compression Only** ⚠️
   - Impact: Higher bandwidth than H.264/VP8
   - Mitigation: 70% savings vs PNG is good for testing
   - Future: Switch to H.264 with FFmpeg in Sprint 3

2. **No Adaptive Quality** ⚠️
   - Impact: Fixed FPS regardless of network conditions
   - Mitigation: Frame dropping if channel backs up
   - Future: Network-aware quality adjustment

3. **Single Stream Per Adapter** ⚠️
   - Impact: Can't stream multiple screens from same device
   - Mitigation: Create new adapter instances for each stream
   - Future: Multi-stream support per adapter

4. **No Frame Loss Recovery** ⚠️
   - Impact: Dropped frames cause visual stutter
   - Mitigation: Brief retry on failure
   - Future: Interpolation or GOP reconstruction

### Production Enhancements Needed:

- [ ] Add TURN server for NAT traversal in restrictive networks
- [ ] Implement H.264/VP8 encoding with FFmpeg integration
- [ ] Add frame interpolation to reduce jitter
- [ ] Implement connection quality monitoring and adaptive bitrate
- [ ] Add audio capture (microphone + device audio)

---

## Troubleshooting

### Issue 1: "simctl screenshot failed"

**Symptoms:** iOS adapter returns command error  
**Causes:**
- Simulator not running
- xcrun path issues
- Permission denied for /tmp/simulator.png

**Fixes:**
```bash
# Verify simulator is running
xcrun simctl list devices

# Check xcrun works
xcrun simctl booted

# Ensure /tmp writable (macOS specific)
chmod 777 /tmp
```

### Issue 2: "ADB pull failed"

**Symptoms:** Android adapter fails to pull screenshot  
**Causes:**
- ADB connection lost
- Storage full on device
- Permission issues

**Fixes:**
```bash
# Reconnect ADB device
adb kill-server
adb start-server
adb devices

# Check device storage
adb -s <device> shell df -h

# Grant screenshot permissions (Android 10+)
adb -s <device> shell grant android.permission.READ_EXTERNAL_STORAGE
```

### Issue 3: High CPU/Memory Usage

**Symptoms:** Server uses >50% RAM with one stream  
**Causes:**
- Frame buffering too large
- Unnecessary copying between buffers

**Fixes:**
1. Reduce frame buffer capacity in delivery system
2. Use `Arc<Vec<u8>>` instead of cloning frames
3. Implement frame reuse strategy

---

## Next Steps After Integration

### Sprint 1 Completion Checklist:
- [x] Screen capture manager created
- [ ] iOS adapter wired to WebRTC delivery ✅ (in progress)
- [ ] Android adapter wired to WebRTC delivery (pending)
- [ ] Frame compression working
- [ ] REST endpoints for stream control
- [ ] Integration testing with real devices

### After Sprint 1:
1. Test with actual iOS simulator and Android emulator
2. Measure end-to-end latency
3. Optimize frame rate based on network conditions
4. Add error recovery mechanisms
5. Implement companion app video display

---

## Success Criteria

The screen capture integration is **complete and functional** when:

- [ ] iOS simulator screen visible in browser (<500ms latency)
- [ ] Android emulator screen visible in browser (<500ms latency)
- [ ] Frame rate stable at target (30 FPS ±5%)
- [ ] JPEG compression applied correctly (~70% size reduction)
- [ ] Multiple concurrent streams work simultaneously
- [ ] Graceful shutdown on stop command

**Estimated Time:** 1 sprint (3-4 days of focused development)

---

## References

- [WebRTC Server Guide](WEbrtc_SERVER_GUIDE.md) - WebRTC signaling reference
- [SCREEN_CAPTURE_SUMMARY.md](SCREEN_CAPTURE_SUMMARY.md) - Screen capture overview
- [IMPLEMENTATION_PRIORITY.md](../IMPLEMENTATION_PRIORITY.md) - Feature priority roadmap

---

**Last Updated:** 2024-01-16  
**Author:** SimBridge Development Team  
**Status:** 🟡 Integration in Progress (Screen Capture Manager Ready, Adapter Wiring Pending)
<EOF>