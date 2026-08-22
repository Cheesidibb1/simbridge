import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:simbridge_client/services/api_client.dart';
import 'package:simbridge_client/services/api_exception.dart';

void main() {
  group('ApiClient', () {
    test('getSimulators parses the documented response shape', () async {
      final mock = MockClient((request) async {
        expect(request.url.path, '/api/v1/simulators');
        return http.Response(
          jsonEncode({
            'simulators': [
              {
                'id': 'android-emu-1',
                'name': 'Pixel 7',
                'platform': 'android',
                'status': 'offline',
              },
            ],
          }),
          200,
          headers: {'content-type': 'application/json'},
        );
      });
      final client = ApiClient(baseUrl: 'http://localhost:8080', client: mock);

      final simulators = await client.getSimulators();

      expect(simulators, hasLength(1));
      expect(simulators.first.name, 'Pixel 7');
    });

    test('createSession posts the expected body and parses the session', () async {
      late Map<String, dynamic> sentBody;
      final mock = MockClient((request) async {
        sentBody = jsonDecode(request.body) as Map<String, dynamic>;
        return http.Response(
          jsonEncode({
            'session_id': 'uuid-1',
            'simulator_id': 'android-emu-1',
            'status': 'active',
          }),
          200,
        );
      });
      final client = ApiClient(baseUrl: 'http://localhost:8080', client: mock);

      final session = await client.createSession(
        simulatorId: 'android-emu-1',
        deviceId: 'device-1',
      );

      expect(sentBody['simulator_id'], 'android-emu-1');
      expect(sentBody['device_id'], 'device-1');
      expect(session.sessionId, 'uuid-1');
      expect(session.simulatorId, 'android-emu-1');
    });

    test('getSessions parses the plain string-array response', () async {
      final mock = MockClient((request) async {
        return http.Response(jsonEncode(['session-id-1', 'session-id-2']), 200);
      });
      final client = ApiClient(baseUrl: 'http://localhost:8080', client: mock);

      final sessions = await client.getSessions();
      expect(sessions, ['session-id-1', 'session-id-2']);
    });

    test('deleteSession hits DELETE /api/v1/sessions/:id', () async {
      String? calledMethod;
      String? calledPath;
      final mock = MockClient((request) async {
        calledMethod = request.method;
        calledPath = request.url.path;
        return http.Response(jsonEncode({'status': 'deleted'}), 200);
      });
      final client = ApiClient(baseUrl: 'http://localhost:8080', client: mock);

      await client.deleteSession('session-id-1');
      expect(calledMethod, 'DELETE');
      expect(calledPath, '/api/v1/sessions/session-id-1');
    });

    test('throws ApiException carrying the status code on a non-2xx response', () async {
      final mock = MockClient((request) async {
        return http.Response(jsonEncode({'message': 'not found'}), 404);
      });
      final client = ApiClient(baseUrl: 'http://localhost:8080', client: mock);

      expect(
        () => client.getSimulators(),
        throwsA(
          isA<ApiException>()
              .having((e) => e.statusCode, 'statusCode', 404)
              .having((e) => e.message, 'message', 'not found'),
        ),
      );
    });

    test('falls back to a generic message when the error body is not JSON', () async {
      final mock = MockClient((request) async {
        return http.Response('Internal Server Error', 500);
      });
      final client = ApiClient(baseUrl: 'http://localhost:8080', client: mock);

      expect(
        () => client.getHealth(),
        throwsA(isA<ApiException>().having((e) => e.statusCode, 'statusCode', 500)),
      );
    });
  });
}
