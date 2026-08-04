// Simulator-related models

use serde::{Deserialize, Serialize};
use crate::protocol::{SimulatorPlatform, SimulatorStatus};

/// Represents a simulator/emulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulator {
    pub id: String,
    pub name: String,
    pub platform: SimulatorPlatform,
    pub os_version: String,
    pub status: SimulatorStatus,
    pub screen_size: ScreenSize,
    pub device_details: DeviceDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDetails {
    pub device_type: String,
    pub model: String,
    pub manufacturer: String,
    pub cpu_cores: u32,
    pub memory_mb: u32,
}

/// Simulator state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorState {
    pub id: String,
    pub is_running: bool,
    pub current_app: Option<String>,
    pub battery_level: Option<f64>,
    pub network_status: NetworkStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub is_connected: bool,
    pub network_type: String,
    pub signal_strength: Option<f64>,
}
