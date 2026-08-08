// Simulator adapter interface

use std::path::Path;
use async_trait::async_trait;
use simbridge_shared::protocol::{
    TouchEvent, Gesture, GpsLocation, DeviceButton, Notification,
    StreamQuality, TransferDirection,
};
use thiserror::Error;

/// Screen stream handle
pub struct ScreenStream {
    pub id: String,
    pub width: u32,
    pub height: u32,
}

impl ScreenStream {
    /// Capture a single frame and return as bytes (PNG/JPEG)
    async fn capture_frame(&self) -> Result<Vec<u8>, AdapterError> {
        Err(AdapterError::NotSupported)
    }

    /// Start continuous recording
    async fn start_recording(&mut self, output_path: &str) -> Result<(), AdapterError> {
        Err(AdapterError::NotSupported)
    }

    /// Stop recording
    async fn stop_recording(&mut self) -> Result<(), AdapterError> {
        Err(AdapterError::NotSupported)
    }
}

/// Simulator adapter trait
#[async_trait]
pub trait SimulatorAdapter: Send + Sync {
    /// Get adapter name
    fn name(&self) -> &str;

    /// Get adapter version
    fn version(&self) -> &str;

    /// Connect to the simulator
    async fn connect(&mut self) -> Result<(), AdapterError>;

    /// Disconnect from the simulator
    async fn disconnect(&mut self) -> Result<(), AdapterError>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Get simulator ID
    fn simulator_id(&self) -> &str;

    /// Get simulator name
    fn simulator_name(&self) -> &str;

    /// Start screen streaming
    async fn start_screen_stream(&mut self, quality: StreamQuality, fps: u32) -> Result<ScreenStream, AdapterError>;

    /// Stop screen streaming
    async fn stop_screen_stream(&mut self) -> Result<(), AdapterError>;

    /// Send touch event to simulator
    async fn send_touch_event(&mut self, event: TouchEvent) -> Result<(), AdapterError>;

    /// Send gesture to simulator
    async fn send_gesture(&mut self, gesture: Gesture) -> Result<(), AdapterError>;

    /// Set GPS location
    async fn set_location(&mut self, location: GpsLocation) -> Result<(), AdapterError>;

    /// Press device button
    async fn press_button(&mut self, button: DeviceButton) -> Result<(), AdapterError>;

    /// Install app
    async fn install_app(&mut self, path: &Path) -> Result<(), AdapterError>;

    /// Launch app
    async fn launch_app(&mut self, bundle_id: &str) -> Result<(), AdapterError>;

    /// Terminate app
    async fn terminate_app(&mut self, bundle_id: &str) -> Result<(), AdapterError>;

    /// Get notifications
    async fn get_notifications(&mut self) -> Result<Vec<Notification>, AdapterError>;

    /// Get clipboard content
    async fn get_clipboard(&mut self) -> Result<String, AdapterError>;

    /// Set clipboard content
    async fn set_clipboard(&mut self, content: &str) -> Result<(), AdapterError>;

    /// Transfer file
    async fn transfer_file(&mut self, direction: TransferDirection, path: &Path) -> Result<Vec<u8>, AdapterError>;

    /// Restart simulator
    async fn restart(&mut self) -> Result<(), AdapterError>;

    /// Get simulator status
    async fn get_status(&mut self) -> Result<SimulatorStatus, AdapterError>;
}

/// Simulator status
#[derive(Debug, Clone)]
pub struct SimulatorStatus {
    pub is_running: bool,
    pub current_app: Option<String>,
    pub battery_level: Option<f64>,
    pub cpu_usage: f64,
    pub memory_usage: u64,
}

/// Adapter errors
#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Not connected")]
    NotConnected,
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Command failed: {0}")]
    CommandFailed(String),
    
    #[error("Not supported")]
    NotSupported,
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("App not found: {0}")]
    AppNotFound(String),
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}
