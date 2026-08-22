/// Models for the `/api/v1/simulators` REST resource and the simulator
/// records embedded in WebSocket payloads.
library;

enum SimulatorPlatform {
  ios,
  android;

  static SimulatorPlatform fromWire(String value) {
    switch (value.toLowerCase()) {
      case 'ios':
        return SimulatorPlatform.ios;
      case 'android':
        return SimulatorPlatform.android;
      default:
        return SimulatorPlatform.android;
    }
  }

  String toWire() => name;
}

enum SimulatorStatus {
  available,
  busy,
  offline,
  error;

  static SimulatorStatus fromWire(String value) {
    return SimulatorStatus.values.firstWhere(
      (s) => s.name == value,
      orElse: () => SimulatorStatus.offline,
    );
  }

  String toWire() => name;
}

class ScreenSize {
  final int width;
  final int height;
  final double scale;

  const ScreenSize({
    required this.width,
    required this.height,
    this.scale = 1.0,
  });

  factory ScreenSize.fromJson(Map<String, dynamic> json) {
    return ScreenSize(
      width: (json['width'] as num?)?.toInt() ?? 0,
      height: (json['height'] as num?)?.toInt() ?? 0,
      scale: (json['scale'] as num?)?.toDouble() ?? 1.0,
    );
  }

  Map<String, dynamic> toJson() => {
        'width': width,
        'height': height,
        'scale': scale,
      };
}

class DeviceDetails {
  final String? deviceType;
  final String? model;
  final String? manufacturer;
  final int? cpuCores;
  final int? memoryMb;

  const DeviceDetails({
    this.deviceType,
    this.model,
    this.manufacturer,
    this.cpuCores,
    this.memoryMb,
  });

  factory DeviceDetails.fromJson(Map<String, dynamic> json) {
    return DeviceDetails(
      deviceType: json['device_type'] as String?,
      model: json['model'] as String?,
      manufacturer: json['manufacturer'] as String?,
      cpuCores: (json['cpu_cores'] as num?)?.toInt(),
      memoryMb: (json['memory_mb'] as num?)?.toInt(),
    );
  }

  Map<String, dynamic> toJson() => {
        if (deviceType != null) 'device_type': deviceType,
        if (model != null) 'model': model,
        if (manufacturer != null) 'manufacturer': manufacturer,
        if (cpuCores != null) 'cpu_cores': cpuCores,
        if (memoryMb != null) 'memory_mb': memoryMb,
      };
}

class Simulator {
  final String id;
  final String name;
  final SimulatorPlatform platform;
  final String? osVersion;
  final SimulatorStatus status;
  final ScreenSize? screenSize;
  final DeviceDetails? deviceDetails;

  const Simulator({
    required this.id,
    required this.name,
    required this.platform,
    required this.status,
    this.osVersion,
    this.screenSize,
    this.deviceDetails,
  });

  factory Simulator.fromJson(Map<String, dynamic> json) {
    return Simulator(
      id: json['id'] as String,
      name: json['name'] as String,
      platform: SimulatorPlatform.fromWire(json['platform'] as String? ?? 'android'),
      osVersion: json['os_version'] as String?,
      status: SimulatorStatus.fromWire(json['status'] as String? ?? 'offline'),
      screenSize: json['screen_size'] != null
          ? ScreenSize.fromJson(json['screen_size'] as Map<String, dynamic>)
          : null,
      deviceDetails: json['device_details'] != null
          ? DeviceDetails.fromJson(json['device_details'] as Map<String, dynamic>)
          : null,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'platform': platform.toWire(),
        if (osVersion != null) 'os_version': osVersion,
        'status': status.toWire(),
        if (screenSize != null) 'screen_size': screenSize!.toJson(),
        if (deviceDetails != null) 'device_details': deviceDetails!.toJson(),
      };

  Simulator copyWith({SimulatorStatus? status}) {
    return Simulator(
      id: id,
      name: name,
      platform: platform,
      osVersion: osVersion,
      status: status ?? this.status,
      screenSize: screenSize,
      deviceDetails: deviceDetails,
    );
  }
}

/// Response wrapper for `GET /api/v1/simulators`.
class SimulatorListResponse {
  final List<Simulator> simulators;

  const SimulatorListResponse(this.simulators);

  factory SimulatorListResponse.fromJson(Map<String, dynamic> json) {
    final rawList = (json['simulators'] as List<dynamic>?) ?? const [];
    return SimulatorListResponse(
      rawList
          .map((e) => Simulator.fromJson(e as Map<String, dynamic>))
          .toList(growable: false),
    );
  }
}
