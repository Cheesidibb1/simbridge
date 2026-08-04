// WebSocket server for SimBridge

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
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
}

/// Handle WebSocket upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketServerState>,
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
                                
                                // TODO: Route message to appropriate handler
                                
                                // Send response
                                let response = ProtocolMessage::new(
                                    simbridge_shared::protocol::MessageType::Pong,
                                    serde_json::json!({"status": "ok"})
                                );
                                
                                match serialize_message(&response) {
                                    Ok(serialized) => {
                                        if sender.send(Message::Text(serialized)).await.is_err() {
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
