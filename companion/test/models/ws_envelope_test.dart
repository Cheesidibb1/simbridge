import 'package:flutter_test/flutter_test.dart';
import 'package:simbridge_client/models/ws_envelope.dart';

void main() {
  group('WsMessage', () {
    test('round-trips a known message type through JSON', () {
      final outgoing = WsMessage.outgoing(
        type: WsMessageType.ping,
        requestId: 'req-1',
      );
      final json = outgoing.toJson();
      expect(json['message_type'], 'ping');
      expect(json['request_id'], 'req-1');
      expect(json['version'], 1);

      final parsed = WsMessage.fromJson(json);
      expect(parsed.type, WsMessageType.ping);
      expect(parsed.requestId, 'req-1');
    });

    test('an unrecognized message_type parses to a null type, not a throw', () {
      final parsed = WsMessage.fromJson({
        'message_type': 'SomethingFromANewerServer',
        'version': 1,
        'timestamp': '2024-01-15T10:30:00Z',
        'payload': <String, dynamic>{},
      });
      expect(parsed.type, isNull);
      expect(parsed.rawType, 'SomethingFromANewerServer');
    });

    test('a missing timestamp falls back instead of throwing', () {
      final parsed = WsMessage.fromJson({
        'message_type': 'pong',
        'payload': {'status': 'ok'},
      });
      expect(parsed.type, WsMessageType.pong);
      expect(parsed.payload['status'], 'ok');
    });

    test('every enum value maps to the exact snake_case wire string', () {
      const expectedWireStrings = {
        WsMessageType.pairRequest: 'pair_request',
        WsMessageType.authRequest: 'auth_request',
        WsMessageType.simulatorList: 'simulator_list',
        WsMessageType.connectSimulator: 'connect_simulator',
        WsMessageType.disconnectSimulator: 'disconnect_simulator',
        WsMessageType.touchEvent: 'touch_event',
        WsMessageType.gesture: 'gesture',
        WsMessageType.gpsUpdate: 'gps_update',
        WsMessageType.headingUpdate: 'heading_update',
        WsMessageType.motionUpdate: 'motion_update',
        WsMessageType.deviceButton: 'device_button',
        WsMessageType.clipboardSync: 'clipboard_sync',
        WsMessageType.fileTransfer: 'file_transfer',
        WsMessageType.startRecording: 'start_recording',
        WsMessageType.stopRecording: 'stop_recording',
        WsMessageType.getRecordings: 'get_recordings',
        WsMessageType.ping: 'ping',
        WsMessageType.pairResponse: 'pair_response',
        WsMessageType.authResponse: 'auth_response',
        WsMessageType.screenFrame: 'screen_frame',
        WsMessageType.notification: 'notification',
        WsMessageType.recordingStatus: 'recording_status',
        WsMessageType.pong: 'pong',
        WsMessageType.error: 'error',
        WsMessageType.settingsUpdate: 'settings_update',
        WsMessageType.sessionInfo: 'session_info',
        WsMessageType.metricsUpdate: 'metrics_update',
        WsMessageType.webrtcOffer: 'webrtc_offer',
        WsMessageType.webrtcAnswer: 'webrtc_answer',
        WsMessageType.webrtcIceCandidate: 'webrtc_ice_candidate',
      };

      for (final entry in expectedWireStrings.entries) {
        expect(entry.key.wire, entry.value);
        expect(WsMessageType.fromWire(entry.value), entry.key);
      }
      expect(expectedWireStrings.length, WsMessageType.values.length);
    });
  });
}
