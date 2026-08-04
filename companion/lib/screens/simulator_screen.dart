// Simulator control screen

import 'package:flutter/material.dart';
import '../models/simulator.dart';
import '../services/simbridge_service.dart';

class SimulatorScreen extends StatefulWidget {
  final Simulator simulator;
  final SimBridgeService service;

  const SimulatorScreen({
    super.key,
    required this.simulator,
    required this.service,
  });

  @override
  State<SimulatorScreen> createState() => _SimulatorScreenState();
}

class _SimulatorScreenState extends State<SimulatorScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.simulator.name),
        actions: [
          IconButton(
            icon: const Icon(Icons.gps_fixed),
            onPressed: () {
              // TODO: Show GPS controls
            },
          ),
          IconButton(
            icon: const Icon(Icons.notifications),
            onPressed: () {
              // TODO: Show notifications
            },
          ),
        ],
      ),
      body: Column(
        children: [
          // Screen placeholder
          Expanded(
            child: Container(
              color: Colors.black,
              child: const Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(
                      Icons.phone_android,
                      size: 64,
                      color: Colors.white24,
                    ),
                    SizedBox(height: 16),
                    Text(
                      'Screen streaming not yet implemented',
                      style: TextStyle(color: Colors.white54),
                    ),
                  ],
                ),
              ),
            ),
          ),
          // Control buttons
          _buildControlPanel(),
        ],
      ),
    );
  }

  Widget _buildControlPanel() {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 4,
            offset: const Offset(0, -2),
          ),
        ],
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          _buildControlButton(
            icon: Icons.home,
            label: 'Home',
            onPressed: () {
              widget.service.sendTouchEvent({
                'simulator_id': widget.simulator.id,
                'button': 'home',
              });
            },
          ),
          _buildControlButton(
            icon: Icons.arrow_back,
            label: 'Back',
            onPressed: () {
              widget.service.sendTouchEvent({
                'simulator_id': widget.simulator.id,
                'button': 'back',
              });
            },
          ),
          _buildControlButton(
            icon: Icons.apps,
            label: 'Apps',
            onPressed: () {
              widget.service.sendTouchEvent({
                'simulator_id': widget.simulator.id,
                'button': 'app_switcher',
              });
            },
          ),
          _buildControlButton(
            icon: Icons.power_settings_new,
            label: 'Lock',
            onPressed: () {
              widget.service.sendTouchEvent({
                'simulator_id': widget.simulator.id,
                'button': 'lock',
              });
            },
          ),
        ],
      ),
    );
  }

  Widget _buildControlButton({
    required IconData icon,
    required String label,
    required VoidCallback onPressed,
  }) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          icon: Icon(icon),
          onPressed: onPressed,
          iconSize: 32,
        ),
        Text(
          label,
          style: Theme.of(context).textTheme.bodySmall,
        ),
      ],
    );
  }
}
