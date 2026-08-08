#[cfg(test)]
mod tests {
    use simbridge_shared::auth::token::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_token_creation() {
        let token = Token::new(
            "device-id-123".to_string(),
            3600, // 1 hour expiry
        );

        assert_eq!(token.device_id, "device-id-123");
        assert_eq!(token.expiry_seconds, 3600);
    }

    #[test]
    fn test_token_with_explicit_expires_at() {
        let expires_at = Utc::now() + Duration::hours(2);
        let token = Token::with_expiration(
            "device-id-456".to_string(),
            expires_at,
        );

        assert_eq!(token.device_id, "device-id-456");
    }

    #[test]
    fn test_token_serialization() {
        let token = Token::new(
            "test-device".to_string(),
            1800, // 30 minutes
        );

        let json = serde_json::to_value(&token).expect("Failed to serialize");
        
        assert_eq!(json["device_id"], "test-device");
    }

    #[test]
    fn test_token_deserialization() {
        let json_str = r#"{"device_id":"test-device","expires_at":"2024-01-01T12:00:00Z"}"#;
        
        let token: Token = serde_json::from_str(json_str).expect("Failed to deserialize");
        
        assert_eq!(token.device_id, "test-device");
    }

    #[test]
    fn test_token_expiration() {
        let token = Token::new(
            "test-device".to_string(),
            60, // 1 minute expiry
        );

        assert!(!token.is_expired());
        
        // Advance time by 2 minutes
        let now = chrono::Utc::now();
        token.expired_at(now + Duration::minutes(2));
        
        assert!(token.is_expired());
    }
}
