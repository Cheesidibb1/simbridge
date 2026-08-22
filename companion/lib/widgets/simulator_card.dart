import 'package:flutter/material.dart';

import '../models/simulator.dart';
import 'status_badge.dart';

class SimulatorCard extends StatelessWidget {
  final Simulator simulator;
  final VoidCallback onTap;

  const SimulatorCard({super.key, required this.simulator, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final icon = simulator.platform == SimulatorPlatform.ios
        ? Icons.phone_iphone_rounded
        : Icons.phone_android_rounded;

    final subtitleParts = <String>[
      simulator.platform.name.toUpperCase(),
      if (simulator.osVersion != null) simulator.osVersion!,
    ];

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      child: ListTile(
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        leading: CircleAvatar(child: Icon(icon)),
        title: Text(simulator.name, style: const TextStyle(fontWeight: FontWeight.w600)),
        subtitle: Text(subtitleParts.join(' · ')),
        trailing: StatusBadge(status: simulator.status),
        onTap: onTap,
      ),
    );
  }
}
