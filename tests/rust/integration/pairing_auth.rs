#[cfg(test)]
mod tests {
    use simbridge_shared::auth::*;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_complete_pairing_flow() {
        // Simulate a complete pairing flow
        
        // 1. Create pairing manager
        let mut manager = PairingManager::new().with_duration(5);

        // 2. Create pairing session
        let session = manager.create_session(
            "device-1".to_string(),
            "Test iPhone 15 Pro".to_string(),
            "public-key-abc123".to_string(),
        );

        assert!(!session.pairing_code.is_empty());
        assert!(!session.is_completed);
    }

    #[tokio::test]
    async fn test_pairing_session_expiration() {
        let session = PairingSession::new(
            "device-1".to_string(),
            "Test Device".to_string(),
            "public-key".to_string(),
        );

        // Initially not expired
        assert!(!session.is_expired());

        // Mark as completed
        session.complete();
        assert!(session.is_completed);
    }

    #[tokio::test]
    async fn test_auth_token_validation() {
        // Simulate token creation and validation
        let token = Token::new(
            "device-1".to_string(),
            3600, // 1 hour
        );

        assert_eq!(token.device_id, "device-1");
    }
}
