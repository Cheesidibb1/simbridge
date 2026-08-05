// WebRTC signaling and streaming

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// WebRTC signaling message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebRTCSignal {
    #[serde(rename = "offer")]
    Offer { 
        sdp: String, 
        session_id: Uuid 
    },
    #[serde(rename = "answer")]
    Answer { 
        sdp: String, 
        session_id: Uuid 
    },
    #[serde(rename = "ice_candidate")]
    IceCandidate { 
        candidate: String, 
        sdp_mid: Option<String>, 
        sdp_mline_index: Option<u16>, 
        session_id: Uuid 
    },
}

/// WebRTC session
#[derive(Debug, Clone)]
pub struct WebRTCSession {
    pub id: Uuid,
    pub simulator_id: String,
    pub device_id: String,
    pub offer_sdp: Option<String>,
    pub answer_sdp: Option<String>,
    pub ice_candidates: Vec<IceCandidate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

/// WebRTC signaling manager
pub struct WebRTCSignalingManager {
    sessions: Arc<RwLock<HashMap<Uuid, WebRTCSession>>>,
}

impl WebRTCSignalingManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new WebRTC session
    pub async fn create_session(
        &self,
        simulator_id: String,
        device_id: String,
    ) -> Result<Uuid, WebRTCError> {
        let session_id = Uuid::new_v4();
        
        let session = WebRTCSession {
            id: session_id,
            simulator_id,
            device_id,
            offer_sdp: None,
            answer_sdp: None,
            ice_candidates: Vec::new(),
            created_at: chrono::Utc::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);
        
        Ok(session_id)
    }

    /// Handle WebRTC offer
    pub async fn handle_offer(
        &self,
        session_id: Uuid,
        sdp: String,
    ) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            session.offer_sdp = Some(sdp);
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Handle WebRTC answer
    pub async fn handle_answer(
        &self,
        session_id: Uuid,
        sdp: String,
    ) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            session.answer_sdp = Some(sdp);
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Handle ICE candidate
    pub async fn handle_ice_candidate(
        &self,
        session_id: Uuid,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    ) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            session.ice_candidates.push(IceCandidate {
                candidate,
                sdp_mid,
                sdp_mline_index,
            });
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Get session
    pub async fn get_session(&self, session_id: Uuid) -> Option<WebRTCSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }

    /// Remove session
    pub async fn remove_session(&self, session_id: Uuid) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&session_id)
            .map(|_| ())
            .ok_or(WebRTCError::SessionNotFound)
    }

    /// Get ICE candidates for a session
    pub async fn get_ice_candidates(&self, session_id: Uuid) -> Result<Vec<IceCandidate>, WebRTCError> {
        let sessions = self.sessions.read().await;
        
        sessions.get(&session_id)
            .map(|s| s.ice_candidates.clone())
            .ok_or(WebRTCError::SessionNotFound)
    }
}

impl Default for WebRTCSignalingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebRTC errors
#[derive(Debug, Error)]
pub enum WebRTCError {
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Invalid SDP")]
    InvalidSdp,
    
    #[error("Connection failed")]
    ConnectionFailed,
    
    #[error("Signaling error: {0}")]
    SignalingError(String),
}
