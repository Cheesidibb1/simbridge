// Session recorder

use simbridge_shared::protocol::Message;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session recorder
pub struct SessionRecorder {
    recording_id: Uuid,
    session_id: Uuid,
    output_path: PathBuf,
    events: Arc<RwLock<Vec<RecordingEvent>>>,
    is_recording: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RecordingEvent {
    timestamp: chrono::DateTime<chrono::Utc>,
    message_type: String,
    data: serde_json::Value,
}

impl SessionRecorder {
    pub fn new(session_id: Uuid, output_path: PathBuf) -> Self {
        Self {
            recording_id: Uuid::new_v4(),
            session_id,
            output_path,
            events: Arc::new(RwLock::new(Vec::new())),
            is_recording: Arc::new(RwLock::new(false)),
        }
    }

    /// Start recording
    pub async fn start(&self) -> Result<(), RecordingError> {
        let mut is_recording = self.is_recording.write().await;
        *is_recording = true;
        Ok(())
    }

    /// Stop recording
    pub async fn stop(&self) -> Result<(), RecordingError> {
        let mut is_recording = self.is_recording.write().await;
        *is_recording = false;

        // Save recording to file
        self.save_to_file().await?;

        Ok(())
    }

    /// Record a message
    pub async fn record_message(&self, message: &Message) -> Result<(), RecordingError> {
        let is_recording = self.is_recording.read().await;
        if !*is_recording {
            return Ok(());
        }
        drop(is_recording);

        let event = RecordingEvent {
            timestamp: chrono::Utc::now(),
            message_type: format!("{:?}", message.message_type),
            data: serde_json::to_value(message)
                .map_err(|e| RecordingError::SerializationError(e.to_string()))?,
        };

        let mut events = self.events.write().await;
        events.push(event);

        Ok(())
    }

    /// Check if recording
    pub async fn is_recording(&self) -> bool {
        let is_recording = self.is_recording.read().await;
        *is_recording
    }

    /// Get event count
    pub async fn event_count(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }

    /// Save recording to file
    async fn save_to_file(&self) -> Result<(), RecordingError> {
        let events = self.events.read().await;

        let recording_data = serde_json::to_string_pretty(&*events)
            .map_err(|e| RecordingError::SerializationError(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| RecordingError::IoError(e.to_string()))?;
        }

        std::fs::write(&self.output_path, recording_data)
            .map_err(|e| RecordingError::IoError(e.to_string()))?;

        Ok(())
    }
}

/// Recording errors
#[derive(Debug, Error)]
pub enum RecordingError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Not recording")]
    NotRecording,

    #[error("Recording already in progress")]
    AlreadyRecording,
}
