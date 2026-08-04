// GPS service for streaming location data

import 'dart:async';
import 'package:geolocator/geolocator.dart';

class GpsService {
  final StreamController<Map<String, dynamic>> _locationController =
      StreamController.broadcast();
  StreamSubscription<Position>? _positionSubscription;
  bool _isStreaming = false;

  Stream<Map<String, dynamic>> get locationStream => _locationController.stream;
  bool get isStreaming => _isStreaming;

  Future<bool> checkPermissions() async {
    bool serviceEnabled = await Geolocator.isLocationServiceEnabled();
    if (!serviceEnabled) {
      return false;
    }

    LocationPermission permission = await Geolocator.checkPermission();
    if (permission == LocationPermission.denied) {
      permission = await Geolocator.requestPermission();
      if (permission == LocationPermission.denied) {
        return false;
      }
    }

    if (permission == LocationPermission.deniedForever) {
      return false;
    }

    return true;
  }

  Future<void> startStreaming({int intervalSeconds = 1}) async {
    if (_isStreaming) return;

    final hasPermission = await checkPermissions();
    if (!hasPermission) {
      throw Exception('Location permission denied');
    }

    final locationSettings = LocationSettings(
      accuracy: LocationAccuracy.high,
      distanceFilter: 0,
    );

    _positionSubscription = Geolocator.getPositionStream(locationSettings: locationSettings)
        .listen((Position position) {
      final locationData = {
        'latitude': position.latitude,
        'longitude': position.longitude,
        'altitude': position.altitude,
        'accuracy': position.accuracy,
        'speed': position.speed,
        'heading': position.heading,
        'timestamp': position.timestamp.toIso8601String(),
      };
      _locationController.add(locationData);
    });

    _isStreaming = true;
  }

  void stopStreaming() {
    _positionSubscription?.cancel();
    _positionSubscription = null;
    _isStreaming = false;
  }

  Future<Position> getCurrentPosition() async {
    final hasPermission = await checkPermissions();
    if (!hasPermission) {
      throw Exception('Location permission denied');
    }

    return await Geolocator.getCurrentPosition(
      desiredAccuracy: LocationAccuracy.high,
    );
  }

  void dispose() {
    stopStreaming();
    _locationController.close();
  }
}
