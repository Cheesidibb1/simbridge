import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import '../models/ws_envelope.dart';
import '../utils/backoff.dart';
import '../utils/constants.dart';
import '../utils/logger.dart';

enum WsConnectionState { disconnected, connecting, connected, reconnecting }

/// Owns a single WebSocket connection to the SimBridge server. Handles:
/// - JSON encode/decode of the [WsMessage] envelope
/// - a keepalive `Ping` sent on [AppDefaults.pingInterval]
/// - reconnection with the doc's prescribed exponential backoff
///   (1s initial, 30s ceiling), via [Backoff]
///
/// Mobile Safari (and any phone browser) suspends sockets when the screen
/// locks or the tab is backgrounded, often *without* firing `onDone`, which
/// would leave the UI showing "connected" on a dead socket. Two guards cover
/// that: a staleness watchdog on the ping timer, and [probe], which the app
/// calls when it returns to the foreground.
///
/// It deliberately does *not* know about simulators, sessions, or auth —
/// [onConnected] is the hook callers use to replay whatever handshake
/// messages (auth, `ConnectSimulator`) are needed after a fresh connection.
class WebSocketService {
  final Uri wsUri;

  WebSocketService({required this.wsUri});

  final AppLogger _log = const AppLogger('WebSocketService');
  final Backoff _backoff = Backoff(
    initial: AppDefaults.reconnectInitialDelay,
    max: AppDefaults.reconnectMaxDelay,
  );

  WebSocketChannel? _channel;
  StreamSubscription<dynamic>? _channelSub;
  Timer? _pingTimer;
  Timer? _probeTimer;
  Timer? _reconnectTimer;
  DateTime _lastRxAt = DateTime.now();
  bool _shouldReconnect = false;
  bool _disposed = false;

  final StreamController<WsMessage> _messageController = StreamController<WsMessage>.broadcast();
  final StreamController<WsConnectionState> _stateController =
      StreamController<WsConnectionState>.broadcast();
  WsConnectionState _state = WsConnectionState.disconnected;

  /// Invoked after every successful (re)connect. Callers use this to send
  /// `AuthRequest`/`ConnectSimulator` again, per the doc's "re-authenticate
  /// on reconnection, restore session state if possible" guidance.
  void Function()? onConnected;

  Stream<WsMessage> get messages => _messageController.stream;
  Stream<WsConnectionState> get connectionState => _stateController.stream;
  WsConnectionState get state => _state;

  /// Opens the connection and enables auto-reconnect until [disconnect] is
  /// called.
  void connect() {
    _shouldReconnect = true;
    unawaited(_connectOnce());
  }

  Future<void> _connectOnce() async {
    if (_disposed) return;
    _setState(
      _backoff.attempt == 0 ? WsConnectionState.connecting : WsConnectionState.reconnecting,
    );
    try {
      final channel = WebSocketChannel.connect(wsUri);
      await channel.ready;
      if (_disposed) {
        await channel.sink.close();
        return;
      }
      _channel = channel;
      _channelSub = channel.stream.listen(
        _handleRaw,
        onError: _handleError,
        onDone: _handleDone,
        cancelOnError: true,
      );
      _backoff.reset();
      _lastRxAt = DateTime.now();
      _setState(WsConnectionState.connected);
      _startPingTimer();
      onConnected?.call();
    } catch (e, st) {
      _log.error('WebSocket connect failed', e, st);
      _scheduleReconnect();
    }
  }

  void _handleRaw(dynamic raw) {
    _lastRxAt = DateTime.now();
    try {
      final text = raw is String ? raw : utf8.decode(raw as List<int>);
      final decoded = jsonDecode(text) as Map<String, dynamic>;
      _messageController.add(WsMessage.fromJson(decoded));
    } catch (e, st) {
      _log.error('Failed to decode incoming WS message', e, st);
    }
  }

  void _handleError(Object error, StackTrace stackTrace) {
    _log.warn('WebSocket stream error: $error');
    _teardownChannel();
    _scheduleReconnect();
  }

  void _handleDone() {
    _log.info('WebSocket closed by remote/local peer');
    _teardownChannel();
    _scheduleReconnect();
  }

  void _scheduleReconnect() {
    if (!_shouldReconnect || _disposed) {
      _setState(WsConnectionState.disconnected);
      return;
    }
    final delay = _backoff.next();
    _setState(WsConnectionState.reconnecting);
    _log.info('Reconnecting in ${delay.inMilliseconds}ms (attempt ${_backoff.attempt})');
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(delay, _connectOnce);
  }

  /// Serializes and sends a message. Silently drops (with a log line) if
  /// not currently connected — callers that need guaranteed delivery should
  /// check [state] first or queue at a higher layer.
  void send(WsMessage message) {
    final channel = _channel;
    if (channel == null || _state != WsConnectionState.connected) {
      _log.warn('Dropping outgoing ${message.rawType}: not connected');
      return;
    }
    channel.sink.add(jsonEncode(message.toJson()));
  }

  void _startPingTimer() {
    _pingTimer?.cancel();
    _pingTimer = Timer.periodic(AppDefaults.pingInterval, (_) {
      // The server answers every message (Pong at minimum), so three silent
      // ping intervals means the socket is dead even if no close event arrived.
      if (DateTime.now().difference(_lastRxAt) > AppDefaults.pingInterval * 3) {
        _forceReconnect('no data for ${AppDefaults.pingInterval.inSeconds * 3}s');
        return;
      }
      send(WsMessage.outgoing(type: WsMessageType.ping));
    });
  }

  /// Checks that a socket that *claims* to be connected really is, e.g. right
  /// after the app returns to the foreground. Sends a ping; if nothing at all
  /// arrives within [timeout], the connection is torn down and re-established.
  void probe({Duration timeout = const Duration(seconds: 4)}) {
    if (_state != WsConnectionState.connected) return;
    final sentAt = DateTime.now();
    send(WsMessage.outgoing(type: WsMessageType.ping));
    _probeTimer?.cancel();
    _probeTimer = Timer(timeout, () {
      if (_state == WsConnectionState.connected && _lastRxAt.isBefore(sentAt)) {
        _forceReconnect('no reply to probe');
      }
    });
  }

  void _forceReconnect(String reason) {
    _log.warn('Forcing reconnect: $reason');
    final channel = _channel;
    _teardownChannel();
    if (channel != null) {
      unawaited(channel.sink.close());
    }
    _scheduleReconnect();
  }

  void _teardownChannel() {
    _pingTimer?.cancel();
    _pingTimer = null;
    _probeTimer?.cancel();
    _probeTimer = null;
    final sub = _channelSub;
    _channelSub = null;
    _channel = null;
    if (sub != null) {
      unawaited(sub.cancel());
    }
  }

  /// Closes the connection and disables auto-reconnect.
  void disconnect() {
    _shouldReconnect = false;
    _reconnectTimer?.cancel();
    final channel = _channel;
    _teardownChannel();
    if (channel != null) {
      unawaited(channel.sink.close());
    }
    _setState(WsConnectionState.disconnected);
  }

  void _setState(WsConnectionState next) {
    _state = next;
    if (!_stateController.isClosed) {
      _stateController.add(next);
    }
  }

  void dispose() {
    _disposed = true;
    disconnect();
    unawaited(_messageController.close());
    unawaited(_stateController.close());
  }
}
