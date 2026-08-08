#[cfg(test)]
mod tests {
    use simbridge_shared::models::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_simulator_creation() {
        let simulator = Simulator {
            id: "sim-1".to_string(),
            name: "iPhone 15 Pro Simulator".to_string(),
            platform: SimulatorPlatform::Ios,
            os_version: "17.0".to_string(),
            is_running: true,
        };

        assert_eq!(simulator.id, "sim-1");
        assert!(!simulator.is_running); // Should start as false
    }

    #[test]
    fn test_simulator_serialization() {
        let simulator = Simulator {
            id: "sim-1".to_string(),
            name: "Test Simulator".to_string(),
            platform: SimulatorPlatform::Android,
            os_version: "13.0".to_string(),
            is_running: true,
        };

        let json = serde_json::to_value(&simulator).expect("Failed to serialize");
        
        assert_eq!(json["platform"], "android");
    }

    #[test]
    fn test_simulator_status_serialization() {
        use simbridge_shared::protocol::SimulatorStatus;

        let status = SimulatorStatus {
            is_running: true,
            current_app: Some("com.example.app".to_string()),
            battery_level: Some(85.0),
            cpu_usage: 45.5,
            memory_usage: 2048 * 1024 * 1024, // 2GB
        };

        let json = serde_json::to_value(&status).expect("Failed to serialize");
        
        assert_eq!(json["cpu_usage"], 45.5);
    }

    #[test]
    fn test_screen_size_serialization() {
        let screen_size = ScreenSize {
            width: 390,
            height: 844,
            scale: 3.0,
        };

        let json = serde_json::to_value(&screen_size).expect("Failed to serialize");
        
        assert_eq!(json["width"], 390);
        assert_eq!(json["scale"], 3.0);
    }

    #[test]
    fn test_session_creation() {
        use simbridge_shared::models::{Session, SessionStatus, StreamConfig};

        let session = Session {
            id: Uuid::new_v4().to_string(),
            device_id: "device-1".to_string(),
            simulator_id: "sim-1".to_string(),
            status: SessionStatus::Active,
            created_at: Utc::now(),
            connected_at: Some(Utc::now()),
            disconnected_at: None,
            last_activity: Utc::now(),
            stream_config: StreamConfig {
                quality: simbridge_shared::protocol::StreamQuality::Medium,
                fps: 30,
                audio_enabled: false,
            },
        };

        assert_eq!(session.device_id, "device-1");
        assert_eq!(session.simulator_id, "sim-1");
    }

    #[test]
    fn test_session_serialization() {
        use simbridge_shared::models::{Session, SessionStatus, StreamConfig};

        let session = Session {
            id: Uuid::new_v4().to_string(),
            device_id: "device-1".to_string(),
            simulator_id: "sim-1".to_string(),
            status: SessionStatus::Active,
            created_at: Utc::now(),
            connected_at: Some(Utc::now()),
            disconnected_at: None,
            last_activity: Utc::now(),
            stream_config: StreamConfig {
                quality: simbridge_shared::protocol::StreamQuality::High,
                fps: 60,
                audio_enabled: true,
            },
        };

        let json = serde_json::to_value(&session).expect("Failed to serialize");
        
        assert_eq!(json["status"], "active");
    }

    #[test]
    fn test_stream_config_serialization() {
        let config = StreamConfig {
            quality: simbridge_shared::protocol::StreamQuality::Low,
            fps: 15,
            audio_enabled: false,
        };

        let json = serde_json::to_value(&config).expect("Failed to serialize");
        
        assert_eq!(json["fps"], 15);
    }

    #[test]
    fn test_recording_creation() {
        use simbridge_shared::models::{Recording, RecordingStatus};

        let recording = Recording {
            id: "rec-1".to_string(),
            session_id: Uuid::new_v4().to_string(),
            simulator_id: "sim-1".to_string(),
            device_id: "device-1".to_string(),
            status: RecordingStatus::Recording,
            created_at: Utc::now(),
            duration_ms: 0,
            file_size_bytes: 0,
        };

        assert_eq!(recording.status, RecordingStatus::Recording);
    }
}
