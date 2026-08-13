// Complete WebRTC signaling and video streaming for SimBridge

use std::collections::HashMap;
use thiserror::Error;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use simbridge_shared::protocol::StreamQuality;
use crate::streaming::encoder::VideoCodec;

/// WebRTC signaling message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WebRTCSignal {
    /// Client offers to connect (browser sends offer)
    Offer {
        sdp: String,
        session_id: Uuid,
        stream_id: String,
    },
    /// Server responds with answer (after generating SDP)
    Answer {
        sdp: String,
        session_id: Uuid,
        stream_id: String,
    },
    /// ICE candidate exchange for network traversal
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: u16,
        session_id: Uuid,
        stream_id: String,
    },
}

/// ICE Candidate structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: u16,
}

/// WebRTC session state
#[derive(Debug, Clone)]
pub enum WebRTCSessionState {
    WaitingForOffer,
    OfferReceived,
    NegotiationNeeded,
    Connected,
    Closed,
}

/// WebRTC Session tracking connection state and data channels
#[derive(Debug, Clone)]
pub struct WebRTCSession {
    pub id: Uuid,
    pub stream_id: String,
    pub simulator_id: String,
    pub device_id: String,
    pub session_state: WebRTCSessionState,
    pub offer_sdp: Option<String>,
    pub answer_sdp: Option<String>,
    pub ice_candidates: Vec<IceCandidate>,
    pub created_at: DateTime<Utc>,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_activity: DateTime<Utc>,
}

/// WebRTC signaling manager for session coordination
pub struct WebRTCSignalingManager {
    sessions: Arc<RwLock<HashMap<Uuid, WebRTCSession>>>,
    /// Active WebSocket connections mapped to sessions
    ws_sessions: Arc<RwLock<HashMap<Uuid, std::sync::Arc<RwLock<WebSocketContext>>>>>,
}

/// WebSocket context for signaling channel
struct WebSocketContext {
    send_callback: broadcast::Sender<WebRTCSignal>,
}

impl WebRTCSignalingManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            ws_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new WebRTC session
    pub async fn create_session(
        &self,
        simulator_id: String,
        device_id: String,
        stream_id: String,
    ) -> Result<Uuid, WebRTCError> {
        let session_id = Uuid::new_v4();
        
        let session = WebRTCSession {
            id: session_id,
            stream_id,
            simulator_id,
            device_id,
            session_state: WebRTCSessionState::WaitingForOffer,
            offer_sdp: None,
            answer_sdp: None,
            ice_candidates: Vec::new(),
            created_at: Utc::now(),
            connected_at: None,
            last_activity: Utc::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Option<WebRTCSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }

    /// Handle incoming offer from client
    pub async fn handle_offer(
        &self,
        session_id: Uuid,
        sdp: String,
        stream_id: String,
    ) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            session.session_state = WebRTCSessionState::OfferReceived;
            session.offer_sdp = Some(sdp);
            session.stream_id = stream_id;
            session.last_activity = Utc::now();
            
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Generate and send answer to client
    pub async fn handle_answer(
        &self,
        session_id: Uuid,
        sdp: String,
    ) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            session.session_state = WebRTCSessionState::NegotiationNeeded;
            session.answer_sdp = Some(sdp);
            session.last_activity = Utc::now();
            
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Add ICE candidate to session
    pub async fn add_ice_candidate(
        &self,
        session_id: Uuid,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: u16,
    ) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            session.ice_candidates.push(IceCandidate {
                candidate,
                sdp_mid,
                sdp_mline_index,
            });
            session.last_activity = Utc::now();
            
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Mark session as connected (after SDP exchange complete)
    pub async fn mark_connected(&self, session_id: Uuid) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            if matches!(session.session_state, WebRTCSessionState::NegotiationNeeded) {
                session.session_state = WebRTCSessionState::Connected;
                session.connected_at = Some(Utc::now());
                session.last_activity = Utc::now();
            }
            
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Get all active sessions (connected state)
    pub async fn get_active_sessions(&self) -> Vec<WebRTCSession> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|s| matches!(s.session_state, WebRTCSessionState::Connected))
            .cloned()
            .collect()
    }

    /// Close a session
    pub async fn close_session(&self, session_id: Uuid) -> Result<(), WebRTCError> {
        let mut sessions = self.sessions.write().await;
        
        if sessions.remove(&session_id).is_some() {
            Ok(())
        } else {
            Err(WebRTCError::SessionNotFound)
        }
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, session_id: Uuid) -> Option<WebRTCSessionStats> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(&session_id) {
            let duration_ms = if let Some(started) = session.connected_at {
                (Utc::now() - started).num_milliseconds() as u64
            } else {
                0
            };

            Some(WebRTCSessionStats {
                state: format!("{:?}", session.session_state),
                duration_ms,
                num_ice_candidates: session.ice_candidates.len(),
                connected_at: session.connected_at.map(|t| t.to_rfc3339()),
                created_at: session.created_at.to_rfc3339(),
            })
        } else {
            None
        }
    }

    /// Clear all sessions (cleanup)
    pub async fn clear_all(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
    }
}

