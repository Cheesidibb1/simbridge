import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';

import 'app.dart';
import 'providers/settings_provider.dart';
import 'services/storage_service.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (kIsWeb) {
    // The mirror decodes a fresh full-resolution frame many times a second.
    // Flutter's default image cache (100 MB / 1000 images) is fine for a
    // desktop browser but can push an iPhone tab over Safari's memory limit,
    // at which point iOS silently reloads the page. Keep only a few frames.
    final cache = PaintingBinding.instance.imageCache;
    cache.maximumSize = 8;
    cache.maximumSizeBytes = 48 << 20; // 48 MB
  }

  final storage = await StorageService.create();
  final settings = await SettingsProvider.load(storage);

  runApp(SimBridgeApp(settings: settings));
}
