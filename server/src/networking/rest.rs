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
use simbridge_server::streaming::webrtc::{WebRTCSignalingManager, WebRTCSessionStats, SignalingHandler};

/// REST API server state
#[derive(Clone)]
pub struct RestServerState {
    pub sessions: Arc<RwLock<Vec<String>>>,
    pub android_adapters: Arc<RwLock<Vec<String>>>,
    pub ios_adapters: Arc<RwLock<Vec<String>>>,
    /// WebRTC signaling manager instance (wrapped in Arc for sharing)
    pub webrtc_manager: Option<Arc<WebRTCSignalingManager>>,
}

impl RestServerState {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(Vec::new())),
            android_adapters: Arc::new(RwLock::new(Vec::new())),
            ios_adapters: Arc::new(RwLock::new(Vec::new())),
            webrtc_manager: None,
        }
    }

    pub fn with_webrtc_manager(webrtc_manager: Arc<WebRTCSignalingManager>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(Vec::new())),
            android_adapters: Arc::new(RwLock::new(Vec::new())),
            ios_adapters: Arc::new(RwLock::new(Vec::new())),
            webrtc_manager: Some(webrtc_manager),
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

/// WebRTC-specific request/response types
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

/// Signaling message request for REST API
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
                    sdp: payload.get("sdp").unwrap_or(&"".to_string()).to_string(),
                    session_id: payload.get("session_id").unwrap_or(&"unknown".to_string())
                        .to_string(),
                    stream_id: payload.get("stream_id").unwrap_or(&"stream-1".to_string())
                        .to_string(),
                }
            }
            simbridge_shared::protocol::MessageType::WebrtcAnswer => {
                WebRtcMessage::Answer {
                    sdp: payload.get("sdp").unwrap_or(&"".to_string()).to_string(),
                    session_id: payload.get("session_id").unwrap_or(&"unknown".to_string())
                        .to_string(),
                    stream_id: payload.get("stream_id").unwrap_or(&"stream-1".to_string())
                        .to_string(),
                }
            }
            simbridge_shared::protocol::MessageType::WebrtcIceCandidate => {
                WebRtcMessage::IceCandidate {
                    candidate: payload.get("candidate").unwrap_or(&"".to_string()).to_string(),
                    sdp_mid: payload.get("sdp_mid"),
                    sdp_mline_index: payload.get("sdp_mline_index").and_then(|v| v.as_i64()).map(|i| i as u16),
                    session_id: payload.get("session_id").unwrap_or(&"unknown".to_string())
                        .to_string(),
                    stream_id: payload.get("stream_id").unwrap_or(&"stream-1".to_string())
                        .to_string(),
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
                    requestId: None,
                    payload,
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
                    requestId: None,
                    payload,
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
                    requestId: None,
                    payload,
                }
            }
        }
    }
}

/// Create REST API router
pub fn create_router() -> Router {
    Router::new()
        // Health and simulator endpoints
        .route("/health", get(health_check))
        .route("/api/v1/simulators", get(list_simulators))
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/sessions", post(create_session))
        .route("/api/v1/sessions/:id", delete(delete_session))

        // WebRTC session management
        .route("/api/v1/webrtc/sessions", post(create_webrtc_session))
        .route("/api/v1/webrtc/sessions/:id", get(get_webrtc_session))
        .route("/api/v1/webrtc/sessions/:id", delete(delete_webrtc_session))
        .route("/api/v1/webrtc/sessions/:id/stats", get(get_webrtc_session_stats))

        // WebRTC signaling message handling
        .route("/api/v1/webrtc/signaling", post(handle_signaling_message))
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
    State(state): State<RestServerState>,
    Json(req): Json<CreateWebrtcSessionRequest>,
) -> Result<Json<WebrtcSessionResponse>, StatusCode> {
    info!("Creating WebRTC session for simulator {}", req.simulator_id);

    // Get WebRTC manager from state
    let webrtc_manager = match &state.webrtc_manager {
        Some(manager) => manager.clone(),
        None => return Err(StatusCode::ServiceUnavailable),
    };

    // Create the session in the signaling manager
    let session_id = webrtc_manager.create_session(
        req.simulator_id,
        req.device_id,
        req.stream_id,
    ).await?;

    Ok(Json(WebrtcSessionResponse {
        session_id: session_id.to_string(),
        simulator_id: req.simulator_id,
        status: "waiting_for_offer".to_string(),
    }))
}

