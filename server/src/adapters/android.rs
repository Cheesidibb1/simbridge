// Android Emulator adapter using ADB

use super::interface::{AdapterError, AdapterSimulatorStatus, ScreenStream, SimulatorAdapter};
use async_trait::async_trait;
use simbridge_shared::protocol::{
    DeviceButton, GestureData, GesturePayload, GpsLocation, Notification, StreamQuality,
    SwipeDirection, TouchEventPayload, TransferDirection,
};
use std::path::Path;
use std::process::Command;

/// Android screen stream handle using ADB commands
pub struct AndroidScreenStream {
    device_id: String,
    stream_id: String,
    width: u32,
    height: u32,
    is_recording: bool,
    recording_pid: Option<u32>,
}

impl AndroidScreenStream {
    pub fn new(stream_id: String, device_id: String, width: u32, height: u32) -> Self {
        Self {
            stream_id,
            device_id,
            width,
            height,
            is_recording: false,
            recording_pid: None,
        }
    }

    /// Capture a single screenshot frame using adb screencap
    pub fn capture_frame(&self) -> Result<Vec<u8>, AdapterError> {
        let screenshot_path = std::env::temp_dir().join("simbridge-android-screen.png");
        let screenshot_path_string = screenshot_path.to_string_lossy().into_owned();

        // Use ADB to copy screenshot from device to server
        let output = Command::new("adb")
            .args([
                "-s",
                &self.device_id,
                "shell",
                "screencap",
                "/sdcard/screen.png",
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("ADB screencap failed: {}", e)))?;

        if !output.status.success() {
            return Err(AdapterError::CommandFailed(
                "Screenshot capture command failed on device".to_string(),
            ));
        }

        // Pull the screenshot file to local system
        std::process::Command::new("adb")
            .args([
                "-s",
                &self.device_id,
                "pull",
                "/sdcard/screen.png",
                &screenshot_path_string,
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("ADB pull failed: {}", e)))?;

        // Read the screenshot file
        let frame = std::fs::read(&screenshot_path).map_err(|e| {
            AdapterError::FileNotFound(screenshot_path.to_string_lossy().into_owned())
        })?;
        let _ = std::fs::remove_file(screenshot_path);
        Ok(frame)
    }

