/// Exponential backoff with a ceiling, matching the protocol doc's
/// reconnection guidance: "Start with 1 second, max 30 seconds."
///
/// Kept as a small, dependency-free, pure class so it's trivial to unit
/// test in isolation from the WebSocket plumbing.
class Backoff {
  final Duration initial;
  final Duration max;
  final double multiplier;

  int _attempt = 0;

  Backoff({
    this.initial = const Duration(seconds: 1),
    this.max = const Duration(seconds: 30),
    this.multiplier = 2.0,
  });

  /// Number of times [next] has been called since the last [reset].
  int get attempt => _attempt;

  /// Returns the delay to wait before the next reconnect attempt, then
  /// advances internal state so the following call returns a longer delay
  /// (until it saturates at [max]).
  Duration next() {
    final rawMs = initial.inMilliseconds * _pow(multiplier, _attempt);
    _attempt++;
    final cappedMs = rawMs.clamp(
      initial.inMilliseconds.toDouble(),
      max.inMilliseconds.toDouble(),
    );
    return Duration(milliseconds: cappedMs.round());
  }

  /// Call after a successful connection so the next failure starts back
  /// at [initial] rather than continuing to climb.
  void reset() => _attempt = 0;

  static double _pow(double base, int exponent) {
    var result = 1.0;
    for (var i = 0; i < exponent; i++) {
      result *= base;
    }
    return result;
  }
}
