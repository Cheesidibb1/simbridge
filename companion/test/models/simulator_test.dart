import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/models/simulator.dart';

void main() {
  group('Simulator', () {
    test('parses a minimal record like the doc example', () {
      final simulator = Simulator.fromJson({
        'id': 'ios-sim-1',
        'name': 'iPhone 15 Pro',
        'platform': 'ios',
        'status': 'offline',
      });

      expect(simulator.id, 'ios-sim-1');
      expect(simulator.name, 'iPhone 15 Pro');
      expect(simulator.platform, SimulatorPlatform.ios);
      expect(simulator.status, SimulatorStatus.offline);
      expect(simulator.screenSize, isNull);
    });

    test('parses the fuller Data Model shape with screen_size and device_details', () {
      final simulator = Simulator.fromJson({
        'id': 'android-emu-1',
        'name': 'Pixel 7',
        'platform': 'android',
        'os_version': '14',
        'status': 'available',
        'screen_size': {'width': 1080, 'height': 2400, 'scale': 2.625},
        'device_details': {
          'device_type': 'phone',
          'model': 'Pixel 7',
          'manufacturer': 'Google',
          'cpu_cores': 8,
          'memory_mb': 8192,
        },
      });

      expect(simulator.screenSize?.width, 1080);
      expect(simulator.screenSize?.scale, closeTo(2.625, 0.0001));
      expect(simulator.deviceDetails?.manufacturer, 'Google');
    });

    test('falls back to a safe default status for an unrecognized value', () {
      final simulator = Simulator.fromJson({
        'id': 'x',
        'name': 'X',
        'platform': 'android',
        'status': 'something-new-from-the-server',
      });
      expect(simulator.status, SimulatorStatus.offline);
    });
  });

  group('SimulatorListResponse', () {
    test('parses the /api/v1/simulators wrapper', () {
      final response = SimulatorListResponse.fromJson({
        'simulators': [
          {'id': 'a', 'name': 'A', 'platform': 'android', 'status': 'available'},
          {'id': 'b', 'name': 'B', 'platform': 'ios', 'status': 'busy'},
        ],
      });

      expect(response.simulators, hasLength(2));
      expect(response.simulators.first.status, SimulatorStatus.available);
      expect(response.simulators.last.platform, SimulatorPlatform.ios);
    });

    test('handles a missing simulators key as an empty list', () {
      final response = SimulatorListResponse.fromJson(<String, dynamic>{});
      expect(response.simulators, isEmpty);
    });
  });
}
