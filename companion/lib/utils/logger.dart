import 'dart:developer' as developer;

/// Thin wrapper around `dart:developer`'s `log()` so call sites read like
/// `logger.info(...)` instead of threading `name:`/`level:` everywhere, and
/// so a future swap to a package like `logging` only touches this file.
class AppLogger {
  final String tag;
  const AppLogger(this.tag);

  void debug(String message) => developer.log(message, name: tag, level: 500);
  void info(String message) => developer.log(message, name: tag, level: 800);
  void warn(String message) => developer.log(message, name: tag, level: 900);

  void error(String message, [Object? error, StackTrace? stackTrace]) {
    developer.log(
      message,
      name: tag,
      level: 1000,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
