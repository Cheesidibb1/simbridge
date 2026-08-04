// Session management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use simbridge_shared::{
    models::{Session, SessionStats, StreamConfig},
    protocol::SessionStatus,
};
use thiserror::Error;

/// Session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    max_sessions: u32,
}

impl SessionManager {
    pub fn new(max_sessions: u32) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_sessions,
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        device_id: String,
        simulator_id: String,
        stream_config: StreamConfig,
    ) -> Result<Session, SessionError> {
        let sessions = self.sessions.read().await;
        
        if sessions.len() >= self.max_sessions as usize {
            return Err(SessionError::MaxSessionsReached);
        }
        
        drop(sessions);
        
        let session = Session {
            id: Uuid::new_v4(),
            device_id,
            simulator_id,
            status: SessionStatus::Active,
            created_at: Utc::now(),
            connected_at: Some(Utc::now()),
            disconnected_at: None,
            last_activity: Utc::now(),
            stream_config,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id, session.clone());
        
        Ok(session)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }

    /// Get all sessions
    pub async fn get_all_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get sessions for a device
    pub async fn get_device_sessions(&self, device_id: &str) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.device_id == device_id)
            .cloned()
            .collect()
    }

    /// Get sessions for a simulator
    pub async fn get_simulator_sessions(&self, simulator_id: &str) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.simulator_id == simulator_id)
            .cloned()
            .collect()
    }

    /// Update session activity
    pub async fn update_activity(&self, session_id: Uuid) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_activity = Utc::now();
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }

    /// Terminate a session
    pub async fn terminate_session(&self, session_id: Uuid) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = SessionStatus::Terminated;
            session.disconnected_at = Some(Utc::now());
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: Uuid) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&session_id)
            .map(|_| ())
            .ok_or(SessionError::NotFound)
    }

    /// Clean up inactive sessions
    pub async fn cleanup_inactive(&self, timeout_seconds: i64) {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        
        sessions.retain(|_, session| {
            let inactive_duration = now.signed_duration_since(session.last_activity);
            inactive_duration.num_seconds() < timeout_seconds
        });
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

/// Session errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    
    #[error("Maximum sessions reached")]
    MaxSessionsReached,
    
    #[error("Session already exists")]
    AlreadyExists,
    
    #[error("Session is inactive")]
    Inactive,
}
