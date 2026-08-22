import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../models/client_payloads.dart';
import '../models/server_payloads.dart';
import '../providers/connection_provider.dart';
import '../services/websocket_service.dart';
import '../widgets/device_button_bar.dart';
import '../widgets/metrics_chip.dart';
import '../widgets/notification_banner.dart';
import '../widgets/screen_mirror.dart';

class ControlScreen extends StatefulWidget {
  const ControlScreen({super.key});

  @override
  State<ControlScreen> createState() => _ControlScreenState();
}

class _ControlScreenState extends State<ControlScreen> {
  ErrorPayload? _lastShownError;

  @override
  void initState() {
    super.initState();
    // Prime the recordings sheet with whatever's already on the server.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) context.read<ConnectionProvider>().requestRecordings();
    });
  }

  Future<void> _confirmDisconnect() async {
    final connection = context.read<ConnectionProvider>();
    final name = connection.currentSimulator?.name ?? 'this simulator';
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Disconnect?'),
        content: Text('End the session with $name?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('Disconnect'),
          ),
        ],
      ),
    );
    if (confirmed == true) {
      await connection.disconnect();
      if (mounted) Navigator.of(context).pop();
    }
  }

  void _showSheet(Widget child) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      builder: (_) => child,
    );
  }

  @override
  Widget build(BuildContext context) {
    final connection = context.watch<ConnectionProvider>();

    // Surface each new server-side error as a one-shot snackbar.
    final error = connection.lastError;
    if (error != null && !identical(error, _lastShownError)) {
      _lastShownError = error;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('${error.code.wire}: ${error.message}')),
        );
      });
    }

    final simulatorId = connection.currentSimulator?.id;

    return Scaffold(
      appBar: AppBar(
        title: Text(connection.currentSimulator?.name ?? 'Simulator'),
        actions: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Center(child: _ConnectionDot(state: connection.wsState)),
          ),
          IconButton(
            icon: const Icon(Icons.link_off_rounded),
            tooltip: 'Disconnect',
            onPressed: _confirmDisconnect,
          ),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: Stack(
              children: [
                Positioned.fill(
                  child: ScreenMirror(
                    frameBytes: connection.latestFrame,
                    simulatorWidth: connection.frameWidth ??
                        connection.currentSimulator?.screenSize?.width ??
                        0,
                    simulatorHeight: connection.frameHeight ??
                        connection.currentSimulator?.screenSize?.height ??
                        0,
                    onTouches: connection.sendTouch,
                    onSwipe: simulatorId == null
                        ? null
                        : (direction, distance) => connection.sendGesture(
                              GesturePayload.swipe(
                                simulatorId: simulatorId,
                                direction: direction,
                                distance: distance,
                              ),
                            ),
                    onPinch: simulatorId == null
                        ? null
                        : (scale) => connection.sendGesture(
                              GesturePayload.pinch(
                                  simulatorId: simulatorId, scale: scale),
                            ),
                    onLongPress: simulatorId == null
                        ? null
                        : (x, y) => connection.sendGesture(
                              GesturePayload.longPress(
                                  simulatorId: simulatorId, x: x, y: y),
                            ),
                  ),
                ),
                Positioned(
                  top: 8,
                  left: 8,
                  child: MetricsChip(metrics: connection.latestMetrics),
                ),
                if (connection.notifications.isNotEmpty)
                  Positioned(
                    top: 8,
                    right: 8,
                    left: 88,
                    child: NotificationBanner(
                      notification: connection.notifications.first,
                      onDismiss: () => connection
                          .dismissNotification(connection.notifications.first),
                    ),
                  ),
              ],
            ),
          ),
          const Divider(height: 1),
          DeviceButtonBar(onPressed: connection.sendDeviceButton),
          const Divider(height: 1),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
            child: Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: () => _showSheet(const _GpsSheet()),
                    icon: const Icon(Icons.location_on_outlined),
                    label: const Text('GPS'),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: () => _showSheet(const _ClipboardSheet()),
                    icon: const Icon(Icons.content_paste_rounded),
                    label: const Text('Clipboard'),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: () => _showSheet(const _RecordingSheet()),
                    icon: const Icon(Icons.fiber_manual_record_rounded),
                    label: const Text('Record'),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _ConnectionDot extends StatelessWidget {
  final WsConnectionState state;

  const _ConnectionDot({required this.state});

  Color get _color {
    switch (state) {
      case WsConnectionState.connected:
        return Colors.greenAccent;
      case WsConnectionState.connecting:
      case WsConnectionState.reconnecting:
        return Colors.orangeAccent;
      case WsConnectionState.disconnected:
        return Colors.redAccent;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: state.name,
      child: Container(
        width: 10,
        height: 10,
        decoration: BoxDecoration(color: _color, shape: BoxShape.circle),
      ),
    );
  }
}

