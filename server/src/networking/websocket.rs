// WebSocket server for SimBridge

use crate::adapters::android::{resolve_adb_path, AndroidEmulatorAdapter};
use crate::adapters::interface::SimulatorAdapter;
use crate::adapters::ios::IosSimulatorAdapter;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use base64::Engine;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use simbridge_shared::protocol::{
    deserialize_message, serialize_message, AuthRequestPayload, DeviceButton, DeviceButtonPayload,
    FrameEncoding, GesturePayload, GpsUpdatePayload, Message as ProtocolMessage, MessageType,
    ScreenFramePayload, StreamQuality, TouchEventPayload,
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info};

/// WebSocket server state
#[derive(Clone)]
pub struct WebSocketServerState {
    // TODO: Add session manager, auth manager, etc.
    pub clients: Arc<RwLock<Vec<String>>>,
    password: Arc<String>,
}

impl WebSocketServerState {
    pub fn new(password: String) -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
            password: Arc::new(password),
        }
    }

    pub fn with_webrtc_manager(_webrtc_manager: Arc<()>, password: String) -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
            password: Arc::new(password),
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
    let (frame_sender, mut frame_receiver) = mpsc::unbounded_channel();
    let mut screen_refresh_task: Option<tokio::task::JoinHandle<()>> = None;
    let challenge = uuid::Uuid::new_v4().simple().to_string();

    let challenge_message = ProtocolMessage::new(
        MessageType::AuthChallenge,
        serde_json::json!({"challenge": challenge}),
    );
    match serialize_message(&challenge_message) {
        Ok(serialized) => {
            if sender.send(Message::Binary(serialized)).await.is_err() {
                return;
            }
        }
        Err(error) => {
            error!("Failed to serialize authentication challenge: {}", error);
            return;
        }
    }

    info!("WebSocket client connected");
    let mut authenticated = false;

    // Add client to list
    {
        let mut clients = state.clients.write().await;
        clients.push("client".to_string());
    }

    // Handle incoming messages and frames produced by the screen refresh task.
    loop {
        tokio::select! {
            Some(frame) = frame_receiver.recv() => {
                match serialize_message(&frame) {
                    Ok(serialized) => {
                        if sender.send(Message::Binary(serialized)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize screen frame: {}", e);
                        break;
                    }
                }
            }
            result = receiver.next() => {
                let Some(result) = result else { break };
                match result {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        // Deserialize protocol message
                        match deserialize_message(text.as_bytes()) {
                            Ok(protocol_msg) => {
                                info!("Received message: {:?}", protocol_msg.message_type);

                                let mut close_after_response = false;
                                let response = if !authenticated {
                                    let request = if protocol_msg.message_type == MessageType::AuthRequest {
                                        serde_json::from_value::<AuthRequestPayload>(protocol_msg.payload.clone()).ok()
                                    } else {
                                        None
                                    };
                                    let valid = request.as_ref().is_some_and(|request| {
                                        verify_password_proof(
                                            &state.password,
                                            &challenge,
                                            request.challenge_response.as_deref(),
                                        )
                                    });
                                    if valid {
                                        authenticated = true;
                                        ProtocolMessage::new(
                                            MessageType::AuthResponse,
                                            serde_json::json!({"success": true}),
                                        )
                                    } else {
                                        close_after_response = true;
                                        ProtocolMessage::new(
                                            MessageType::AuthResponse,
                                            serde_json::json!({
                                                "success": false,
                                                "message": "Authentication failed",
                                            }),
                                        )
                                    }
                                } else {
                                    // Route based on message type after authentication.
                                    match protocol_msg.message_type {
                                    MessageType::ConnectSimulator => {
                                        if let Some(task) = screen_refresh_task.take() {
                                            task.abort();
                                        }

                                        let response = handle_connect_simulator(&protocol_msg).await;
                                        if response.message_type == MessageType::ScreenFrame {
                                            if let Some(simulator_id) = protocol_msg
                                                .payload
                                                .get("simulator_id")
                                                .and_then(|value| value.as_str())
                                            {
                                                let (quality, fps) = stream_preferences(&protocol_msg);
                                                screen_refresh_task = Some(start_screen_refresh(
                                                    simulator_id.to_string(),
                                                    quality,
                                                    fps,
                                                    frame_sender.clone(),
                                                ));
                                            }
                                        }
                                        response
                                    }
                                    MessageType::DisconnectSimulator => {
                                        if let Some(task) = screen_refresh_task.take() {
                                            task.abort();
                                        }
                                        ProtocolMessage::new(
                                            MessageType::Pong,
                                            serde_json::json!({"status": "ok"}),
                                        )
                                    }
                                    MessageType::TouchEvent => {
                                        handle_touch_event(&protocol_msg).await
                                    }
                                    MessageType::Gesture => {
                                        handle_gesture(&protocol_msg).await
                                    }
                                    MessageType::GpsUpdate => {
                                        handle_gps_update(&protocol_msg).await
                                    }
                                    MessageType::DeviceButton => {
                                        handle_device_button(&protocol_msg).await
                                    }
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
                                if close_after_response {
                                    let _ = sender.send(Message::Close(None)).await;
                                    break;
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
        }
    }

    if let Some(task) = screen_refresh_task {
        task.abort();
    }

    // Remove client from list
    {
        let mut clients = state.clients.write().await;
        clients.retain(|c| c != "client");
    }

    info!("WebSocket client disconnected");
}

fn verify_password_proof(password: &str, challenge: &str, proof: Option<&str>) -> bool {
    let Some(proof) = proof.and_then(|value| hex::decode(value).ok()) else {
        return false;
    };
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(password.as_bytes()) else {
        return false;
    };
    mac.update(challenge.as_bytes());
    mac.verify_slice(&proof).is_ok()
}

async fn handle_touch_event(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    let payload = match serde_json::from_value::<TouchEventPayload>(protocol_msg.payload.clone()) {
        Ok(payload) => payload,
        Err(error) => return make_input_error(format!("Invalid touch event: {}", error)),
    };
    let simulator_id = payload.simulator_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut adapter = AndroidEmulatorAdapter::new(simulator_id, String::new())
            .with_adb_path(resolve_adb_path());
        futures::executor::block_on(adapter.send_touch_event(payload))
    })
    .await;
    match result {
        Ok(Ok(())) => ProtocolMessage::new(MessageType::Pong, serde_json::json!({"status": "ok"})),
        Ok(Err(error)) => make_input_error(error.to_string()),
        Err(error) => make_input_error(format!("Touch dispatch task failed: {}", error)),
    }
}

async fn handle_gesture(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    let payload = match serde_json::from_value::<GesturePayload>(protocol_msg.payload.clone()) {
        Ok(payload) => payload,
        Err(error) => return make_input_error(format!("Invalid gesture: {}", error)),
    };
    let simulator_id = payload.simulator_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut adapter = AndroidEmulatorAdapter::new(simulator_id, String::new())
            .with_adb_path(resolve_adb_path());
        futures::executor::block_on(adapter.send_gesture(payload))
    })
    .await;
    match result {
        Ok(Ok(())) => ProtocolMessage::new(MessageType::Pong, serde_json::json!({"status": "ok"})),
        Ok(Err(error)) => make_input_error(error.to_string()),
        Err(error) => make_input_error(format!("Gesture dispatch task failed: {}", error)),
    }
}

async fn handle_gps_update(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    let payload = match serde_json::from_value::<GpsUpdatePayload>(protocol_msg.payload.clone()) {
        Ok(payload) => payload,
        Err(error) => return make_input_error(format!("Invalid GPS update: {}", error)),
    };
    let simulator_id = payload.simulator_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut adapter = AndroidEmulatorAdapter::new(simulator_id, String::new())
            .with_adb_path(resolve_adb_path());
        futures::executor::block_on(adapter.set_location(payload.location))
    })
    .await;
    match result {
        Ok(Ok(())) => ProtocolMessage::new(MessageType::Pong, serde_json::json!({"status": "ok"})),
        Ok(Err(error)) => make_input_error(error.to_string()),
        Err(error) => make_input_error(format!("GPS dispatch task failed: {}", error)),
    }
}

async fn handle_device_button(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    let payload = match serde_json::from_value::<DeviceButtonPayload>(protocol_msg.payload.clone())
    {
        Ok(payload) => payload,
        Err(error) => return make_input_error(format!("Invalid device button: {}", error)),
    };
    let simulator_id = payload.simulator_id;

    if matches!(payload.button, DeviceButton::Screenshot) {
        return match capture_screen_frame(&simulator_id).await {
            Ok(Ok(frame)) => make_screen_frame_message(simulator_id, frame, StreamQuality::Low),
            Ok(Err(error)) => make_input_error(error.to_string()),
            Err(error) => make_input_error(format!("Screenshot task failed: {}", error)),
        };
    }

    let button = payload.button;
    let result = tokio::task::spawn_blocking(move || {
        futures::executor::block_on(async move {
            if is_ios_simulator_id(&simulator_id) {
                let mut adapter = IosSimulatorAdapter::new(simulator_id, String::new());
                adapter.press_button(button).await
            } else {
                let mut adapter = AndroidEmulatorAdapter::new(simulator_id, String::new())
                    .with_adb_path(resolve_adb_path());
                adapter.press_button(button).await
            }
        })
    })
    .await;

    match result {
        Ok(Ok(())) => ProtocolMessage::new(MessageType::Pong, serde_json::json!({"status": "ok"})),
        Ok(Err(error)) => make_input_error(error.to_string()),
        Err(error) => make_input_error(format!("Device button task failed: {}", error)),
    }
}

fn is_ios_simulator_id(simulator_id: &str) -> bool {
    simulator_id.starts_with("ios-")
        || (simulator_id.len() == 36 && simulator_id.matches('-').count() == 4)
        || (simulator_id.len() == 40
            && simulator_id
                .chars()
                .all(|character| character.is_ascii_hexdigit()))
}

fn make_input_error(message: String) -> ProtocolMessage {
    ProtocolMessage::new(
        MessageType::Error,
        serde_json::json!({"code": "input_dispatch_failed", "message": message}),
    )
}

/// Capture one real frame when the companion completes the simulator handshake.
/// A first frame is enough to unblock the mirror while the streaming path is
/// brought up; failures are returned as protocol errors instead of leaving the
/// client in an indefinite loading state.
async fn handle_connect_simulator(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    let simulator_id = protocol_msg
        .payload
        .get("simulator_id")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let (quality, _) = stream_preferences(protocol_msg);

    match capture_screen_frame(&simulator_id).await {
        Ok(Ok(frame)) => make_screen_frame_message(simulator_id, frame, quality),
        Ok(Err(error)) => ProtocolMessage::new(
            MessageType::Error,
            serde_json::json!({
                "code": "screen_capture_failed",
                "message": error.to_string(),
            }),
        ),
        Err(error) => ProtocolMessage::new(
            MessageType::Error,
            serde_json::json!({
                "code": "screen_capture_failed",
                "message": format!("Screen capture task failed: {}", error),
            }),
        ),
    }
}

fn start_screen_refresh(
    simulator_id: String,
    quality: StreamQuality,
    fps: u32,
    frame_sender: mpsc::UnboundedSender<ProtocolMessage>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval_ms = 1000 / u64::from(fps.clamp(1, 15));
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            match capture_screen_frame(&simulator_id).await {
                Ok(Ok(frame)) => {
                    if frame_sender
                        .send(make_screen_frame_message(
                            simulator_id.clone(),
                            frame,
                            quality,
                        ))
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(Err(error)) => {
                    error!("Screen refresh failed for {}: {}", simulator_id, error);
                }
                Err(error) => {
                    error!("Screen refresh failed for {}: {}", simulator_id, error);
                }
            }
        }
    })
}

