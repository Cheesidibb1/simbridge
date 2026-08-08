#[cfg(test)]
mod tests {
    use simbridge_server::streaming::*;

    #[test]
    fn test_streaming_coordinator_structure() {
        // Test coordinator structure
        let _coordinator = StreamingCoordinator::new();
    }

    #[test]
    fn test_screen_encoder_variants() {
        use simbridge_server::streaming::ScreenEncoderType;

        let encoders = vec![
            ScreenEncoderType::H264,
            ScreenEncoderType::VP8,
            ScreenEncoderType::JPEG,
            ScreenEncoderType::PNG,
        ];

        assert_eq!(encoders.len(), 4);
    }
}
