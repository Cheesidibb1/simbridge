import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import './providers/settings_provider.dart';
import 'simulator_list_screen.dart';

class OnboardingScreen extends StatefulWidget {
  const OnboardingScreen({super.key});

  @override
  State<OnboardingScreen> createState() => _OnboardingScreenState();
}

class _OnboardingScreenState extends State<OnboardingScreen> {
  final _formKey = GlobalKey<FormState>();
  late final TextEditingController _hostController;
  late final TextEditingController _portController;
  late final TextEditingController _nameController;
  bool _useTls = false;

  @override
  void initState() {
    super.initState();
    final settings = context.read<SettingsProvider>();
    _hostController = TextEditingController(text: settings.serverHost);
    _portController = TextEditingController(text: settings.serverPort.toString());
    _nameController = TextEditingController(text: settings.deviceName);
    _useTls = settings.useTls;
  }

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _continue() async {
    if (!_formKey.currentState!.validate()) return;
    final settings = context.read<SettingsProvider>();
    await settings.updateServer(
      host: _hostController.text.trim(),
      port: int.parse(_portController.text.trim()),
      tls: _useTls,
    );
    await settings.updateDeviceName(_nameController.text.trim());
    if (!mounted) return;
    Navigator.of(context).pushReplacement(
      MaterialPageRoute(builder: (_) => const SimulatorListScreen()),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 420),
              child: Form(
                key: _formKey,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Icon(
                      Icons.phonelink_rounded,
                      size: 64,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'Connect to SimBridge',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'Point this device at the SimBridge server running on your Mac.',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                    const SizedBox(height: 28),
                    TextFormField(
                      controller: _hostController,
                      decoration: const InputDecoration(
                        labelText: 'Server host or IP',
                        hintText: '192.168.1.20',
                        prefixIcon: Icon(Icons.dns_outlined),
                        border: OutlineInputBorder(),
                      ),
                      validator: (value) =>
                          (value == null || value.trim().isEmpty) ? 'Required' : null,
                    ),
                    const SizedBox(height: 12),
                    TextFormField(
                      controller: _portController,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                        labelText: 'Port',
                        prefixIcon: Icon(Icons.numbers_rounded),
                        border: OutlineInputBorder(),
                      ),
                      validator: (value) {
                        final port = int.tryParse(value?.trim() ?? '');
                        if (port == null || port <= 0 || port > 65535) {
                          return 'Enter a valid port';
                        }
                        return null;
                      },
                    ),
                    const SizedBox(height: 12),
                    TextFormField(
                      controller: _nameController,
                      decoration: const InputDecoration(
                        labelText: 'This device\'s name',
                        prefixIcon: Icon(Icons.smartphone_rounded),
                        border: OutlineInputBorder(),
                      ),
                      validator: (value) =>
                          (value == null || value.trim().isEmpty) ? 'Required' : null,
                    ),
                    const SizedBox(height: 4),
                    SwitchListTile(
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Use TLS (wss:// / https://)'),
                      value: _useTls,
                      onChanged: (value) => setState(() => _useTls = value),
                    ),
                    const SizedBox(height: 20),
                    FilledButton.icon(
                      onPressed: _continue,
                      icon: const Icon(Icons.arrow_forward_rounded),
                      label: const Padding(
                        padding: EdgeInsets.symmetric(vertical: 12),
                        child: Text('Continue'),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
