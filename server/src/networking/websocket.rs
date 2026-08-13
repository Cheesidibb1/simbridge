// WebSocket server for SimBridge

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use simbridge_shared::protocol::{Message as ProtocolMessage, serialize_message, deserialize_message};

/// WebSocket server state
#[derive(Clone)]
pub struct WebSocketServerState {
    // TODO: Add session manager, auth manager, etc.
    pub clients: Arc<RwLock<Vec<String>>>,
}

impl WebSocketServerState {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_webrtc_manager(_webrtc_manager: Arc<()>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// Handle WebSocket upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    state: WebSocketServerState,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: WebSocketServerState) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket client connected");

    // Add client to list
    {
        let mut clients = state.clients.write().await;
        clients.push("client".to_string());
    }

    // Handle incoming messages
    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        // Deserialize protocol message
                        match deserialize_message(text.as_bytes()) {
                            Ok(protocol_msg) => {
                                info!("Received message: {:?}", protocol_msg.message_type);

                                // Route based on message type
                                let response = match protocol_msg.message_type {
                                    simbridge_shared::protocol::MessageType::WebrtcOffer => {
                                        // Handle WebRTC offer - forward to signaling handler
                                        handle_webrtc_offer(&protocol_msg).await
                                    }
                                    simbridge_shared::protocol::MessageType::WebrtcIceCandidate => {
                                        // Handle ICE candidate
                                        handle_ice_candidate(&protocol_msg).await
                                    }
                                    _ => {
                                        // Default response for other messages
                                        ProtocolMessage::new(
                                            simbridge_shared::protocol::MessageType::Pong,
                                            serde_json::json!({"status": "ok"})
                                        )
                                    }
                                };

                                match serialize_message(&response) {
                                    Ok(serialized) => {
                                        if sender.send(Message::Binary(serialized)).await.is_err() {
                                            error!("Failed to send response");
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to serialize response: {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize message: {}", e);
                            }
                        }
                    }
                    Message::Close(_) => {
                        info!("Client requested close");
                        break;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Remove client from list
    {
        let mut clients = state.clients.write().await;
        clients.retain(|c| c != "client");
    }

    info!("WebSocket client disconnected");
}

/// Handle WebRTC offer message
async fn handle_webrtc_offer(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    // Extract SDP and session info from payload
    let payload = &protocol_msg.payload;
    let sdp = payload.get("sdp").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let session_id = payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    let stream_id = payload.get("stream_id").and_then(|v| v.as_str()).unwrap_or("stream-1");

    info!("Handling WebRTC offer for session: {}", session_id);

    // In a real implementation, this would:
    // 1. Parse the session UUID
    // 2. Store the offer in the signaling manager
    // 3. Generate an answer SDP
    // 4. Return the answer to the client

    ProtocolMessage::new(
        simbridge_shared::protocol::MessageType::WebrtcAnswer,
        serde_json::json!({
            "session_id": session_id,
            "stream_id": stream_id,
            "sdp": sdp, // In production, this would be a generated answer
            "type": "answer"
        })
    )
}

/// Handle ICE candidate message
async fn handle_ice_candidate(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    let payload = &protocol_msg.payload;
    let candidate = payload.get("candidate").and_then(|v| v.as_str()).unwrap_or("");
    let session_id = payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    let stream_id = payload.get("stream_id").and_then(|v| v.as_str()).unwrap_or("stream-1");

    info!("Handling ICE candidate for session: {}", session_id);

    // In a real implementation, this would:
    // 1. Parse the session UUID
    // 2. Add the candidate to the session's ICE candidates list

    ProtocolMessage::new(
        simbridge_shared::protocol::MessageType::WebrtcIceCandidate,
        serde_json::json!({
            "session_id": session_id,
            "stream_id": stream_id,
            "candidate": candidate,
            "type": "ice_candidate"
        })
    )
}
