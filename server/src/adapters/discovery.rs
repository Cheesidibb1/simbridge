// Device discovery for Android and iOS

use std::process::Command;
use super::android::AndroidEmulatorAdapter;
use super::ios::IosSimulatorAdapter;
use super::interface::SimulatorAdapter;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Device discovery service
pub struct DeviceDiscovery {
    android_adapters: Arc<Mutex<Vec<AndroidEmulatorAdapter>>>,
    ios_adapters: Arc<Mutex<Vec<IosSimulatorAdapter>>>,
}

impl DeviceDiscovery {
    pub fn new() -> Self {
        Self {
            android_adapters: Arc::new(Mutex::new(Vec::new())),
            ios_adapters: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Discover Android emulators and devices
    pub async fn discover_android(&self) -> Result<Vec<AndroidEmulatorAdapter>, String> {
        let output = Command::new("adb")
            .args(["devices", "-l"])
            .output()
            .map_err(|e| format!("ADB command failed: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut adapters = Vec::new();

        for line in output_str.lines().skip(1) {
            if line.contains("device") && !line.contains("unauthorized") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let device_id = parts[0].to_string();
                    let device_name = parts.get(2).map(|s| s.to_string()).unwrap_or_else(|| "Android Device".to_string());
                    
                    // Clean up the device name (remove model: prefix)
                    let device_name = device_name.replace("model:", "");
                    
                    let adapter = AndroidEmulatorAdapter::new(device_id, device_name);
                    adapters.push(adapter);
                }
            }
        }

        let mut android_adapters = self.android_adapters.lock().await;
        *android_adapters = adapters.clone();
        
        Ok(adapters)
    }

    /// Discover iOS simulators and devices
    pub async fn discover_ios(&self) -> Result<Vec<IosSimulatorAdapter>, String> {
        let mut adapters = Vec::new();

        // Discover simulators
        let output = Command::new("xcrun")
            .args(["simctl", "list", "devices", "available"])
            .output()
            .map_err(|e| format!("simctl command failed: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.contains("(Booted)") || line.contains("--") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let device_id = parts.last().unwrap().trim_end_matches(')').to_string();
                    let device_name = parts.iter().take(parts.len() - 1).cloned().collect::<Vec<_>>().join(" ");
                    
                    let adapter = IosSimulatorAdapter::new(device_id, device_name);
                    adapters.push(adapter);
                }
            }
        }

        // Discover physical devices
        let output = Command::new("idevice_id")
            .args(["-l"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.len() == 40 { // UDID is 40 characters
                    let device_id = line.to_string();
                    let device_name = "iOS Device".to_string();
                    
                    let adapter = IosSimulatorAdapter::new(device_id, device_name);
                    adapters.push(adapter);
                }
            }
        }

        let mut ios_adapters = self.ios_adapters.lock().await;
        *ios_adapters = adapters.clone();
        
        Ok(adapters)
    }

    /// Get all discovered Android adapters
    pub async fn get_android_adapters(&self) -> Vec<AndroidEmulatorAdapter> {
        let android_adapters = self.android_adapters.lock().await;
        android_adapters.clone()
    }

    /// Get all discovered iOS adapters
    pub async fn get_ios_adapters(&self) -> Vec<IosSimulatorAdapter> {
        let ios_adapters = self.ios_adapters.lock().await;
        ios_adapters.clone()
    }

    /// Refresh all discoveries
    pub async fn refresh_all(&self) -> Result<(Vec<AndroidEmulatorAdapter>, Vec<IosSimulatorAdapter>), String> {
        let android = self.discover_android().await?;
        let ios = self.discover_ios().await?;
        Ok((android, ios))
    }
}

impl Default for DeviceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
