// Serialization utilities for protocol messages

use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use base64::{Engine as _, engine::general_purpose};
use crate::protocol::Message;

/// Serialize a message to JSON bytes
pub fn serialize_message(message: &Message) -> Result<Vec<u8>, SerializationError> {
    serde_json::to_vec(message)
        .map_err(|e| SerializationError::JsonError(e.to_string()))
}

/// Deserialize a message from JSON bytes
pub fn deserialize_message(data: &[u8]) -> Result<Message, SerializationError> {
    serde_json::from_slice(data)
        .map_err(|e| SerializationError::JsonError(e.to_string()))
}

/// Serialize a message to a base64-encoded string
pub fn serialize_message_to_base64(message: &Message) -> Result<String, SerializationError> {
    let bytes = serialize_message(message)?;
    Ok(general_purpose::STANDARD.encode(bytes))
}

/// Deserialize a message from a base64-encoded string
pub fn deserialize_message_from_base64(data: &str) -> Result<Message, SerializationError> {
    let bytes = general_purpose::STANDARD
        .decode(data)
        .map_err(|e| SerializationError::Base64Error(e.to_string()))?;
    deserialize_message(&bytes)
}

/// Serialize a message with length prefix for binary protocols
pub fn serialize_message_with_length(message: &Message) -> Result<Vec<u8>, SerializationError> {
    let data = serialize_message(message)?;
    let len = data.len() as u32;
    
    let mut result = Vec::with_capacity(4 + data.len());
    result.extend_from_slice(&len.to_be_bytes());
    result.extend_from_slice(&data);
    
    Ok(result)
}

/// Deserialize a message with length prefix
pub fn deserialize_message_with_length<R: Read>(reader: &mut R) -> Result<Message, SerializationError> {
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)
        .map_err(|e| SerializationError::IoError(e.to_string()))?;
    
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    let mut data = vec![0u8; len];
    reader.read_exact(&mut data)
        .map_err(|e| SerializationError::IoError(e.to_string()))?;
    
    deserialize_message(&data)
}

/// Errors that can occur during serialization/deserialization
#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("JSON error: {0}")]
    JsonError(String),
    
    #[error("Base64 error: {0}")]
    Base64Error(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Invalid message version")]
    InvalidVersion,
    
    #[error("Unknown message type")]
    UnknownMessageType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MessageType, PROTOCOL_VERSION};
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn test_serialize_deserialize_message() {
        let message = Message::new(
            MessageType::Ping,
            json!({"data": "test"})
        );
        
        let serialized = serialize_message(&message).unwrap();
        let deserialized = deserialize_message(&serialized).unwrap();
        
        assert_eq!(deserialized.message_type, MessageType::Ping);
        assert_eq!(deserialized.version, PROTOCOL_VERSION);
    }

    #[test]
    fn test_base64_serialization() {
        let message = Message::new(
            MessageType::Pong,
            json!({"response": "data"})
        );
        
        let base64 = serialize_message_to_base64(&message).unwrap();
        let deserialized = deserialize_message_from_base64(&base64).unwrap();
        
        assert_eq!(deserialized.message_type, MessageType::Pong);
    }

    #[test]
    fn test_length_prefix_serialization() {
        let message = Message::new(
            MessageType::Ping,
            json!({"test": true})
        );
        
        let serialized = serialize_message_with_length(&message).unwrap();
        assert!(serialized.len() >= 4);
        
        let mut cursor = io::Cursor::new(serialized);
        let deserialized = deserialize_message_with_length(&mut cursor).unwrap();
        
        assert_eq!(deserialized.message_type, MessageType::Ping);
    }
}
