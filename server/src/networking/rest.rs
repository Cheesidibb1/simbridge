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
use tracing::info;

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

/// Create REST API router
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/simulators", get(list_simulators))
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/sessions", post(create_session))
        .route("/api/v1/sessions/:id", delete(delete_session))
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
    
    // TODO: Create actual session
    let mut sessions = state.sessions.write().await;
    sessions.push(req.simulator_id.clone());
    
    Ok(Json(serde_json::json!({
        "session_id": "new-session-id",
        "simulator_id": req.simulator_id,
        "status": "active"
    })))
}

/// Delete a session
async fn delete_session(
    State(state): State<RestServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting session {}", id);
    
    // TODO: Delete actual session
    let mut sessions = state.sessions.write().await;
    sessions.retain(|s| s != &id);
    
    Ok(Json(serde_json::json!({
        "status": "deleted",
        "session_id": id
    })))
}
