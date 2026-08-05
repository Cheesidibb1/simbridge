// Simulator control screen

import 'package:flutter/material.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';
import '../models/simulator.dart';
import '../services/simbridge_service.dart';
import '../services/webrtc_service.dart';
import '../widgets/touch_canvas.dart';

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
  WebRTCService? _webrtcService;
  MediaStream? _remoteStream;
  RTCVideoRenderer _remoteRenderer = RTCVideoRenderer();
  bool _isGpsStreaming = false;

  @override
  void initState() {
    super.initState();
    _initRenderer();
    _startWebRTC();
  }

  Future<void> _initRenderer() async {
    await _remoteRenderer.initialize();
  }

  Future<void> _startWebRTC() async {
    _webrtcService = WebRTCService(
      wsClient: widget.service.wsClient,
      onRemoteStream: (stream) {
        setState(() {
          _remoteStream = stream;
          _remoteRenderer.srcObject = stream;
        });
      },
      onError: (error) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('WebRTC error: $error')),
        );
      },
    );

    await _webrtcService!.createOffer('session-id');
  }

  void _toggleGpsStreaming() {
    setState(() {
      _isGpsStreaming = !_isGpsStreaming;
    });

    if (_isGpsStreaming) {
      widget.service.startGpsStreaming(widget.simulator.id);
    } else {
      widget.service.stopGpsStreaming();
    }
  }

  @override
  void dispose() {
    widget.service.stopGpsStreaming();
    _remoteRenderer.dispose();
    _webrtcService?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.simulator.name),
        actions: [
          IconButton(
            icon: Icon(_isGpsStreaming ? Icons.gps_fixed : Icons.gps_not_fixed),
            onPressed: _toggleGpsStreaming,
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
          // Screen streaming
          Expanded(
            child: _remoteStream != null
                ? TouchCanvas(
                    simulatorId: widget.simulator.id,
                    service: widget.service,
                    child: RTCVideoView(_remoteRenderer),
                  )
                : Container(
                    color: Colors.black,
                    child: const Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          CircularProgressIndicator(),
                          SizedBox(height: 16),
                          Text(
                            'Connecting to simulator...',
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
            color: Colors.black.withValues(alpha: 0.1),
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
