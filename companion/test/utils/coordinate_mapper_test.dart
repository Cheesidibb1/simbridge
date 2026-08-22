import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/utils/coordinate_mapper.dart';

void main() {
  group('mapWidgetPointToSimulator', () {
    test('matches the documented example exactly', () {
      // From the protocol doc's Coordinate System section:
      // simulator 1080x1920, Flutter widget 400x800,
      // Flutter touch at (200, 400) -> Simulator touch at (540, 960).
      final result = mapWidgetPointToSimulator(
        widgetX: 200,
        widgetY: 400,
        widgetWidth: 400,
        widgetHeight: 800,
        simulatorWidth: 1080,
        simulatorHeight: 1920,
      );
      expect(result.x, closeTo(540, 0.001));
      expect(result.y, closeTo(960, 0.001));
    });

    test('identity mapping when widget and simulator sizes match', () {
      final result = mapWidgetPointToSimulator(
        widgetX: 42,
        widgetY: 99,
        widgetWidth: 300,
        widgetHeight: 600,
        simulatorWidth: 300,
        simulatorHeight: 600,
      );
      expect(result.x, closeTo(42, 0.001));
      expect(result.y, closeTo(99, 0.001));
    });

    test('returns the origin instead of dividing by zero for an empty widget', () {
      final result = mapWidgetPointToSimulator(
        widgetX: 50,
        widgetY: 50,
        widgetWidth: 0,
        widgetHeight: 0,
        simulatorWidth: 1080,
        simulatorHeight: 1920,
      );
      expect(result, const SimulatorPoint(0, 0));
    });
  });
}