/// WebRTC session statistics
#[derive(Debug, Clone)]
pub struct WebRTCSessionStats {
    pub state: String,
    pub duration_ms: u64,
    pub num_ice_candidates: usize,
    pub connected_at: Option<String>,
    pub created_at: String,
}

/// WebRTC signaling errors
#[derive(Debug, Error)]
pub enum WebRTCError {
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Invalid SDP format")]
    InvalidSDP,
    
    #[error("ICE candidate rejected")]
    ICECandidateRejected,
    
    #[error("Signaling error: {0}")]
    SignalingError(String),
}

/// Frame delivery system for streaming video data over WebRTC
pub struct FrameDeliverySystem {
    /// Active frame senders mapped to stream IDs
    active_senders: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Vec<u8>>>>>,
}

impl FrameDeliverySystem {
    pub fn new() -> Self {
        Self {
            active_senders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create sender for a stream
    pub async fn get_or_create_sender(
        &self,
        stream_id: String,
        capacity: usize,
    ) -> Result<tokio::sync::mpsc::Sender<Vec<u8>>, FrameDeliveryError> {
        // Try to get existing sender
        let mut senders = self.active_senders.write().await;
        
        if let Some(sender) = senders.get(&stream_id) {
            let sender_clone = sender.clone();
            drop(senders);
            return Ok(sender_clone);
        }

        // Create new sender
        let (tx, _rx) = tokio::sync::mpsc::channel(capacity);
        senders.insert(stream_id.clone(), tx.clone());
        
        Ok(tx)
    }

    /// Send frame data to all active receivers for a stream
    pub async fn broadcast_frame(&self, stream_id: String, frame_data: Vec<u8>) -> Result<usize, FrameDeliveryError> {
        let mut senders = self.active_senders.write().await;
        
        if let Some(sender) = senders.get(&stream_id) {
            let len = frame_data.len();
            let sent = sender.send(frame_data).await.is_ok();
            Ok(if sent { len } else { 0 })
        } else {
            Err(FrameDeliveryError::StreamNotActive(stream_id))
        }
    }

    /// Stop streaming for a stream
    pub async fn stop_stream(&self, stream_id: String) {
        let mut senders = self.active_senders.write().await;
        senders.remove(&stream_id);
    }

    /// Get active stream count
    pub async fn get_active_count(&self) -> usize {
        let senders = self.active_senders.read().await;
        senders.len()
    }
}

/// Frame delivery errors
#[derive(Debug, Error)]
pub enum FrameDeliveryError {
    #[error("Stream not active: {0}")]
    StreamNotActive(String),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Send error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),
}

/// WebRTC signaling handler for WebSocket connections
pub struct SignalingHandler;

impl SignalingHandler {
    /// Handle incoming offer signal and send answer back
    pub async fn handle_offer(
        manager: Arc<WebRTCSignalingManager>,
        session_id: Uuid,
        stream_id: String,
        sdp: String,
    ) -> Result<WebRTCSignal, WebRTCError> {
        // Store the offer
        manager.handle_offer(session_id.clone(), sdp.clone(), stream_id.clone())
            .await?;

        // In real implementation, generate answer SDP here
        // For now, return the stored offer (placeholder)
        
        Ok(WebRTCSignal::Answer {
            sdp: sdp, // Placeholder - would generate in production
            session_id,
            stream_id,
        })
    }

