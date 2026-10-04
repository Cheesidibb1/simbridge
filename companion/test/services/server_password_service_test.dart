import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/services/server_password_service.dart';

void main() {
  test('creates an HMAC-SHA256 proof for the server challenge', () {
    expect(
      ServerPasswordService.createProof(
        'key',
        'The quick brown fox jumps over the lazy dog',
      ),
      'f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8',
    );
  });
}
