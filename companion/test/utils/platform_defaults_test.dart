import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/utils/constants.dart';
import 'package:simbridge_client/utils/platform_defaults.dart';

void main() {
  group('PlatformDefaults.resolve', () {
    test('native builds keep the original defaults and never force TLS', () {
      final d = PlatformDefaults.resolve(
        isWeb: false,
        pageUri: Uri.parse('https://example.com:9999/'),
      );

      expect(d.host, AppDefaults.serverHost);
      expect(d.port, AppDefaults.serverPort);
      expect(d.tls, AppDefaults.useTls);
      expect(d.tlsRequired, isFalse);
    });

    test('a page served over http by the server reuses its host and port', () {
      final d = PlatformDefaults.resolve(
        isWeb: true,
        pageUri: Uri.parse('http://192.168.1.20:8080/'),
      );

      expect(d.host, '192.168.1.20');
      expect(d.port, 8080);
      expect(d.tls, isFalse);
      expect(d.tlsRequired, isFalse);
    });

    test('a page served over https forces TLS (Safari blocks mixed content)', () {
      final d = PlatformDefaults.resolve(
        isWeb: true,
        pageUri: Uri.parse('https://sim.example.ts.net:8443/'),
      );

      expect(d.host, 'sim.example.ts.net');
      expect(d.port, 8443);
      expect(d.tls, isTrue);
      expect(d.tlsRequired, isTrue);
    });

    test('https without an explicit port falls back to 443', () {
      final d = PlatformDefaults.resolve(
        isWeb: true,
        pageUri: Uri.parse('https://sim.example.com/'),
      );

      expect(d.port, 443);
      expect(d.tlsRequired, isTrue);
    });

    test('http without an explicit port falls back to 80', () {
      final d = PlatformDefaults.resolve(
        isWeb: true,
        pageUri: Uri.parse('http://mac.local/'),
      );

      expect(d.host, 'mac.local');
      expect(d.port, 80);
      expect(d.tlsRequired, isFalse);
    });

    test('pages with no usable origin (file://, about:blank) use the native defaults', () {
      for (final raw in ['file:///index.html', 'about:blank']) {
        final d = PlatformDefaults.resolve(isWeb: true, pageUri: Uri.parse(raw));
        expect(d.host, AppDefaults.serverHost, reason: raw);
        expect(d.port, AppDefaults.serverPort, reason: raw);
        expect(d.tlsRequired, isFalse, reason: raw);
      }
    });
  });
}
