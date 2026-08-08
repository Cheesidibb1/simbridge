// GPS Service tests

import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_companion/services/gps_service.dart';
import 'package:geolocator/geolocator.dart';

void main() {
  group('GpsService', () {
    late GpsService service;

    setUp(() {
      service = GpsService();
    });

    tearDown(() {
      service.dispose();
    });

    test('should initialize gps service correctly', () {
      expect(service, isNotNull);
      expect(service.isStreaming, isFalse);
    });

    test('should have broadcast stream controller', () {
      final controller = service.locationStream;
      
      expect(controller.runtimeType, equals(StreamController<Map<String, dynamic>>.broadcast));
    });

    group('checkPermissions', () {
      test('should return false when location service is disabled', () async {
        // Note: This test requires actual device permissions
        // Skipping in unit tests, would need to be run with integration tests
        skip('Requires physical device testing');
        
        final hasPermission = await service.checkPermissions();
        expect(hasPermission, isFalse);
      });

      test('should return true when permission granted', () async {
        // Note: Requires actual device with location permissions
        skip('Requires physical device testing');
        
        final hasPermission = await service.checkPermissions();
        expect(hasPermission, isTrue);
      });
    });

    group('startStreaming', () {
      test('should not start if already streaming', () {
        // This should be idempotent and safe to call multiple times
        service.startStreaming(intervalSeconds: 1);
        service.startStreaming(intervalSeconds: 2);
        
        expect(service.isStreaming, isTrue);
      });

      test('should handle start with different intervals', () async {
        // Note: Requires actual location permissions on device
        skip('Requires physical device with location permissions');
        
        final hasPermission = await service.checkPermissions();
        if (hasPermission) {
          await service.startStreaming(intervalSeconds: 1);
          
          expect(service.isStreaming, isTrue);
          service.stopStreaming();
          expect(service.isStreaming, isFalse);
        }
      });
    });

    group('stopStreaming', () {
      test('should stop streaming gracefully', () async {
        skip('Requires physical device testing');
        
        await service.startStreaming(intervalSeconds: 1);
        service.stopStreaming();
        
        expect(service.isStreaming, isFalse);
      });
    });

    group('getCurrentPosition', () {
      test('should return current position when available', () async {
        skip('Requires physical device with location permissions');
        
        final hasPermission = await service.checkPermissions();
        if (hasPermission) {
          try {
            final position = await service.getCurrentPosition();
            expect(position.latitude, isNotNull);
            expect(position.longitude, isNotNull);
          } catch (_) {
            // Expected in some cases
          }
        }
      });
    });

    test('should dispose location controller', () {
      // Verify disposal doesn't throw errors
      service.dispose();
      
      expect(service.isStreaming, isFalse);
    });
  });
}
