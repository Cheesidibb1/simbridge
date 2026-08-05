// Protocol message definitions for companion app

import 'dart:convert';

enum MessageType {
  // Client → Server messages
  pairRequest,
  authRequest,
  simulatorListRequest,
  connectSimulator,
  disconnectSimulator,
  touchEvent,
  gesture,
  gpsUpdate,
  headingUpdate,
  motionUpdate,
  deviceButton,
  clipboardSyncRequest,
  fileTransferRequest,
  startRecording,
  stopRecording,
  getRecordings,
  ping,

  // Server → Client messages
  pairResponse,
  authResponse,
  simulatorListResponse,
  screenFrame,
  notification,
  clipboardSyncResponse,
  fileTransferResponse,
  recordingStatus,
  recordingData,
  pong,
  error,

  // Bidirectional
  settingsUpdate,
  sessionInfo,
  metricsUpdate,
}

class Message {
  final MessageType messageType;
  final int version;
  final DateTime timestamp;
  final String? requestId;
  final Map<String, dynamic> payload;

  Message({
    required this.messageType,
    this.version = 1,
    DateTime? timestamp,
    this.requestId,
    required this.payload,
  }) : timestamp = timestamp ?? DateTime.now();

  Map<String, dynamic> toJson() {
    return {
      'message_type': messageType.toString().split('.').last,
      'version': version,
      'timestamp': timestamp.toIso8601String(),
      'request_id': requestId,
      'payload': payload,
    };
  }

  factory Message.fromJson(Map<String, dynamic> json) {
    return Message(
      messageType: _parseMessageType(json['message_type']),
      version: json['version'] ?? 1,
      timestamp: DateTime.parse(json['timestamp']),
      requestId: json['request_id'],
      payload: json['payload'] as Map<String, dynamic>,
    );
  }

  String toJsonString() => jsonEncode(toJson());

  factory Message.fromJsonString(String data) {
    return Message.fromJson(jsonDecode(data));
  }

  static MessageType _parseMessageType(String type) {
    return MessageType.values.firstWhere(
      (e) => e.toString().split('.').last == type,
      orElse: () => MessageType.error,
    );
  }
}