class _GpsSheet extends StatefulWidget {
  const _GpsSheet();

  @override
  State<_GpsSheet> createState() => _GpsSheetState();
}

class _GpsSheetState extends State<_GpsSheet> {
  final _latController = TextEditingController(text: '37.7749');
  final _lngController = TextEditingController(text: '-122.4194');

  static const List<(String, double, double)> _presets = [
    ('San Francisco', 37.7749, -122.4194),
    ('New York', 40.7128, -74.0060),
    ('London', 51.5074, -0.1278),
    ('Tokyo', 35.6762, 139.6503),
  ];

  @override
  void dispose() {
    _latController.dispose();
    _lngController.dispose();
    super.dispose();
  }

  void _send() {
    final lat = double.tryParse(_latController.text.trim());
    final lng = double.tryParse(_lngController.text.trim());
    if (lat == null || lng == null) return;
    context.read<ConnectionProvider>().sendGps(latitude: lat, longitude: lng);
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(
        left: 20,
        right: 20,
        top: 20,
        bottom: MediaQuery.of(context).viewInsets.bottom + 20,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Set GPS location',
              style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _latController,
                  keyboardType: const TextInputType.numberWithOptions(
                      decimal: true, signed: true),
                  decoration: const InputDecoration(
                      labelText: 'Latitude', border: OutlineInputBorder()),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: TextField(
                  controller: _lngController,
                  keyboardType: const TextInputType.numberWithOptions(
                      decimal: true, signed: true),
                  decoration: const InputDecoration(
                      labelText: 'Longitude', border: OutlineInputBorder()),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: _presets.map((preset) {
              final (label, lat, lng) = preset;
              return ActionChip(
                label: Text(label),
                onPressed: () {
                  _latController.text = lat.toString();
                  _lngController.text = lng.toString();
                },
              );
            }).toList(),
          ),
          const SizedBox(height: 20),
          FilledButton.icon(
            onPressed: _send,
            icon: const Icon(Icons.send_rounded),
            label: const Text('Send to simulator'),
          ),
        ],
      ),
    );
  }
}

class _ClipboardSheet extends StatefulWidget {
  const _ClipboardSheet();

  @override
  State<_ClipboardSheet> createState() => _ClipboardSheetState();
}

class _ClipboardSheetState extends State<_ClipboardSheet> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _send() {
    if (_controller.text.isEmpty) return;
    context.read<ConnectionProvider>().sendClipboard(_controller.text);
    Navigator.of(context).pop();
  }

  Future<void> _pasteFromDevice() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    final text = data?.text;
    if (text != null && mounted) {
      setState(() => _controller.text = text);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(
        left: 20,
        right: 20,
        top: 20,
        bottom: MediaQuery.of(context).viewInsets.bottom + 20,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Sync clipboard',
              style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 12),
          TextField(
            controller: _controller,
            maxLines: 3,
            decoration: InputDecoration(
              labelText: 'Text to send',
              border: const OutlineInputBorder(),
              suffixIcon: IconButton(
                icon: const Icon(Icons.content_paste_go_rounded),
                tooltip: 'Paste from this device',
                onPressed: _pasteFromDevice,
              ),
            ),
          ),
          const SizedBox(height: 20),
          FilledButton.icon(
            onPressed: _send,
            icon: const Icon(Icons.send_rounded),
            label: const Text('Send to simulator'),
          ),
        ],
      ),
    );
  }
}

class _RecordingSheet extends StatelessWidget {
  const _RecordingSheet();

  @override
  Widget build(BuildContext context) {
    return Consumer<ConnectionProvider>(
      builder: (context, connection, _) {
        final status = connection.recordingStatus;
        final isActive = status?.status == RecordingStatusValue.recording ||
            status?.status == RecordingStatusValue.paused;
        return Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text('Screen recording',
                  style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 12),
              Text(
                status != null
                    ? '${status.status.name} · ${status.durationSeconds}s · '
                        '${(status.fileSizeBytes / 1024).toStringAsFixed(0)} KB'
                    : 'No active recording.',
              ),
              const SizedBox(height: 20),
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      onPressed: isActive ? null : connection.startRecording,
                      icon: const Icon(Icons.fiber_manual_record_rounded,
                          color: Colors.red),
                      label: const Text('Start'),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: OutlinedButton.icon(
                      onPressed: isActive ? connection.stopRecording : null,
                      icon: const Icon(Icons.stop_rounded),
                      label: const Text('Stop'),
                    ),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }
}
