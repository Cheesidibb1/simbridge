// TLS configuration and utilities

use std::path::Path;
use thiserror::Error;

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub client_auth_enabled: bool,
    pub client_ca_path: Option<String>,
}

impl TlsConfig {
    pub fn new(cert_path: &str, key_path: &str) -> Self {
        Self {
            cert_path: cert_path.to_string(),
            key_path: key_path.to_string(),
            client_auth_enabled: false,
            client_ca_path: None,
        }
    }

    pub fn with_client_auth(mut self, client_ca_path: &str) -> Self {
        self.client_auth_enabled = true;
        self.client_ca_path = Some(client_ca_path.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), TlsError> {
        if !Path::new(&self.cert_path).exists() {
            return Err(TlsError::CertNotFound(self.cert_path.clone()));
        }
        
        if !Path::new(&self.key_path).exists() {
            return Err(TlsError::KeyNotFound(self.key_path.clone()));
        }
        
        if self.client_auth_enabled {
            if let Some(ca_path) = &self.client_ca_path {
                if !Path::new(ca_path).exists() {
                    return Err(TlsError::CaNotFound(ca_path.clone()));
                }
            }
        }
        
        Ok(())
    }
}

/// TLS errors
#[derive(Debug, Error)]
pub enum TlsError {
    #[error("Certificate not found: {0}")]
    CertNotFound(String),
    
    #[error("Private key not found: {0}")]
    KeyNotFound(String),
    
    #[error("CA certificate not found: {0}")]
    CaNotFound(String),
    
    #[error("Invalid certificate: {0}")]
    InvalidCert(String),
    
    #[error("Invalid private key: {0}")]
    InvalidKey(String),
}
