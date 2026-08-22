import 'package:flutter/material.dart';

import 'app.dart';
import 'providers/settings_provider.dart';
import 'services/storage_service.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final storage = await StorageService.create();
  final settings = await SettingsProvider.load(storage);

  runApp(SimBridgeApp(settings: settings));
}