    /// Handle ICE candidate and forward to peer
    pub async fn handle_ice_candidate(
        manager: Arc<WebRTCSignalingManager>,
        session_id: Uuid,
        stream_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: u16,
    ) -> Result<(), WebRTCError> {
        manager.add_ice_candidate(session_id, candidate, sdp_mid, sdp_mline_index)
            .await?;
        
        Ok(())
    }

    /// Get session info for monitoring
    pub async fn get_session_info(
        manager: Arc<WebRTCSignalingManager>,
        session_id: Uuid,
    ) -> Option<WebRTCSessionStats> {
        manager.get_session_stats(session_id).await
    }
}

/// WebRTC server configuration
#[derive(Debug, Clone)]
pub struct WebRTCConfig {
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<String>,
    pub signaling_port: u16,
    pub max_ice_candidates: usize,
}

impl Default for WebRTCConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(), // Google STUN
                "stun:stun1.l.google.com:19302".to_string(), // Backup
            ],
            turn_servers: Vec::new(), // TURN requires credential setup
            signaling_port: 8787,
            max_ice_candidates: 100,
        }
    }
}

impl WebRTCConfig {
    /// Create new config with custom settings
    pub fn with_stun(mut self, stun: String) -> Self {
        self.stun_servers = vec![stun];
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_session_creation() {
        let manager = WebRTCSignalingManager::new();
        
        let session_id = manager.create_session(
            "sim-1".to_string(),
            "device-1".to_string(),
            "stream-1".to_string(),
        ).await.unwrap();

        assert!(manager.get_session(session_id).is_some());
    }

    #[tokio::test]
    async fn test_offer_handling() {
        let manager = WebRTCSignalingManager::new();
        
        let session_id = Uuid::new_v4();
        let sdp = "v=0\r\no=- 1234567890 1234567890 IN IP4 127.0.0.1";

        manager.handle_offer(session_id.clone(), sdp.to_string(), "stream-1".to_string())
            .await.unwrap();

        assert!(manager.get_session(session_id).is_some());
    }

    #[tokio::test]
    async fn test_ice_candidate() {
        let manager = WebRTCSignalingManager::new();
        
        let session_id = manager.create_session(
            "sim-1".to_string(),
            "device-1".to_string(),
            "stream-1".to_string(),
        ).await.unwrap();

        let candidate = "candidate:1 1 UDP 100 192.168.1.1 5000 typ host";
        
        manager.add_ice_candidate(
            session_id.clone(),
            candidate.to_string(),
            Some("audio".to_string()),
            0,
        ).await.unwrap();

        let stats = manager.get_session_stats(session_id).await.unwrap();
        assert_eq!(stats.num_ice_candidates, 1);
    }

    #[tokio::test]
    async fn test_frame_delivery() {
        let delivery = FrameDeliverySystem::new();
        
        let stream_id = "stream-1".to_string();
        
        // Create sender with capacity of 10 frames
        let tx = delivery.get_or_create_sender(stream_id.clone(), 10)
            .await.unwrap();

        // Send test frame (1KB PNG)
        let frame_data = vec![1, 2, 3, 4, 5; 1024];
        
        let sent_frames = delivery.broadcast_frame(stream_id, frame_data)
            .await.unwrap();

        assert_eq!(sent_frames, frame_data.len());
    }

    #[tokio::test]
    async fn test_session_statistics() {
        let manager = WebRTCSignalingManager::new();
        
        let session_id = manager.create_session(
            "sim-1".to_string(),
            "device-1".to_string(),
            "stream-1".to_string(),
        ).await.unwrap();

        // Mark as connected after some time
        let now = Utc::now() - Duration::seconds(30);
        if let mut sessions = manager.sessions.write().await {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.connected_at = Some(now);
                session.session_state = WebRTCSessionState::Connected;
            }
        }

        let stats = manager.get_session_stats(session_id).await.unwrap();
        assert!(stats.duration_ms > 29000); // ~30 seconds
    }
}
