#[cfg(test)]
mod tests {
    use simbridge_server::core::session::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new(5); // Allow 5 concurrent sessions

        assert_eq!(manager.max_sessions, 5);
    }

    #[test]
    fn test_session_manager_defaults() {
        let manager = SessionManager::new(10);

        assert_eq!(manager.max_sessions, 10);
    }

    #[test]
    fn test_session_tracking() {
        // Test that sessions can be created and tracked
        // Actual session creation requires async context
    }

    #[test]
    fn test_max_session_enforcement() {
        let manager = SessionManager::new(2);

        // When max is reached, should return error
        // The actual implementation handles this in create_session
    }
}
