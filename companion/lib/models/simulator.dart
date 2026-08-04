// Simulator model

class Simulator {
  final String id;
  final String name;
  final SimulatorPlatform platform;
  final String osVersion;
  final SimulatorStatus status;
  final ScreenSize screenSize;

  Simulator({
    required this.id,
    required this.name,
    required this.platform,
    required this.osVersion,
    required this.status,
    required this.screenSize,
  });

  factory Simulator.fromJson(Map<String, dynamic> json) {
    return Simulator(
      id: json['id'],
      name: json['name'],
      platform: SimulatorPlatform.fromString(json['platform']),
      osVersion: json['os_version'],
      status: SimulatorStatus.fromString(json['status']),
      screenSize: ScreenSize.fromJson(json['screen_size']),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'platform': platform.toString().split('.').last,
      'os_version': osVersion,
      'status': status.toString().split('.').last,
      'screen_size': screenSize.toJson(),
    };
  }
}

enum SimulatorPlatform {
  ios,
  android;

  static SimulatorPlatform fromString(String value) {
    return SimulatorPlatform.values.firstWhere(
      (e) => e.toString().split('.').last == value.toLowerCase(),
      orElse: () => SimulatorPlatform.android,
    );
  }
}

enum SimulatorStatus {
  available,
  busy,
  offline,
  error;

  static SimulatorStatus fromString(String value) {
    return SimulatorStatus.values.firstWhere(
      (e) => e.toString().split('.').last == value.toLowerCase(),
      orElse: () => SimulatorStatus.offline,
    );
  }
}

class ScreenSize {
  final int width;
  final int height;
  final double scale;

  ScreenSize({
    required this.width,
    required this.height,
    required this.scale,
  });

  factory ScreenSize.fromJson(Map<String, dynamic> json) {
    return ScreenSize(
      width: json['width'],
      height: json['height'],
      scale: json['scale'].toDouble(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'width': width,
      'height': height,
      'scale': scale,
    };
  }
}
