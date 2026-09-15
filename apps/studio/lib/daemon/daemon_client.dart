/// Child-process transport for the Studio ↔ bdld protocol.
///
/// The only place in Studio that talks to the daemon.  It knows nothing
/// about application state: requests go in, responses and events come out.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:fixnum/fixnum.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'framing.dart';

class DaemonError implements Exception {
  const DaemonError(this.code, this.message, {this.detailsJson = ''});
  final String code;
  final String message;
  final String detailsJson;
  @override
  String toString() => '$code: $message';
}

/// What the effect executor needs from a daemon connection.  `DaemonClient`
/// is the real one over a child process; tests substitute a fake.
abstract interface class DaemonLink {
  Stream<pb.Event> get events;
  Stream<String> get stderrLines;
  Future<int> get exitCode;

  /// Send one request; resolves with the response payload or throws
  /// [DaemonError] when the daemon answered with an error.
  Future<pb.Response> request(pb.ClientMessage message);
  Future<void> shutdown();
}

class DaemonClient implements DaemonLink {
  DaemonClient._(this._process, this.executable);

  final Process _process;
  final String executable;
  final FrameDecoder _decoder = FrameDecoder();
  final Map<int, Completer<pb.Response>> _inflight = {};
  final StreamController<pb.Event> _events = StreamController.broadcast();
  final StreamController<String> _stderr = StreamController.broadcast();
  final Completer<int> _exit = Completer();
  int _nextId = 1;

  @override
  Stream<pb.Event> get events => _events.stream;
  @override
  Stream<String> get stderrLines => _stderr.stream;
  @override
  Future<int> get exitCode => _exit.future;

  static Future<DaemonClient> spawn(String executable) async {
    final process = await Process.start(executable, const ['serve']);
    final client = DaemonClient._(process, executable);
    process.stdout.listen(client._onStdout, onDone: client._onStdoutDone);
    process.stderr
        .transform(utf8.decoder)
        .transform(const LineSplitter())
        .listen(client._stderr.add);
    unawaited(process.exitCode.then(client._onExit));
    return client;
  }

  void _onStdout(List<int> chunk) {
    for (final frame in _decoder.feed(chunk)) {
      final msg = pb.ServerMessage.fromBuffer(frame);
      switch (msg.whichPayload()) {
        case pb.ServerMessage_Payload.response:
          final r = msg.response;
          _inflight.remove(r.requestId.toInt())?.complete(r);
        case pb.ServerMessage_Payload.event:
          _events.add(msg.event);
        case pb.ServerMessage_Payload.notSet:
          break;
      }
    }
  }

  void _onStdoutDone() {
    for (final c in _inflight.values) {
      c.completeError(const DaemonError('daemon.exited', 'the compiler service exited'));
    }
    _inflight.clear();
  }

  void _onExit(int code) {
    if (!_exit.isCompleted) _exit.complete(code);
    _events.close();
    _stderr.close();
  }

  @override
  Future<pb.Response> request(pb.ClientMessage message) {
    final id = _nextId++;
    message.requestId = Int64(id);
    final completer = Completer<pb.Response>();
    _inflight[id] = completer;
    _process.stdin.add(encodeFrame(message.writeToBuffer()));
    return completer.future.then((r) {
      if (r.whichPayload() == pb.Response_Payload.error) {
        throw DaemonError(r.error.code, r.error.message, detailsJson: r.error.detailsJson);
      }
      return r;
    });
  }

  @override
  Future<void> shutdown() async {
    try {
      await request(pb.ClientMessage(shutdown: pb.ShutdownRequest()))
          .timeout(const Duration(seconds: 2));
    } catch (_) {
      _process.kill();
    }
    await _process.stdin.close();
  }
}
