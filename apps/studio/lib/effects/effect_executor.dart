/// The imperative shell: executes [Effect]s and feeds [ResponseAction]s back.
///
/// This is the only code that touches the daemon process.  It holds the
/// client handle (an asynchronous resource), nothing semantic.
library;

import 'dart:async';

import 'package:fixnum/fixnum.dart';

import '../app/actions.dart';
import '../app/effects.dart';
import '../daemon/daemon_client.dart';
import '../daemon/daemon_locator.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../protocol/versions.dart';

typedef Dispatch = void Function(AppAction action);

class EffectExecutor {
  EffectExecutor(this._dispatch, {String Function()? locate}) : _locate = locate ?? locateDaemon;

  final Dispatch _dispatch;
  final String Function() _locate;
  DaemonClient? _client;
  final List<StreamSubscription<Object?>> _subs = [];

  Future<void> run(Effect effect) async {
    switch (effect) {
      case ConnectDaemon():
        await _connect();
      case OpenProject(:final rootPath):
        await _project(pb.ClientMessage(openProject: pb.OpenProjectRequest(rootPath: rootPath)));
      case InitProject(:final rootPath, :final name):
        await _project(
          pb.ClientMessage(
            initProject: pb.InitProjectRequest(rootPath: rootPath, name: name),
          ),
        );
      case SaveProject():
        await _project(pb.ClientMessage(saveProject: pb.SaveProjectRequest()));
      case CloseProject():
        await _call(pb.ClientMessage(closeProject: pb.CloseProjectRequest()), (_) {
          _dispatch(const ProjectClosed());
        });
      case SubscribeProject():
        await _call(
          pb.ClientMessage(subscribeProject: pb.SubscribeProjectRequest()),
          (_) {},
          counted: false,
        );
      case ApplyEdit(:final baseRevision, :final op):
        await _call(
          pb.ClientMessage(
            applyEdit: pb.ApplyEditRequest(baseRevision: Int64(baseRevision), op: op),
          ),
          _onEditApplied,
        );
      case SetLayout(:final layout):
        // Layout is not a revision and is not counted as pending.
        await _call(
          pb.ClientMessage(setLayout: pb.SetLayoutRequest(layout: layout)),
          (_) {},
          counted: false,
        );
      case Undo():
        await _call(pb.ClientMessage(undo: pb.UndoRequest()), _onEditApplied);
      case Redo():
        await _call(pb.ClientMessage(redo: pb.RedoRequest()), _onEditApplied);
    }
  }

  Future<void> _connect() async {
    await dispose();
    final executable = _locate();
    try {
      final client = await DaemonClient.spawn(executable);
      _client = client;
      _subs.add(client.events.listen(_onEvent));
      _subs.add(client.stderrLines.listen((l) => _dispatch(DaemonLogged(l))));
      unawaited(client.exitCode.then((code) => _dispatch(DaemonExited(code))));
      final r = await client.request(
        pb.ClientMessage(
          handshake: pb.HandshakeRequest(
            clientProtocolVersion: kClientProtocolVersion,
            clientName: kClientName,
            clientVersion: kStudioVersion,
          ),
        ),
      );
      _dispatch(DaemonConnected(executable: executable, handshake: r.handshake));
    } catch (e) {
      _dispatch(DaemonConnectionFailed('could not start `$executable`: $e'));
    }
  }

  void _onEvent(pb.Event event) {
    switch (event.whichPayload()) {
      case pb.Event_Payload.projectChanged:
        final pc = event.projectChanged;
        _dispatch(
          ProjectReceived(
            pc.project,
            outcome: pc.hasOutcome() ? pc.outcome : null,
            fromRequest: false,
          ),
        );
      case pb.Event_Payload.log:
        _dispatch(DaemonLogged('[${event.log.level}] ${event.log.message}'));
      case pb.Event_Payload.notSet:
        break;
    }
  }

  void _onEditApplied(pb.Response r) {
    final e = r.editApplied;
    _dispatch(ProjectReceived(e.project, outcome: e.hasOutcome() ? e.outcome : null));
  }

  Future<void> _project(pb.ClientMessage m) =>
      _call(m, (r) => _dispatch(ProjectReceived(r.project.project)));

  /// Send one request.  A `counted` request was registered as pending by the
  /// reducer and must be settled by exactly one terminal action: whatever
  /// [onOk] dispatches for a payload response, [RequestSucceeded] for an ack,
  /// or [RequestFailed].
  Future<void> _call(
    pb.ClientMessage m,
    void Function(pb.Response) onOk, {
    bool counted = true,
  }) async {
    final client = _client;
    if (client == null) {
      if (counted) {
        _dispatch(const RequestFailed(code: 'studio.not_connected', message: 'not connected'));
      }
      return;
    }
    try {
      final r = await client.request(m);
      onOk(r);
      if (counted && r.whichPayload() == pb.Response_Payload.ack) {
        _dispatch(const RequestSucceeded());
      }
    } on DaemonError catch (e) {
      if (counted) {
        _dispatch(RequestFailed(code: e.code, message: e.message, details: e.detailsJson));
      } else {
        _dispatch(DaemonLogged('request failed: $e'));
      }
    } catch (e) {
      if (counted) {
        _dispatch(RequestFailed(code: 'studio.transport', message: e.toString()));
      } else {
        _dispatch(DaemonLogged('transport failure: $e'));
      }
    }
  }

  Future<void> dispose() async {
    for (final s in _subs) {
      await s.cancel();
    }
    _subs.clear();
    final c = _client;
    _client = null;
    if (c != null) await c.shutdown();
  }
}