    /// Start recording screen to video file
    pub fn start_recording(&mut self) -> Result<(), AdapterError> {
        if self.is_recording {
            return Err(AdapterError::NotSupported);
        }

        let output = Command::new("adb")
            .args([
                "-s",
                &self.device_id,
                "shell",
                "screenrecord",
                "/sdcard/android_recording.mp4",
            ])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("screenrecord failed: {}", e)))?;

        if !output.status.success() {
            return Err(AdapterError::CommandFailed(format!(
                "Screen recording failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        self.is_recording = true;
        Ok(())
    }

    /// Stop recording and return video path
    pub fn stop_recording(&mut self) -> Option<String> {
        if !self.is_recording {
            return None;
        }

        // Kill the screenrecord process
        if let Some(pid) = self.recording_pid.take() {
            let _ = Command::new("adb")
                .args(["-s", &self.device_id, "shell", "kill", &pid.to_string()])
                .output();
        }

        self.is_recording = false;
        Some("/sdcard/android_recording.mp4".to_string())
    }
}

/// Android Emulator adapter
#[derive(Clone)]
pub struct AndroidEmulatorAdapter {
    device_id: String,
    device_name: String,
    connected: bool,
    adb_path: String,
}

/// Resolve ADB without requiring the Android SDK to be on the server PATH.
pub fn resolve_adb_path() -> String {
    if let Ok(path) = std::env::var("SIMBRIDGE_ADB_PATH") {
        if !path.trim().is_empty() {
            return path;
        }
    }

    for variable in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(sdk_root) = std::env::var(variable) {
            let candidate = Path::new(&sdk_root)
                .join("platform-tools")
                .join(adb_executable());
            if candidate.is_file() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }

    #[cfg(windows)]
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let candidate = Path::new(&local_app_data)
            .join("Android")
            .join("Sdk")
            .join("platform-tools")
            .join(adb_executable());
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    "adb".to_string()
}

fn adb_executable() -> &'static str {
    if cfg!(windows) {
        "adb.exe"
    } else {
        "adb"
    }
}

impl AndroidEmulatorAdapter {
    pub fn new(device_id: String, device_name: String) -> Self {
        Self {
            device_id,
            device_name,
            connected: false,
            adb_path: resolve_adb_path(),
        }
    }

    pub fn with_adb_path(mut self, adb_path: String) -> Self {
        self.adb_path = adb_path;
        self
    }

    fn run_adb_command(&self, args: &[&str]) -> Result<String, AdapterError> {
        let output = Command::new(&self.adb_path)
            .args(["-s", &self.device_id])
            .args(args)
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("ADB command failed: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AdapterError::CommandFailed(format!(
                "ADB command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    fn run_adb_shell_command(&self, command: &str) -> Result<String, AdapterError> {
        self.run_adb_command(&["shell", command])
    }

    fn screen_size(&self) -> Result<(i32, i32), AdapterError> {
        let output = self.run_adb_command(&["shell", "wm", "size"])?;
        let size = output
            .lines()
            .rev()
            .find_map(|line| line.rsplit_once(':').map(|(_, value)| value.trim()))
            .unwrap_or(output.trim());
        let (width, height) = size.split_once('x').ok_or_else(|| {
            AdapterError::InvalidParameter("Could not determine screen size".into())
        })?;
        let width = width
            .parse::<i32>()
            .map_err(|_| AdapterError::InvalidParameter("Invalid screen width".into()))?;
        let height = height
            .parse::<i32>()
            .map_err(|_| AdapterError::InvalidParameter("Invalid screen height".into()))?;
        Ok((width, height))
    }
}

#[async_trait]
impl SimulatorAdapter for AndroidEmulatorAdapter {
    fn name(&self) -> &str {
        "android-emulator"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        // Query the selected serial directly. `adb -s <serial> devices` is not
        // a valid connectivity check because `devices` is a global command.
        let state = self.run_adb_command(&["get-state"])?;

        if state.trim() != "device" {
            return Err(AdapterError::ConnectionFailed(format!(
                "Device {} not found",
                self.device_id
            )));
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
        // Stream the PNG directly to avoid the extra device-file and pull round trip.
        let output = Command::new(&self.adb_path)
            .args(["-s", &self.device_id, "exec-out", "screencap", "-p"])
            .output()
            .map_err(|e| AdapterError::CommandFailed(format!("screencap failed: {}", e)))?;

        if !output.status.success() {
            return Err(AdapterError::CommandFailed(format!(
                "Screenshot command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        if output.stdout.is_empty() {
            return Err(AdapterError::CommandFailed(
                "Screenshot command returned no image data".to_string(),
            ));
        }

        Ok(output.stdout)
    }

    async fn start_screen_stream(
        &mut self,
        _quality: StreamQuality,
        _fps: u32,
    ) -> Result<ScreenStream, AdapterError> {
        // Validate device is connected and running
        if !self.is_connected() {
            return Err(AdapterError::NotConnected);
        }

        // Check device screen resolution
        let output = self.run_adb_command(&["shell", "wm", "size"])?;

        // Parse width and height from output (e.g., "1080x2400")
        let parts: Vec<&str> = output.split('x').collect();
        let (width, height) = if parts.len() == 2 {
            (
                parts[0].parse().unwrap_or(1080),
                parts[1].parse().unwrap_or(2400),
            )
        } else {
            (1080, 2400) // Default dimensions
        };

        Ok(ScreenStream {
            id: format!("android-stream-{}", uuid::Uuid::new_v4()),
            width,
            height,
        })
    }

    async fn stop_screen_stream(&mut self) -> Result<(), AdapterError> {
        // No cleanup needed for Android (commands are stateless)
        Ok(())
    }

    async fn send_touch_event(&mut self, event: TouchEventPayload) -> Result<(), AdapterError> {
        for touch in &event.touches {
            let x = touch.x as i32;
            let y = touch.y as i32;

            // Convert phase to string for matching
            let phase_str = format!("{:?}", touch.phase).to_lowercase();

            if phase_str.contains("began") || phase_str.contains("moved") {
                self.run_adb_shell_command(&format!("input tap {} {}", x, y))?;
            }
            // Ended and Cancelled phases don't need explicit commands
        }
        Ok(())
    }

    async fn send_gesture(&mut self, gesture: GesturePayload) -> Result<(), AdapterError> {
        match gesture.data {
            GestureData::Swipe {
                direction,
                distance,
            } => {
                if !distance.is_finite() || distance < 0.0 {
                    return Err(AdapterError::InvalidParameter(
                        "Swipe distance must be non-negative".into(),
                    ));
                }
                let (width, height) = self.screen_size()?;
                let (start_x, start_y) = (width / 2, height / 2);
                let distance = distance.round() as i32;
                let (end_x, end_y) = match direction {
                    SwipeDirection::Up => (start_x, (start_y - distance).max(0)),
                    SwipeDirection::Down => (start_x, (start_y + distance).min(height - 1)),
                    SwipeDirection::Left => ((start_x - distance).max(0), start_y),
                    SwipeDirection::Right => ((start_x + distance).min(width - 1), start_y),
                };
                self.run_adb_shell_command(&format!(
                    "input swipe {} {} {} {} 300",
                    start_x, start_y, end_x, end_y
                ))?;
            }
            GestureData::LongPress { x, y, duration_ms } => {
                self.run_adb_shell_command(&format!(
                    "input swipe {} {} {} {} {}",
                    x.round() as i32,
                    y.round() as i32,
                    x.round() as i32,
                    y.round() as i32,
                    duration_ms
                ))?;
            }
            GestureData::DoubleTap { x, y } => {
                let x = x.round() as i32;
                let y = y.round() as i32;
                self.run_adb_shell_command(&format!("input tap {} {}", x, y))?;
                self.run_adb_shell_command(&format!("input tap {} {}", x, y))?;
            }
            GestureData::Pinch { .. } | GestureData::Rotation { .. } => {
                return Err(AdapterError::NotSupported);
            }
        }
        Ok(())
    }

    async fn set_location(&mut self, location: GpsLocation) -> Result<(), AdapterError> {
        // Use adb geo fix to set location
        let lat = location.latitude;
        let lon = location.longitude;
        let alt = location.altitude.unwrap_or(0.0);

        self.run_adb_command(&[
            "geo",
            "fix",
            &lon.to_string(),
            &lat.to_string(),
            &alt.to_string(),
        ])?;
        Ok(())
    }

    async fn press_button(&mut self, button: DeviceButton) -> Result<(), AdapterError> {
        match button {
            DeviceButton::Home => {
                self.run_adb_shell_command("input keyevent KEYCODE_HOME")?;
            }
            DeviceButton::Back => {
                self.run_adb_shell_command("input keyevent KEYCODE_BACK")?;
            }
            DeviceButton::AppSwitcher => {
                self.run_adb_shell_command("input keyevent KEYCODE_APP_SWITCH")?;
            }
            DeviceButton::VolumeUp => {
                self.run_adb_shell_command("input keyevent KEYCODE_VOLUME_UP")?;
            }
            DeviceButton::VolumeDown => {
                self.run_adb_shell_command("input keyevent KEYCODE_VOLUME_DOWN")?;
            }
            DeviceButton::Mute => {
                self.run_adb_shell_command("input keyevent KEYCODE_MUTE")?;
            }
            DeviceButton::Lock => {
                self.run_adb_shell_command("input keyevent KEYCODE_POWER")?;
            }
            DeviceButton::Unlock => {
                // Turn screen on and unlock
                self.run_adb_shell_command("input keyevent KEYCODE_WAKEUP")?;
                self.run_adb_shell_command("input keyevent KEYCODE_MENU")?;
            }
            DeviceButton::Screenshot => {
                self.run_adb_shell_command("screencap -p /sdcard/screenshot.png")?;
            }
            _ => {
                return Err(AdapterError::NotSupported);
            }
        }
        Ok(())
    }

    async fn install_app(&mut self, _path: &Path) -> Result<(), AdapterError> {
        // TODO: Implement app installation using adb install
        Ok(())
    }

    async fn launch_app(&mut self, _bundle_id: &str) -> Result<(), AdapterError> {
        // TODO: Implement app launch using adb shell am start
        Ok(())
    }

    async fn terminate_app(&mut self, _bundle_id: &str) -> Result<(), AdapterError> {
        // TODO: Implement app termination using adb shell am force-stop
        Ok(())
    }

    async fn get_notifications(&mut self) -> Result<Vec<Notification>, AdapterError> {
        // TODO: Implement notification monitoring using adb shell dumpsys notification
        Ok(vec![])
    }

    async fn get_clipboard(&mut self) -> Result<String, AdapterError> {
        // TODO: Implement clipboard reading using adb shell
        Ok(String::new())
    }

    async fn set_clipboard(&mut self, _content: &str) -> Result<(), AdapterError> {
        // TODO: Implement clipboard writing using adb shell
        Ok(())
    }

    async fn transfer_file(
        &mut self,
        _direction: TransferDirection,
        _path: &Path,
    ) -> Result<Vec<u8>, AdapterError> {
        // TODO: Implement file transfer using adb push/pull
        Ok(vec![])
    }

    async fn restart(&mut self) -> Result<(), AdapterError> {
        // TODO: Implement emulator restart
        Ok(())
    }

    async fn get_status(&mut self) -> Result<AdapterSimulatorStatus, AdapterError> {
        // TODO: Get actual emulator status
        Ok(AdapterSimulatorStatus {
            is_running: true,
            current_app: None,
            battery_level: Some(100.0),
            cpu_usage: 8.0,
            memory_usage: 2 * 1024 * 1024 * 1024,
        })
    }
}
