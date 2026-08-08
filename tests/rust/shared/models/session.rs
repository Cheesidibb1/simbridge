#[cfg(test)]
mod tests {
    use simbridge_shared::models::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    #[test]
    fn test_session_stats_creation() {
        let stats = SessionStats {
            total_touch_events: 1500,
            total_gestures: 75,
            total_gps_updates: 3600, // 1 second at 1 Hz
            total_file_transfers: 25,
            total_notifications_received: 10,
        };

        assert_eq!(stats.total_touch_events, 1500);
    }

    #[test]
    fn test_session_creation_and_status() {
        use simbridge_shared::protocol::SessionStatus;

        let session = Session {
            id: Uuid::new_v4().to_string(),
            device_id: "device-1".to_string(),
            simulator_id: "sim-1".to_string(),
            status: SessionStatus::Active,
            created_at: Utc::now(),
            connected_at: Some(Utc::now()),
            disconnected_at: None,
            last_activity: Utc::now(),
            stream_config: simbridge_shared::protocol::StreamConfig {
                quality: simbridge_shared::protocol::StreamQuality::Medium,
                fps: 30,
                audio_enabled: false,
            },
        };

        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_with_stream_config() {
        let session = Session {
            id: Uuid::new_v4().to_string(),
            device_id: "device-1".to_string(),
            simulator_id: "sim-1".to_string(),
            status: SessionStatus::Active,
            created_at: Utc::now(),
            connected_at: Some(Utc::now()),
            disconnected_at: None,
            last_activity: Utc::now(),
            stream_config: simbridge_shared::protocol::StreamConfig {
                quality: simbridge_shared::protocol::StreamQuality::High,
                fps: 60,
                audio_enabled: true,
            },
        };

        assert_eq!(session.stream_config.fps, 60);
    }
}
