import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';

import '../models/client_payloads.dart';
import '../models/server_payloads.dart';
import '../models/session.dart';
import '../models/shared_payloads.dart';
import '../models/simulator.dart';
import '../models/ws_envelope.dart';
import '../services/api_client.dart';
import '../services/api_exception.dart';
import '../services/websocket_service.dart';
import '../utils/logger.dart';

/// Orchestrates one active simulator connection end-to-end:
/// 1. `POST /api/v1/sessions` to register the session over REST.
/// 2. Opens the WebSocket and sends `ConnectSimulator` (replayed
///    automatically on every reconnect via [WebSocketService.onConnected]).
/// 3. Fans incoming messages out into typed fields the UI can listen to
///    via [ChangeNotifier], and exposes typed methods for every outgoing
///    message the control screen needs to send.
class ConnectionProvider extends ChangeNotifier {
  final ApiClient apiClient;
  final String deviceId;

  ConnectionProvider({required this.apiClient, required this.deviceId});

  final AppLogger _log = const AppLogger('ConnectionProvider');

  WebSocketService? _ws;
  StreamSubscription<WsMessage>? _msgSub;
  StreamSubscription<WsConnectionState>? _stateSub;

  Simulator? currentSimulator;
  Session? currentSession;
  WsConnectionState wsState = WsConnectionState.disconnected;
  StreamConfig streamConfig = const StreamConfig();

  Uint8List? latestFrame;
  int? frameWidth;
  int? frameHeight;

  MetricsUpdatePayload? latestMetrics;
  final List<SimNotification> notifications = [];
  RecordingStatusPayload? recordingStatus;
  ErrorPayload? lastError;

  bool get isConnected => wsState == WsConnectionState.connected && currentSimulator != null;

  /// Creates a REST session for [simulator], then opens the WebSocket at
  /// [wsUri] and sends `ConnectSimulator`. Throws [ApiException] if the
  /// session cannot be created; the WebSocket step itself never throws —
  /// connection failures surface through [wsState] instead.
  Future<void> connectToSimulator(
    Simulator simulator,
    Uri wsUri, {
    StreamConfig? config,
  }) async {
    if (config != null) streamConfig = config;
    lastError = null;

    currentSession = await apiClient.createSession(
      simulatorId: simulator.id,
      deviceId: deviceId,
    );
    currentSimulator = simulator;
    notifyListeners();

    await _msgSub?.cancel();
    await _stateSub?.cancel();
    _ws?.dispose();

    final ws = WebSocketService(wsUri: wsUri);
    _ws = ws;
    ws.onConnected = () => _sendConnectSimulator(simulator.id);
    _msgSub = ws.messages.listen(_handleMessage);
    _stateSub = ws.connectionState.listen((state) {
      wsState = state;
      notifyListeners();
    });
    ws.connect();
  }

  void _sendConnectSimulator(String simulatorId) {
    _ws?.send(WsMessage.outgoing(
      type: WsMessageType.connectSimulator,
      payload: ConnectSimulatorPayload(
        simulatorId: simulatorId,
        streamConfig: streamConfig,
      ).toJson(),
    ));
  }

  Future<void> disconnect() async {
    final simulatorId = currentSimulator?.id;
    if (simulatorId != null) {
      _ws?.send(WsMessage.outgoing(
        type: WsMessageType.disconnectSimulator,
        payload: DisconnectSimulatorPayload(simulatorId: simulatorId).toJson(),
      ));
    }

    final sessionId = currentSession?.sessionId;
    await _msgSub?.cancel();
    await _stateSub?.cancel();
    _ws?.dispose();
    _ws = null;

    currentSimulator = null;
    currentSession = null;
    latestFrame = null;
    frameWidth = null;
    frameHeight = null;
    wsState = WsConnectionState.disconnected;
    notifyListeners();

    if (sessionId != null) {
      try {
        await apiClient.deleteSession(sessionId);
      } on ApiException catch (e) {
        _log.warn('Failed to delete session on disconnect: $e');
      }
    }
  }

