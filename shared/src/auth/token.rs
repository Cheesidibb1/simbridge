// Token generation and validation

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token_id: Uuid,
    pub device_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

impl AuthToken {
    pub fn new(device_id: &str, duration_hours: u64) -> Self {
        let now = Utc::now();
        Self {
            token_id: Uuid::new_v4(),
            device_id: device_id.to_string(),
            issued_at: now,
            expires_at: now + Duration::hours(duration_hours as i64),
            scopes: vec!["simulator:control".to_string(), "simulator:view".to_string()],
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// Session token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    pub session_id: Uuid,
    pub device_id: String,
    pub simulator_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl SessionToken {
    pub fn new(device_id: &str, simulator_id: &str, duration_minutes: u64) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            device_id: device_id.to_string(),
            simulator_id: simulator_id.to_string(),
            created_at: now,
            expires_at: now + Duration::minutes(duration_minutes as i64),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Token manager
pub struct TokenManager {
    default_token_duration_hours: u64,
    default_session_duration_minutes: u64,
}

impl TokenManager {
    pub fn new() -> Self {
        Self {
            default_token_duration_hours: 24,
            default_session_duration_minutes: 60,
        }
    }

    pub fn with_durations(mut self, token_hours: u64, session_minutes: u64) -> Self {
        self.default_token_duration_hours = token_hours;
        self.default_session_duration_minutes = session_minutes;
        self
    }

    pub fn generate_auth_token(&self, device_id: &str) -> AuthToken {
        AuthToken::new(device_id, self.default_token_duration_hours)
    }

    pub fn generate_session_token(&self, device_id: &str, simulator_id: &str) -> SessionToken {
        SessionToken::new(device_id, simulator_id, self.default_session_duration_minutes)
    }

    pub fn validate_token(&self, token: &AuthToken) -> Result<(), TokenError> {
        if token.is_expired() {
            return Err(TokenError::Expired);
        }
        Ok(())
    }

    pub fn validate_session_token(&self, token: &SessionToken) -> Result<(), TokenError> {
        if token.is_expired() {
            return Err(TokenError::Expired);
        }
        Ok(())
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Token errors
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Token is expired")]
    Expired,
    
    #[error("Token is invalid")]
    Invalid,
    
    #[error("Token not found")]
    NotFound,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
}
