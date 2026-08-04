// Configuration models

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
    pub max_sessions: u32,
    pub session_timeout_seconds: u64,
    pub database_path: PathBuf,
    pub recordings_path: PathBuf,
    pub plugins_path: PathBuf,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            max_sessions: 10,
            session_timeout_seconds: 3600,
            database_path: PathBuf::from("simbridge.db"),
            recordings_path: PathBuf::from("recordings"),
            plugins_path: PathBuf::from("plugins"),
            log_level: "info".to_string(),
        }
    }
}

/// Stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub default_quality: String,
    pub default_fps: u32,
    pub max_bitrate: u32,
    pub min_bitrate: u32,
    pub adaptive_bitrate: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            default_quality: "medium".to_string(),
            default_fps: 30,
            max_bitrate: 5000,
            min_bitrate: 500,
            adaptive_bitrate: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_auth: bool,
    pub token_expiry_hours: u64,
    pub max_failed_attempts: u32,
    pub lockout_duration_seconds: u64,
    pub rate_limit_requests_per_minute: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: true,
            token_expiry_hours: 24,
            max_failed_attempts: 5,
            lockout_duration_seconds: 300,
            rate_limit_requests_per_minute: 60,
        }
    }
}
