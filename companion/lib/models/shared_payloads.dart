/// Payload models for bidirectional messages (`SettingsUpdate`,
/// `MetricsUpdate`; `SessionInfo` lives in session.dart) and the WebRTC
/// signaling messages. WebRTC payloads are modeled now so the wire format
/// is ready even though this client drives its screen mirror over the
/// WebSocket `ScreenFrame` path per the doc's current backend limitations.
library;

class MetricsUpdatePayload {
  final String simulatorId;
  final double cpuUsage;
  final int memoryUsage;
  final double bandwidth;
  final double fps;
  final double latency;

  const MetricsUpdatePayload({
    required this.simulatorId,
    required this.cpuUsage,
    required this.memoryUsage,
    required this.bandwidth,
    required this.fps,
    required this.latency,
  });

  factory MetricsUpdatePayload.fromJson(Map<String, dynamic> json) {
    return MetricsUpdatePayload(
      simulatorId: json['simulator_id'] as String,
      cpuUsage: (json['cpu_usage'] as num?)?.toDouble() ?? 0,
      memoryUsage: (json['memory_usage'] as num?)?.toInt() ?? 0,
      bandwidth: (json['bandwidth'] as num?)?.toDouble() ?? 0,
      fps: (json['fps'] as num?)?.toDouble() ?? 0,
      latency: (json['latency'] as num?)?.toDouble() ?? 0,
    );
  }
}

/// The doc marks this payload's format as TBD server-side; this client
/// treats it as an opaque map so it can round-trip whatever shape the
/// server eventually settles on without a protocol-breaking client update.
class SettingsUpdatePayload {
  final Map<String, dynamic> settings;

  const SettingsUpdatePayload(this.settings);

  factory SettingsUpdatePayload.fromJson(Map<String, dynamic> json) {
    return SettingsUpdatePayload(json);
  }

  Map<String, dynamic> toJson() => settings;
}

class WebrtcOfferPayload {
  final String sdp;
  final String sessionId;
  final String streamId;

  const WebrtcOfferPayload({
    required this.sdp,
    required this.sessionId,
    required this.streamId,
  });

  factory WebrtcOfferPayload.fromJson(Map<String, dynamic> json) {
    return WebrtcOfferPayload(
      sdp: json['sdp'] as String,
      sessionId: json['session_id'] as String,
      streamId: json['stream_id'] as String,
    );
  }

  Map<String, dynamic> toJson() => {
        'sdp': sdp,
        'session_id': sessionId,
        'stream_id': streamId,
      };
}

class WebrtcAnswerPayload {
  final String sdp;
  final String sessionId;
  final String streamId;

  const WebrtcAnswerPayload({
    required this.sdp,
    required this.sessionId,
    required this.streamId,
  });

  factory WebrtcAnswerPayload.fromJson(Map<String, dynamic> json) {
    return WebrtcAnswerPayload(
      sdp: json['sdp'] as String,
      sessionId: json['session_id'] as String,
      streamId: json['stream_id'] as String,
    );
  }

  Map<String, dynamic> toJson() => {
        'sdp': sdp,
        'session_id': sessionId,
        'stream_id': streamId,
      };
}

class WebrtcIceCandidatePayload {
  final String candidate;
  final String sdpMid;
  final int sdpMlineIndex;
  final String sessionId;
  final String streamId;

  const WebrtcIceCandidatePayload({
    required this.candidate,
    required this.sdpMid,
    required this.sdpMlineIndex,
    required this.sessionId,
    required this.streamId,
  });

  factory WebrtcIceCandidatePayload.fromJson(Map<String, dynamic> json) {
    return WebrtcIceCandidatePayload(
      candidate: json['candidate'] as String,
      sdpMid: json['sdp_mid'] as String,
      sdpMlineIndex: (json['sdp_mline_index'] as num?)?.toInt() ?? 0,
      sessionId: json['session_id'] as String,
      streamId: json['stream_id'] as String,
    );
  }

  Map<String, dynamic> toJson() => {
        'candidate': candidate,
        'sdp_mid': sdpMid,
        'sdp_mline_index': sdpMlineIndex,
        'session_id': sessionId,
        'stream_id': streamId,
      };
}
