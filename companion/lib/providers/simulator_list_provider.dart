import 'package:flutter/foundation.dart';

import '../models/simulator.dart';
import '../services/api_client.dart';
import '../services/api_exception.dart';

enum LoadState { idle, loading, loaded, error }

/// Fetches and holds the simulator/emulator list from `GET
/// /api/v1/simulators`, exposing loading and error state for the list
/// screen to render directly.
class SimulatorListProvider extends ChangeNotifier {
  final ApiClient apiClient;

  SimulatorListProvider({required this.apiClient});

  LoadState loadState = LoadState.idle;
  List<Simulator> simulators = const [];
  String? errorMessage;

  Future<void> refresh() async {
    loadState = LoadState.loading;
    errorMessage = null;
    notifyListeners();
    try {
      simulators = await apiClient.getSimulators();
      loadState = LoadState.loaded;
    } on ApiException catch (e) {
      errorMessage = e.message;
      loadState = LoadState.error;
    } catch (e) {
      errorMessage = 'Unexpected error: $e';
      loadState = LoadState.error;
    }
    notifyListeners();
  }
}
