#[cfg(test)]
mod tests {
    use simbridge_server::core::plugin::*;

    #[test]
    fn test_plugin_manager_creation() {
        let plugin_context = PluginContext {
            config_dir: std::path::PathBuf::from(".config"),
            data_dir: std::path::PathBuf::from(".data"),
            server_version: "1.0.0".to_string(),
        };

        let manager = PluginManager::new(plugin_context);

        assert!(manager.config_dir.is_absolute());
    }

    #[test]
    fn test_plugin_registration() {
        // Test that plugins can be registered
        // The actual plugin loading happens at runtime
    }
}
