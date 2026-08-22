import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/simulator.dart';
import '../providers/connection_provider.dart';
import '../providers/settings_provider.dart';
import '../providers/simulator_list_provider.dart';
import '../services/api_exception.dart';
import '../widgets/simulator_card.dart';
import 'control_screen.dart';
import 'settings_screen.dart';

class SimulatorListScreen extends StatefulWidget {
  const SimulatorListScreen({super.key});

  @override
  State<SimulatorListScreen> createState() => _SimulatorListScreenState();
}

class _SimulatorListScreenState extends State<SimulatorListScreen> {
  bool _connecting = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) context.read<SimulatorListProvider>().refresh();
    });
  }

  Future<void> _connect(Simulator simulator) async {
    if (_connecting) return;
    setState(() => _connecting = true);

    final settings = context.read<SettingsProvider>();
    final connection = context.read<ConnectionProvider>();
    try {
      await connection.connectToSimulator(
        simulator,
        settings.wsUri,
        config: settings.streamConfig,
      );
      if (!mounted) return;
      await Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => const ControlScreen()),
      );
    } on ApiException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text("Couldn't start session: ${e.message}")),
      );
    } finally {
      if (mounted) setState(() => _connecting = false);
    }
  }

  Widget _buildBody(SimulatorListProvider provider) {
    switch (provider.loadState) {
      case LoadState.idle:
      case LoadState.loading:
        return const Center(child: CircularProgressIndicator());
      case LoadState.error:
        return _MessageState(
          icon: Icons.cloud_off_rounded,
          message: provider.errorMessage ?? 'Something went wrong.',
          onRetry: () => context.read<SimulatorListProvider>().refresh(),
        );
      case LoadState.loaded:
        if (provider.simulators.isEmpty) {
          return _MessageState(
            icon: Icons.devices_other_rounded,
            message: 'No simulators found on the server yet.',
            onRetry: () => context.read<SimulatorListProvider>().refresh(),
          );
        }
        return ListView.builder(
          padding: const EdgeInsets.symmetric(vertical: 8),
          itemCount: provider.simulators.length,
          itemBuilder: (context, index) {
            final simulator = provider.simulators[index];
            return SimulatorCard(
              simulator: simulator,
              onTap: () => _connect(simulator),
            );
          },
        );
    }
  }

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<SimulatorListProvider>();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Simulators'),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            tooltip: 'Settings',
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => const SettingsScreen()),
            ),
          ),
        ],
      ),
      body: Stack(
        children: [
          RefreshIndicator(
            onRefresh: () => context.read<SimulatorListProvider>().refresh(),
            child: _buildBody(provider),
          ),
          if (_connecting)
            Container(
              color: Colors.black.withAlpha(60),
              child: const Center(child: CircularProgressIndicator()),
            ),
        ],
      ),
    );
  }
}

class _MessageState extends StatelessWidget {
  final IconData icon;
  final String message;
  final VoidCallback onRetry;

  const _MessageState({
    required this.icon,
    required this.message,
    required this.onRetry,
  });

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) => SingleChildScrollView(
        physics: const AlwaysScrollableScrollPhysics(),
        child: ConstrainedBox(
          constraints: BoxConstraints(minHeight: constraints.maxHeight),
          child: Center(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(icon, size: 48, color: Theme.of(context).disabledColor),
                  const SizedBox(height: 12),
                  Text(message, textAlign: TextAlign.center),
                  const SizedBox(height: 16),
                  OutlinedButton(onPressed: onRetry, child: const Text('Retry')),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
