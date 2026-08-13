// Screen capture manager for coordinating capture from multiple devices

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::Duration;
use tracing::{info, warn, error};
use simbridge_shared::protocol::StreamQuality;
use crate::streaming::encoder::VideoEncoder;

/// Active screen capture stream information
#[derive(Clone)]
pub struct CaptureStreamInfo {
    pub simulator_id: String,
    pub stream_id: String,
    pub is_active: bool,
    pub started_at: tokio::time::Instant,
    pub frame_count: u64,
    pub last_frame_time: tokio::time::Instant,
}

impl CaptureStreamInfo {
    pub fn new(simulator_id: String, stream_id: String) -> Self {
        Self {
            simulator_id,
            stream_id,
            is_active: false,
            started_at: tokio::time::Instant::now(),
            frame_count: 0,
            last_frame_time: tokio::time::Instant::now(),
        }
    }

    pub fn mark_active(&mut self) {
        self.is_active = true;
    }

    pub fn update_frame(&mut self) {
        self.frame_count += 1;
        self.last_frame_time = tokio::time::Instant::now();
    }
}

/// Screen capture coordinator managing multiple simultaneous captures
pub struct ScreenCaptureManager {
    /// Active capture streams
    streams: Arc<RwLock<HashMap<String, CaptureStreamInfo>>>,

    /// Channel for frame data delivery to WebRTC system
    webrtc_tx: Option<mpsc::Sender<Vec<u8>>>,

    /// Encoder configuration
    encoder: VideoEncoder,

    /// Frame rate target (FPS)
    target_fps: u32,
}

impl ScreenCaptureManager {
    /// Create new capture manager with default settings
    pub fn new(target_fps: u32) -> Self {
        let (webrtc_tx, webrtc_rx) = mpsc::channel::<Vec<u8>>(100); // Buffer for 100 frames

        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            webrtc_tx: Some(webrtc_tx),
            encoder: VideoEncoder::default_encoder(),
            target_fps,
        }
    }

    /// Start background capture task for a simulator
    pub async fn start_capture_stream(
        &self,
        simulator_id: String,
        stream_id: String,
        quality: StreamQuality,
    ) -> Result<(), CaptureManagerError> {
        let mut streams = self.streams.write().await;

        // Check if already capturing for this simulator
        if streams.contains_key(&simulator_id) {
            warn!("Already capturing stream for simulator {}", simulator_id);
            return Ok(());
        }

        let capture_info = CaptureStreamInfo::new(simulator_id.clone(), stream_id.clone());
        streams.insert(simulator_id.clone(), capture_info);

        // Determine frame interval based on target FPS
        let frame_interval = Duration::from_millis(1000 / self.target_fps as u64);

        let webrtc_tx = self.webrtc_tx.clone();
        let streams_arc = self.streams.clone();

        // Start async capture task
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Capture frame and deliver (every frame_interval)
                tokio::time::sleep(frame_interval).await;

                let streams_ref = streams_arc.read().await;
                if let Err(e) = capture_frame_and_deliver(
                    &simulator_id,
                    &stream_id,
                    quality,
                    &streams_ref,
                    webrtc_tx.as_ref(),
                ).await {
                    error!("Failed to capture frame: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Stop capture stream for a simulator
    pub async fn stop_capture_stream(&self, simulator_id: &str) -> Result<(), CaptureManagerError> {
        let mut streams = self.streams.write().await;

        if streams.remove(simulator_id).is_some() {
            info!("Stopped capture stream for simulator {}", simulator_id);
            Ok(())
        } else {
            warn!("No active capture stream found for simulator {}", simulator_id);
            Err(CaptureManagerError::StreamNotFound(simulator_id.to_string()))
        }
    }

    /// Gracefully shut down all capture streams
    pub async fn shutdown_all(&self) {
        // Clear all streams
        self.streams.write().await.clear();
        info!("All capture streams shut down");
    }

    /// Get all active capture streams
    pub async fn get_active_streams(&self) -> Vec<CaptureStreamInfo> {
        let streams = self.streams.read().await;
        streams.values()
            .filter(|info| info.is_active)
            .cloned()
            .collect()
    }

    /// Get capture stream info by simulator ID
    pub async fn get_stream_info(&self, simulator_id: &str) -> Option<CaptureStreamInfo> {
        let streams = self.streams.read().await;
        streams.get(simulator_id).cloned()
    }

    /// Get statistics for all active streams
    pub async fn get_statistics(&self) -> Vec<CaptureStreamStats> {
        let mut streams = self.streams.read().await;
        let now = tokio::time::Instant::now();
        
        streams.values()
            .filter(|info| info.is_active)
            .map(|info| CaptureStreamStats {
                simulator_id: info.simulator_id.clone(),
                stream_id: info.stream_id.clone(),
                is_active: true,
                duration_ms: now.duration_since(info.started_at).as_millis() as u64,
                frame_count: info.frame_count,
                avg_fps: {
                    let duration_sec = now.duration_since(info.started_at).as_secs_f64();
                    if duration_sec > 0.0 {
                        (info.frame_count as f64 / duration_sec).round() as u32
                    } else {
                        0
                    }
                },
            })
            .collect()
    }

    /// Get encoder with configured quality
    fn get_encoder(&self, quality: StreamQuality) -> VideoEncoder {
        let codec = match quality {
            StreamQuality::Low => crate::streaming::encoder::VideoCodec::JPEG,
            StreamQuality::Medium => crate::streaming::encoder::VideoCodec::JPEG,
            StreamQuality::High => crate::streaming::encoder::VideoCodec::H264,
            StreamQuality::Ultra => crate::streaming::encoder::VideoCodec::H264,
        };

        VideoEncoder::new(crate::streaming::encoder::VideoEncoderConfig {
            codec,
            quality: crate::streaming::encoder::EncoderQuality::from(quality),
            width: None,
            height: None,
            fps: self.target_fps,
        })
    }
}

/// Statistics for a capture stream
#[derive(Debug, Clone)]
pub struct CaptureStreamStats {
    pub simulator_id: String,
    pub stream_id: String,
    pub is_active: bool,
    pub duration_ms: u64,
    pub frame_count: u64,
    pub avg_fps: u32,
}

/// Errors from the capture manager
#[derive(Debug)]
pub enum CaptureManagerError {
    StreamAlreadyActive(String),
    StreamNotFound(String),
    CaptureFailed(String),
    ShutdownInProgress(String),
}

impl std::fmt::Display for CaptureManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureManagerError::StreamAlreadyActive(id) => write!(f, "Stream already active for simulator: {}", id),
            CaptureManagerError::StreamNotFound(id) => write!(f, "No active stream found for simulator: {}", id),
            CaptureManagerError::CaptureFailed(msg) => write!(f, "Capture failed: {}", msg),
            CaptureManagerError::ShutdownInProgress(id) => write!(f, "Shutdown in progress for: {}", id),
        }
    }
}

