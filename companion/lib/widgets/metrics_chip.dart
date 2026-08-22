import 'package:flutter/material.dart';

import '../models/shared_payloads.dart';

class MetricsChip extends StatelessWidget {
  final MetricsUpdatePayload? metrics;

  const MetricsChip({super.key, required this.metrics});

  @override
  Widget build(BuildContext context) {
    final m = metrics;
    if (m == null) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
      decoration: BoxDecoration(
        color: Colors.black.withAlpha(150),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        '${m.fps.toStringAsFixed(0)} fps  ·  '
        '${m.latency.toStringAsFixed(0)} ms  ·  '
        'CPU ${m.cpuUsage.toStringAsFixed(0)}%',
        style: const TextStyle(color: Colors.white, fontSize: 11),
      ),
    );
  }
}
