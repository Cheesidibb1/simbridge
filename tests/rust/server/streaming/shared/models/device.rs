#[cfg(test)]
mod tests {
    use simbridge_shared::models::*;
    use uuid::Uuid;

    #[test]
    fn test_device_creation() {
        let device = Device {
            id: "device-123".to_string(),
            name: "Test iPhone".to_string(),
            platform: DevicePlatform::Ios,
            is_physical: false,
            capabilities: vec![
                Capabilities::GpsStreaming,
                Capabilities::TouchControl,
                Capabilities::ClipboardSync,
            ],
        };

        assert_eq!(device.id, "device-123");
        assert_eq!(device.name, "Test iPhone");
        assert!(!device.is_physical);
    }

    #[test]
    fn test_device_serialization() {
        let device = Device {
            id: Uuid::new_v4().to_string(),
            name: "Pixel 7".to_string(),
            platform: DevicePlatform::Android,
            is_physical: true,
            capabilities: vec![
                Capabilities::GpsStreaming,
                Capabilities::TouchControl,
            ],
        };

        let json = serde_json::to_value(&device).expect("Failed to serialize");
        
        assert_eq!(json["id"], device.id);
        assert_eq!(json["platform"], "android");
        assert!(json["is_physical"].as_bool().unwrap());
    }

    #[test]
    fn test_device_deserialization() {
        let json_str = r#"{
            "id": "test-id-123",
            "name": "Test Device",
            "platform": "ios",
            "is_physical": false,
            "capabilities": ["gps_streaming", "touch_control"]
        }"#;

        let device: Device = serde_json::from_str(json_str).expect("Failed to deserialize");
        
        assert_eq!(device.name, "Test Device");
    }
}
