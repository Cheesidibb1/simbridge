// Authentication management

use simbridge_shared::{
    auth::{crypto::Crypto, pairing::PairingManager, token::TokenManager},
    models::Device,
};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Authentication manager
pub struct AuthManager {
    token_manager: TokenManager,
    pairing_manager: Arc<RwLock<PairingManager>>,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    max_failed_attempts: u32,
    lockout_duration_seconds: u64,
}

impl AuthManager {
    pub fn new(max_failed_attempts: u32, lockout_duration_seconds: u64) -> Self {
        Self {
            token_manager: TokenManager::new(),
            pairing_manager: Arc::new(RwLock::new(PairingManager::new())),
            devices: Arc::new(RwLock::new(HashMap::new())),
            max_failed_attempts,
            lockout_duration_seconds,
        }
    }

    /// Create a new pairing session
    pub async fn create_pairing(
        &self,
        device_id: String,
        device_name: String,
        public_key: String,
    ) -> Result<String, AuthError> {
        let mut pairing_manager = self.pairing_manager.write().await;

        // Check if device is already paired
        let devices = self.devices.read().await;
        if devices.contains_key(&device_id) {
            return Err(AuthError::DeviceAlreadyPaired);
        }
        drop(devices);

        let session = pairing_manager.create_session(device_id, device_name, public_key);
        Ok(session.pairing_code)
    }

    /// Complete pairing with code
    pub async fn complete_pairing(&self, code: &str) -> Result<Device, AuthError> {
        let mut pairing_manager = self.pairing_manager.write().await;
        let session = pairing_manager
            .complete_session(code)
            .map_err(|e| AuthError::PairingFailed(e.to_string()))?;

        let device = Device {
            id: session.device_id.clone(),
            name: session.device_name,
            device_type: simbridge_shared::models::DeviceType::Desktop, // Will be updated based on actual device
            platform: "unknown".to_string(),
            os_version: "unknown".to_string(),
            paired_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            is_trusted: true,
            public_key: session.public_key,
        };

        let mut devices = self.devices.write().await;
        devices.insert(device.id.clone(), device.clone());

        Ok(device)
    }

    /// Generate auth token for a device
    pub async fn generate_token(
        &self,
        device_id: &str,
    ) -> Result<simbridge_shared::auth::token::AuthToken, AuthError> {
        let devices = self.devices.read().await;

        if !devices.contains_key(device_id) {
            return Err(AuthError::DeviceNotFound);
        }

        Ok(self.token_manager.generate_auth_token(device_id))
    }

    /// Validate an auth token
    pub async fn validate_token(
        &self,
        token: &simbridge_shared::auth::token::AuthToken,
    ) -> Result<(), AuthError> {
        let devices = self.devices.read().await;

        if !devices.contains_key(&token.device_id) {
            return Err(AuthError::DeviceNotFound);
        }

        self.token_manager
            .validate_token(token)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(())
    }

    /// Get a device by ID
    pub async fn get_device(&self, device_id: &str) -> Option<Device> {
        let devices = self.devices.read().await;
        devices.get(device_id).cloned()
    }

    /// Get all devices
    pub async fn get_all_devices(&self) -> Vec<Device> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    /// Remove a device
    pub async fn remove_device(&self, device_id: &str) -> Result<(), AuthError> {
        let mut devices = self.devices.write().await;
        devices
            .remove(device_id)
            .map(|_| ())
            .ok_or(AuthError::DeviceNotFound)
    }

    /// Update device last seen
    pub async fn update_last_seen(&self, device_id: &str) -> Result<(), AuthError> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.last_seen = chrono::Utc::now();
            Ok(())
        } else {
            Err(AuthError::DeviceNotFound)
        }
    }

    /// Clean up expired pairing sessions
    pub async fn cleanup_pairing_sessions(&self) {
        let mut pairing_manager = self.pairing_manager.write().await;
        pairing_manager.cleanup_expired();
    }
}

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Device not found")]
    DeviceNotFound,

    #[error("Device already paired")]
    DeviceAlreadyPaired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Too many failed attempts")]
    TooManyFailedAttempts,

    #[error("Account locked")]
    AccountLocked,

    #[error("Pairing session not found")]
    PairingSessionNotFound,

    #[error("Pairing session expired")]
    PairingSessionExpired,

    #[error("Pairing failed: {0}")]
    PairingFailed(String),
}
