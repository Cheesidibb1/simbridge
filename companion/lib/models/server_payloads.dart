/// Payload models for every server -> client WebSocket message defined in
/// the protocol doc. Each type is parsed from `WsMessage.payload`.
library;

class PairResponsePayload {
  final bool success;
  final String? pairingCode;
  final String? message;

  const PairResponsePayload({required this.success, this.pairingCode, this.message});

  factory PairResponsePayload.fromJson(Map<String, dynamic> json) {
    return PairResponsePayload(
      success: json['success'] as bool? ?? false,
      pairingCode: json['pairing_code'] as String?,
      message: json['message'] as String?,
    );
  }
}

class AuthResponsePayload {
  final bool success;
  final String? sessionToken;
  final DateTime? expiresAt;
  final String? message;

  const AuthResponsePayload({
    required this.success,
    this.sessionToken,
    this.expiresAt,
    this.message,
  });

  factory AuthResponsePayload.fromJson(Map<String, dynamic> json) {
    final rawExpiry = json['expires_at'] as String?;
    return AuthResponsePayload(
      success: json['success'] as bool? ?? false,
      sessionToken: json['session_token'] as String?,
      expiresAt: rawExpiry != null ? DateTime.tryParse(rawExpiry) : null,
      message: json['message'] as String?,
    );
  }
}

enum FrameEncoding {
  h264,
  vp8,
  jpeg,
  png;

  static FrameEncoding fromWire(String value) {
    return FrameEncoding.values.firstWhere(
      (e) => e.name == value,
      orElse: () => FrameEncoding.jpeg,
    );
  }
}

class ScreenFramePayload {
  final String simulatorId;
  final String frameData; // base64
  final FrameEncoding encoding;
  final int width;
  final int height;
  final DateTime timestamp;

  const ScreenFramePayload({
    required this.simulatorId,
    required this.frameData,
    required this.encoding,
    required this.width,
    required this.height,
    required this.timestamp,
  });

  factory ScreenFramePayload.fromJson(Map<String, dynamic> json) {
    return ScreenFramePayload(
      simulatorId: json['simulator_id'] as String,
      frameData: json['frame_data'] as String,
      encoding: FrameEncoding.fromWire(json['encoding'] as String? ?? 'jpeg'),
      width: (json['width'] as num?)?.toInt() ?? 0,
      height: (json['height'] as num?)?.toInt() ?? 0,
      timestamp: DateTime.tryParse(json['timestamp'] as String? ?? '') ?? DateTime.now().toUtc(),
    );
  }
}

class SimNotification {
  final String id;
  final String appName;
  final String title;
  final String body;
  final DateTime timestamp;
  final String? icon;
  final String? action;

  const SimNotification({
    required this.id,
    required this.appName,
    required this.title,
    required this.body,
    required this.timestamp,
    this.icon,
    this.action,
  });

  factory SimNotification.fromJson(Map<String, dynamic> json) {
    return SimNotification(
      id: json['id'] as String,
      appName: json['app_name'] as String? ?? '',
      title: json['title'] as String? ?? '',
      body: json['body'] as String? ?? '',
      timestamp: DateTime.tryParse(json['timestamp'] as String? ?? '') ?? DateTime.now().toUtc(),
      icon: json['icon'] as String?,
      action: json['action'] as String?,
    );
  }
}

class NotificationPayload {
  final String simulatorId;
  final SimNotification notification;

  const NotificationPayload({required this.simulatorId, required this.notification});

  factory NotificationPayload.fromJson(Map<String, dynamic> json) {
    return NotificationPayload(
      simulatorId: json['simulator_id'] as String,
      notification: SimNotification.fromJson(json['notification'] as Map<String, dynamic>),
    );
  }
}

enum RecordingStatusValue {
  recording,
  paused,
  stopped,
  processing,
  completed,
  error;

  static RecordingStatusValue fromWire(String value) {
    return RecordingStatusValue.values.firstWhere(
      (e) => e.name == value,
      orElse: () => RecordingStatusValue.error,
    );
  }
}

class RecordingStatusPayload {
  final String recordingId;
  final RecordingStatusValue status;
  final int durationSeconds;
  final int fileSizeBytes;

  const RecordingStatusPayload({
    required this.recordingId,
    required this.status,
    required this.durationSeconds,
    required this.fileSizeBytes,
  });

  factory RecordingStatusPayload.fromJson(Map<String, dynamic> json) {
    return RecordingStatusPayload(
      recordingId: json['recording_id'] as String,
      status: RecordingStatusValue.fromWire(json['status'] as String? ?? 'error'),
      durationSeconds: (json['duration_seconds'] as num?)?.toInt() ?? 0,
      fileSizeBytes: (json['file_size_bytes'] as num?)?.toInt() ?? 0,
    );
  }
}

class PongPayload {
  final String status;

  const PongPayload({required this.status});

  factory PongPayload.fromJson(Map<String, dynamic> json) {
    return PongPayload(status: json['status'] as String? ?? 'ok');
  }
}

enum ErrorCode {
  authenticationFailed('AuthenticationFailed'),
  invalidRequest('InvalidRequest'),
  simulatorNotFound('SimulatorNotFound'),
  simulatorBusy('SimulatorBusy'),
  connectionError('ConnectionError'),
  streamError('StreamError'),
  fileTransferError('FileTransferError'),
  recordingError('RecordingError'),
  internalError('InternalError'),
  rateLimited('RateLimited'),
  permissionDenied('PermissionDenied'),
  unknown('Unknown');

  final String wire;
  const ErrorCode(this.wire);

  static ErrorCode fromWire(String value) {
    return ErrorCode.values.firstWhere(
      (e) => e.wire == value,
      orElse: () => ErrorCode.unknown,
    );
  }
}

class ErrorPayload {
  final ErrorCode code;
  final String message;
  final String? details;

  const ErrorPayload({required this.code, required this.message, this.details});

  factory ErrorPayload.fromJson(Map<String, dynamic> json) {
    return ErrorPayload(
      code: ErrorCode.fromWire(json['code'] as String? ?? 'Unknown'),
      message: json['message'] as String? ?? 'Unknown error',
      details: json['details'] as String?,
    );
  }
}
