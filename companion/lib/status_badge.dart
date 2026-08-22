import 'package:flutter/material.dart';

import './models/simulator.dart';

class StatusBadge extends StatelessWidget {
  final SimulatorStatus status;

  const StatusBadge({super.key, required this.status});

  Color _colorFor(SimulatorStatus status) {
    switch (status) {
      case SimulatorStatus.available:
        return Colors.green;
      case SimulatorStatus.busy:
        return Colors.orange;
      case SimulatorStatus.offline:
        return Colors.grey;
      case SimulatorStatus.error:
        return Colors.red;
    }
  }

  String _labelFor(SimulatorStatus status) {
    switch (status) {
      case SimulatorStatus.available:
        return 'Available';
      case SimulatorStatus.busy:
        return 'Busy';
      case SimulatorStatus.offline:
        return 'Offline';
      case SimulatorStatus.error:
        return 'Error';
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = _colorFor(status);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withAlpha(30),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(color: color, shape: BoxShape.circle),
          ),
          const SizedBox(width: 6),
          Text(
            _labelFor(status),
            style: TextStyle(color: color, fontSize: 12, fontWeight: FontWeight.w600),
          ),
        ],
      ),
    );
  }
}
