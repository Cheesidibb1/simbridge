import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'providers/connection_provider.dart';
import 'providers/settings_provider.dart';
import 'providers/simulator_list_provider.dart';
import 'screens/onboarding_screen.dart';
import 'screens/simulator_list_screen.dart';
import 'services/api_client.dart';

/// Root widget. Owns the long-lived, non-widget objects (the REST client
/// and the two top-level providers) for the app's whole lifetime, and
/// keeps [ApiClient.baseUrl] in sync whenever the user edits the server
/// address in Settings.
class SimBridgeApp extends StatefulWidget {
  final SettingsProvider settings;

  const SimBridgeApp({super.key, required this.settings});

  @override
  State<SimBridgeApp> createState() => _SimBridgeAppState();
}

class _SimBridgeAppState extends State<SimBridgeApp> {
  late final ApiClient _apiClient;
  late final SimulatorListProvider _simulatorListProvider;
  late final ConnectionProvider _connectionProvider;

  @override
  void initState() {
    super.initState();
    _apiClient = ApiClient(baseUrl: widget.settings.httpBaseUrl);
    _simulatorListProvider = SimulatorListProvider(apiClient: _apiClient);
    _connectionProvider = ConnectionProvider(
      apiClient: _apiClient,
      deviceId: widget.settings.deviceId,
    );
    widget.settings.addListener(_syncApiClientBaseUrl);
  }

  void _syncApiClientBaseUrl() {
    _apiClient.baseUrl = widget.settings.httpBaseUrl;
  }

  @override
  void dispose() {
    widget.settings.removeListener(_syncApiClientBaseUrl);
    _connectionProvider.dispose();
    _simulatorListProvider.dispose();
    _apiClient.close();
    super.dispose();
  }

  ThemeMode _themeModeFor(String mode) {
    switch (mode) {
      case 'light':
        return ThemeMode.light;
      case 'dark':
        return ThemeMode.dark;
      default:
        return ThemeMode.system;
    }
  }

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider<SettingsProvider>.value(value: widget.settings),
        ChangeNotifierProvider<SimulatorListProvider>.value(value: _simulatorListProvider),
        ChangeNotifierProvider<ConnectionProvider>.value(value: _connectionProvider),
      ],
      child: Consumer<SettingsProvider>(
        builder: (context, settings, _) {
          return MaterialApp(
            title: 'SimBridge',
            debugShowCheckedModeBanner: false,
            themeMode: _themeModeFor(settings.themeMode),
            theme: ThemeData(
              colorSchemeSeed: Colors.indigo,
              useMaterial3: true,
              brightness: Brightness.light,
            ),
            darkTheme: ThemeData(
              colorSchemeSeed: Colors.indigo,
              useMaterial3: true,
              brightness: Brightness.dark,
            ),
            home: settings.onboardingComplete
                ? const SimulatorListScreen()
                : const OnboardingScreen(),
          );
        },
      ),
    );
  }
}
