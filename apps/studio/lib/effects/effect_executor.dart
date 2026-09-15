/// The imperative shell: executes [Effect]s and feeds [ResponseAction]s back.
///
/// This is the only code that touches the daemon process.  It holds the
/// client handle (an asynchronous resource), nothing semantic.
library;

import 'dart:async';

import 'package:file_selector/file_selector.dart' as fs;
import 'package:fixnum/fixnum.dart';
import 'package:path/path.dart' as p;

import '../app/actions.dart';
import '../app/effects.dart';
import '../daemon/daemon_client.dart';
import '../daemon/daemon_locator.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../protocol/versions.dart';
import 'recent_store.dart';

typedef Dispatch = void Function(AppAction action);

class EffectExecutor {
  EffectExecutor(this._dispatch, {String Function()? locate, RecentStore? recent})
    : _locate = locate ?? locateDaemon,
      _recent = recent ?? RecentStore();

  final Dispatch _dispatch;
  final String Function() _locate;
  final RecentStore _recent;
  DaemonClient? _client;
  final List<StreamSubscription<Object?>> _subs = [];

  Future<void> run(Effect effect) async {
    switch (effect) {
      case ConnectDaemon():
        await _connect();
      case LoadRecentProjects():
        _dispatch(RecentProjectsLoaded(await _recent.load()));
      case SaveRecentProjects(:final recent):
        try {
          await _recent.save(recent);
        } catch (e) {
          _dispatch(DaemonLogged('could not save recent projects: $e'));
        }
      case PickProjectToOpen():
        final dir = await _picker(() => fs.getDirectoryPath(confirmButtonText: 'Open Project'));
        if (dir != null) _dispatch(OpenProjectRequested(dir));
      case PickNewProjectLocation():
        // A save dialog names the new project directory — the native idiom
        // for creating a document on both macOS and Windows.
        final loc = await _picker(
          () => fs.getSaveLocation(
            suggestedName: 'Untitled Project',
            confirmButtonText: 'Create Project',
          ),
        );
        if (loc != null) {
          _dispatch(
            NewProjectRequested(rootPath: loc.path, name: p.basenameWithoutExtension(loc.path)),
          );
        }
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
      case RunAnalysis():
        await _call(
          pb.ClientMessage(runAnalysis: pb.RunAnalysisRequest()),
          (r) => _dispatch(AnalysisReceived(r.analysis.analysis)),
          counted: false,
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

  /// Run an OS picker.  A `null` that comes back faster than a person could
  /// dismiss a dialog means the dialog was never shown (the view-bridge
  /// refuses children of sandboxed hosts); report that instead of silence.
  Future<T?> _picker<T>(Future<T?> Function() show) async {
    final started = DateTime.now();
    T? result;
    try {
      result = await show();
    } catch (e) {
      _dispatch(const PickerUnavailable());
      _dispatch(DaemonLogged('file dialog failed: $e'));
      return null;
    }
    if (result == null && DateTime.now().difference(started) < const Duration(milliseconds: 400)) {
      _dispatch(const PickerUnavailable());
    }
    return result;
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
      case pb.Event_Payload.analysisReady:
        _dispatch(AnalysisReceived(event.analysisReady.analysis));
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
