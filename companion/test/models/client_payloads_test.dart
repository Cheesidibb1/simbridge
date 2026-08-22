import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/models/client_payloads.dart';

void main() {
  group('TouchEventPayload', () {
    test('serializes touches in the documented wire format', () {
      const payload = TouchEventPayload(
        simulatorId: 'android-emu-1',
        touches: [
          Touch(
            id: 1,
            x: 100.5,
            y: 200.3,
            phase: TouchPhase.began,
            force: 0.5,
            majorRadius: 20.0,
          ),
        ],
      );

      final json = payload.toJson();
      expect(json['simulator_id'], 'android-emu-1');
      final touchJson = (json['touches'] as List<dynamic>).single as Map<String, dynamic>;
      expect(touchJson['id'], 1);
      expect(touchJson['x'], 100.5);
      expect(touchJson['phase'], 'began');
      expect(touchJson['force'], 0.5);
      expect(touchJson['major_radius'], 20.0);
    });

    test('omits optional force/major_radius when not provided', () {
      const touch = Touch(id: 2, x: 0, y: 0, phase: TouchPhase.ended);
      final json = touch.toJson();
      expect(json.containsKey('force'), isFalse);
      expect(json.containsKey('major_radius'), isFalse);
    });
  });

  group('GesturePayload', () {
    test('swipe factory builds the documented data shape', () {
      final gesture = GesturePayload.swipe(
        simulatorId: 'android-emu-1',
        direction: SwipeDirection.up,
        distance: 500.0,
      );
      final json = gesture.toJson();
      expect(json['gesture_type'], 'swipe');
      expect(json['data'], {'direction': 'up', 'distance': 500.0});
    });

    test('pinch factory reports scale as gesture data', () {
      final gesture = GesturePayload.pinch(simulatorId: 'sim-1', scale: 1.5);
      expect(gesture.toJson()['data'], {'scale': 1.5});
      expect(gesture.toJson()['gesture_type'], 'pinch');
    });
  });

  group('DeviceButtonPayload', () {
    test('uses snake_case wire values for multi-word buttons', () {
      const payload = DeviceButtonPayload(
        simulatorId: 'sim-1',
        button: DeviceButtonType.appSwitcher,
      );
      expect(payload.toJson()['button'], 'app_switcher');
    });

    test('covers every documented button name', () {
      const expected = {
        DeviceButtonType.home: 'home',
        DeviceButtonType.back: 'back',
        DeviceButtonType.appSwitcher: 'app_switcher',
        DeviceButtonType.lock: 'lock',
        DeviceButtonType.unlock: 'unlock',
        DeviceButtonType.volumeUp: 'volume_up',
        DeviceButtonType.volumeDown: 'volume_down',
        DeviceButtonType.mute: 'mute',
        DeviceButtonType.rotateLeft: 'rotate_left',
        DeviceButtonType.rotateRight: 'rotate_right',
        DeviceButtonType.shake: 'shake',
        DeviceButtonType.screenshot: 'screenshot',
      };
      for (final entry in expected.entries) {
        expect(entry.key.wire, entry.value);
      }
      expect(expected.length, DeviceButtonType.values.length);
    });
  });

  group('ClipboardSyncPayload', () {
    test('defaults content_type to text', () {
      const payload = ClipboardSyncPayload(simulatorId: 'sim-1', content: 'hello');
      expect(payload.toJson()['content_type'], 'text');
    });
  });
}
