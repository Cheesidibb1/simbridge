// This is a basic Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:simbridge_client/app.dart';
import 'package:simbridge_client/providers/settings_provider.dart';
import 'package:simbridge_client/services/storage_service.dart';

void main() {
  testWidgets('shows onboarding on first launch', (WidgetTester tester) async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final settings = await SettingsProvider.load(StorageService(preferences));

    await tester.pumpWidget(SimBridgeApp(settings: settings));

    expect(find.text('Connect to SimBridge'), findsOneWidget);
  });
}
