// Touch canvas for capturing and sending touch events

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import '../services/simbridge_service.dart';

class TouchCanvas extends StatefulWidget {
  final String simulatorId;
  final SimBridgeService service;
  final Widget? child;

  const TouchCanvas({
    super.key,
    required this.simulatorId,
    required this.service,
    this.child,
  });

  @override
  State<TouchCanvas> createState() => _TouchCanvasState();
}

class _TouchCanvasState extends State<TouchCanvas> {
  final Map<int, Offset> _touchPoints = {};

  @override
  Widget build(BuildContext context) {
    return RawGestureDetector(
      behavior: HitTestBehavior.opaque,
      gestures: {
        ScaleGestureRecognizer: GestureRecognizerFactoryWithHandlers<ScaleGestureRecognizer>(
          () => ScaleGestureRecognizer(),
          (ScaleGestureRecognizer instance) {
            instance.onStart = _handleScaleStart;
            instance.onUpdate = _handleScaleUpdate;
            instance.onEnd = _handleScaleEnd;
          },
        ),
        PanGestureRecognizer: GestureRecognizerFactoryWithHandlers<PanGestureRecognizer>(
          () => PanGestureRecognizer(),
          (PanGestureRecognizer instance) {
            instance.onStart = _handlePanStart;
            instance.onUpdate = _handlePanUpdate;
            instance.onEnd = _handlePanEnd;
          },
        ),
        TapGestureRecognizer: GestureRecognizerFactoryWithHandlers<TapGestureRecognizer>(
          () => TapGestureRecognizer(),
          (TapGestureRecognizer instance) {
            instance.onTapDown = _handleTapDown;
            instance.onTapUp = _handleTapUp;
          },
        ),
        LongPressGestureRecognizer: GestureRecognizerFactoryWithHandlers<LongPressGestureRecognizer>(
          () => LongPressGestureRecognizer(),
          (LongPressGestureRecognizer instance) {
            instance.onLongPressStart = _handleLongPressStart;
          },
        ),
      },
      child: Listener(
        onPointerDown: _handlePointerDown,
        onPointerMove: _handlePointerMove,
        onPointerUp: _handlePointerUp,
        onPointerCancel: _handlePointerCancel,
        child: widget.child ?? const SizedBox.expand(),
      ),
    );
  }

  void _handlePointerDown(PointerDownEvent event) {
    _touchPoints[event.pointer] = event.position;
    _sendTouchEvent('began', event.position, event.pointer);
  }

  void _handlePointerMove(PointerMoveEvent event) {
    _touchPoints[event.pointer] = event.position;
    _sendTouchEvent('moved', event.position, event.pointer);
  }

  void _handlePointerUp(PointerUpEvent event) {
    _touchPoints.remove(event.pointer);
    _sendTouchEvent('ended', event.position, event.pointer);
  }

  void _handlePointerCancel(PointerCancelEvent event) {
    _touchPoints.remove(event.pointer);
    _sendTouchEvent('cancelled', event.position, event.pointer);
  }

  void _handleTapDown(TapDownDetails details) {
    _sendGestureEvent('tap', details.localPosition);
  }

  void _handleTapUp(TapUpDetails details) {
    _sendGestureEvent('tap_up', details.localPosition);
  }

  void _handleLongPressStart(LongPressStartDetails details) {
    _sendGestureEvent('long_press', details.localPosition);
  }

  void _handleScaleStart(ScaleStartDetails details) {
    _sendGestureEvent('pinch_start', details.localFocalPoint);
  }

  void _handleScaleUpdate(ScaleUpdateDetails details) {
    _sendGestureEvent('pinch', details.localFocalPoint, extra: {
      'scale': details.scale,
    });
  }

  void _handleScaleEnd(ScaleEndDetails details) {
    _sendGestureEvent('pinch_end', Offset.zero);
  }

  void _handlePanStart(DragStartDetails details) {
    _sendGestureEvent('pan_start', details.localPosition);
  }

  void _handlePanUpdate(DragUpdateDetails details) {
    _sendGestureEvent('pan', details.localPosition, extra: {
      'delta': {'dx': details.delta.dx, 'dy': details.delta.dy},
    });
  }

  void _handlePanEnd(DragEndDetails details) {
    _sendGestureEvent('pan_end', Offset.zero, extra: {
      'velocity': {'vx': details.velocity.pixelsPerSecond.dx, 'vy': details.velocity.pixelsPerSecond.dy},
    });
  }

  void _sendTouchEvent(String phase, Offset position, int pointer) {
    final touches = _touchPoints.entries.map((entry) {
      return {
        'id': entry.key,
        'x': entry.value.dx,
        'y': entry.value.dy,
        'phase': _touchPoints[pointer] == position ? phase : 'stationary',
      };
    }).toList();

    widget.service.sendTouchEvent({
      'simulator_id': widget.simulatorId,
      'touches': touches,
    });
  }

  void _sendGestureEvent(String gestureType, Offset position, {Map<String, dynamic>? extra}) {
    final payload = <String, dynamic>{
      'simulator_id': widget.simulatorId,
      'gesture_type': gestureType,
      'data': {
        'x': position.dx,
        'y': position.dy,
        ...?extra,
      },
    };

    widget.service.sendGesture(payload);
  }
}
