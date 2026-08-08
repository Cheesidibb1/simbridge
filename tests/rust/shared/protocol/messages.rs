use simbridge_shared::protocol::*;
use chrono::{Duration, Utc};

#[test]
fn test_message_creation() {
    let payload = serde_json::json!({
        "key": "value",
        "number": 42
    });

    let message = Message::new(MessageType::Ping, payload);

    assert_eq!(message.message_type, MessageType::Ping);
    assert_eq!(message.version, PROTOCOL_VERSION);
    assert_eq!(message.request_id, None);
}

#[test]
fn test_message_with_request_id() {
    let request_id = uuid::Uuid::new_v4();
    let payload = serde_json::json!({});

    let message = Message::new(MessageType::Ping, payload)
        .with_request_id(request_id);

    assert_eq!(message.request_id, Some(request_id));
}

#[test]
fn test_message_serialization() {
    let request_id = uuid::Uuid::new_v4();
    let now = Utc::now();
    let payload = serde_json::json!({
        "simulator_id": "test-sim-1",
        "quality": "high"
    });

    let message = Message {
        message_type: MessageType::ConnectSimulator,
        version: PROTOCOL_VERSION,
        timestamp: now,
        request_id: Some(request_id),
        payload: payload.clone(),
    };

    // Test JSON serialization
    let json_str = serde_json::to_string(&message).expect("Failed to serialize");
    let parsed: Message = serde_json::from_str(&json_str)
        .expect("Failed to deserialize");

    assert_eq!(parsed.message_type, MessageType::ConnectSimulator);
    assert_eq!(parsed.payload, payload);
}

#[test]
fn test_message_type_serialization() {
    // Test client → server messages
    assert_eq!(MessageType::PairRequest.to_string(), "pair_request");
    assert_eq!(MessageType::TouchEvent.to_string(), "touch_event");
    assert_eq!(MessageType::GpsUpdate.to_string(), "gps_update");

    // Test server → client messages  
    assert_eq!(MessageType::AuthResponse.to_string(), "auth_response");
    assert_eq!(MessageType::ScreenFrame.to_string(), "screen_frame");
    assert_eq!(MessageType::PairResponse.to_string(), "pair_response");

    // Test bidirectional messages
    assert_eq!(MessageType::SettingsUpdate.to_string(), "settings_update");
}

#[test]
fn test_pair_request_payload() {
    let payload = PairRequestPayload {
        device_id: "abc123def456".to_string(),
        device_name: "Test iPhone 15 Pro".to_string(),
        device_type: DeviceType::IosPhysical,
        public_key: "test-public-key-123456789".to_string(),
    };

    let json = serde_json::to_value(&payload).expect("Failed to serialize");
    
    assert_eq!(json["device_id"], "abc123def456");
    assert_eq!(json["device_name"], "Test iPhone 15 Pro");
}

#[test]
fn test_gps_location_serialization() {
    let location = GpsLocation {
        latitude: 37.7749,
        longitude: -122.4194,
        altitude: Some(100.0),
        accuracy: Some(5.0),
        speed: Some(45.5),
        heading: Some(270.0),
        timestamp: Utc::now(),
    };

    let payload = GpsUpdatePayload {
        simulator_id: "sim-1".to_string(),
        location: location.clone(),
    };

    let json = serde_json::to_value(&payload).expect("Failed to serialize");
    
    assert_eq!(json["simulator_id"], "sim-1");
    assert!((json["latitude"] as f64 - 37.7749).abs() < 0.001);
    assert!((json["longitude"] as f64 - (-122.4194)).abs() < 0.001);
}

#[test]
fn test_touch_event_payload() {
    let payload = TouchEventPayload {
        simulator_id: "sim-1".to_string(),
        touches: vec![
            Touch {
                id: 1,
                x: 100.0,
                y: 200.0,
                phase: TouchPhase::Began,
                force: Some(0.8),
                major_radius: Some(5.0),
            },
            Touch {
                id: 2,
                x: 150.0,
                y: 220.0,
                phase: TouchPhase::Moved,
                force: Some(0.7),
                major_radius: None,
            },
        ],
    };

    let json = serde_json::to_value(&payload).expect("Failed to serialize");
    
    assert_eq!(json["simulator_id"], "sim-1");
    assert_eq!(json["touches"].len(), 2);
}

#[test]
fn test_gesture_payload() {
    // Test swipe gesture
    let payload = GesturePayload {
        simulator_id: "sim-1".to_string(),
        gesture_type: GestureType::Swipe,
        data: GestureData::Swipe {
            direction: SwipeDirection::Right,
            distance: 200.0,
        },
    };

    let json = serde_json::to_value(&payload).expect("Failed to serialize");
    
    assert_eq!(json["simulator_id"], "sim-1");
}

#[test]
fn test_device_button_payload() {
    // Test home button press
    let payload = DeviceButtonPayload {
        simulator_id: "sim-1".to_string(),
        button: DeviceButton::Home,
    };

    let json = serde_json::to_value(&payload).expect("Failed to serialize");
    
    assert_eq!(json["simulator_id"], "sim-1");
}

#[test]
fn test_file_transfer_payload() {
    use uuid::Uuid;
    
    let payload = FileTransferPayload {
        transfer_id: Uuid::new_v4(),
        simulator_id: Some("sim-1".to_string()),
        direction: TransferDirection::Upload,
        file_name: "test.txt".to_string(),
        file_size: 1024,
        chunk_data: None,
        chunk_index: Some(0),
        total_chunks: Some(5),
    };

    let json = serde_json::to_value(&payload).expect("Failed to serialize");
    
    assert_eq!(json["direction"], "upload");
    assert!(!json["simulator_id"].is_null());
}