impl std::error::Error for CaptureManagerError {}

/// Async function to capture a frame and deliver it through WebRTC
async fn capture_frame_and_deliver(
    simulator_id: &str,
    stream_id: &str,
    quality: StreamQuality,
    streams: &HashMap<String, CaptureStreamInfo>,
    webrtc_tx: Option<&mpsc::Sender<Vec<u8>>>,
) -> Result<(), CaptureManagerError> {
    // Get stream info for stats update
    let mut stream_info = match streams.get(simulator_id) {
        Some(info) => info.clone(),
        None => return Err(CaptureManagerError::StreamNotFound(simulator_id.to_string())),
    };

    // Update frame count
    stream_info.update_frame();

    // Try to capture frame (adapter-specific logic here)
    // This is a placeholder - actual implementation in adapter modules
    let frame_data = match simulate_frame_capture().await {
        Ok(data) => data,
        Err(e) => {
            error!("Frame capture failed: {}", e);
            return Err(CaptureManagerError::CaptureFailed(format!("{} for simulator {}", e, simulator_id)));
        }
    };

    // Deliver through WebRTC frame delivery system
    if let Some(tx) = webrtc_tx {
        // Attempt to send the raw frame data
        // In production, this would be encoded first
        match tx.send(frame_data).await {
            Ok(_) => info!("Frame sent for stream {}", stream_id),
            Err(e) => warn!("Failed to send frame: {}", e),
        }
    }

    Ok(())
}

/// Simulate frame capture (placeholder for actual adapter implementation)
async fn simulate_frame_capture() -> Result<Vec<u8>, String> {
    // This will be replaced by actual adapter code that calls:
    // - IosSimulatorAdapter.start_screen_stream().await?
    // - android_screen_stream.capture_frame().await?
    
    // Return a simple test pattern (red square on black background)
    let width = 1920;
    let height = 1080;
    let mut frame = vec![0u8; (width * height * 4) as usize]; // RGBA
    
    // Draw red square in center
    for y in (height / 2 - 50)..(height / 2 + 50) {
        for x in (width / 2 - 50)..(width / 2 + 50) {
            let idx = (y * width + x) * 4;
            frame[idx] = 255;     // R
            frame[idx + 1] = 0;   // G
            frame[idx + 2] = 0;   // B
            frame[idx + 3] = 255; // A
        }
    }
    
    Ok(frame)
}