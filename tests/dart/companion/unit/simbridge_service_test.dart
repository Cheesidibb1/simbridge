// SimBridge Service tests

import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_companion/services/simbridge_service.dart';
import 'package:mockito/mockito.dart';

void main() {
  group('SimBridgeService', () {
    test('should initialize service correctly', () {
      final service = SimBridgeService();
      
      expect(service, isNotNull);
    });

    test('should have correct stream properties', () {
      final service = SimBridgeService();
      
      expect(service.simulatorsStream.runtimeType, equals(Stream<List<simbridge_protocol.Simulator>>));
      expect(service.messageStream.runtimeType, equals(Stream<simbridge_protocol.Message>));
      expect(service.errorStream.runtimeType, equals(Stream<String>));
    });

    test('should create websocket client on connect', () {
      final service = SimBridgeService();
      
      expect(service.wsClient, isNotNull);
    });

    test('should dispose properly', () {
      final service = SimBridgeService();
      
      expect(() => service.dispose(), returnsNormally);
    });
  });
}
