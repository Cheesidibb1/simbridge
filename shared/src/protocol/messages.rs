// Message types for SimBridge WebSocket and REST communication

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Base message structure for all WebSocket communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,
    pub payload: serde_json::Value,
}

impl Message {
    pub fn new(message_type: MessageType, payload: serde_json::Value) -> Self {
        Self {
            message_type,
            version: PROTOCOL_VERSION,
            timestamp: Utc::now(),
            request_id: None,
            payload,
        }
    }

    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// All message types in the protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    // Client → Server messages
    PairRequest,
    AuthRequest,
    SimulatorList,
    ConnectSimulator,
    DisconnectSimulator,
    TouchEvent,
    Gesture,
    GpsUpdate,
    HeadingUpdate,
    MotionUpdate,
    DeviceButton,
    ClipboardSync,
    FileTransfer,
    StartRecording,
    StopRecording,
    GetRecordings,
    Ping,

    // Server → Client messages (unique names only)
    PairResponse,
    AuthResponse,
    ScreenFrame,
    Notification,
    RecordingStatus,
    RecordingData,
    Pong,
    Error,

    // Bidirectional
    SettingsUpdate,
    SessionInfo,
    MetricsUpdate,

    // WebRTC-specific messages (bidirectional)
    WebrtcOffer,
    WebrtcAnswer,
    WebrtcIceCandidate,
}

// Client → Server request payloads

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequestPayload {
    pub device_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequestPayload {
    pub device_id: String,
    pub token: String,
    pub challenge_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectSimulatorPayload {
    pub simulator_id: String,
    pub stream_config: StreamConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub quality: StreamQuality,
    pub fps: u32,
    pub audio_enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchEventPayload {
    pub simulator_id: String,
    pub touches: Vec<Touch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Touch {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub phase: TouchPhase,
    pub force: Option<f64>,
    pub major_radius: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TouchPhase {
    Began,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GesturePayload {
    pub simulator_id: String,
    pub gesture_type: GestureType,
    pub data: GestureData,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureType {
    Swipe,
    Pinch,
    Rotation,
    LongPress,
    DoubleTap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GestureData {
    Swipe { direction: SwipeDirection, distance: f64 },
    Pinch { scale: f64, center: (f64, f64) },
    Rotation { angle: f64, center: (f64, f64) },
    LongPress { location: (f64, f64), duration: f64 },
    DoubleTap { location: (f64, f64) },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsUpdatePayload {
    pub simulator_id: String,
    pub location: GpsLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingUpdatePayload {
    pub simulator_id: String,
    pub heading: f64,
    pub accuracy: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionUpdatePayload {
    pub simulator_id: String,
    pub acceleration: Option<(f64, f64, f64)>,
    pub gravity: Option<(f64, f64, f64)>,
    pub rotation_rate: Option<(f64, f64, f64)>,
    pub attitude: Option<Attitude>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attitude {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceButtonPayload {
    pub simulator_id: String,
    pub button: DeviceButton,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceButton {
    Home,
    Back,
    AppSwitcher,
    Lock,
    Unlock,
    VolumeUp,
    VolumeDown,
    Mute,
    RotateLeft,
    RotateRight,
    Shake,
    Screenshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncPayload {
    pub simulator_id: Option<String>,
    pub content: String,
    pub content_type: ClipboardContentType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardContentType {
    Text,
    Image,
    Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferPayload {
    pub transfer_id: Uuid,
    pub simulator_id: Option<String>,
    pub direction: TransferDirection,
    pub file_name: String,
    pub file_size: u64,
    pub chunk_data: Option<String>, // Base64 encoded
    pub chunk_index: Option<u32>,
    pub total_chunks: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    Upload,
    Download,
}

// Server → Client response payloads

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairResponsePayload {
    pub success: bool,
    pub pairing_code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponsePayload {
    pub success: bool,
    pub session_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorListPayload {
    pub simulators: Vec<SimulatorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorInfo {
    pub id: String,
    pub name: String,
    pub platform: SimulatorPlatform,
    pub os_version: String,
    pub status: SimulatorStatus,
    pub screen_size: ScreenSize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimulatorPlatform {
    Ios,
    Android,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulatorStatus {
    Available,
    Busy,
    Offline,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenFramePayload {
    pub simulator_id: String,
    pub frame_data: String, // Base64 encoded
    pub encoding: FrameEncoding,
    pub width: u32,
    pub height: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameEncoding {
    H264,
    Vp8,
    Jpeg,
    Png,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub simulator_id: String,
    pub notification: Notification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatusPayload {
    pub recording_id: Uuid,
    pub status: RecordingStatus,
    pub duration_seconds: Option<u64>,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingStatus {
    Recording,
    Paused,
    Stopped,
    Processing,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    AuthenticationFailed,
    InvalidRequest,
    SimulatorNotFound,
    SimulatorBusy,
    ConnectionError,
    StreamError,
    FileTransferError,
    RecordingError,
    InternalError,
    RateLimited,
    PermissionDenied,
}

// Common types

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Android,
    Ios,
    Desktop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfoPayload {
    pub session_id: Uuid,
    pub device_id: String,
    pub simulator_id: String,
    pub connected_at: DateTime<Utc>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Paused,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdatePayload {
    pub simulator_id: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub bandwidth: f64,
    pub fps: f64,
    pub latency: f64,
}
