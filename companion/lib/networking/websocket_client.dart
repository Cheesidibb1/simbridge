// WebSocket client for companion app

import 'dart:async';
import 'package:web_socket_channel/web_socket_channel.dart';
import '../protocol/message.dart';

class WebSocketClient {
  WebSocketChannel? _channel;
  final StreamController<Message> _messageController = StreamController.broadcast();
  final StreamController<String> _errorController = StreamController.broadcast();
  bool _isConnected = false;

  Stream<Message> get messageStream => _messageController.stream;
  Stream<String> get errorStream => _errorController.stream;
  bool get isConnected => _isConnected;

  Future<void> connect(String url) async {
    try {
      _channel = WebSocketChannel.connect(Uri.parse(url));
      _isConnected = true;

      _channel!.stream.listen(
        (data) {
          try {
            final message = Message.fromJsonString(data);
            _messageController.add(message);
          } catch (e) {
            _errorController.add('Failed to parse message: $e');
          }
        },
        onError: (error) {
          _errorController.add('WebSocket error: $error');
          _isConnected = false;
        },
        onDone: () {
          _isConnected = false;
          _errorController.add('WebSocket connection closed');
        },
      );
    } catch (e) {
      _errorController.add('Failed to connect: $e');
      _isConnected = false;
    }
  }

  void disconnect() {
    _channel?.sink.close();
    _channel = null;
    _isConnected = false;
  }

  void send(Message message) {
    if (_isConnected && _channel != null) {
      _channel!.sink.add(message.toJsonString());
    }
  }

  void dispose() {
    disconnect();
    _messageController.close();
    _errorController.close();
  }
}