fn stream_preferences(protocol_msg: &ProtocolMessage) -> (StreamQuality, u32) {
    let config = protocol_msg.payload.get("stream_config");
    let quality = config
        .and_then(|config| config.get("quality"))
        .cloned()
        .and_then(|value| serde_json::from_value::<StreamQuality>(value).ok())
        .unwrap_or(StreamQuality::Low);
    let fps = config
        .and_then(|config| config.get("fps"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(3)
        .clamp(1, 15) as u32;
    (quality, fps)
}

fn make_screen_frame_message(
    simulator_id: String,
    frame: Vec<u8>,
    quality: StreamQuality,
) -> ProtocolMessage {
    let decoded = image::load_from_memory(&frame).ok();
    let original_dimensions = decoded
        .as_ref()
        .map(|image| (image.width(), image.height()));
    let encoded = decoded.and_then(|decoded| {
        let max_dimension = match quality {
            StreamQuality::Low => 1600,
            StreamQuality::Medium => 1920,
            StreamQuality::High => 2560,
            StreamQuality::Ultra => 3840,
        };
        let resized = decoded.resize(
            max_dimension,
            max_dimension,
            image::imageops::FilterType::Triangle,
        );
        let jpeg_quality = match quality {
            StreamQuality::Low => 55,
            StreamQuality::Medium => 68,
            StreamQuality::High => 80,
            StreamQuality::Ultra => 90,
        };
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, jpeg_quality)
            .encode_image(&resized)
            .ok()?;
        Some(jpeg)
    });
    let (frame, encoding) = encoded
        .map(|jpeg| (jpeg, FrameEncoding::Jpeg))
        .unwrap_or((frame, FrameEncoding::Png));
    let (width, height) = original_dimensions.unwrap_or((0, 0));
    let payload = ScreenFramePayload {
        simulator_id,
        frame_data: base64::engine::general_purpose::STANDARD.encode(frame),
        encoding,
        width,
        height,
        timestamp: chrono::Utc::now(),
    };
    ProtocolMessage::new(
        MessageType::ScreenFrame,
        serde_json::to_value(payload).unwrap(),
    )
}

async fn capture_screen_frame(
    simulator_id: &str,
) -> Result<Result<Vec<u8>, crate::adapters::interface::AdapterError>, tokio::task::JoinError> {
    let simulator_id = simulator_id.to_string();
    tokio::task::spawn_blocking(move || {
        futures::executor::block_on(async move {
            if is_ios_simulator_id(&simulator_id) {
                let mut adapter = IosSimulatorAdapter::new(simulator_id, String::new());
                adapter.start_screenshot().await
            } else {
                let mut adapter = AndroidEmulatorAdapter::new(simulator_id, String::new())
                    .with_adb_path(resolve_adb_path());
                adapter.start_screenshot().await
            }
        })
    })
    .await
}

/// Handle WebRTC offer message
async fn handle_webrtc_offer(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    // Extract SDP and session info from payload
    let payload = &protocol_msg.payload;
    let sdp = payload
        .get("sdp")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let stream_id = payload
        .get("stream_id")
        .and_then(|v| v.as_str())
        .unwrap_or("stream-1");

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
        }),
    )
}

/// Handle ICE candidate message
async fn handle_ice_candidate(protocol_msg: &ProtocolMessage) -> ProtocolMessage {
    let payload = &protocol_msg.payload;
    let candidate = payload
        .get("candidate")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let stream_id = payload
        .get("stream_id")
        .and_then(|v| v.as_str())
        .unwrap_or("stream-1");

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
        }),
    )
}
