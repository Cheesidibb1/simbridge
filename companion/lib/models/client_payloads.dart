/// Payload models for every client -> server WebSocket message defined in
/// the protocol doc. Each type mirrors the exact JSON shape so
/// `toJson()` can be dropped straight into `WsMessage.outgoing(payload: ...)`.
library;

import 'session.dart';

// ---------------------------------------------------------------------------
// Pairing / auth
// ---------------------------------------------------------------------------

class PairRequestPayload {
  final String deviceId;
  final String deviceName;
  final String deviceType; // 'android' | 'ios' | 'desktop'
  final String publicKey;

  const PairRequestPayload({
    required this.deviceId,
    required this.deviceName,
    required this.deviceType,
    required this.publicKey,
  });

  Map<String, dynamic> toJson() => {
        'device_id': deviceId,
        'device_name': deviceName,
        'device_type': deviceType,
        'public_key': publicKey,
      };
}

class AuthRequestPayload {
  final String deviceId;
  final String token;
  final String? challengeResponse;

  const AuthRequestPayload({
    required this.deviceId,
    required this.token,
    this.challengeResponse,
  });

  Map<String, dynamic> toJson() => {
        'device_id': deviceId,
        'token': token,
        if (challengeResponse != null) 'challenge_response': challengeResponse,
      };
}

// ---------------------------------------------------------------------------
// Simulator connection lifecycle
// ---------------------------------------------------------------------------

class ConnectSimulatorPayload {
  final String simulatorId;
  final StreamConfig streamConfig;

  const ConnectSimulatorPayload({
    required this.simulatorId,
    this.streamConfig = const StreamConfig(),
  });

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'stream_config': streamConfig.toJson(),
      };
}

class DisconnectSimulatorPayload {
  final String simulatorId;

  const DisconnectSimulatorPayload({required this.simulatorId});

  Map<String, dynamic> toJson() => {'simulator_id': simulatorId};
}

// ---------------------------------------------------------------------------
// Touch
// ---------------------------------------------------------------------------

enum TouchPhase {
  began,
  moved,
  ended,
  cancelled;

  String toWire() => name;
}

class Touch {
  final int id;
  final double x;
  final double y;
  final TouchPhase phase;
  final double? force;
  final double? majorRadius;

  const Touch({
    required this.id,
    required this.x,
    required this.y,
    required this.phase,
    this.force,
    this.majorRadius,
  });

  Map<String, dynamic> toJson() => {
        'id': id,
        'x': x,
        'y': y,
        'phase': phase.toWire(),
        if (force != null) 'force': force,
        if (majorRadius != null) 'major_radius': majorRadius,
      };
}

class TouchEventPayload {
  final String simulatorId;
  final List<Touch> touches;

  const TouchEventPayload({required this.simulatorId, required this.touches});

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'touches': touches.map((t) => t.toJson()).toList(growable: false),
      };
}

// ---------------------------------------------------------------------------
// Gestures
// ---------------------------------------------------------------------------

enum GestureType {
  swipe('swipe'),
  pinch('pinch'),
  rotation('rotation'),
  longPress('long_press'),
  doubleTap('double_tap');

  final String wire;
  const GestureType(this.wire);
}

enum SwipeDirection {
  up,
  down,
  left,
  right;

  String toWire() => name;
}

class GesturePayload {
  final String simulatorId;
  final GestureType gestureType;
  final Map<String, dynamic> data;

  const GesturePayload({
    required this.simulatorId,
    required this.gestureType,
    this.data = const {},
  });

  factory GesturePayload.swipe({
    required String simulatorId,
    required SwipeDirection direction,
    required double distance,
  }) {
    return GesturePayload(
      simulatorId: simulatorId,
      gestureType: GestureType.swipe,
      data: {'direction': direction.toWire(), 'distance': distance},
    );
  }

  factory GesturePayload.pinch({
    required String simulatorId,
    required double scale,
  }) {
    return GesturePayload(
      simulatorId: simulatorId,
      gestureType: GestureType.pinch,
      data: {'scale': scale},
    );
  }

  factory GesturePayload.rotation({
    required String simulatorId,
    required double angleDegrees,
  }) {
    return GesturePayload(
      simulatorId: simulatorId,
      gestureType: GestureType.rotation,
      data: {'angle': angleDegrees},
    );
  }

  factory GesturePayload.longPress({
    required String simulatorId,
    required double x,
    required double y,
    int durationMs = 500,
  }) {
    return GesturePayload(
      simulatorId: simulatorId,
      gestureType: GestureType.longPress,
      data: {'x': x, 'y': y, 'duration_ms': durationMs},
    );
  }

