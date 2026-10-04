import 'package:flutter/foundation.dart' show kIsWeb;

import 'constants.dart';

/// Where the app should point itself the very first time it runs, and
/// whether it is *allowed* to use plain `ws://` / `http://`.
///
/// This exists for the browser build (iPhone Safari in particular):
///
/// * On a phone, `localhost` is the phone itself, so the old hard-coded
///   default of `localhost:8080` can never work. When the app is served by
///   the SimBridge server (the normal setup), the address in Safari's URL bar
///   *is* the server, so we reuse it.
/// * Safari blocks "mixed content": a page loaded over `https://` may not open
///   `ws://` sockets or `http://` requests. If the page is https we must use
///   `wss://` / `https://` no matter what the TLS switch says.
///
/// Everything here is a pure function of its inputs ([resolve]) so it can be
/// unit tested without a browser; [current] feeds it the real environment.
class PlatformDefaults {
  /// Default server host for a fresh install.
  final String host;

  /// Default server port for a fresh install.
  final int port;

  /// Default for the "Use TLS" switch on a fresh install.
  final bool tls;

  /// True when the page itself was loaded over https. In that case the TLS
  /// switch is forced on (and shown as locked in the UI) because the browser
  /// would refuse the insecure alternative anyway.
  final bool tlsRequired;

  const PlatformDefaults({
    required this.host,
    required this.port,
    required this.tls,
    required this.tlsRequired,
  });

  /// The original, non-web defaults (unchanged behaviour for native builds).
  static const PlatformDefaults native = PlatformDefaults(
    host: AppDefaults.serverHost,
    port: AppDefaults.serverPort,
    tls: AppDefaults.useTls,
    tlsRequired: false,
  );

  /// Pure resolution logic. [pageUri] is the URL the app was loaded from
  /// (`Uri.base` in a browser).
  static PlatformDefaults resolve({required bool isWeb, required Uri pageUri}) {
    if (!isWeb) return native;

    final isHttp = pageUri.scheme == 'http';
    final isHttps = pageUri.scheme == 'https';
    // `file://`, `about:blank`, an empty host, etc. → nothing useful to infer.
    if ((!isHttp && !isHttps) || pageUri.host.isEmpty) return native;

    return PlatformDefaults(
      host: pageUri.host,
      // Uri.port is the scheme default (80 / 443) when the URL has no port.
      port: pageUri.hasPort ? pageUri.port : (isHttps ? 443 : 80),
      tls: isHttps,
      tlsRequired: isHttps,
    );
  }

  /// Defaults for the environment this app is actually running in.
  static PlatformDefaults get current => resolve(isWeb: kIsWeb, pageUri: Uri.base);
}
