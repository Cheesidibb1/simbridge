import 'package:flutter/material.dart';

import './models/client_payloads.dart';

class DeviceButtonBar extends StatelessWidget {
  final void Function(DeviceButtonType button) onPressed;

  const DeviceButtonBar({super.key, required this.onPressed});

  static const List<(IconData, String, DeviceButtonType)> _buttons = [
    (Icons.arrow_back_rounded, 'Back', DeviceButtonType.back),
    (Icons.circle_outlined, 'Home', DeviceButtonType.home),
    (Icons.apps_rounded, 'Recents', DeviceButtonType.appSwitcher),
    (Icons.rotate_left_rounded, 'Rotate L', DeviceButtonType.rotateLeft),
    (Icons.rotate_right_rounded, 'Rotate R', DeviceButtonType.rotateRight),
    (Icons.volume_down_rounded, 'Vol -', DeviceButtonType.volumeDown),
    (Icons.volume_up_rounded, 'Vol +', DeviceButtonType.volumeUp),
    (Icons.volume_off_rounded, 'Mute', DeviceButtonType.mute),
    (Icons.lock_outline_rounded, 'Lock', DeviceButtonType.lock),
    (Icons.lock_open_rounded, 'Unlock', DeviceButtonType.unlock),
    (Icons.vibration_rounded, 'Shake', DeviceButtonType.shake),
    (Icons.camera_alt_outlined, 'Screenshot', DeviceButtonType.screenshot),
  ];

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 56,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        itemCount: _buttons.length,
        separatorBuilder: (_, __) => const SizedBox(width: 4),
        itemBuilder: (context, index) {
          final (icon, label, type) = _buttons[index];
          return IconButton.filledTonal(
            icon: Icon(icon),
            tooltip: label,
            onPressed: () => onPressed(type),
          );
        },
      ),
    );
  }
}
