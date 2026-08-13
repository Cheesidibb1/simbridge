// REST API server for SimBridge

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use uuid::Uuid;
// use crate::streaming::webrtc::{WebRTCSignalingManager, WebRTCSessionStats, SignalingHandler};

/// REST API server state
#[derive(Clone)]
pub struct RestServerState {
    pub sessions: Arc<RwLock<Vec<String>>>,
    pub android_adapters: Arc<RwLock<Vec<String>>>,
    pub ios_adapters: Arc<RwLock<Vec<String>>>,
}

impl RestServerState {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(Vec::new())),
            android_adapters: Arc::new(RwLock::new(Vec::new())),
            ios_adapters: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Simulators list response
#[derive(Debug, Serialize)]
struct SimulatorsResponse {
    simulators: Vec<SimulatorInfo>,
}

#[derive(Debug, Serialize)]
struct SimulatorInfo {
    id: String,
    name: String,
    platform: String,
    status: String,
}

/// WebRTC-specific request/response types (commented out for basic build)
/*
#[derive(Debug, Deserialize)]
struct CreateWebrtcSessionRequest {
    simulator_id: String,
    device_id: String,
    stream_id: String,
}

#[derive(Debug, Serialize)]
struct WebrtcSessionResponse {
    session_id: String,
    simulator_id: String,
    status: String,
}

/// WebRTC message types for WebSocket communication
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebRtcMessage {
    /// Client sends offer to server (browser-initiated)
    Offer {
        sdp: String,
        session_id: String,
        stream_id: String,
    },
    /// Server responds with answer
    Answer {
        sdp: String,
        session_id: String,
        stream_id: String,
    },
    /// ICE candidate exchange
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: u16,
        session_id: String,
        stream_id: String,
    },
}
*/

/// Signaling message request for REST API (commented out for basic build)
/*
#[derive(Debug, Serialize, Deserialize)]
pub struct SignalingMessageRequest {
    pub session_id: String,
    pub stream_id: String,
    #[serde(flatten)]
    pub message: WebRtcMessage,
}

impl WebRtcMessage {
    pub fn from_protocol_message(msg: simbridge_shared::protocol::Message) -> Self {
        // Extract payload and dispatch to appropriate message type
        let payload = msg.payload.clone();
        let message_type = msg.message_type;

        match message_type {
            simbridge_shared::protocol::MessageType::WebrtcOffer => {
                WebRtcMessage::Offer {
                    sdp: payload.get("sdp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    session_id: payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    stream_id: payload.get("stream_id").and_then(|v| v.as_str()).unwrap_or("stream-1").to_string(),
                }
            }
            simbridge_shared::protocol::MessageType::WebrtcAnswer => {
                WebRtcMessage::Answer {
                    sdp: payload.get("sdp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    session_id: payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    stream_id: payload.get("stream_id").and_then(|v| v.as_str()).unwrap_or("stream-1").to_string(),
                }
            }
            simbridge_shared::protocol::MessageType::WebrtcIceCandidate => {
                WebRtcMessage::IceCandidate {
                    candidate: payload.get("candidate").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    sdp_mid: payload.get("sdp_mid").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    sdp_mline_index: payload.get("sdp_mline_index").and_then(|v| v.as_i64()).map(|i| i as u16).unwrap_or(0),
                    session_id: payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    stream_id: payload.get("stream_id").and_then(|v| v.as_str()).unwrap_or("stream-1").to_string(),
                }
            }
            _ => WebRtcMessage::IceCandidate {
                candidate: String::new(),
                sdp_mid: None,
                sdp_mline_index: 0,
                session_id: "unknown".to_string(),
                stream_id: "stream-1".to_string(),
            },
        }
    }

    pub fn to_protocol_message(&self) -> simbridge_shared::protocol::Message {
        match self {
            WebRtcMessage::Offer { sdp, session_id, stream_id } => {
                let mut payload = serde_json::Map::new();
                payload.insert("type".to_string(), serde_json::Value::String("offer".to_string()));
                payload.insert("sdp".to_string(), serde_json::Value::String(sdp.clone()));
                payload.insert("session_id".to_string(), serde_json::Value::String(session_id.clone()));
                payload.insert("stream_id".to_string(), serde_json::Value::String(stream_id.clone()));

                simbridge_shared::protocol::Message {
                    message_type: simbridge_shared::protocol::MessageType::WebrtcOffer,
                    version: 1,
                    timestamp: chrono::Utc::now(),
                    request_id: None,
                    payload: serde_json::Value::Object(payload),
                }
            }
            WebRtcMessage::Answer { sdp, session_id, stream_id } => {
                let mut payload = serde_json::Map::new();
                payload.insert("type".to_string(), serde_json::Value::String("answer".to_string()));
                payload.insert("sdp".to_string(), serde_json::Value::String(sdp.clone()));
                payload.insert("session_id".to_string(), serde_json::Value::String(session_id.clone()));
                payload.insert("stream_id".to_string(), serde_json::Value::String(stream_id.clone()));

                simbridge_shared::protocol::Message {
                    message_type: simbridge_shared::protocol::MessageType::WebrtcAnswer,
                    version: 1,
                    timestamp: chrono::Utc::now(),
                    request_id: None,
                    payload: serde_json::Value::Object(payload),
                }
            }
            WebRtcMessage::IceCandidate { candidate, sdp_mid, sdp_mline_index, session_id, stream_id } => {
                let mut payload = serde_json::Map::new();
                payload.insert("type".to_string(), serde_json::Value::String("ice_candidate".to_string()));
                payload.insert("candidate".to_string(), serde_json::Value::String(candidate.clone()));
                if let Some(mid) = sdp_mid {
                    payload.insert("sdp_mid".to_string(), serde_json::Value::String(mid.clone()));
                }
                payload.insert("sdp_mline_index".to_string(), serde_json::Value::Number((*sdp_mline_index).into()));
                payload.insert("session_id".to_string(), serde_json::Value::String(session_id.clone()));
                payload.insert("stream_id".to_string(), serde_json::Value::String(stream_id.clone()));

                simbridge_shared::protocol::Message {
                    message_type: simbridge_shared::protocol::MessageType::WebrtcIceCandidate,
                    version: 1,
                    timestamp: chrono::Utc::now(),
                    request_id: None,
                    payload: serde_json::Value::Object(payload),
                }
            }
        }
    }
*/