/// Get WebRTC session info
async fn get_webrtc_session(
    State(state): State<RestServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting WebRTC session: {}", session_id);

    let webrtc_manager = match &state.webrtc_manager {
        Some(manager) => manager.clone(),
        None => return Err(StatusCode::ServiceUnavailable),
    };

    // Parse UUID and get session
    match Uuid::parse_str(&session_id) {
        Ok(uuid) => {
            if let Some(session) = webrtc_manager.get_session(uuid).await {
                return Ok(Json(serde_json::json!({
                    "id": session.id.to_string(),
                    "simulator_id": session.simulator_id,
                    "device_id": session.device_id,
                    "stream_id": session.stream_id,
                    "state": format!("{:?}", session.session_state),
                    "connected_at": session.connected_at.map(|t| t.to_rfc3339()),
                    "created_at": session.created_at.to_rfc3339(),
                })));
            }
        }
        Err(_) => return Err(StatusCode::BadRequest),
    }

    Err(StatusCode::NotFound)
}

/// Delete a session
async fn delete_session(
    State(state): State<RestServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting session: {}", session_id);

    let mut sessions = state.sessions.write().await;

    if sessions.remove(&session_id) {
        Ok(Json(serde_json::json!({ "status": "deleted", "session_id": session_id })))
    } else {
        Err(StatusCode::NotFound)
    }
}

/// Delete WebRTC session
async fn delete_webrtc_session(
    State(state): State<RestServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting WebRTC session: {}", session_id);

    let webrtc_manager = match &state.webrtc_manager {
        Some(manager) => manager.clone(),
        None => return Err(StatusCode::ServiceUnavailable),
    };

    // Parse UUID and delete session
    if let Ok(uuid) = Uuid::parse_str(&session_id) {
        webrtc_manager.close_session(uuid).await?;
        return Ok(Json(serde_json::json!({ "status": "deleted" })));
    }

    Err(StatusCode::BadRequest)
}

/// Get WebRTC session statistics
async fn get_webrtc_session_stats(
    State(state): State<RestServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<WebRTCSessionStats>, StatusCode> {
    info!("Getting WebRTC session stats: {}", session_id);

    let webrtc_manager = match &state.webrtc_manager {
        Some(manager) => manager.clone(),
        None => return Err(StatusCode::ServiceUnavailable),
    };

    if let Some(stats) = webrtc_manager
        .get_session_stats(Uuid::parse_str(&session_id).map_err(|_| StatusCode::BadRequest)?)
        .await
    {
        Ok(Json(stats))
    } else {
        Err(StatusCode::NotFound)
    }
}

/// Handle incoming WebRTC signaling message (offer, answer, ICE candidate)
async fn handle_signaling_message(
    State(state): State<RestServerState>,
    Json(request): Json<SignalingMessageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Received signaling message for session: {}", request.session_id);

    let webrtc_manager = match &state.webrtc_manager {
        Some(manager) => manager.clone(),
        None => return Err(StatusCode::ServiceUnavailable),
    };

    // Parse session UUID
    let session_id = Uuid::parse_str(&request.session_id)
        .map_err(|_| StatusCode::BadRequest)?;

    match request.message {
        WebRtcMessage::Offer { sdp, stream_id, .. } => {
            // Handle offer and generate answer
            match SignalingHandler::handle_offer(
                webrtc_manager,
                session_id,
                stream_id,
                sdp,
            ).await {
                Ok(answer_signal) => {
                    if let WebRTCSignal::Answer { sdp: answer_sdp, .. } = answer_signal {
                        Ok(Json(serde_json::json!({
                            "type": "answer",
                            "sdp": answer_sdp,
                            "session_id": request.session_id
                        })))
                    } else {
                        Err(StatusCode::InternalServerError)
                    }
                }
                Err(e) => {
                    error!("Failed to handle offer: {}", e);
                    Err(StatusCode::BadRequest)
                }
            }
        }
        WebRtcMessage::IceCandidate { candidate, sdp_mid, sdp_mline_index, .. } => {
            // Add ICE candidate
            match SignalingHandler::handle_ice_candidate(
                webrtc_manager,
                session_id,
                request.stream_id,
                candidate,
                sdp_mid,
                sdp_mline_index,
            ).await {
                Ok(()) => Ok(Json(serde_json::json!({
                    "type": "ice_candidate",
                    "status": "received",
                    "session_id": request.session_id
                }))),
                Err(e) => {
                    error!("Failed to handle ICE candidate: {}", e);
                    Err(StatusCode::BadRequest)
                }
            }
        }
        WebRtcMessage::Answer { .. } => {
            // For now, return not implemented - answer would come from browser via WebSocket
            Err(StatusCode::NotImplemented)
        }
    }
}