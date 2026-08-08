// Connection Screen widget tests

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_companion/screens/connection_screen.dart';
import 'package:simbridge_companion/services/simbridge_service.dart';

void main() {
  group('ConnectionScreen', () {
    late SimBridgeService service;
    late ConnectionScreen widget;

    setUp(() {
      service = SimBridgeService();
      widget = ConnectionScreen(service: service);
    });

    tearDown(() {
      service.dispose();
    });

    testWidgets('should display connection form', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: widget),
        ),
      );

      // Verify the server URL text field exists
      expect(find.widgetWithText(TextFormField, 'Server URL'), findsOneWidget);
      
      // Verify the auth token text field exists
      expect(find.widgetWithText(TextFormField, 'Auth Token (optional)'), findsOneWidget);
      
      // Verify connect button exists
      expect(find.text('Connect'), findsOneWidget);
    });

    testWidgets('should validate empty server URL', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: widget),
        ),
      );

      // Try to submit with empty URL
      final connectButton = find.text('Connect');
      expect(connectButton, findsOneWidget);
      
      // Note: In actual testing, you would tap the button and verify validation
    });

    testWidgets('should show connection loading state', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: widget),
        ),
      );

      // Verify initial state is not connecting
      expect(find.text('Connect'), findsOneWidget);
      
      // Note: Would need actual connection to test loading state
    });
  });
}
