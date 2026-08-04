// WebSocket client and server utilities

use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream};
use tokio::net::TcpStream;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::protocol::{Message as ProtocolMessage, serialize_message, deserialize_message};
use thiserror::Error;

/// WebSocket client
pub struct WebSocketClient {
    url: String,
    write: Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
}

impl WebSocketClient {
    pub async fn connect(url: &str) -> Result<Self, WebSocketError> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            url: url.to_string(),
            write: Arc::new(Mutex::new(ws_stream)),
        })
    }

    pub async fn send(&self, message: &ProtocolMessage) -> Result<(), WebSocketError> {
        let serialized = serialize_message(message)
            .map_err(|e| WebSocketError::SerializationError(e.to_string()))?;
        
        let ws_message = Message::Text(serialized);
        
        let mut write = self.write.lock().await;
        write.send(ws_message)
            .await
            .map_err(|e| WebSocketError::SendError(e.to_string()))?;
        
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<ProtocolMessage, WebSocketError> {
        let mut write = self.write.lock().await;
        
        if let Some(message) = write.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    deserialize_message(text.as_bytes())
                        .map_err(|e| WebSocketError::DeserializationError(e.to_string()))
                }
                Ok(Message::Binary(data)) => {
                    deserialize_message(&data)
                        .map_err(|e| WebSocketError::DeserializationError(e.to_string()))
                }
                Ok(Message::Close(_)) => Err(WebSocketError::ConnectionClosed),
                Err(e) => Err(WebSocketError::ReceiveError(e.to_string())),
                _ => Err(WebSocketError::UnexpectedMessageType),
            }
        } else {
            Err(WebSocketError::ConnectionClosed)
        }
    }
}

/// WebSocket errors
#[derive(Debug, Error)]
pub enum WebSocketError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Send error: {0}")]
    SendError(String),
    
    #[error("Receive error: {0}")]
    ReceiveError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Unexpected message type")]
    UnexpectedMessageType,
}
