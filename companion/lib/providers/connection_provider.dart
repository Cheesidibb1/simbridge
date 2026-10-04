import 'dart:async';
import 'dart:convert';

import 'package:crypto/crypto.dart' as crypto;
import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart' show AppLifecycleState, WidgetsBinding, WidgetsBindingObserver;

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
/// 2. Authenticates the WebSocket, then sends `ConnectSimulator`.
/// 3. Fans incoming messages out into typed fields the UI can listen to
///    via [ChangeNotifier], and exposes typed methods for every outgoing
///    message the control screen needs to send.
class ConnectionProvider extends ChangeNotifier with WidgetsBindingObserver {
  final ApiClient apiClient;
  final String deviceId;

  ConnectionProvider({required this.apiClient, required this.deviceId});

  final AppLogger _log = const AppLogger('ConnectionProvider');

  WebSocketService? _ws;
  StreamSubscription<WsMessage>? _msgSub;
  StreamSubscription<WsConnectionState>? _stateSub;
  Completer<void>? _authenticationCompleter;
  String? _serverPassword;
  bool _authenticated = false;

  // Registered only while a connection exists (see connectToSimulator), so
  // constructing a ConnectionProvider in a test never touches WidgetsBinding.
  bool _observingLifecycle = false;

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

  bool get isConnected =>
      wsState == WsConnectionState.connected && currentSimulator != null && _authenticated;

  /// Creates a REST session for [simulator], then opens the WebSocket at
  /// [wsUri], authenticates with [password], then connects to the simulator.
  /// Throws [ApiException] if session creation or authentication fails.
  Future<void> connectToSimulator(
    Simulator simulator,
    Uri wsUri, {
    StreamConfig? config,
    required String password,
  }) async {
    if (config != null) streamConfig = config;
    lastError = null;

    currentSession = await apiClient.createSession(
      simulatorId: simulator.id,
      deviceId: deviceId,
    );
    currentSimulator = simulator;
    _serverPassword = password;
    _authenticated = false;
    notifyListeners();

    await _msgSub?.cancel();
    await _stateSub?.cancel();
    _ws?.dispose();

    final ws = WebSocketService(wsUri: wsUri);
    _ws = ws;
    ws.onConnected = () => _authenticated = false;
    _msgSub = ws.messages.listen(_handleMessage);
    _stateSub = ws.connectionState.listen((state) {
      wsState = state;
      notifyListeners();
    });
    final authenticationCompleter = Completer<void>();
    _authenticationCompleter = authenticationCompleter;
    ws.connect();

    if (!_observingLifecycle) {
      WidgetsBinding.instance.addObserver(this);
      _observingLifecycle = true;
    }

    try {
      await authenticationCompleter.future.timeout(const Duration(seconds: 15));
    } on TimeoutException {
      await _cleanupFailedConnection(ws, currentSession!.sessionId);
      throw const ApiException('Timed out authenticating with the server.');
    } on ApiException {
      await _cleanupFailedConnection(ws, currentSession!.sessionId);
      rethrow;
    }
  }

  Future<void> _cleanupFailedConnection(WebSocketService ws, String sessionId) async {
    await _msgSub?.cancel();
    await _stateSub?.cancel();
    ws.dispose();
    if (identical(_ws, ws)) _ws = null;
    _authenticationCompleter = null;
    _serverPassword = null;
    _authenticated = false;
    currentSimulator = null;
    currentSession = null;
    latestFrame = null;
    frameWidth = null;
    frameHeight = null;
    wsState = WsConnectionState.disconnected;
    _stopObservingLifecycle();
    notifyListeners();
    try {
      await apiClient.deleteSession(sessionId);
    } on ApiException catch (error) {
      _log.warn('Failed to delete unauthenticated session: $error');
    }
  }

  /// Phone browsers suspend sockets while the screen is locked or the tab is
  /// hidden. On return, verify the socket is still alive (reconnecting if not)
  /// rather than showing a frozen mirror marked "connected".
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      _ws?.probe();
    }
  }

  void _stopObservingLifecycle() {
    if (_observingLifecycle) {
      WidgetsBinding.instance.removeObserver(this);
      _observingLifecycle = false;
    }
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
    if (simulatorId != null && _authenticated) {
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
    _authenticationCompleter = null;
    _serverPassword = null;
    _authenticated = false;
    _stopObservingLifecycle();

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
      case WsMessageType.authChallenge:
        final challenge = message.payload['challenge'] as String?;
        final password = _serverPassword;
        if (challenge == null || password == null) {
          _log.warn('Received an invalid authentication challenge');
          break;
        }
        final proof = crypto.Hmac(crypto.sha256, utf8.encode(password))
            .convert(utf8.encode(challenge))
            .toString();
        _ws?.send(WsMessage.outgoing(
          type: WsMessageType.authRequest,
          payload: AuthRequestPayload(
            deviceId: deviceId,
            token: '',
            challengeResponse: proof,
          ).toJson(),
        ));
        break;
      case WsMessageType.authResponse:
        final response = AuthResponsePayload.fromJson(message.payload);
        if (!response.success) {
          _authenticated = false;
          final error = ApiException(response.message ?? 'Server password was rejected.');
          _ws?.disconnect();
          final completer = _authenticationCompleter;
          if (completer != null && !completer.isCompleted) {
            completer.completeError(error);
          }
          break;
        }
        _authenticated = true;
        final simulatorId = currentSimulator?.id;
        if (simulatorId != null) _sendConnectSimulator(simulatorId);
        final completer = _authenticationCompleter;
        if (completer != null && !completer.isCompleted) completer.complete();
        notifyListeners();
        break;
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
    if (!_authenticated || simulatorId == null) return;
    _ws?.send(WsMessage.outgoing(
      type: WsMessageType.touchEvent,
      payload: TouchEventPayload(simulatorId: simulatorId, touches: touches).toJson(),
    ));
  }

  void sendGesture(GesturePayload gesture) {
    if (!_authenticated || currentSimulator == null) return;
    _ws?.send(WsMessage.outgoing(type: WsMessageType.gesture, payload: gesture.toJson()));
  }

  void sendDeviceButton(DeviceButtonType button) {
    final simulatorId = currentSimulator?.id;
    if (!_authenticated || simulatorId == null) return;
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
    if (!_authenticated || simulatorId == null) return;
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
    if (!_authenticated || simulatorId == null) return;
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
    if (!_authenticated || simulatorId == null) return;
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
    if (!_authenticated) return;
    _ws?.send(WsMessage.outgoing(type: WsMessageType.startRecording));
  }

  void stopRecording() {
    if (!_authenticated) return;
    _ws?.send(WsMessage.outgoing(type: WsMessageType.stopRecording));
  }

  void requestRecordings() {
    if (!_authenticated) return;
    _ws?.send(WsMessage.outgoing(type: WsMessageType.getRecordings));
  }

  @override
  void dispose() {
    final msgSub = _msgSub;
    final stateSub = _stateSub;
    if (msgSub != null) unawaited(msgSub.cancel());
    if (stateSub != null) unawaited(stateSub.cancel());
    _ws?.dispose();
    _stopObservingLifecycle();
    super.dispose();
  }
}
