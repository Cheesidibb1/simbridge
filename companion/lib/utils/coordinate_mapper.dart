/// Translates a point in the Flutter widget's local coordinate space into
/// the simulator's screen coordinate space, per the protocol doc:
///
/// > Origin (0,0) at top-left, X increases right, Y increases downward.
/// > Example: simulator 1080x1920, Flutter widget 400x800:
/// > Flutter touch at (200, 400) -> Simulator touch at (540, 960)
library;

class SimulatorPoint {
  final double x;
  final double y;
  const SimulatorPoint(this.x, this.y);

  @override
  bool operator ==(Object other) =>
      other is SimulatorPoint && other.x == x && other.y == y;

  @override
  int get hashCode => Object.hash(x, y);

  @override
  String toString() => 'SimulatorPoint($x, $y)';
}

/// Pure function — no Flutter dependency — so it can be unit tested without
/// a widget test harness.
SimulatorPoint mapWidgetPointToSimulator({
  required double widgetX,
  required double widgetY,
  required double widgetWidth,
  required double widgetHeight,
  required int simulatorWidth,
  required int simulatorHeight,
}) {
  if (widgetWidth <= 0 || widgetHeight <= 0) {
    return const SimulatorPoint(0, 0);
  }
  final scaleX = simulatorWidth / widgetWidth;
  final scaleY = simulatorHeight / widgetHeight;
  return SimulatorPoint(widgetX * scaleX, widgetY * scaleY);
}
