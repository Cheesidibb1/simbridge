/// Represents this client (or a remembered client) as the server sees it.
/// Used by the pairing flow and stored locally once paired.
library;

class Device {
  final String id;
  final String name;
  final String deviceType; // 'android' | 'ios' | 'desktop'
  final String? platform;
  final String? osVersion;
  final DateTime? pairedAt;
  final DateTime? lastSeen;
  final bool isTrusted;
  final String? publicKey;

  const Device({
    required this.id,
    required this.name,
    required this.deviceType,
    this.platform,
    this.osVersion,
    this.pairedAt,
    this.lastSeen,
    this.isTrusted = false,
    this.publicKey,
  });

  factory Device.fromJson(Map<String, dynamic> json) {
    return Device(
      id: json['id'] as String,
      name: json['name'] as String,
      deviceType: json['device_type'] as String? ?? 'android',
      platform: json['platform'] as String?,
      osVersion: json['os_version'] as String?,
      pairedAt: json['paired_at'] != null ? DateTime.parse(json['paired_at'] as String) : null,
      lastSeen: json['last_seen'] != null ? DateTime.parse(json['last_seen'] as String) : null,
      isTrusted: json['is_trusted'] as bool? ?? false,
      publicKey: json['public_key'] as String?,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'device_type': deviceType,
        if (platform != null) 'platform': platform,
        if (osVersion != null) 'os_version': osVersion,
        if (pairedAt != null) 'paired_at': pairedAt!.toIso8601String(),
        if (lastSeen != null) 'last_seen': lastSeen!.toIso8601String(),
        'is_trusted': isTrusted,
        if (publicKey != null) 'public_key': publicKey,
      };
}
