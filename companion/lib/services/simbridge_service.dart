// SimBridge service for managing connections and simulators

import 'dart:async';
import '../networking/websocket_client.dart';
import '../protocol/message.dart';
import '../models/simulator.dart';
import 'gps_service.dart';

class SimBridgeService {
  final WebSocketClient _wsClient = WebSocketClient();
  final List<Simulator> _simulators = [];
  final StreamController<List<Simulator>> _simulatorsController =
      StreamController.broadcast();
  final GpsService _gpsService = GpsService();
  StreamSubscription<Map<String, dynamic>>? _gpsSubscription;

  Stream<List<Simulator>> get simulatorsStream => _simulatorsController.stream;
  Stream<Message> get messageStream => _wsClient.messageStream;
  Stream<String> get errorStream => _wsClient.errorStream;
  bool get isConnected => _wsClient.isConnected;
  List<Simulator> get simulators => List.unmodifiable(_simulators);
  WebSocketClient get wsClient => _wsClient;
  GpsService get gpsService => _gpsService;

  Future<void> connect(String serverUrl, String authToken) async {
    // Convert HTTP URL to WebSocket URL
    final wsUrl = '${serverUrl.replaceFirst('http', 'ws')}/ws';
    await _wsClient.connect(wsUrl);

    // Listen for messages
    _wsClient.messageStream.listen(_handleMessage);
  }

  void disconnect() {
    _gpsSubscription?.cancel();
    _gpsService.stopStreaming();
    _wsClient.disconnect();
    _simulators.clear();
    _simulatorsController.add([]);
  }

  void _handleMessage(Message message) {
    switch (message.messageType) {
      case MessageType.simulatorListResponse:
        _handleSimulatorList(message);
        break;
      default:
        break;
    }
  }

  void _handleSimulatorList(Message message) {
    final simulatorsData = message.payload['simulators'] as List;
    _simulators.clear();
    for (final data in simulatorsData) {
      _simulators.add(Simulator.fromJson(data as Map<String, dynamic>));
    }
    _simulatorsController.add(List.from(_simulators));
  }

  void requestSimulatorList() {
    final message = Message(
      messageType: MessageType.simulatorListRequest,
      payload: {},
    );
    _wsClient.send(message);
  }

  void connectToSimulator(String simulatorId) {
    final message = Message(
      messageType: MessageType.connectSimulator,
      payload: {
        'simulator_id': simulatorId,
        'stream_config': {
          'quality': 'medium',
          'fps': 30,
          'audio_enabled': false,
        },
      },
    );
    _wsClient.send(message);
  }

  void sendTouchEvent(Map<String, dynamic> touchData) {
    final message = Message(
      messageType: MessageType.touchEvent,
      payload: touchData,
    );
    _wsClient.send(message);
  }

  void sendGesture(Map<String, dynamic> gestureData) {
    final message = Message(
      messageType: MessageType.gesture,
      payload: gestureData,
    );
    _wsClient.send(message);
  }

  Future<void> startGpsStreaming(String simulatorId) async {
    await _gpsService.startStreaming();
    
    _gpsSubscription = _gpsService.locationStream.listen((locationData) {
      final payload = {
        'simulator_id': simulatorId,
        'location': locationData,
      };
      sendGpsUpdate(payload);
    });
  }

  void stopGpsStreaming() {
    _gpsSubscription?.cancel();
    _gpsService.stopStreaming();
  }

  void sendGpsUpdate(Map<String, dynamic> gpsData) {
    final message = Message(
      messageType: MessageType.gpsUpdate,
      payload: gpsData,
    );
    _wsClient.send(message);
  }

  void dispose() {
    _gpsSubscription?.cancel();
    _gpsService.dispose();
    _wsClient.dispose();
    _simulatorsController.close();
  }
}
