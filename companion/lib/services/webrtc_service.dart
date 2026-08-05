// WebRTC service for screen streaming

import 'package:flutter_webrtc/flutter_webrtc.dart';
import '../networking/websocket_client.dart';
import '../protocol/message.dart' as protocol;

class WebRTCService {
  final WebSocketClient wsClient;
  RTCPeerConnection? _peerConnection;
  MediaStream? _localStream;
  MediaStream? _remoteStream;
  final Function(MediaStream)? onRemoteStream;
  final Function(String)? onError;

  WebRTCService({
    required this.wsClient,
    this.onRemoteStream,
    this.onError,
  });

  Future<void> initialize() async {
    final configuration = <String, dynamic>{
      'iceServers': [
        {'urls': 'stun:stun.l.google.com:19302'},
      ],
    };

    _peerConnection = await createPeerConnection(configuration);

    _peerConnection!.onIceCandidate = (candidate) {
      // Send ICE candidate to server
      _sendIceCandidate(candidate);
    };

    _peerConnection!.onIceConnectionState = (state) {
      print('ICE connection state: $state');
    };

    _peerConnection!.onAddStream = (stream) {
      print('Remote stream added');
      _remoteStream = stream;
      onRemoteStream?.call(stream);
    };
  }

  Future<void> createOffer(String sessionId) async {
    if (_peerConnection == null) {
      await initialize();
    }

    final offer = await _peerConnection!.createOffer();
    await _peerConnection!.setLocalDescription(offer);

    final message = protocol.Message(
      messageType: protocol.MessageType.screenFrame,
      payload: {
        'webrtc_type': 'offer',
        'sdp': offer.sdp,
        'session_id': sessionId,
      },
    );
    wsClient.send(message);
  }

  Future<void> handleAnswer(Map<String, dynamic> data) async {
    final sdp = data['sdp'] as String;
    final answer = RTCSessionDescription(sdp, 'answer');
    await _peerConnection?.setRemoteDescription(answer);
  }

  Future<void> handleIceCandidate(Map<String, dynamic> data) async {
    final candidate = data['candidate'] as String;
    final sdpMid = data['sdp_mid'] as String?;
    final sdpMlineIndex = data['sdp_mline_index'] as int?;

    final iceCandidate = RTCIceCandidate(
      candidate,
      sdpMid,
      sdpMlineIndex,
    );
    await _peerConnection?.addCandidate(iceCandidate);
  }

  void _sendIceCandidate(RTCIceCandidate candidate) {
    final message = protocol.Message(
      messageType: protocol.MessageType.screenFrame,
      payload: {
        'webrtc_type': 'ice_candidate',
        'candidate': candidate.candidate,
        'sdp_mid': candidate.sdpMid,
        'sdp_mline_index': candidate.sdpMLineIndex,
      },
    );
    wsClient.send(message);
  }

  Future<void> dispose() async {
    await _localStream?.dispose();
    await _remoteStream?.dispose();
    await _peerConnection?.close();
    _peerConnection = null;
  }
}
