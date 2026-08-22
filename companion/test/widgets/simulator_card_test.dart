import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/models/simulator.dart';
import 'package:simbridge_client/widgets/simulator_card.dart';
import 'package:simbridge_client/widgets/status_badge.dart';

void main() {
  testWidgets('SimulatorCard shows the name, platform and status, and reports taps',
      (tester) async {
    const simulator = Simulator(
      id: 'ios-sim-1',
      name: 'iPhone 15 Pro',
      platform: SimulatorPlatform.ios,
      status: SimulatorStatus.available,
    );
    var tapped = false;

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SimulatorCard(
            simulator: simulator,
            onTap: () => tapped = true,
          ),
        ),
      ),
    );

    expect(find.text('iPhone 15 Pro'), findsOneWidget);
    expect(find.text('Available'), findsOneWidget);
    expect(find.byType(StatusBadge), findsOneWidget);

    await tester.tap(find.byType(SimulatorCard));
    await tester.pump();
    expect(tapped, isTrue);
  });

  testWidgets('shows the OS version when present', (tester) async {
    const simulator = Simulator(
      id: 'android-emu-1',
      name: 'Pixel 7',
      platform: SimulatorPlatform.android,
      status: SimulatorStatus.busy,
      osVersion: '14',
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SimulatorCard(simulator: simulator, onTap: () {}),
        ),
      ),
    );

    expect(find.textContaining('14'), findsOneWidget);
    expect(find.text('Busy'), findsOneWidget);
  });
}
