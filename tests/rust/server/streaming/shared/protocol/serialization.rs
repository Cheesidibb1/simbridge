#[cfg(test)]
mod tests {
    use simbridge_shared::protocol::*;

    #[test]
    fn test_message_type_round_trip() {
        let types = vec![
            MessageType::PairRequest,
            MessageType::AuthRequest,
            MessageType::SimulatorList,
            MessageType::ConnectSimulator,
            MessageType::DisconnectSimulator,
            MessageType::TouchEvent,
            MessageType::Gesture,
            MessageType::GpsUpdate,
            MessageType::DeviceButton,
            MessageType::ClipboardSync,
            MessageType::FileTransfer,
            MessageType::StartRecording,
            MessageType::StopRecording,
            MessageType::GetRecordings,
            MessageType::Ping,
            MessageType::PairResponse,
            MessageType::AuthResponse,
            MessageType::ScreenFrame,
            MessageType::Notification,
            MessageType::Error,
        ];

        for msg_type in types {
            let json_value = serde_json::json!({});
            let message = Message::new(msg_type.clone(), json_value);
            
            let serialized = serde_json::to_string(&message).expect("Failed to serialize");
            let deserialized: Message = serde_json::from_str(&serialized)
                .expect("Failed to deserialize");

            assert_eq!(deserialized.message_type, msg_type, 
                "Round-trip failed for {:?}", msg_type);
        }
    }

    #[test]
    fn test_payload_with_null_values() {
        let json_value = serde_json::json!({});
        
        let message = Message::new(MessageType::Ping, json_value.clone());
        
        // Test with None request_id (should be omitted in JSON)
        let serialized_no_id = serde_json::to_string(&message).expect("Failed to serialize");
        assert!(!serialized_no_id.contains("request_id"));

        // Test with Some request_id
        let message_with_id = message.with_request_id(uuid::Uuid::new_v4());
        let serialized_with_id = serde_json::to_string(&message_with_id).expect("Failed to serialize");
        assert!(serialized_with_id.contains("request_id"));
    }

    #[test]
    fn test_simulator_info_serialization() {
        let simulator_info = SimulatorInfo {
            id: "sim-123".to_string(),
            name: "Test Simulator".to_string(),
            platform: SimulatorPlatform::Ios,
            os_version: "17.0".to_string(),
            status: SimulatorStatus::Available,
            screen_size: ScreenSize {
                width: 390,
                height: 844,
                scale: 3.0,
            },
        };

        let json = serde_json::to_value(&simulator_info).expect("Failed to serialize");
        
        assert_eq!(json["id"], "sim-123");
        assert_eq!(json["platform"], "ios");
    }

    #[test]
    fn test_screen_frame_payload_serialization() {
        let now = chrono::Utc::now();
        
        let payload = ScreenFramePayload {
            simulator_id: "sim-1".to_string(),
            frame_data: "iVBORw0KGgo=".to_string(), // Base64 dummy data
            encoding: FrameEncoding::H264,
            width: 390,
            height: 844,
            timestamp: now,
        };

        let json = serde_json::to_value(&payload).expect("Failed to serialize");
        
        assert_eq!(json["simulator_id"], "sim-1");
        assert!(!json["frame_data"].is_null());
    }

    #[test]
    fn test_stream_config_serialization() {
        let config = StreamConfig {
            quality: StreamQuality::High,
            fps: 30,
            audio_enabled: true,
        };

        let json = serde_json::to_value(&config).expect("Failed to serialize");
        
        assert_eq!(json["quality"], "high");
        assert_eq!(json["fps"], 30);
        assert!(json["audio_enabled"].as_bool().unwrap());
    }

    #[test]
    fn test_connection_payload_with_various_configs() {
        let configs = vec![
            (StreamQuality::Low, 15),
            (StreamQuality::Medium, 30),
            (StreamQuality::High, 60),
            (StreamQuality::Ultra, 120),
        ];

        for (quality, fps) in configs {
            let payload = ConnectSimulatorPayload {
                simulator_id: "sim-1".to_string(),
                stream_config: StreamConfig {
                    quality,
                    fps,
                    audio_enabled: false,
                },
            };

            let json = serde_json::to_value(&payload).expect("Failed to serialize");
            
            assert_eq!(json["stream_config"]["quality"], quality.to_string());
            assert_eq!(json["stream_config"]["fps"], fps);
        }
    }
}
