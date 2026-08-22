import 'dart:typed_data';

import 'package:flutter/material.dart';

import '../models/client_payloads.dart';
import '../utils/coordinate_mapper.dart';

/// Renders the latest [frameBytes] (a decoded JPEG/PNG frame from a
/// `ScreenFrame` message) and turns on-screen interaction into protocol
/// messages two ways at once, matching the doc's separate `TouchEvent` and
/// `Gesture` message types:
///
/// - A confirmed tap is forwarded via [onTouches] as a [Touch].
/// - The same interaction is classified into a single-finger swipe,
///   a two-finger pinch, or a long press, and reported once via
///   [onSwipe] / [onPinch] / [onLongPress] — useful when the server's
///   simulator backend only exposes higher-level gesture commands rather
///   than raw multitouch injection.
///
/// The mirror is wrapped in an [AspectRatio] matching the simulator's
/// reported resolution so the displayed image and the touch-mapping math
/// always agree on the same rectangle — no letterboxing to account for.
class ScreenMirror extends StatefulWidget {
  final Uint8List? frameBytes;
  final int simulatorWidth;
  final int simulatorHeight;
  final void Function(List<Touch> touches) onTouches;
  final void Function(SwipeDirection direction, double distance)? onSwipe;
  final void Function(double scale)? onPinch;
  final void Function(double simX, double simY)? onLongPress;
  final void Function(double simX, double simY)? onDoubleTap;

  const ScreenMirror({
    super.key,
    required this.frameBytes,
    required this.simulatorWidth,
    required this.simulatorHeight,
    required this.onTouches,
    this.onSwipe,
    this.onPinch,
    this.onLongPress,
    this.onDoubleTap,
  });

  @override
  State<ScreenMirror> createState() => _ScreenMirrorState();
}

class _ScreenMirrorState extends State<ScreenMirror> {
  static const double _minSwipeDistance = 24;
  static const double _minPinchDelta = 0.05;

  Size _renderedSize = Size.zero;

  Offset? _gestureStart;
  Offset _gestureCurrent = Offset.zero;
  int _pointerCountAtStart = 0;
  double _scaleAtEnd = 1.0;

  bool get _hasResolution =>
      widget.simulatorWidth > 0 && widget.simulatorHeight > 0;

  SimulatorPoint _mapLocal(Offset local) {
    return mapWidgetPointToSimulator(
      widgetX: local.dx,
      widgetY: local.dy,
      widgetWidth: _renderedSize.width,
      widgetHeight: _renderedSize.height,
      simulatorWidth: widget.simulatorWidth,
      simulatorHeight: widget.simulatorHeight,
    );
  }

  void _emitTap(TapUpDetails details) {
    if (_renderedSize == Size.zero || !_hasResolution) return;
    final mapped = _mapLocal(details.localPosition);
    widget.onTouches([
      Touch(id: 0, x: mapped.x, y: mapped.y, phase: TouchPhase.began),
    ]);
  }

  void _handleScaleStart(ScaleStartDetails details) {
    _gestureStart = details.localFocalPoint;
    _gestureCurrent = details.localFocalPoint;
    _pointerCountAtStart = details.pointerCount;
    _scaleAtEnd = 1.0;
  }

  void _handleScaleUpdate(ScaleUpdateDetails details) {
    _gestureCurrent = details.localFocalPoint;
    _scaleAtEnd = details.scale;
    if (details.pointerCount > _pointerCountAtStart) {
      _pointerCountAtStart = details.pointerCount;
    }
  }

  void _handleScaleEnd(ScaleEndDetails details) {
    final start = _gestureStart;
    _gestureStart = null;
    if (start == null || !_hasResolution) return;

    if (_pointerCountAtStart >= 2) {
      if (widget.onPinch != null &&
          (_scaleAtEnd - 1.0).abs() > _minPinchDelta) {
        widget.onPinch!(_scaleAtEnd);
      }
      return;
    }

    final delta = _gestureCurrent - start;
    if (widget.onSwipe != null && delta.distance >= _minSwipeDistance) {
      final direction = delta.dx.abs() > delta.dy.abs()
          ? (delta.dx > 0 ? SwipeDirection.right : SwipeDirection.left)
          : (delta.dy > 0 ? SwipeDirection.down : SwipeDirection.up);
      widget.onSwipe!(direction, delta.distance);
    }
  }

  void _handleLongPressStart(LongPressStartDetails details) {
    if (widget.onLongPress == null || !_hasResolution) return;
    final mapped = _mapLocal(details.localPosition);
    widget.onLongPress!(mapped.x, mapped.y);
  }

  @override
  Widget build(BuildContext context) {
    final aspect = _hasResolution
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
                onScaleStart: _handleScaleStart,
                onScaleUpdate: _handleScaleUpdate,
                onScaleEnd: _handleScaleEnd,
                onLongPressStart: _handleLongPressStart,
                onTapUp: _emitTap,
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
              );
            },
          ),
        ),
      ),
    );
  }
}