/// Create REST API router
pub fn create_router() -> Router<RestServerState> {
    Router::new()
        // Health and simulator endpoints
        .route("/health", get(health_check))
        .route("/api/v1/simulators", get(list_simulators))
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/sessions", post(create_session))
        .route("/api/v1/sessions/:id", delete(delete_session))
        // WebRTC endpoints (commented out for basic build)
        // .route("/api/v1/webrtc/sessions", post(create_webrtc_session))
        // .route("/api/v1/webrtc/sessions/:id", get(get_webrtc_session))
        // .route("/api/v1/webrtc/sessions/:id", delete(delete_webrtc_session))
        // .route("/api/v1/webrtc/sessions/:id/stats", get(get_webrtc_session_stats))
        // .route("/api/v1/webrtc/signaling", post(handle_signaling_message))
}

/// Health check endpoint
async fn health_check(State(state): State<RestServerState>) -> Json<HealthResponse> {
    info!("Health check requested");
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// List available simulators
async fn list_simulators(State(state): State<RestServerState>) -> Json<SimulatorsResponse> {
    info!("Listing simulators");
    
    let android_adapters = state.android_adapters.read().await;
    let ios_adapters = state.ios_adapters.read().await;
    
    let mut simulators = vec![];
    
    // Add Android devices
    for device_id in android_adapters.iter() {
        simulators.push(SimulatorInfo {
            id: device_id.clone(),
            name: format!("Android Device ({})", device_id),
            platform: "android".to_string(),
            status: "available".to_string(),
        });
    }
    
    // Add iOS devices
    for device_id in ios_adapters.iter() {
        simulators.push(SimulatorInfo {
            id: device_id.clone(),
            name: format!("iOS Device ({})", device_id),
            platform: "ios".to_string(),
            status: "available".to_string(),
        });
    }
    
    // Add default examples if no devices found
    if simulators.is_empty() {
        simulators.push(SimulatorInfo {
            id: "android-emu-1".to_string(),
            name: "Pixel 7".to_string(),
            platform: "android".to_string(),
            status: "offline".to_string(),
        });
        simulators.push(SimulatorInfo {
            id: "ios-sim-1".to_string(),
            name: "iPhone 15 Pro".to_string(),
            platform: "ios".to_string(),
            status: "offline".to_string(),
        });
    }
    
    Json(SimulatorsResponse { simulators })
}

/// List active sessions
async fn list_sessions(State(state): State<RestServerState>) -> Json<Vec<String>> {
    info!("Listing sessions");
    let sessions = state.sessions.read().await;
    Json(sessions.clone())
}

/// Create a new session
#[derive(Debug, Deserialize)]
struct CreateSessionRequest {
    simulator_id: String,
    device_id: String,
}

async fn create_session(
    State(state): State<RestServerState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Creating session for simulator {}", req.simulator_id);

    // Generate a new UUID for the session
    let session_id = Uuid::new_v4();

    let mut sessions = state.sessions.write().await;
    sessions.push(session_id.to_string());

    Ok(Json(serde_json::json!({
        "session_id": session_id.to_string(),
        "simulator_id": req.simulator_id,
        "status": "active"
    })))
}

/// Create WebRTC session endpoint
async fn create_webrtc_session(
    State(_state): State<RestServerState>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Creating WebRTC session (not implemented)");
    Err(StatusCode::METHOD_NOT_ALLOWED)
}

/// Get WebRTC session info
async fn get_webrtc_session(
    State(_state): State<RestServerState>,
    Path(_session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting WebRTC session (not implemented)");
    Err(StatusCode::METHOD_NOT_ALLOWED)
}

/// Delete a session
async fn delete_session(
    State(state): State<RestServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting session: {}", session_id);

    let mut sessions = state.sessions.write().await;

    if let Some(pos) = sessions.iter().position(|id| id == &session_id) {
        sessions.remove(pos);
        Ok(Json(serde_json::json!({ "status": "deleted", "session_id": session_id })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Delete WebRTC session
async fn delete_webrtc_session(
    State(_state): State<RestServerState>,
    Path(_session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting WebRTC session (not implemented)");
    Err(StatusCode::METHOD_NOT_ALLOWED)
}

/// Get WebRTC session statistics
async fn get_webrtc_session_stats(
    State(_state): State<RestServerState>,
    Path((_session_id, _stream_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting WebRTC session stats (not implemented)");
    Err(StatusCode::METHOD_NOT_ALLOWED)
}

/// Handle incoming WebRTC signaling message (offer, answer, ICE candidate)
async fn handle_signaling_message(
    State(_state): State<RestServerState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Received signaling message (not implemented)");
    Err(StatusCode::METHOD_NOT_ALLOWED)
}