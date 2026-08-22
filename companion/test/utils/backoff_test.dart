import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/utils/backoff.dart';

void main() {
  group('Backoff', () {
    test('starts at the initial delay and doubles up to the max', () {
      final backoff = Backoff(
        initial: const Duration(seconds: 1),
        max: const Duration(seconds: 30),
      );

      expect(backoff.next(), const Duration(seconds: 1));
      expect(backoff.next(), const Duration(seconds: 2));
      expect(backoff.next(), const Duration(seconds: 4));
      expect(backoff.next(), const Duration(seconds: 8));
      expect(backoff.next(), const Duration(seconds: 16));
      expect(backoff.next(), const Duration(seconds: 30)); // would be 32, capped
      expect(backoff.next(), const Duration(seconds: 30)); // stays capped
    });

    test('reset returns the sequence to the initial delay', () {
      final backoff = Backoff(
        initial: const Duration(seconds: 1),
        max: const Duration(seconds: 30),
      );
      backoff.next();
      backoff.next();
      expect(backoff.attempt, 2);

      backoff.reset();
      expect(backoff.attempt, 0);
      expect(backoff.next(), const Duration(seconds: 1));
    });

    test('never exceeds max even with a large multiplier', () {
      final backoff = Backoff(
        initial: const Duration(seconds: 1),
        max: const Duration(seconds: 30),
        multiplier: 10,
      );
      expect(backoff.next(), const Duration(seconds: 1)); // 1 * 10^0
      expect(backoff.next(), const Duration(seconds: 10)); // 1 * 10^1
      expect(backoff.next(), const Duration(seconds: 30)); // 1 * 10^2 = 100s, capped
    });
  });
}
