import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../providers/settings_provider.dart';
import '../services/api_exception.dart';
import '../services/server_password_service.dart';
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
  late final TextEditingController _passwordController;
  bool _useTls = false;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    final settings = context.read<SettingsProvider>();
    _hostController = TextEditingController(text: settings.serverHost);
    _portController =
        TextEditingController(text: settings.serverPort.toString());
    _nameController = TextEditingController(text: settings.deviceName);
    _passwordController = TextEditingController();
    _useTls = settings.effectiveTls;
  }

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    _nameController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  Future<void> _continue() async {
    if (_saving || !_formKey.currentState!.validate()) return;
    final settings = context.read<SettingsProvider>();
    final host = _hostController.text.trim();
    final port = int.parse(_portController.text.trim());
    final password = _passwordController.text;
    final tls = settings.tlsRequired || _useTls;
    setState(() => _saving = true);
    try {
      await ServerPasswordService.verify(
        Uri(scheme: tls ? 'wss' : 'ws', host: host, port: port, path: '/ws'),
        password,
      );
      await settings.updateServer(
        host: host,
        port: port,
        tls: tls,
        serverPassword: password,
      );
      await settings.updateDeviceName(_nameController.text.trim());
      await settings.completeOnboarding();
      if (!mounted) return;
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => const SimulatorListScreen()),
      );
    } on ApiException catch (error) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(error.message)),
      );
    } finally {
      if (mounted) setState(() => _saving = false);
    }
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
                      // iOS keyboards otherwise capitalise the first letter and
                      // "correct" IP addresses / hostnames.
                      keyboardType: TextInputType.url,
                      autocorrect: false,
                      enableSuggestions: false,
                      textCapitalization: TextCapitalization.none,
                      decoration: const InputDecoration(
                        labelText: 'Server host or IP',
                        hintText: '192.168.1.20',
                        prefixIcon: Icon(Icons.dns_outlined),
                        border: OutlineInputBorder(),
                      ),
                      validator: (value) =>
                          (value == null || value.trim().isEmpty)
                              ? 'Required'
                              : null,
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
                      controller: _passwordController,
                      obscureText: true,
                      autocorrect: false,
                      enableSuggestions: false,
                      decoration: const InputDecoration(
                        labelText: 'Server passcode',
                        prefixIcon: Icon(Icons.lock_outline_rounded),
                        border: OutlineInputBorder(),
                      ),
                      validator: (value) => (value == null || value.length < 12)
                          ? 'Use at least 12 characters'
                          : null,
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
                          (value == null || value.trim().isEmpty)
                              ? 'Required'
                              : null,
                    ),
                    const SizedBox(height: 4),
                    SwitchListTile(
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Use TLS (wss:// / https://)'),
                      // A page loaded over https can't open insecure sockets
                      // (Safari blocks mixed content), so the switch is locked on.
                      subtitle: context.read<SettingsProvider>().tlsRequired
                          ? const Text(
                              'Required: this page was loaded over https')
                          : null,
                      value: _useTls,
                      onChanged: context.read<SettingsProvider>().tlsRequired
                          ? null
                          : (value) => setState(() => _useTls = value),
                    ),
                    const SizedBox(height: 20),
                    FilledButton.icon(
                      onPressed: _saving ? null : _continue,
                      icon: _saving
                          ? const SizedBox.square(
                              dimension: 18,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.arrow_forward_rounded),
                      label: const Padding(
                        padding: EdgeInsets.symmetric(vertical: 12),
                        child: Text('Verify server and continue'),
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
