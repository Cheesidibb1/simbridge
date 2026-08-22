// iOS Device/Emulator adapter using libimobiledevice and simctl

use super::interface::{AdapterError, AdapterSimulatorStatus, ScreenStream, SimulatorAdapter};
use async_trait::async_trait;
use simbridge_shared::protocol::{
    DeviceButton, GesturePayload, GpsLocation, Notification, StreamQuality, TouchEventPayload,
    TransferDirection,
};
use std::path::Path;
use std::process::Command;

/// Screen stream handle for iOS simulator
pub struct IosScreenStream {
    stream_id: String,
    device_id: String,
    width: u32,
    height: u32,
    quality: StreamQuality,
    fps: u32,
    is_recording: bool,
    recording_path: Option<String>,
}

impl IosScreenStream {
    pub fn new(
        stream_id: String,
        device_id: String,
        width: u32,
        height: u32,
        quality: StreamQuality,
        fps: u32,
    ) -> Self {
        Self {
            stream_id,
            device_id,
            width,
            height,
            quality,
            fps,
            is_recording: false,
            recording_path: None,
        }
    }

    pub fn capture_frame(&self) -> Result<Vec<u8>, AdapterError> {
        // For iOS, use simctl to capture a screenshot
        let output = Command::new("xcrun")
            .args([
                "simctl",
                "io",
                &self.device_id,
                "screenshot",
                "/tmp/ios_frame.png",
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("screenshot failed: {}", e)))?;

        if !output.status.success() {
            return Err(AdapterError::CommandFailed(
                "screenshot command failed".to_string(),
            ));
        }

        // Read the captured frame
        let frame_data = std::fs::read("/tmp/ios_frame.png")
            .map_err(|e| AdapterError::FileNotFound(format!("Failed to read frame: {}", e)))?;

        Ok(frame_data)
    }

    pub fn start_recording(&mut self, path: &str) -> Result<(), AdapterError> {
        self.recording_path = Some(path.to_string());
        self.is_recording = true;
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<(), AdapterError> {
        self.is_recording = false;
        if let Some(path) = self.recording_path.take() {
            std::fs::remove_file(&path).map_err(|e| AdapterError::FileNotFound(path))?;
        }
        Ok(())
    }
}

/// iOS Device/Emulator adapter
#[derive(Clone)]
pub struct IosSimulatorAdapter {
    device_id: String,
    device_name: String,
    connected: bool,
    is_physical_device: bool,
    simctl_path: String,
    idevice_path: String,
    recording_path: Option<String>,
}

impl IosSimulatorAdapter {
    pub fn new(device_id: String, device_name: String) -> Self {
        // Determine if it's a physical device or simulator
        let is_physical_device = device_id.len() == 40; // UDID is 40 chars

        Self {
            device_id,
            device_name,
            connected: false,
            is_physical_device,
            simctl_path: "xcrun simctl".to_string(),
            idevice_path: "idevice".to_string(),
            recording_path: None,
        }
    }

    pub fn with_paths(mut self, simctl_path: String, idevice_path: String) -> Self {
        self.simctl_path = simctl_path;
        self.idevice_path = idevice_path;
        self
    }

