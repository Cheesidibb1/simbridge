// Recording-related models

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Represents a recorded session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub duration_seconds: u64,
    pub file_size_bytes: u64,
    pub file_path: PathBuf,
    pub metadata: RecordingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    pub simulator_id: String,
    pub simulator_platform: String,
    pub device_id: String,
    pub event_count: u64,
    pub frame_count: u64,
    pub has_gps: bool,
    pub has_touch: bool,
    pub has_notifications: bool,
}

/// Recording event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: RecordingEventType,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingEventType {
    Touch,
    Gesture,
    Gps,
    Heading,
    Motion,
    Notification,
    Clipboard,
    DeviceButton,
    ScreenFrame,
}
