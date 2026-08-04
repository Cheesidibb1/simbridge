import 'package:flutter/material.dart';

void main() {
  runApp(const SimBridgeDesktop());
}

class SimBridgeDesktop extends StatelessWidget {
  const SimBridgeDesktop({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'SimBridge Desktop',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      home: const DashboardScreen(),
    );
  }
}

class DashboardScreen extends StatefulWidget {
  const DashboardScreen({super.key});

  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('SimBridge Dashboard'),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () {
              // TODO: Settings
            },
          ),
        ],
      ),
      body: Row(
        children: [
          // Sidebar
          Container(
            width: 250,
            color: Theme.of(context).colorScheme.surfaceVariant,
            child: ListView(
              children: [
                _buildSidebarItem(
                  icon: Icons.dashboard,
                  title: 'Dashboard',
                  isSelected: true,
                ),
                _buildSidebarItem(
                  icon: Icons.devices,
                  title: 'Devices',
                ),
                _buildSidebarItem(
                  icon: Icons.phone_android,
                  title: 'Simulators',
                ),
                _buildSidebarItem(
                  icon: Icons.router,
                  title: 'Sessions',
                ),
                _buildSidebarItem(
                  icon: Icons.videocam,
                  title: 'Recordings',
                ),
                _buildSidebarItem(
                  icon: Icons.extension,
                  title: 'Plugins',
                ),
                const Divider(),
                _buildSidebarItem(
                  icon: Icons.analytics,
                  title: 'Metrics',
                ),
                _buildSidebarItem(
                  icon: Icons.description,
                  title: 'Logs',
                ),
              ],
            ),
          ),
          // Main content
          Expanded(
            child: _buildDashboardContent(),
          ),
        ],
      ),
    );
  }

  Widget _buildSidebarItem({
    required IconData icon,
    required String title,
    bool isSelected = false,
  }) {
    return ListTile(
      leading: Icon(icon),
      title: Text(title),
      selected: isSelected,
      selectedTileColor: Theme.of(context).colorScheme.primaryContainer,
      onTap: () {
        // TODO: Navigate
      },
    );
  }

  Widget _buildDashboardContent() {
    return Padding(
      padding: const EdgeInsets.all(24.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Dashboard',
            style: TextStyle(fontSize: 32, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 24),
          // Stats cards
          Row(
            children: [
              _buildStatCard(
                icon: Icons.devices,
                title: 'Connected Devices',
                value: '0',
                color: Colors.blue,
              ),
              const SizedBox(width: 16),
              _buildStatCard(
                icon: Icons.phone_android,
                title: 'Active Simulators',
                value: '0',
                color: Colors.green,
              ),
              const SizedBox(width: 16),
              _buildStatCard(
                icon: Icons.router,
                title: 'Active Sessions',
                value: '0',
                color: Colors.orange,
              ),
              const SizedBox(width: 16),
              _buildStatCard(
                icon: Icons.videocam,
                title: 'Recordings',
                value: '0',
                color: Colors.purple,
              ),
            ],
          ),
          const SizedBox(height: 24),
          // Activity log placeholder
          Expanded(
            child: Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Recent Activity',
                      style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 16),
                    const Center(
                      child: Text(
                        'No recent activity',
                        style: TextStyle(color: Colors.grey),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatCard({
    required IconData icon,
    required String title,
    required String value,
    required Color color,
  }) {
    return Expanded(
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(icon, color: color, size: 32),
              const SizedBox(height: 8),
              Text(
                title,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              const SizedBox(height: 4),
              Text(
                value,
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                      color: color,
                      fontWeight: FontWeight.bold,
                    ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
