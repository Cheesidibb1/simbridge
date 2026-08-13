#[cfg(test)]
mod tests {
    use simbridge_shared::auth::pairing::*;

    #[test]
    fn test_pairing_session_creation() {
        let session = PairingSession::new(
            "device-123".to_string(),
            "Test iPhone".to_string(),
            "public-key-data".to_string(),
        );

        assert_eq!(session.device_id, "device-123");
        assert!(!session.pairing_code.is_empty());
        assert!(!session.public_key.is_empty());
        assert!(!session.is_expired());
        assert!(!session.is_completed);
    }

    #[test]
    fn test_pairing_session_expiration() {
        let session = PairingSession::new(
            "device-123".to_string(),
            "Test iPhone".to_string(),
            "public-key-data".to_string(),
        );

        // Should not be expired initially
        assert!(!session.is_expired());

        // Mark as completed
        session.complete();
        assert!(session.is_completed);
    }

    #[test]
    fn test_pairing_manager() {
        let mut manager = PairingManager::new()
            .with_duration(5); // 5 minutes

        // Create a pairing session
        let session1 = manager.create_session(
            "device-1".to_string(),
            "Test iPhone 1".to_string(),
            "key1".to_string(),
        );

        assert_eq!(manager.active_sessions.len(), 1);
        assert_eq!(session1.device_id, "device-1");

        // Create another session
        let session2 = manager.create_session(
            "device-2".to_string(),
            "Test iPhone 2".to_string(),
            "key2".to_string(),
        );

        assert_eq!(manager.active_sessions.len(), 2);
    }

    #[test]
    fn test_find_session_by_code() {
        let mut manager = PairingManager::new();

        let session = manager.create_session(
            "device-1".to_string(),
            "Test iPhone".to_string(),
            "key1".to_string(),
        );

        // Find by code - should succeed
        assert!(manager.find_session_by_code(&session.pairing_code).is_some());

        // Find by wrong code - should fail
        assert!(manager.find_session_by_code("wrong-code").is_none());
    }

    #[test]
    fn test_pairing_manager_with_duration() {
        let manager = PairingManager::new()
            .with_duration(10); // 10 minutes

        assert_eq!(manager.session_duration_minutes, 10);
    }
}
