/// App-wide constant values. Kept in one place so defaults referenced from
/// settings, providers, and services can't drift out of sync with each
/// other.
class PrefsKeys {
  static const String serverHost = 'server_host';
  static const String serverPort = 'server_port';
  static const String useTls = 'use_tls';
  static const String deviceId = 'device_id';
  static const String deviceName = 'device_name';
  static const String streamQuality = 'stream_quality';
  static const String streamFps = 'stream_fps';
  static const String audioEnabled = 'audio_enabled';
  static const String themeMode = 'theme_mode';
  static const String authToken = 'auth_token';
  static const String lastSimulatorId = 'last_simulator_id';
  static const String onboardingComplete = 'onboarding_complete';
}

class AppDefaults {
  static const String serverHost = 'localhost';
  static const int serverPort = 8080;
  static const bool useTls = false;
  static const int streamFps = 30;
  static const Duration pingInterval = Duration(seconds: 15);
  static const Duration reconnectInitialDelay = Duration(seconds: 1);
  static const Duration reconnectMaxDelay = Duration(seconds: 30);
  static const int protocolVersion = 1;
}
