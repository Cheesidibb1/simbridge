// Device pairing utilities

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::auth::crypto::Crypto;
use crate::models::Device;
use thiserror::Error;

/// Pairing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingRequest {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub public_key: String,
    pub timestamp: DateTime<Utc>,
}

/// Pairing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingResponse {
    pub pairing_code: String,
    pub server_public_key: String,
    pub expires_at: DateTime<Utc>,
}

/// Pairing session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingSession {
    pub session_id: Uuid,
    pub pairing_code: String,
    pub device_id: String,
    pub device_name: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_completed: bool,
}

impl PairingSession {
    pub fn new(device_id: String, device_name: String, public_key: String) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::minutes(5); // 5 minute expiry
        
        Self {
            session_id: Uuid::new_v4(),
            pairing_code: Crypto::generate_pairing_code(),
            device_id,
            device_name,
            public_key,
            created_at: now,
            expires_at,
            is_completed: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn complete(&mut self) {
        self.is_completed = true;
    }
}

/// Pairing manager
pub struct PairingManager {
    active_sessions: Vec<PairingSession>,
    session_duration_minutes: i64,
}

impl PairingManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Vec::new(),
            session_duration_minutes: 5,
        }
    }

    pub fn with_duration(mut self, minutes: i64) -> Self {
        self.session_duration_minutes = minutes;
        self
    }

    /// Create a new pairing session
    pub fn create_session(&mut self, device_id: String, device_name: String, public_key: String) -> PairingSession {
        let session = PairingSession::new(device_id, device_name, public_key);
        self.active_sessions.push(session.clone());
        session
    }

    /// Find a pairing session by code
    pub fn find_session_by_code(&self, code: &str) -> Option<&PairingSession> {
        self.active_sessions
            .iter()
            .find(|s| s.pairing_code == code && !s.is_expired() && !s.is_completed)
    }

    /// Complete a pairing session
    pub fn complete_session(&mut self, code: &str) -> Result<PairingSession, PairingError> {
        let session = self.find_session_by_code(code)
            .ok_or(PairingError::SessionNotFound)?;
        
        let session_id = session.session_id;
        
        if let Some(s) = self.active_sessions.iter_mut().find(|s| s.session_id == session_id) {
            s.complete();
            Ok(s.clone())
        } else {
            Err(PairingError::SessionNotFound)
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) {
        self.active_sessions.retain(|s| !s.is_expired());
    }

    /// Get active session count
    pub fn active_count(&self) -> usize {
        self.active_sessions
            .iter()
            .filter(|s| !s.is_expired() && !s.is_completed)
            .count()
    }
}

impl Default for PairingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pairing errors
#[derive(Debug, Error)]
pub enum PairingError {
    #[error("Pairing session not found")]
    SessionNotFound,
    
    #[error("Pairing session expired")]
    SessionExpired,
    
    #[error("Pairing already completed")]
    AlreadyCompleted,
    
    #[error("Invalid public key")]
    InvalidPublicKey,
    
    #[error("Device already paired")]
    DeviceAlreadyPaired,
}
