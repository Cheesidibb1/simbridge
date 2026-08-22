import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/session.dart';
import '../providers/settings_provider.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  late final TextEditingController _hostController;
  late final TextEditingController _portController;
  late final TextEditingController _nameController;

  @override
  void initState() {
    super.initState();
    final settings = context.read<SettingsProvider>();
    _hostController = TextEditingController(text: settings.serverHost);
    _portController = TextEditingController(text: settings.serverPort.toString());
    _nameController = TextEditingController(text: settings.deviceName);
  }

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    _nameController.dispose();
    super.dispose();
  }

  void _saveServer(SettingsProvider settings) {
    final port = int.tryParse(_portController.text.trim());
    if (port == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Enter a valid port number')),
      );
      return;
    }
    settings.updateServer(
      host: _hostController.text.trim(),
      port: port,
      tls: settings.useTls,
    );
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Server address saved')),
    );
  }

  @override
  Widget build(BuildContext context) {
    final settings = context.watch<SettingsProvider>();

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const _SectionHeader('Server'),
          TextField(
            controller: _hostController,
            decoration:
                const InputDecoration(labelText: 'Host or IP', border: OutlineInputBorder()),
            onSubmitted: (_) => _saveServer(settings),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _portController,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(labelText: 'Port', border: OutlineInputBorder()),
            onSubmitted: (_) => _saveServer(settings),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: const Text('Use TLS (wss:// / https://)'),
            value: settings.useTls,
            onChanged: (value) => settings.updateServer(
              host: _hostController.text.trim(),
              port: int.tryParse(_portController.text.trim()) ?? settings.serverPort,
              tls: value,
            ),
          ),
          const SizedBox(height: 4),
          FilledButton(
            onPressed: () => _saveServer(settings),
            child: const Text('Save server address'),
          ),
          const Divider(height: 32),
          const _SectionHeader('This device'),
          TextField(
            controller: _nameController,
            decoration:
                const InputDecoration(labelText: 'Device name', border: OutlineInputBorder()),
            onSubmitted: (value) => settings.updateDeviceName(value.trim()),
          ),
          const SizedBox(height: 8),
          Text(
            'Device ID: ${settings.deviceId}',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const Divider(height: 32),
          const _SectionHeader('Screen mirroring'),
          DropdownButtonFormField<StreamQuality>(
            initialValue: settings.streamQuality,
            decoration:
                const InputDecoration(labelText: 'Stream quality', border: OutlineInputBorder()),
            items: StreamQuality.values
                .map((quality) => DropdownMenuItem(value: quality, child: Text(quality.name)))
                .toList(),
            onChanged: (value) {
              if (value != null) settings.updateStreamConfig(quality: value);
            },
          ),
          const SizedBox(height: 16),
          Text('Target frame rate: ${settings.streamFps} fps'),
          Slider(
            value: settings.streamFps.toDouble(),
            min: 5,
            max: 60,
            divisions: 11,
            label: '${settings.streamFps} fps',
            onChanged: (value) => settings.updateStreamConfig(fps: value.round()),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: const Text('Audio'),
            value: settings.audioEnabled,
            onChanged: (value) => settings.updateStreamConfig(audioEnabled: value),
          ),
          const Divider(height: 32),
          const _SectionHeader('Appearance'),
          SegmentedButton<String>(
            segments: const [
              ButtonSegment(value: 'system', label: Text('System')),
              ButtonSegment(value: 'light', label: Text('Light')),
              ButtonSegment(value: 'dark', label: Text('Dark')),
            ],
            selected: {settings.themeMode},
            onSelectionChanged: (selection) => settings.updateThemeMode(selection.first),
          ),
        ],
      ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  final String title;

  const _SectionHeader(this.title);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(
        title,
        style: Theme.of(context).textTheme.titleSmall?.copyWith(
              color: Theme.of(context).colorScheme.primary,
              fontWeight: FontWeight.bold,
            ),
      ),
    );
  }
}
