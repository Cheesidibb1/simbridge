import 'dart:async';
import 'dart:convert';

import 'package:crypto/crypto.dart' as crypto;
import 'package:web_socket_channel/web_socket_channel.dart';

import '../models/client_payloads.dart';
import '../models/ws_envelope.dart';
import 'api_exception.dart';

class ServerPasswordService {
  static String createProof(String password, String challenge) {
    return crypto.Hmac(crypto.sha256, utf8.encode(password))
        .convert(utf8.encode(challenge))
        .toString();
  }

  static Future<void> verify(Uri wsUri, String password) async {
    final channel = WebSocketChannel.connect(wsUri);
    final completer = Completer<void>();
    StreamSubscription<dynamic>? subscription;
    var challengeAnswered = false;

    subscription = channel.stream.listen(
      (raw) {
        try {
          final text = raw is String ? raw : utf8.decode(raw as List<int>);
          final message =
              WsMessage.fromJson(jsonDecode(text) as Map<String, dynamic>);
          if (message.type == WsMessageType.authChallenge) {
            if (challengeAnswered) return;
            final challenge = message.payload['challenge'] as String?;
            if (challenge == null || challenge.isEmpty) {
              throw const ApiException(
                  'Server sent an invalid authentication challenge.');
            }
            challengeAnswered = true;
            channel.sink.add(jsonEncode(WsMessage.outgoing(
              type: WsMessageType.authRequest,
              payload: AuthRequestPayload(
                deviceId: 'server-setup',
                token: '',
                challengeResponse: createProof(password, challenge),
              ).toJson(),
            ).toJson()));
          } else if (message.type == WsMessageType.authResponse) {
            if (message.payload['success'] == true) {
              if (!completer.isCompleted) {
                completer.complete();
              }
            } else if (!completer.isCompleted) {
              completer.completeError(const ApiException('Incorrect server passcode.'));
            }
          }
        } catch (error, stackTrace) {
          if (!completer.isCompleted) {
            completer.completeError(error, stackTrace);
          }
        }
      },
      onError: (Object error, StackTrace stackTrace) {
        if (!completer.isCompleted) {
          completer.completeError(
            ApiException('Could not verify the server passcode: $error'),
            stackTrace,
          );
        }
      },
      onDone: () {
        if (!completer.isCompleted) {
          completer.completeError(const ApiException(
              'Server closed before passcode verification.'));
        }
      },
      cancelOnError: true,
    );

    try {
      await channel.ready.timeout(const Duration(seconds: 8));
      await completer.future.timeout(const Duration(seconds: 8));
    } on TimeoutException {
      throw const ApiException('Timed out verifying the server passcode.');
    } on ApiException {
      rethrow;
    } catch (error) {
      throw ApiException('Could not verify the server passcode: $error');
    } finally {
      await subscription.cancel();
      await channel.sink.close();
    }
  }
}
