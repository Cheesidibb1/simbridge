// Screen streaming coordinator

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use thiserror::Error;

/// Stream coordinator
pub struct StreamCoordinator {
    streams: Arc<RwLock<HashMap<Uuid, StreamInfo>>>,
}

#[derive(Debug, Clone)]
struct StreamInfo {
    id: Uuid,
    session_id: Uuid,
    simulator_id: String,
    quality: String,
    fps: u32,
    started_at: chrono::DateTime<chrono::Utc>,
}

impl StreamCoordinator {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new stream
    pub async fn start_stream(
        &self,
        session_id: Uuid,
        simulator_id: String,
        quality: String,
        fps: u32,
    ) -> Result<Uuid, StreamError> {
        let stream_id = Uuid::new_v4();
        
        let stream_info = StreamInfo {
            id: stream_id,
            session_id,
            simulator_id,
            quality,
            fps,
            started_at: chrono::Utc::now(),
        };

        let mut streams = self.streams.write().await;
        streams.insert(stream_id, stream_info);
        
        Ok(stream_id)
    }

    /// Stop a stream
    pub async fn stop_stream(&self, stream_id: Uuid) -> Result<(), StreamError> {
        let mut streams = self.streams.write().await;
        streams.remove(&stream_id)
            .map(|_| ())
            .ok_or(StreamError::NotFound)
    }

    /// Get stream info
    pub async fn get_stream(&self, stream_id: Uuid) -> Option<StreamInfo> {
        let streams = self.streams.read().await;
        streams.get(&stream_id).cloned()
    }

    /// Get all streams
    pub async fn get_all_streams(&self) -> Vec<StreamInfo> {
        let streams = self.streams.read().await;
        streams.values().cloned().collect()
    }

    /// Get streams for a session
    pub async fn get_session_streams(&self, session_id: Uuid) -> Vec<StreamInfo> {
        let streams = self.streams.read().await;
        streams
            .values()
            .filter(|s| s.session_id == session_id)
            .cloned()
            .collect()
    }
}

/// Stream errors
#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Stream not found")]
    NotFound,
    
    #[error("Stream already exists")]
    AlreadyExists,
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Encoder error: {0}")]
    EncoderError(String),
}