  void _handleMessage(WsMessage message) {
    final type = message.type;
    if (type == null) {
      _log.warn('Ignoring message with unrecognized type: ${message.rawType}');
      return;
    }
    switch (type) {
      case WsMessageType.screenFrame:
        final payload = ScreenFramePayload.fromJson(message.payload);
        try {
          latestFrame = base64Decode(payload.frameData);
        } catch (e) {
          _log.error('Failed to decode frame_data', e);
          break;
        }
        frameWidth = payload.width;
        frameHeight = payload.height;
        notifyListeners();
        break;
      case WsMessageType.notification:
        final payload = NotificationPayload.fromJson(message.payload);
        notifications.insert(0, payload.notification);
        if (notifications.length > 50) notifications.removeLast();
        notifyListeners();
        break;
      case WsMessageType.metricsUpdate:
        latestMetrics = MetricsUpdatePayload.fromJson(message.payload);
        notifyListeners();
        break;
      case WsMessageType.recordingStatus:
        recordingStatus = RecordingStatusPayload.fromJson(message.payload);
        notifyListeners();
        break;
      case WsMessageType.error:
        lastError = ErrorPayload.fromJson(message.payload);
        _log.warn('Server error ${lastError!.code.wire}: ${lastError!.message}');
        notifyListeners();
        break;
      case WsMessageType.pong:
        break; // keepalive ack, nothing to surface
      case WsMessageType.sessionInfo:
        break; // informational; session id/status already tracked locally
      default:
        _log.info('Unhandled message type: ${type.wire}');
    }
  }

  // --- Outgoing actions ---------------------------------------------------

  void sendTouch(List<Touch> touches) {
    final simulatorId = currentSimulator?.id;
    if (simulatorId == null) return;
    _ws?.send(WsMessage.outgoing(
      type: WsMessageType.touchEvent,
      payload: TouchEventPayload(simulatorId: simulatorId, touches: touches).toJson(),
    ));
  }

  void sendGesture(GesturePayload gesture) {
    if (currentSimulator == null) return;
    _ws?.send(WsMessage.outgoing(type: WsMessageType.gesture, payload: gesture.toJson()));
  }

  void sendDeviceButton(DeviceButtonType button) {
    final simulatorId = currentSimulator?.id;
    if (simulatorId == null) return;
    _ws?.send(WsMessage.outgoing(
      type: WsMessageType.deviceButton,
      payload: DeviceButtonPayload(simulatorId: simulatorId, button: button).toJson(),
    ));
  }

  void sendGps({
    required double latitude,
    required double longitude,
    double? altitude,
    double? accuracy,
    double? speed,
    double? heading,
  }) {
    final simulatorId = currentSimulator?.id;
    if (simulatorId == null) return;
    _ws?.send(WsMessage.outgoing(
      type: WsMessageType.gpsUpdate,
      payload: GpsUpdatePayload(
        simulatorId: simulatorId,
        location: GpsLocation(
          latitude: latitude,
          longitude: longitude,
          altitude: altitude,
          accuracy: accuracy,
          speed: speed,
          heading: heading,
          timestamp: DateTime.now().toUtc(),
        ),
      ).toJson(),
    ));
  }

  void sendHeading(double heading, {double? accuracy}) {
    final simulatorId = currentSimulator?.id;
    if (simulatorId == null) return;
    _ws?.send(WsMessage.outgoing(
      type: WsMessageType.headingUpdate,
      payload: HeadingUpdatePayload(
        simulatorId: simulatorId,
        heading: heading,
        accuracy: accuracy,
        timestamp: DateTime.now().toUtc(),
      ).toJson(),
    ));
  }

  void sendClipboard(String content, {ClipboardContentType type = ClipboardContentType.text}) {
    final simulatorId = currentSimulator?.id;
    if (simulatorId == null) return;
    _ws?.send(WsMessage.outgoing(
      type: WsMessageType.clipboardSync,
      payload: ClipboardSyncPayload(
        simulatorId: simulatorId,
        content: content,
        contentType: type,
      ).toJson(),
    ));
  }

  void dismissNotification(SimNotification notification) {
    notifications.remove(notification);
    notifyListeners();
  }

  void startRecording() {
    _ws?.send(WsMessage.outgoing(type: WsMessageType.startRecording));
  }

  void stopRecording() {
    _ws?.send(WsMessage.outgoing(type: WsMessageType.stopRecording));
  }

  void requestRecordings() {
    _ws?.send(WsMessage.outgoing(type: WsMessageType.getRecordings));
  }

  @override
  void dispose() {
    final msgSub = _msgSub;
    final stateSub = _stateSub;
    if (msgSub != null) unawaited(msgSub.cancel());
    if (stateSub != null) unawaited(stateSub.cancel());
    _ws?.dispose();
    super.dispose();
  }
}
