// Device-related models

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a paired device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub platform: String,
    pub os_version: String,
    pub paired_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_trusted: bool,
    pub public_key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Android,
    Ios,
    Desktop,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub supports_gps: bool,
    pub supports_heading: bool,
    pub supports_motion: bool,
    pub supports_notifications: bool,
    pub supports_clipboard: bool,
    pub supports_file_transfer: bool,
    pub max_simultaneous_streams: u32,
    pub supported_codecs: Vec<String>,
}
