/// The `message_type`/`version`/`timestamp`/`request_id`/`payload` envelope
/// used by every WebSocket message, plus the enum of message types the
/// SimBridge protocol defines.
library;

enum WsMessageType {
  // Client -> Server
  pairRequest('pair_request'),
  authRequest('auth_request'),
  simulatorList('simulator_list'),
  connectSimulator('connect_simulator'),
  disconnectSimulator('disconnect_simulator'),
  touchEvent('touch_event'),
  gesture('gesture'),
  gpsUpdate('gps_update'),
  headingUpdate('heading_update'),
  motionUpdate('motion_update'),
  deviceButton('device_button'),
  clipboardSync('clipboard_sync'),
  fileTransfer('file_transfer'),
  startRecording('start_recording'),
  stopRecording('stop_recording'),
  getRecordings('get_recordings'),
  ping('ping'),

  // Server -> Client
  pairResponse('pair_response'),
  authResponse('auth_response'),
  screenFrame('screen_frame'),
  notification('notification'),
  recordingStatus('recording_status'),
  pong('pong'),
  error('error'),

  // Bidirectional
  settingsUpdate('settings_update'),
  sessionInfo('session_info'),
  metricsUpdate('metrics_update'),

  // WebRTC signaling
  webrtcOffer('webrtc_offer'),
  webrtcAnswer('webrtc_answer'),
  webrtcIceCandidate('webrtc_ice_candidate');

  final String wire;
  const WsMessageType(this.wire);

  /// Returns null (rather than throwing) for a type string the client
  /// doesn't recognize, so a protocol addition on the server never crashes
  /// an older client — it just surfaces as an unhandled message.
  static WsMessageType? fromWire(String value) {
    for (final candidate in WsMessageType.values) {
      if (candidate.wire == value) return candidate;
    }
    return null;
  }
}

/// The envelope every WebSocket message is wrapped in, per the protocol doc:
/// ```json
/// {
///   "message_type": "MessageType",
///   "version": 1,
///   "timestamp": "2024-01-15T10:30:00Z",
///   "request_id": "optional-uuid",
///   "payload": { ... }
/// }
/// ```
class WsMessage {
  /// Parsed enum value, or null if the server sent a type this client
  /// build doesn't know about yet.
  final WsMessageType? type;

  /// The raw `message_type` string as received/sent, kept even when [type]
  /// is null so callers can log or ignore it deliberately.
  final String rawType;

  final int version;
  final DateTime timestamp;
  final String? requestId;
  final Map<String, dynamic> payload;

  const WsMessage({
    required this.type,
    required this.rawType,
    required this.timestamp,
    required this.payload,
    this.version = 1,
    this.requestId,
  });

  factory WsMessage.fromJson(Map<String, dynamic> json) {
    final rawType = json['message_type'] as String? ?? '';
    final rawTimestamp = json['timestamp'] as String?;
    return WsMessage(
      type: WsMessageType.fromWire(rawType),
      rawType: rawType,
      version: (json['version'] as num?)?.toInt() ?? 1,
      timestamp: rawTimestamp != null
          ? (DateTime.tryParse(rawTimestamp) ?? DateTime.now().toUtc())
          : DateTime.now().toUtc(),
      requestId: json['request_id'] as String?,
      payload: (json['payload'] as Map<String, dynamic>?) ?? const {},
    );
  }

  Map<String, dynamic> toJson() => {
        'message_type': rawType,
        'version': version,
        'timestamp': timestamp.toIso8601String(),
        if (requestId != null) 'request_id': requestId,
        'payload': payload,
      };

  /// Convenience constructor for messages this client originates.
  factory WsMessage.outgoing({
    required WsMessageType type,
    Map<String, dynamic> payload = const {},
    String? requestId,
    int version = 1,
  }) {
    return WsMessage(
      type: type,
      rawType: type.wire,
      version: version,
      timestamp: DateTime.now().toUtc(),
      requestId: requestId,
      payload: payload,
    );
  }

  @override
  String toString() => 'WsMessage(${rawType.isEmpty ? '<unknown>' : rawType}, requestId: $requestId)';
}
