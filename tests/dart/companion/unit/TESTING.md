# SimBridge Companion App Tests

## Overview

Unit and widget tests for the SimBridge companion mobile application running on Android and iOS devices.

## Test Organization

```
tests/dart/companion/unit/
├── protocol_test.dart      # Protocol message tests
├── simbridge_service_test.dart  # Service layer tests
├── gps_service_test.dart   # GPS/location service tests
├── connection_screen_test.dart  # UI widget tests
└── mock/                   # Mock implementations
```

## Running Tests

### Unit Tests (All)

```bash
cd companion
flutter test unit/
```

### Run Specific Test Files

```bash
# Run protocol tests only
flutter test unit/protocol_test.dart

# Run GPS service tests
flutter test unit/gps_service_test.dart

# Run all service layer tests
flutter test test/services/
```

### Coverage Report

```bash
# Install lcov and genhtml if needed
cd /path/to/lcov-report
genhtml --branch-coverage coverage/lcov.info -o coverage/html
```

## Test Types

### Unit Tests

Test individual components in isolation:
- Protocol message serialization
- Service initialization
- Data transformation logic

Example:
```dart
test('should serialize touch event correctly', () {
  final payload = {'x': 100, 'y': 200};
  // ... test assertion
});
```

### Widget Tests

Test Flutter UI components:
- Connection screen validation
- Simulator list display
- Touch controls layout
- Gesture feedback

Example:
```dart
testWidgets('connection form should validate input', (tester) async {
  await tester.pumpWidget(MaterialApp(body: widget));
  // ... test assertion
});
```

### Integration Tests

Test component interactions:
- GPS service with WebSocket client
- Service initialization flow
- Permission handling

Example:
```dart
test('gps service should stream location on connect', () async {
  await service.connect();
  // ... test assertion
});
```

## Test Setup

### Before Running Tests

1. Ensure Flutter is installed
2. Have a connected physical device or emulator
3. Grant location permissions (for GPS tests)

### Mock Services

Use Mockito for isolating component tests:

```dart
import 'package:mockito/mockito.dart';

class MockWebSocketClient extends Mock {
  @override
  bool get isConnected => false;
}
```

## GPS Service Tests

GPS service tests require special setup:

### On Android Device

1. Install app on real device (emulator lacks GPS)
2. Grant location permissions in app settings
3. Run `flutter test unit/gps_service_test.dart`

### Expected Behavior

- Should request location permissions
- Should stream location at configured interval
- Should handle permission denial gracefully
- Should dispose properly when stopped

## Test Coverage Goals

| Component | Target Coverage | Current Status |
|-----------|----------------|----------------|
| Protocol Messages | 100% | - |
| SimBridge Service | 95% | - |
| GPS Service | 90% | - |
| Connection Screen | 85% | - |
| Touch Controls | 95% | - |

## Running E2E Tests

End-to-end tests will be added in a future iteration using Flutter Driver.

```bash
# E2E tests (future)
flutter test integration/
```

## Debugging Tests

### View Test Output Verbosely

```bash
flutter test --verbose unit/protocol_test.dart
```

### Run Single Test Case

```bash
flutter test unit/protocol_test.dart --name "should serialize ping message"
```

### Update Golden Files (for widget tests)

```bash
flutter test --update-goldens unit/connection_screen_test.dart
```

## Continuous Integration

Tests run on every pull request in CI:

1. Unit tests pass → ✅
2. Widget tests pass → ✅
3. No linting errors → ✅

### Example GitHub Actions Workflow

```yaml
name: Companion App Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Flutter
        uses: subosito/flutter-action@v2
        with:
          channel: 'stable'
      
      - name: Run unit tests
        run: flutter test companion/test/
```

## Common Issues

### Permission Denies

```bash
# On first run, ensure location permissions are granted
adb shell pm grant com.simbridge android.permission.ACCESS_FINE_LOCATION
```

### Test Timing Issues

Use `await tester.pumpAndSettle()` for async operations:

```dart
await tester.pumpWidget(widget);
await tester.pumpAndSettle(); // Allow animations to complete
```

## Documentation Updates

When adding new test files, update this `TESTING.md` file with:
- Test coverage metrics
- New test categories
- Breaking changes in test setup
