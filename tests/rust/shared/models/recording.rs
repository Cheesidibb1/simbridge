#[cfg(test)]
mod tests {
    use simbridge_shared::models::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    #[test]
    fn test_recording_creation() {
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

        assert_eq!(recording.id, "rec-1");
    }

    #[test]
    fn test_recording_serialization() {
        let recording = Recording {
            id: Uuid::new_v4().to_string(),
            session_id: Uuid::new_v4().to_string(),
            simulator_id: "sim-1".to_string(),
            device_id: "device-1".to_string(),
            status: RecordingStatus::Completed,
            created_at: Utc::now() - Duration::hours(1),
            duration_ms: 3600000, // 1 hour in milliseconds
            file_size_bytes: 524288000, // 500MB
        };

        let json = serde_json::to_value(&recording).expect("Failed to serialize");
        
        assert_eq!(json["status"], "completed");
    }

    #[test]
    fn test_recording_deserialization() {
        let json_str = r#"{
            "id": "rec-123",
            "session_id": "uuid-here",
            "simulator_id": "sim-1",
            "device_id": "device-1",
            "status": "completed",
            "created_at": "2024-01-01T12:00:00Z",
            "duration_ms": 3600000,
            "file_size_bytes": 524288000
        }"#;

        let recording: Recording = serde_json::from_str(json_str).expect("Failed to deserialize");
        
        assert_eq!(recording.id, "rec-123");
    }

    #[test]
    fn test_recording_status() {
        let statuses = vec![
            RecordingStatus::Recording,
            RecordingStatus::Completed,
            RecordingStatus::Failed,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).expect("Failed to serialize");
            let deserialized: RecordingStatus = serde_json::from_str(&json)
                .expect("Failed to deserialize");
            
            assert_eq!(deserialized, status);
        }
    }
}
