import 'dart:convert';

import 'package:http/http.dart' as http;

import '../models/session.dart';
import '../models/simulator.dart';
import 'api_exception.dart';

/// Thin wrapper over the SimBridge REST API described in the protocol doc.
///
/// Takes an [http.Client] so tests can inject `http.testing.MockClient`
/// instead of hitting the network.
class ApiClient {
  /// Origin only, no trailing slash — e.g. `http://192.168.1.20:8080`.
  final String baseUrl;
  final http.Client _client;

  ApiClient({required this.baseUrl, http.Client? client}) : _client = client ?? http.Client();

  static const Map<String, String> _headers = {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  };

  Uri _uri(String path) => Uri.parse('$baseUrl$path');

  /// GET /health
  Future<Map<String, dynamic>> getHealth() async {
    final response = await _client.get(_uri('/health'), headers: _headers);
    _checkStatus(response);
    return jsonDecode(response.body) as Map<String, dynamic>;
  }

  /// GET /api/v1/simulators
  Future<List<Simulator>> getSimulators() async {
    final response = await _client.get(_uri('/api/v1/simulators'), headers: _headers);
    _checkStatus(response);
    final decoded = jsonDecode(response.body) as Map<String, dynamic>;
    return SimulatorListResponse.fromJson(decoded).simulators;
  }

  /// GET /api/v1/sessions
  Future<List<String>> getSessions() async {
    final response = await _client.get(_uri('/api/v1/sessions'), headers: _headers);
    _checkStatus(response);
    final decoded = jsonDecode(response.body) as List<dynamic>;
    return decoded.cast<String>();
  }

  /// POST /api/v1/sessions
  Future<Session> createSession({
    required String simulatorId,
    required String deviceId,
  }) async {
    final response = await _client.post(
      _uri('/api/v1/sessions'),
      headers: _headers,
      body: jsonEncode({'simulator_id': simulatorId, 'device_id': deviceId}),
    );
    _checkStatus(response);
    return Session.fromJson(jsonDecode(response.body) as Map<String, dynamic>);
  }

  /// DELETE /api/v1/sessions/:id
  Future<void> deleteSession(String sessionId) async {
    final response = await _client.delete(
      _uri('/api/v1/sessions/$sessionId'),
      headers: _headers,
    );
    _checkStatus(response);
  }

  void _checkStatus(http.Response response) {
    if (response.statusCode >= 200 && response.statusCode < 300) return;

    var message = 'Request failed with status ${response.statusCode}';
    try {
      final decoded = jsonDecode(response.body);
      if (decoded is Map && decoded['message'] is String) {
        message = decoded['message'] as String;
      }
    } catch (_) {
      // Body wasn't JSON (or was empty) — keep the default message.
    }
    throw ApiException(message, statusCode: response.statusCode);
  }

  void close() => _client.close();
}
