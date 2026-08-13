// Session-related models

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::protocol::SessionStatus;

/// Represents an active session between a device and simulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub device_id: String,
    pub simulator_id: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub connected_at: Option<DateTime<Utc>>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub last_activity: DateTime<Utc>,
    pub stream_config: SessionStreamConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStreamConfig {
    pub quality: StreamQuality,
    pub fps: u32,
    pub audio_enabled: bool,
    pub video_codec: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamQuality {
    Low,
    Medium,
    High,
    Ultra,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: Uuid,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub frames_sent: u64,
    pub touch_events: u64,
    pub gps_updates: u64,
    pub notifications: u64,
    pub average_latency_ms: f64,
    pub uptime_seconds: u64,
}
