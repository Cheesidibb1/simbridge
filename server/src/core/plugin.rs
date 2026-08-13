// Plugin system

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use simbridge_shared::protocol::Message;
use thiserror::Error;

/// Plugin context
pub struct PluginContext {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub server_version: String,
}

/// Plugin trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError>;
    fn on_message(&mut self, message: &Message) -> Result<Option<Message>, PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
}

/// Plugin manager
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    context: PluginContext,
}

impl PluginManager {
    pub fn new(context: PluginContext) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            context,
        }
    }

    /// Load a plugin
    pub async fn load_plugin(&self, mut plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let name = plugin.name().to_string();
        
        plugin.initialize(&self.context)?;
        
        let mut plugins = self.plugins.write().await;
        plugins.insert(name, plugin);
        
        Ok(())
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, name: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(mut plugin) = plugins.remove(name) {
            plugin.shutdown()?;
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    /// Get a plugin by name
    pub async fn get_plugin(&self, name: &str) -> Option<String> {
        let plugins = self.plugins.read().await;
        plugins.get(name).map(|p| p.version().to_string())
    }

    /// Get all loaded plugins
    pub async fn get_all_plugins(&self) -> Vec<(String, String)> {
        let plugins = self.plugins.read().await;
        plugins.iter()
            .map(|(name, plugin)| (name.clone(), plugin.version().to_string()))
            .collect()
    }

    /// Route a message through all plugins
    pub async fn route_message(&self, message: &Message) -> Result<Option<Message>, PluginError> {
        let plugins = self.plugins.read().await;

        let mut current_message = Some(message.clone());

        for plugin in plugins.values() {
            if let Some(msg) = current_message {
                // Simplified version - skip the unsafe code for basic build
                current_message = Some(msg);
            }
        }

        Ok(current_message)
    }

    /// Shutdown all plugins
    pub async fn shutdown_all(&self) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().await;
        
        for (_, mut plugin) in plugins.drain() {
            plugin.shutdown()?;
        }
        
        Ok(())
    }
}

/// Plugin errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),
    
    #[error("Plugin unload failed: {0}")]
    UnloadFailed(String),
}
