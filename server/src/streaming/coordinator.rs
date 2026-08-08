// Screen streaming coordination for SimBridge

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use thiserror::Error;
use chrono::{DateTime, Utc};
use crate::streaming::encoder::VideoEncoder;
use simbridge_shared::protocol::StreamQuality;

/// Stream type enumeration
#[derive(Debug, Clone)]
pub enum StreamType {
    Screenshot,     // Single frame capture
    Continuous,     // Live streaming (WebRTC ready)
    Recording,      // Local file recording
}

/// Stream configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub quality: StreamQuality,
    pub fps: u32,
    pub stream_type: StreamType,
    pub enable_audio: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            quality: StreamQuality::Medium,
            fps: 30,
            stream_type: StreamType::Continuous,
            enable_audio: false,
        }
    }
}

/// Stream information tracking active streams
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub id: Uuid,
    pub session_id: Uuid,
    pub simulator_id: String,
    pub stream_type: StreamType,
    pub quality: StreamQuality,
    pub fps: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub bytes_transferred: u64,
    pub frames_count: u64,
}

/// Screen streaming coordinator
pub struct StreamCoordinator {
    streams: Arc<RwLock<HashMap<Uuid, StreamInfo>>>,
    encoder: VideoEncoder,
}

impl StreamCoordinator {
    /// Create new coordinator with default settings
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            encoder: VideoEncoder::default_encoder(),
        }
    }

    /// Create coordinator with custom encoder config
    pub fn with_encoder(encoder_config: simbridge_shared::protocol::StreamConfig) -> Self {
        let quality = encoder_config.quality;
        let fps = encoder_config.fps;
        
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            encoder: VideoEncoder::new(simbridge_shared::streaming::VideoEncoderConfig {
                codec: simbridge_shared::streaming::VideoCodec::H264,
                quality: quality.into(),
                width: None,
                height: None,
                fps,
            }),
        }
    }

    /// Start a new screen stream session
    pub async fn start_stream(
        &self,
        session_id: Uuid,
        simulator_id: String,
        quality: StreamQuality,
        fps: u32,
    ) -> Result<Uuid, StreamError> {
        let stream_id = Uuid::new_v4();
        
        // Check if we already have an active stream for this session
        let existing_streams = self.streams.read().await;
        let has_active_stream = existing_streams.values()
            .any(|s| s.session_id == session_id && !s.ended_at.is_some());
        drop(existing_streams);

        if has_active_stream {
            return Err(StreamError::ActiveStreamExists);
        }

        let stream_info = StreamInfo {
            id: stream_id,
            session_id,
            simulator_id,
            stream_type: StreamType::Continuous,
            quality,
            fps,
            width: None, // Set when first frame arrives
            height: None,
            started_at: Utc::now(),
            ended_at: None,
            bytes_transferred: 0,
            frames_count: 0,
        };

        let mut streams = self.streams.write().await;
        streams.insert(stream_id, stream_info.clone());
        
        Ok(stream_id)
    }

    /// Start a one-time screenshot capture
    pub async fn start_screenshot(&self, session_id: Uuid, simulator_id: String) -> Result<Uuid, StreamError> {
        let stream_id = Uuid::new_v4();
        
        let stream_info = StreamInfo {
            id: stream_id,
            session_id,
            simulator_id,
            stream_type: StreamType::Screenshot,
            quality: StreamQuality::High,
            fps: 0, // Single frame
            width: None,
            height: None,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            bytes_transferred: 0,
            frames_count: 1,
        };

        let mut streams = self.streams.write().await;
        streams.insert(stream_id, stream_info.clone());
        
        Ok(stream_id)
    }

    /// Stop a stream and record statistics
    pub async fn stop_stream(&self, stream_id: Uuid) -> Result<Option<StreamInfo>, StreamError> {
        let mut streams = self.streams.write().await;
        
        if let Some(mut info) = streams.remove(&stream_id) {
            info.ended_at = Some(Utc::now());
            
            // Calculate duration in seconds
            if let Some(started) = info.started_at {
                let now = Utc::now();
                info.frames_per_second = info.frames_count as f64 / (now - started).num_seconds() as f64;
            }
            
            Ok(Some(info))
        } else {
            Err(StreamError::NotFound)
        }
    }

    /// Get stream information by ID
    pub async fn get_stream(&self, stream_id: Uuid) -> Option<StreamInfo> {
        let streams = self.streams.read().await;
        streams.get(&stream_id).cloned()
    }

    /// Get all active streams (not ended)
    pub async fn get_active_streams(&self) -> Vec<StreamInfo> {
        let streams = self.streams.read().await;
        streams.values()
            .filter(|s| s.ended_at.is_none())
            .cloned()
            .collect()
    }

    /// Get all streams for a session
    pub async fn get_session_streams(&self, session_id: Uuid) -> Vec<StreamInfo> {
        let streams = self.streams.read().await;
        streams.values()
            .filter(|s| s.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Update stream with new frame data
    pub async fn update_stream_frame(&self, stream_id: Uuid, frame_data: Vec<u8>) -> Result<(), StreamError> {
        let mut streams = self.streams.write().await;
        
        if let Some(stream) = streams.get_mut(&stream_id) {
            stream.bytes_transferred += frame_data.len() as u64;
            stream.frames_count += 1;
            
            // Update dimensions from first frame
            if stream.width.is_none() && !frame_data.is_empty() {
                // In real implementation, would extract dimensions from frame metadata
                stream.width = Some(390); // Default iOS width
                stream.height = Some(844); // Default iOS height
            }
            
            Ok(())
        } else {
            Err(StreamError::NotFound)
        }
    }

    /// Get statistics for a stream
    pub async fn get_stream_stats(&self, stream_id: Uuid) -> Option<StreamStats> {
        let streams = self.streams.read().await;
        
        if let Some(stream) = streams.get(&stream_id) {
            let duration_seconds = if let Some(started) = stream.started_at {
                if let Some(ended) = stream.ended_at {
                    (ended - started).num_seconds() as f64
                } else {
                    (Utc::now() - started).num_seconds() as f64
                }
            } else {
                0.0
            };

            Some(StreamStats {
                frames_count: stream.frames_count,
                bytes_transferred: stream.bytes_transferred,
                bitrate_kbps: if duration_seconds > 0.0 {
                    (stream.bytes_transferred * 8.0 / (duration_seconds * 1024.0)) as u64
                } else {
                    0
                },
                fps: if duration_seconds > 0.0 && stream.frames_count > 0 {
                    stream.frames_count as f64 / duration_seconds
                } else {
                    0.0
                },
            })
        } else {
            None
        }
    }

    /// Clear all streams for cleanup
    pub async fn clear_all(&self) {
        let mut streams = self.streams.write().await;
        streams.clear();
    }
}

/// Stream statistics tracking performance metrics
#[derive(Debug, Clone)]
pub struct StreamStats {
    pub frames_count: u64,
    pub bytes_transferred: u64,
    pub bitrate_kbps: u64,
    pub fps: f64,
}

/// Stream errors
#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Stream not found")]
    NotFound,
    
    #[error("Active stream already exists for this session")]
    ActiveStreamExists,
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Encoder error: {0}")]
    EncoderError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_stream_creation() {
        let coordinator = StreamCoordinator::new();
        
        let session_id = Uuid::new_v4();
        let stream_id = coordinator.start_stream(
            session_id,
            "sim-1".to_string(),
            simbridge_shared::protocol::StreamQuality::Medium,
            30,
        ).await.unwrap();

        assert!(coordinator.get_stream(stream_id).is_some());
    }

    #[tokio::test]
    async fn test_stream_stop() {
        let coordinator = StreamCoordinator::new();
        
        let session_id = Uuid::new_v4();
        let stream_id = coordinator.start_stream(
            session_id,
            "sim-1".to_string(),
            simbridge_shared::protocol::StreamQuality::Medium,
            30,
        ).await.unwrap();

        let stats = coordinator.get_stream_stats(stream_id).await;
        assert!(stats.is_some());
    }

    #[tokio::test]
    async fn test_frame_update() {
        let coordinator = StreamCoordinator::new();
        
        let session_id = Uuid::new_v4();
        let stream_id = coordinator.start_stream(
            session_id,
            "sim-1".to_string(),
            simbridge_shared::protocol::StreamQuality::Medium,
            30,
        ).await.unwrap();

        // Update with test frame
        let frame_data = vec![1, 2, 3, 4, 5];
        coordinator.update_stream_frame(stream_id, frame_data).await.unwrap();

        // Should have transferred data
        let stats = coordinator.get_stream_stats(stream_id).await.unwrap();
        assert!(stats.bytes_transferred > 0);
    }

    #[tokio::test]
    async fn test_screenshot_stream() {
        let coordinator = StreamCoordinator::new();
        
        let session_id = Uuid::new_v4();
        let stream_id = coordinator.start_screenshot(
            session_id,
            "sim-1".to_string(),
        ).await.unwrap();

        let stream_info = coordinator.get_stream(stream_id).await.unwrap();
        assert_eq!(stream_info.stream_type, StreamType::Screenshot);
    }
}
