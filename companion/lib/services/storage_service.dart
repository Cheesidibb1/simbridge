import 'package:shared_preferences/shared_preferences.dart';

import '../utils/constants.dart';

/// Thin, typed wrapper over [SharedPreferences] so the rest of the app
/// never touches raw string keys directly.
class StorageService {
  final SharedPreferences _prefs;

  StorageService(this._prefs);

  static Future<StorageService> create() async {
    final prefs = await SharedPreferences.getInstance();
    return StorageService(prefs);
  }

  String get serverHost => _prefs.getString(PrefsKeys.serverHost) ?? AppDefaults.serverHost;
  Future<void> setServerHost(String value) => _prefs.setString(PrefsKeys.serverHost, value);

  int get serverPort => _prefs.getInt(PrefsKeys.serverPort) ?? AppDefaults.serverPort;
  Future<void> setServerPort(int value) => _prefs.setInt(PrefsKeys.serverPort, value);

  bool get useTls => _prefs.getBool(PrefsKeys.useTls) ?? AppDefaults.useTls;
  Future<void> setUseTls(bool value) => _prefs.setBool(PrefsKeys.useTls, value);

  String? get deviceId => _prefs.getString(PrefsKeys.deviceId);
  Future<void> setDeviceId(String value) => _prefs.setString(PrefsKeys.deviceId, value);

  String get deviceName => _prefs.getString(PrefsKeys.deviceName) ?? 'Flutter Companion';
  Future<void> setDeviceName(String value) => _prefs.setString(PrefsKeys.deviceName, value);

  String get streamQuality => _prefs.getString(PrefsKeys.streamQuality) ?? 'high';
  Future<void> setStreamQuality(String value) =>
      _prefs.setString(PrefsKeys.streamQuality, value);

  int get streamFps => _prefs.getInt(PrefsKeys.streamFps) ?? AppDefaults.streamFps;
  Future<void> setStreamFps(int value) => _prefs.setInt(PrefsKeys.streamFps, value);

  bool get audioEnabled => _prefs.getBool(PrefsKeys.audioEnabled) ?? false;
  Future<void> setAudioEnabled(bool value) => _prefs.setBool(PrefsKeys.audioEnabled, value);

  /// One of 'system', 'light', 'dark'.
  String get themeMode => _prefs.getString(PrefsKeys.themeMode) ?? 'system';
  Future<void> setThemeMode(String value) => _prefs.setString(PrefsKeys.themeMode, value);

  String? get authToken => _prefs.getString(PrefsKeys.authToken);
  Future<void> setAuthToken(String? value) async {
    if (value == null) {
      await _prefs.remove(PrefsKeys.authToken);
    } else {
      await _prefs.setString(PrefsKeys.authToken, value);
    }
  }

  String? get lastSimulatorId => _prefs.getString(PrefsKeys.lastSimulatorId);
  Future<void> setLastSimulatorId(String? value) async {
    if (value == null) {
      await _prefs.remove(PrefsKeys.lastSimulatorId);
    } else {
      await _prefs.setString(PrefsKeys.lastSimulatorId, value);
    }
  }
}