    fn run_simctl_command(&self, args: &[&str]) -> Result<String, AdapterError> {
        let output = Command::new("xcrun")
            .args(["simctl", &self.device_id])
            .args(args)
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("simctl command failed: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AdapterError::CommandFailed(format!(
                "simctl command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    fn run_idevice_command(&self, args: &[&str]) -> Result<String, AdapterError> {
        let output = Command::new(&self.idevice_path)
            .args(["-u", &self.device_id])
            .args(args)
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("idevice command failed: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AdapterError::CommandFailed(format!(
                "idevice command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}

#[async_trait]
impl SimulatorAdapter for IosSimulatorAdapter {
    fn name(&self) -> &str {
        "ios-simulator"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        if self.is_physical_device {
            // Check if physical device is available using idevice_id
            let output = Command::new("idevice_id")
                .args(["-l"])
                .output()
                .map_err(|e| AdapterError::ConnectionFailed(format!("idevice_id failed: {}", e)))?;

            let devices = String::from_utf8_lossy(&output.stdout);
            if !devices.contains(&self.device_id) {
                return Err(AdapterError::ConnectionFailed(format!(
                    "iOS device {} not found",
                    self.device_id
                )));
            }
        } else {
            // Check if simulator is available using simctl
            let output = Command::new("xcrun")
                .args(["simctl", "list", "devices"])
                .output()
                .map_err(|e| {
                    AdapterError::ConnectionFailed(format!("simctl list failed: {}", e))
                })?;

            let devices = String::from_utf8_lossy(&output.stdout);
            if !devices.contains(&self.device_id) {
                return Err(AdapterError::ConnectionFailed(format!(
                    "iOS simulator {} not found",
                    self.device_id
                )));
            }
        }

        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), AdapterError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn simulator_id(&self) -> &str {
        &self.device_id
    }

    fn simulator_name(&self) -> &str {
        &self.device_name
    }

    async fn start_screenshot(&mut self) -> Result<Vec<u8>, AdapterError> {
        let output = Command::new("xcrun")
            .args([
                "simctl",
                "io",
                &self.device_id,
                "screenshot",
                "/tmp/simulator.png",
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("screenshot failed: {}", e)))?;

        if !output.status.success() {
            return Err(AdapterError::CommandFailed(format!(
                "Screenshot command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        std::fs::read("/tmp/simulator.png")
            .map_err(|e| AdapterError::FileNotFound("/tmp/simulator.png".to_string()))
    }

    async fn start_screen_stream(
        &mut self,
        quality: StreamQuality,
        fps: u32,
    ) -> Result<ScreenStream, AdapterError> {
        // Validate device is connected and running
        if !self.is_connected() {
            return Err(AdapterError::NotConnected);
        }

        // Get device status to ensure it's bootable
        let output = Command::new("xcrun")
            .args(["simctl", "booted"])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("simctl booted failed: {}", e)))?;

        if !output.status.success() {
            return Err(AdapterError::ConnectionFailed(
                "iOS simulator is not running. Please start it first.".to_string(),
            ));
        }

        // Determine screen size based on device type
        let (width, height) = (390, 844); // Default iPhone 15 Pro dimensions

        Ok(ScreenStream {
            id: format!("ios-stream-{}", uuid::Uuid::new_v4()),
            width,
            height,
        })
    }

    async fn stop_screen_stream(&mut self) -> Result<(), AdapterError> {
        // Stop any active recording
        if let Some(path) = self.recording_path.take() {
            std::fs::remove_file(&path).map_err(|e| AdapterError::FileNotFound(path))?;
        }
        Ok(())
    }

    async fn send_touch_event(&mut self, event: TouchEventPayload) -> Result<(), AdapterError> {
        if self.is_physical_device {
            // For physical devices, use idevicerestool or similar
            return Err(AdapterError::NotSupported);
        } else {
            // For simulators, use simctl io tap
            for touch in &event.touches {
                let x = touch.x;
                let y = touch.y;
                self.run_simctl_command(&["io", "tap", &x.to_string(), &y.to_string()])?;
            }
        }
        Ok(())
    }

    async fn send_gesture(&mut self, _gesture: GesturePayload) -> Result<(), AdapterError> {
        if self.is_physical_device {
            return Err(AdapterError::NotSupported);
        }
        Err(AdapterError::NotSupported)
    }

    async fn set_location(&mut self, location: GpsLocation) -> Result<(), AdapterError> {
        if self.is_physical_device {
            return Err(AdapterError::NotSupported);
        } else {
            // Use simctl location set
            let lat = location.latitude;
            let lon = location.longitude;
            self.run_simctl_command(&["location", "set", &lat.to_string(), &lon.to_string()])?;
        }
        Ok(())
    }

    async fn press_button(&mut self, button: DeviceButton) -> Result<(), AdapterError> {
        if self.is_physical_device {
            return Err(AdapterError::NotSupported);
        }

        match button {
            DeviceButton::Home => {
                self.run_simctl_command(&["ui", "home"])?;
            }
            DeviceButton::Lock => {
                self.run_simctl_command(&["shutdown"])?;
            }
            DeviceButton::Screenshot => {
                self.run_simctl_command(&["io", "screenshot", "/tmp/screenshot.png"])?;
            }
            DeviceButton::VolumeUp | DeviceButton::VolumeDown | DeviceButton::Mute => {
                // Volume controls on simulator
                self.run_simctl_command(&["ui", "volume", "up"])?;
            }
            _ => {
                return Err(AdapterError::NotSupported);
            }
        }
        Ok(())
    }

    async fn install_app(&mut self, path: &Path) -> Result<(), AdapterError> {
        if self.is_physical_device {
            // Use ideviceinstaller for physical devices
            let path_str = path
                .to_str()
                .ok_or(AdapterError::InvalidParameter("Invalid path".to_string()))?;
            self.run_idevice_command(&["installer", "install", path_str])?;
        } else {
            // Use simctl install for simulators
            let path_str = path
                .to_str()
                .ok_or(AdapterError::InvalidParameter("Invalid path".to_string()))?;
            self.run_simctl_command(&["install", path_str])?;
        }
        Ok(())
    }

    async fn launch_app(&mut self, bundle_id: &str) -> Result<(), AdapterError> {
        if self.is_physical_device {
            // Use idevicediagnostics or similar
            return Err(AdapterError::NotSupported);
        } else {
            self.run_simctl_command(&["launch", bundle_id])?;
        }
        Ok(())
    }

    async fn terminate_app(&mut self, bundle_id: &str) -> Result<(), AdapterError> {
        if self.is_physical_device {
            return Err(AdapterError::NotSupported);
        } else {
            self.run_simctl_command(&["terminate", bundle_id])?;
        }
        Ok(())
    }

    async fn get_notifications(&mut self) -> Result<Vec<Notification>, AdapterError> {
        // TODO: Implement notification monitoring
        Ok(vec![])
    }

    async fn get_clipboard(&mut self) -> Result<String, AdapterError> {
        // TODO: Implement clipboard reading
        Ok(String::new())
    }

    async fn set_clipboard(&mut self, _content: &str) -> Result<(), AdapterError> {
        // TODO: Implement clipboard writing
        Ok(())
    }

    async fn transfer_file(
        &mut self,
        _direction: TransferDirection,
        _path: &Path,
    ) -> Result<Vec<u8>, AdapterError> {
        // TODO: Implement file transfer
        Ok(vec![])
    }

    async fn restart(&mut self) -> Result<(), AdapterError> {
        // TODO: Implement simulator restart using simctl
        Ok(())
    }

    async fn get_status(&mut self) -> Result<AdapterSimulatorStatus, AdapterError> {
        // TODO: Get actual simulator status
        Ok(AdapterSimulatorStatus {
            is_running: true,
            current_app: None,
            battery_level: Some(100.0),
            cpu_usage: 5.0,
            memory_usage: 512 * 1024 * 1024,
        })
    }
}
