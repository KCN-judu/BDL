/// Test doubles around the real reducer and effect executor.
///
/// [TestStore] is the store without Riverpod: every dispatched action goes
/// through `reduce`, every effect through a real [EffectExecutor] whose
/// daemon is whatever [DaemonLink] the test supplies — a [FakeDaemon] with
/// scripted answers, or the real `bdld` over a `DaemonClient`.
library;

import 'dart:async';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/effects/effect_executor.dart';
import 'package:bdl_studio/effects/recent_store.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;

class TestStore {
  TestStore({
    required SpawnDaemon spawn,
    String executable = 'fake-bdld',
    Duration draftDebounce = const Duration(milliseconds: 20),
    AppState initial = const AppState(),
  }) : state = initial {
    executor = EffectExecutor(
      dispatch,
      locate: () => executable,
      recent: _NoRecent(),
      spawn: spawn,
      draftDebounce: draftDebounce,
    );
  }

  late final EffectExecutor executor;
  AppState state;
  final List<AppAction> actions = [];
  final StreamController<AppState> _changes = StreamController.broadcast();

  void dispatch(AppAction action) {
    actions.add(action);
    final t = reduce(state, action);
    state = t.state;
    _changes.add(state);
    for (final e in t.effects) {
      unawaited(executor.run(e));
    }
  }

  /// Resolve once the state satisfies [test] (immediately if it already does).
  Future<AppState> until(bool Function(AppState) test, {Duration? timeout}) {
    if (test(state)) return Future.value(state);
    return _changes.stream
        .firstWhere(test)
        .timeout(timeout ?? const Duration(seconds: 10), onTimeout: () => state);
  }

  Future<void> dispose() => executor.dispose();
}

class _NoRecent extends RecentStore {
  @override
  Future<List<RecentProject>> load() async => const [];
  @override
  Future<void> save(List<RecentProject> recent) async {}
}

typedef FakeHandler = FutureOr<pb.Response> Function(pb.ClientMessage request);

/// A daemon that answers from a handler.  Records every request.
class FakeDaemon implements DaemonLink {
  FakeDaemon(this.handler);
  FakeHandler handler;
  final List<pb.ClientMessage> requests = [];
  final StreamController<pb.Event> _events = StreamController.broadcast();
  final StreamController<String> _stderr = StreamController.broadcast();
  final Completer<int> _exit = Completer();

  @override
  Stream<pb.Event> get events => _events.stream;
  @override
  Stream<String> get stderrLines => _stderr.stream;
  @override
  Future<int> get exitCode => _exit.future;

  void push(pb.Event event) => _events.add(event);

  @override
  Future<pb.Response> request(pb.ClientMessage message) async {
    requests.add(message);
    final r = await handler(message);
    if (r.whichPayload() == pb.Response_Payload.error) {
      throw DaemonError(r.error.code, r.error.message);
    }
    return r;
  }

  @override
  Future<void> shutdown() async {
    if (!_exit.isCompleted) _exit.complete(0);
  }
}

pb.Response okHandshake() => pb.Response(
  handshake: pb.HandshakeResponse(
    compatible: true,
    compilerVersion: 'fake',
    protocolVersion: pb.Version(major: 0, minor: 4),
  ),
);

pb.Response errorResponse(String code, String message) => pb.Response(
  error: pb.Error(code: code, message: message),
);
