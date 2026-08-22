/// Models for `/api/v1/sessions` and the `SessionInfo` WS payload.
library;

enum SessionStatus {
  active,
  paused,
  terminated;

  static SessionStatus fromWire(String value) {
    return SessionStatus.values.firstWhere(
      (s) => s.name == value,
      orElse: () => SessionStatus.terminated,
    );
  }

  String toWire() => name;
}

enum StreamQuality {
  low,
  medium,
  high,
  ultra;

  static StreamQuality fromWire(String value) {
    return StreamQuality.values.firstWhere(
      (s) => s.name == value,
      orElse: () => StreamQuality.medium,
    );
  }

  String toWire() => name;
}

class StreamConfig {
  final StreamQuality quality;
  final int fps;
  final bool audioEnabled;
  final String? videoCodec;

  const StreamConfig({
    this.quality = StreamQuality.high,
    this.fps = 30,
    this.audioEnabled = false,
    this.videoCodec,
  });

  factory StreamConfig.fromJson(Map<String, dynamic> json) {
    return StreamConfig(
      quality: StreamQuality.fromWire(json['quality'] as String? ?? 'high'),
      fps: (json['fps'] as num?)?.toInt() ?? 30,
      audioEnabled: json['audio_enabled'] as bool? ?? false,
      videoCodec: json['video_codec'] as String?,
    );
  }

  Map<String, dynamic> toJson() => {
        'quality': quality.toWire(),
        'fps': fps,
        'audio_enabled': audioEnabled,
        if (videoCodec != null) 'video_codec': videoCodec,
      };

  StreamConfig copyWith({
    StreamQuality? quality,
    int? fps,
    bool? audioEnabled,
  }) {
    return StreamConfig(
      quality: quality ?? this.quality,
      fps: fps ?? this.fps,
      audioEnabled: audioEnabled ?? this.audioEnabled,
      videoCodec: videoCodec,
    );
  }
}

/// The response body returned by `POST /api/v1/sessions`.
class Session {
  final String sessionId;
  final String simulatorId;
  final String status;

  const Session({
    required this.sessionId,
    required this.simulatorId,
    required this.status,
  });

  factory Session.fromJson(Map<String, dynamic> json) {
    return Session(
      sessionId: json['session_id'] as String,
      simulatorId: json['simulator_id'] as String,
      status: json['status'] as String? ?? 'active',
    );
  }
}

/// The richer `SessionInfo` WS payload (bidirectional message).
class SessionInfo {
  final String sessionId;
  final String deviceId;
  final String simulatorId;
  final DateTime connectedAt;
  final SessionStatus status;

  const SessionInfo({
    required this.sessionId,
    required this.deviceId,
    required this.simulatorId,
    required this.connectedAt,
    required this.status,
  });

  factory SessionInfo.fromJson(Map<String, dynamic> json) {
    return SessionInfo(
      sessionId: json['session_id'] as String,
      deviceId: json['device_id'] as String,
      simulatorId: json['simulator_id'] as String,
      connectedAt: DateTime.parse(json['connected_at'] as String),
      status: SessionStatus.fromWire(json['status'] as String? ?? 'active'),
    );
  }
}
