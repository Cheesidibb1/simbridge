import 'package:flutter/foundation.dart';
import 'package:uuid/uuid.dart';

import '../models/session.dart';
import '../services/storage_service.dart';

/// Holds the user-editable connection and stream settings, persisting each
/// change through [StorageService]. Created once at startup via [load] so
/// the device id is generated (and saved) exactly once per install.
class SettingsProvider extends ChangeNotifier {
  final StorageService _storage;

  String serverHost;
  int serverPort;
  bool useTls;
  final String deviceId;
  String deviceName;
  StreamQuality streamQuality;
  int streamFps;
  bool audioEnabled;
  String themeMode; // 'system' | 'light' | 'dark'
  bool onboardingComplete;

  SettingsProvider._({
    required StorageService storage,
    required this.serverHost,
    required this.serverPort,
    required this.useTls,
    required this.deviceId,
    required this.deviceName,
    required this.streamQuality,
    required this.streamFps,
    required this.audioEnabled,
    required this.themeMode,
    required this.onboardingComplete,
  }) : _storage = storage;

  static Future<SettingsProvider> load(StorageService storage) async {
    var deviceId = storage.deviceId;
    if (deviceId == null) {
      deviceId = const Uuid().v4();
      await storage.setDeviceId(deviceId);
    }
    return SettingsProvider._(
      storage: storage,
      serverHost: storage.serverHost,
      serverPort: storage.serverPort,
      useTls: storage.useTls,
      deviceId: deviceId,
      deviceName: storage.deviceName,
      streamQuality: StreamQuality.fromWire(storage.streamQuality),
      streamFps: storage.streamFps,
      audioEnabled: storage.audioEnabled,
      themeMode: storage.themeMode,
      onboardingComplete: storage.onboardingComplete,
    );
  }

  String get httpBaseUrl => '${useTls ? 'https' : 'http'}://$serverHost:$serverPort';

  Uri get wsUri => Uri(
        scheme: useTls ? 'wss' : 'ws',
        host: serverHost,
        port: serverPort,
        path: '/ws',
      );

  StreamConfig get streamConfig => StreamConfig(
        quality: streamQuality,
        fps: streamFps,
        audioEnabled: audioEnabled,
      );

  Future<void> updateServer({
    required String host,
    required int port,
    required bool tls,
  }) async {
    serverHost = host;
    serverPort = port;
    useTls = tls;
    await _storage.setServerHost(host);
    await _storage.setServerPort(port);
    await _storage.setUseTls(tls);
    notifyListeners();
  }

  Future<void> updateDeviceName(String name) async {
    deviceName = name;
    await _storage.setDeviceName(name);
    notifyListeners();
  }

  Future<void> updateStreamConfig({
    StreamQuality? quality,
    int? fps,
    bool? audioEnabled,
  }) async {
    if (quality != null) {
      streamQuality = quality;
      await _storage.setStreamQuality(quality.toWire());
    }
    if (fps != null) {
      streamFps = fps;
      await _storage.setStreamFps(fps);
    }
    if (audioEnabled != null) {
      this.audioEnabled = audioEnabled;
      await _storage.setAudioEnabled(audioEnabled);
    }
    notifyListeners();
  }

  Future<void> updateThemeMode(String mode) async {
    themeMode = mode;
    await _storage.setThemeMode(mode);
    notifyListeners();
  }

  Future<void> completeOnboarding() async {
    onboardingComplete = true;
    await _storage.setOnboardingComplete(true);
    notifyListeners();
  }
}
