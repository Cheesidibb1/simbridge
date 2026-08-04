import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'services/simbridge_service.dart';
import 'screens/connection_screen.dart';
import 'screens/simulator_screen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
    DeviceOrientation.landscapeLeft,
    DeviceOrientation.landscapeRight,
  ]);
  runApp(const SimBridgeApp());
}

class SimBridgeApp extends StatelessWidget {
  const SimBridgeApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'SimBridge',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
        brightness: Brightness.light,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blue,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
      home: const HomeScreen(),
    );
  }
}

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final SimBridgeService _service = SimBridgeService();
  bool _isConnected = false;

  @override
  void initState() {
    super.initState();
    _service.messageStream.listen((message) {
      // Handle messages
    });
    _service.errorStream.listen((error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error: $error')),
        );
      }
    });
  }

  @override
  void dispose() {
    _service.dispose();
    super.dispose();
  }

  Future<void> _showConnectionDialog() async {
    final result = await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (context) => ConnectionScreen(service: _service),
      ),
    );

    if (result == true && mounted) {
      setState(() {
        _isConnected = true;
      });
      _service.requestSimulatorList();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('SimBridge'),
        actions: [
          if (_isConnected)
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () {
                _service.requestSimulatorList();
              },
            ),
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () {
              // TODO: Navigate to settings
            },
          ),
        ],
      ),
      body: _isConnected ? _buildSimulatorList() : _buildNoConnection(),
      floatingActionButton: _isConnected
          ? null
          : FloatingActionButton.extended(
              onPressed: _showConnectionDialog,
              icon: const Icon(Icons.add),
              label: const Text('Connect Server'),
            ),
    );
  }

  Widget _buildNoConnection() {
    return const Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.cloud_off,
            size: 64,
            color: Colors.grey,
          ),
          SizedBox(height: 16),
          Text(
            'No server connected',
            style: TextStyle(fontSize: 18, color: Colors.grey),
          ),
          SizedBox(height: 8),
          Text(
            'Connect to a SimBridge server to get started',
            style: TextStyle(fontSize: 14, color: Colors.grey),
          ),
        ],
      ),
    );
  }

  Widget _buildSimulatorList() {
    return StreamBuilder<List>(
      stream: _service.simulatorsStream,
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          return const Center(child: CircularProgressIndicator());
        }

        final simulators = snapshot.data!;

        if (simulators.isEmpty) {
          return const Center(
            child: Text('No simulators available'),
          );
        }

        return ListView.builder(
          itemCount: simulators.length,
          itemBuilder: (context, index) {
            final simulator = simulators[index];
            return ListTile(
              leading: Icon(
                simulator.platform.name == 'ios' ? Icons.phone_iphone : Icons.android,
              ),
              title: Text(simulator.name),
              subtitle: Text('${simulator.osVersion} • ${simulator.status.name}'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(
                    builder: (context) => SimulatorScreen(
                      simulator: simulator,
                      service: _service,
                    ),
                  ),
                );
              },
            );
          },
        );
      },
    );
  }
}
