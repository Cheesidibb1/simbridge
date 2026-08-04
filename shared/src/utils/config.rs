// Configuration utilities

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a TOML file
    pub fn load_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Load configuration from a JSON file
    pub fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;
        
        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save configuration to a TOML file
    pub fn save_toml<T: Serialize>(path: &Path, config: &T) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        
        Ok(())
    }

    /// Save configuration to a JSON file
    pub fn save_json<T: Serialize>(path: &Path, config: &T) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        
        Ok(())
    }

    /// Get default configuration directory
    pub fn get_config_dir() -> PathBuf {
        if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(|s| PathBuf::from(s).join("simbridge"))
                .unwrap_or_else(|_| PathBuf::from(".simbridge"))
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|s| PathBuf::from(s).join(".config").join("simbridge"))
                .unwrap_or_else(|_| PathBuf::from(".simbridge"))
        } else {
            // Linux and others
            std::env::var("XDG_CONFIG_HOME")
                .map(|s| PathBuf::from(s).join("simbridge"))
                .unwrap_or_else(|_| {
                    std::env::var("HOME")
                        .map(|s| PathBuf::from(s).join(".config").join("simbridge"))
                        .unwrap_or_else(|_| PathBuf::from(".simbridge"))
                })
        }
    }

    /// Ensure configuration directory exists
    pub fn ensure_config_dir() -> Result<PathBuf, ConfigError> {
        let config_dir = Self::get_config_dir();
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| ConfigError::CreateDirError(e.to_string()))?;
        Ok(config_dir)
    }
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Read error: {0}")]
    ReadError(String),
    
    #[error("Write error: {0}")]
    WriteError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Serialize error: {0}")]
    SerializeError(String),
    
    #[error("Create directory error: {0}")]
    CreateDirError(String),
}