  factory GesturePayload.doubleTap({
    required String simulatorId,
    required double x,
    required double y,
  }) {
    return GesturePayload(
      simulatorId: simulatorId,
      gestureType: GestureType.doubleTap,
      data: {'x': x, 'y': y},
    );
  }

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'gesture_type': gestureType.wire,
        'data': data,
      };
}

// ---------------------------------------------------------------------------
// GPS / heading / motion
// ---------------------------------------------------------------------------

class GpsLocation {
  final double latitude;
  final double longitude;
  final double? altitude;
  final double? accuracy;
  final double? speed;
  final double? heading;
  final DateTime timestamp;

  const GpsLocation({
    required this.latitude,
    required this.longitude,
    required this.timestamp,
    this.altitude,
    this.accuracy,
    this.speed,
    this.heading,
  });

  Map<String, dynamic> toJson() => {
        'latitude': latitude,
        'longitude': longitude,
        if (altitude != null) 'altitude': altitude,
        if (accuracy != null) 'accuracy': accuracy,
        if (speed != null) 'speed': speed,
        if (heading != null) 'heading': heading,
        'timestamp': timestamp.toIso8601String(),
      };
}

class GpsUpdatePayload {
  final String simulatorId;
  final GpsLocation location;

  const GpsUpdatePayload({required this.simulatorId, required this.location});

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'location': location.toJson(),
      };
}

class HeadingUpdatePayload {
  final String simulatorId;
  final double heading;
  final double? accuracy;
  final DateTime timestamp;

  const HeadingUpdatePayload({
    required this.simulatorId,
    required this.heading,
    required this.timestamp,
    this.accuracy,
  });

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'heading': heading,
        if (accuracy != null) 'accuracy': accuracy,
        'timestamp': timestamp.toIso8601String(),
      };
}

class Attitude {
  final double roll;
  final double pitch;
  final double yaw;

  const Attitude({required this.roll, required this.pitch, required this.yaw});

  Map<String, dynamic> toJson() => {'roll': roll, 'pitch': pitch, 'yaw': yaw};
}

class MotionUpdatePayload {
  final String simulatorId;
  final List<double> acceleration; // [x, y, z]
  final List<double> gravity; // [x, y, z]
  final List<double> rotationRate; // [x, y, z]
  final Attitude attitude;
  final DateTime timestamp;

  const MotionUpdatePayload({
    required this.simulatorId,
    required this.acceleration,
    required this.gravity,
    required this.rotationRate,
    required this.attitude,
    required this.timestamp,
  });

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'acceleration': acceleration,
        'gravity': gravity,
        'rotation_rate': rotationRate,
        'attitude': attitude.toJson(),
        'timestamp': timestamp.toIso8601String(),
      };
}

// ---------------------------------------------------------------------------
// Device buttons
// ---------------------------------------------------------------------------

enum DeviceButtonType {
  home('home'),
  back('back'),
  appSwitcher('app_switcher'),
  lock('lock'),
  unlock('unlock'),
  volumeUp('volume_up'),
  volumeDown('volume_down'),
  mute('mute'),
  rotateLeft('rotate_left'),
  rotateRight('rotate_right'),
  shake('shake'),
  screenshot('screenshot');

  final String wire;
  const DeviceButtonType(this.wire);
}

class DeviceButtonPayload {
  final String simulatorId;
  final DeviceButtonType button;

  const DeviceButtonPayload({required this.simulatorId, required this.button});

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'button': button.wire,
      };
}

// ---------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------

enum ClipboardContentType {
  text,
  image,
  url;

  String toWire() => name;
}

class ClipboardSyncPayload {
  final String simulatorId;
  final String content;
  final ClipboardContentType contentType;

  const ClipboardSyncPayload({
    required this.simulatorId,
    required this.content,
    this.contentType = ClipboardContentType.text,
  });

  Map<String, dynamic> toJson() => {
        'simulator_id': simulatorId,
        'content': content,
        'content_type': contentType.toWire(),
      };
}

// ---------------------------------------------------------------------------
// File transfer
// ---------------------------------------------------------------------------

enum TransferDirection {
  upload,
  download;

  String toWire() => name;
}

class FileTransferPayload {
  final String transferId;
  final String simulatorId;
  final TransferDirection direction;
  final String fileName;
  final int fileSize;
  final String chunkData; // base64
  final int chunkIndex;
  final int totalChunks;

  const FileTransferPayload({
    required this.transferId,
    required this.simulatorId,
    required this.direction,
    required this.fileName,
    required this.fileSize,
    required this.chunkData,
    required this.chunkIndex,
    required this.totalChunks,
  });

  Map<String, dynamic> toJson() => {
        'transfer_id': transferId,
        'simulator_id': simulatorId,
        'direction': direction.toWire(),
        'file_name': fileName,
        'file_size': fileSize,
        'chunk_data': chunkData,
        'chunk_index': chunkIndex,
        'total_chunks': totalChunks,
      };
}
