#[cfg(test)]
mod tests {
    use simbridge_server::adapters::*;

    #[test]
    fn test_adapter_trait_methods() {
        // Test that the adapter trait is properly defined
        // We can't instantiate adapters without actual implementations, but we can verify the trait
    }

    #[test]
    fn test_screen_stream_structure() {
        let stream = ScreenStream {
            id: "stream-1".to_string(),
            width: 390,
            height: 844,
        };

        assert_eq!(stream.width, 390);
        assert_eq!(stream.height, 844);
    }

    #[test]
    fn test_simulator_status_fields() {
        let status = SimulatorStatus {
            is_running: true,
            current_app: Some("com.example.app".to_string()),
            battery_level: Some(85.0),
            cpu_usage: 45.5,
            memory_usage: 2048 * 1024 * 1024,
        };

        assert!(status.is_running);
    }

    #[test]
    fn test_adapter_error_variants() {
        use simbridge_server::adapters::AdapterError;

        // Verify all error variants can be created
        let _not_connected = AdapterError::NotConnected;
        let _connection_failed = AdapterError::ConnectionFailed("test".to_string());
        let _command_failed = AdapterError::CommandFailed("test".to_string());
        let _not_supported = AdapterError::NotSupported;
        let _invalid_param = AdapterError::InvalidParameter("test".to_string());
        let _file_not_found = AdapterError::FileNotFound("test".to_string());
        let _app_not_found = AdapterError::AppNotFound("test".to_string());
        let _stream_error = AdapterError::StreamError("test".to_string());
        let _internal_error = AdapterError::InternalError("test".to_string());
    }
}
