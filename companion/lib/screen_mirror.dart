import 'dart:typed_data';

import 'package:flutter/material.dart';

import './models/client_payloads.dart';
import './utils/coordinate_mapper.dart';

/// Renders the latest [frameBytes] (a decoded JPEG/PNG frame from a
/// `ScreenFrame` message) and forwards pointer events, mapped into the
/// simulator's coordinate space, via [onTouches].
///
/// The mirror is wrapped in an [AspectRatio] matching the simulator's
/// reported resolution so the displayed image and the touch-mapping math
/// always agree on the same rectangle — no letterboxing to account for.
class ScreenMirror extends StatefulWidget {
  final Uint8List? frameBytes;
  final int simulatorWidth;
  final int simulatorHeight;
  final void Function(List<Touch> touches) onTouches;
  final VoidCallback? onDoubleTap;

  const ScreenMirror({
    super.key,
    required this.frameBytes,
    required this.simulatorWidth,
    required this.simulatorHeight,
    required this.onTouches,
    this.onDoubleTap,
  });

  @override
  State<ScreenMirror> createState() => _ScreenMirrorState();
}

class _ScreenMirrorState extends State<ScreenMirror> {
  Size _renderedSize = Size.zero;

  void _emit(PointerEvent event, TouchPhase phase) {
    if (_renderedSize == Size.zero ||
        widget.simulatorWidth <= 0 ||
        widget.simulatorHeight <= 0) {
      return;
    }
    final mapped = mapWidgetPointToSimulator(
      widgetX: event.localPosition.dx,
      widgetY: event.localPosition.dy,
      widgetWidth: _renderedSize.width,
      widgetHeight: _renderedSize.height,
      simulatorWidth: widget.simulatorWidth,
      simulatorHeight: widget.simulatorHeight,
    );
    widget.onTouches([
      Touch(id: event.pointer, x: mapped.x, y: mapped.y, phase: phase),
    ]);
  }

  @override
  Widget build(BuildContext context) {
    final hasResolution = widget.simulatorWidth > 0 && widget.simulatorHeight > 0;
    final aspect = hasResolution
        ? widget.simulatorWidth / widget.simulatorHeight
        : 9 / 19.5; // sensible placeholder before the first frame arrives

    return ColoredBox(
      color: Colors.black,
      child: Center(
        child: AspectRatio(
          aspectRatio: aspect,
          child: LayoutBuilder(
            builder: (context, constraints) {
              _renderedSize = Size(constraints.maxWidth, constraints.maxHeight);
              return GestureDetector(
                onDoubleTap: widget.onDoubleTap,
                child: Listener(
                  behavior: HitTestBehavior.opaque,
                  onPointerDown: (e) => _emit(e, TouchPhase.began),
                  onPointerMove: (e) => _emit(e, TouchPhase.moved),
                  onPointerUp: (e) => _emit(e, TouchPhase.ended),
                  onPointerCancel: (e) => _emit(e, TouchPhase.cancelled),
                  child: widget.frameBytes == null
                      ? const Center(
                          child: Text(
                            'Waiting for screen…',
                            style: TextStyle(color: Colors.white54),
                          ),
                        )
                      : Image.memory(
                          widget.frameBytes!,
                          gaplessPlayback: true,
                          fit: BoxFit.fill,
                          width: double.infinity,
                          height: double.infinity,
                        ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}
