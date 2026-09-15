/// The pure reducer: `State × Action → State × Effects`.
///
/// No I/O, no Flutter, no daemon.  Everything about *how the application
/// behaves* is readable here and testable without a widget tree.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'actions.dart';
import 'effects.dart';
import 'state.dart';

@immutable
class Transition {
  const Transition(this.state, [this.effects = const []]);
  final AppState state;
  final List<Effect> effects;
}

const int _maxLogLines = 200;

Transition reduce(AppState s, AppAction action) {
  return switch (action) {
    AppStarted() || ConnectRequested() => _connect(s),

    OpenProjectRequested(:final rootPath) => _whenConnected(
      s,
      () => Transition(_pending(s), [OpenProject(rootPath)]),
    ),
    NewProjectRequested(:final rootPath, :final name) => _whenConnected(
      s,
      () => Transition(_pending(s), [InitProject(rootPath: rootPath, name: name)]),
    ),
    SaveRequested() => _whenProject(s, () => Transition(_pending(s), const [SaveProject()])),
    CloseProjectRequested() => _whenProject(
      s,
      () => Transition(_pending(s), const [CloseProject()]),
    ),
    UndoRequested() => _whenProject(
      s,
      () => s.project!.canUndo ? Transition(_pending(s), const [Undo()]) : Transition(s),
    ),
    RedoRequested() => _whenProject(
      s,
      () => s.project!.canRedo ? Transition(_pending(s), const [Redo()]) : Transition(s),
    ),

    CreateConceptRequested(:final name, :final description) => _edit(
      s,
      pb.EditOp(
        createConcept: pb.CreateConcept(name: name, description: description),
      ),
    ),
    CreateMappingRequested(:final name, :final inputs, :final output) => _edit(
      s,
      pb.EditOp(
        createMapping: pb.CreateMapping(
          name: name,
          signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(output)),
        ),
      ),
    ),
    AttachFormulaRequested(:final mappingId, :final source) => _edit(
      s,
      pb.EditOp(
        attachDefinition: pb.AttachDefinition(
          id: Int64(mappingId),
          definition: pb.Definition(formula: source),
        ),
      ),
    ),

    SelectionChanged(:final selection) => Transition(
      s.copyWith(editor: s.editor.copyWith(selection: selection)),
    ),
    ErrorDismissed() => Transition(s.copyWith(editor: s.editor.copyWith(clearError: true))),

    DaemonConnected(:final executable, :final handshake) => Transition(
      s.copyWith(
        connection: handshake.compatible
            ? Connected(executable: executable, handshake: handshake)
            : ConnectionFailed(
                'protocol mismatch: daemon speaks '
                '${handshake.protocolVersion.major}.x, Studio does not',
              ),
      ),
    ),
    DaemonConnectionFailed(:final reason) => Transition(
      s.copyWith(connection: ConnectionFailed(reason)),
    ),
    DaemonExited(:final exitCode) => Transition(
      s.copyWith(
        connection: ConnectionFailed('compiler service exited (code $exitCode)'),
        clearProject: true,
        editor: s.editor.copyWith(pendingRequests: 0, selection: const NoSelection()),
      ),
    ),
    DaemonLogged(:final line) => Transition(
      s.copyWith(render: s.render.copyWith(daemonLog: _appendLog(s.render.daemonLog, line))),
    ),

    ProjectReceived(:final project, :final fromRequest) => _projectReceived(
      s,
      project,
      fromRequest,
    ),
    ProjectClosed() => Transition(
      s.copyWith(
        clearProject: true,
        editor: s.editor.copyWith(pendingRequests: _dec(s), selection: const NoSelection()),
      ),
    ),
    RequestSucceeded() => Transition(
      s.copyWith(editor: s.editor.copyWith(pendingRequests: _dec(s))),
    ),
    RequestFailed(:final code, :final message, :final details) => Transition(
      s.copyWith(
        editor: s.editor.copyWith(
          pendingRequests: _dec(s),
          lastError: UserFacingError(code: code, message: message, details: details),
        ),
      ),
    ),
  };
}

Transition _connect(AppState s) {
  if (s.connection is Connecting) return Transition(s);
  return Transition(s.copyWith(connection: const Connecting('')), const [ConnectDaemon()]);
}

Transition _whenConnected(AppState s, Transition Function() then) =>
    s.connection is Connected ? then() : Transition(s);

Transition _whenProject(AppState s, Transition Function() then) =>
    s.connection is Connected && s.project != null ? then() : Transition(s);

/// Every semantic edit is sent against the revision Studio currently holds;
/// the daemon refuses it if the project has moved on.
Transition _edit(AppState s, pb.EditOp op) =>
    _whenProject(s, () => Transition(_pending(s), [ApplyEdit(baseRevision: s.revision, op: op)]));

AppState _pending(AppState s) =>
    s.copyWith(editor: s.editor.copyWith(pendingRequests: s.editor.pendingRequests + 1));

int _dec(AppState s) => s.editor.pendingRequests > 0 ? s.editor.pendingRequests - 1 : 0;

/// Accept a projection only if it is at least as new as what we hold for the
/// same project.  Responses for an older revision are discarded — this is
/// the stale-result rule of the protocol.
Transition _projectReceived(AppState s, pb.ProjectProjection incoming, bool fromRequest) {
  final current = s.project;
  final sameProject = current != null && current.rootPath == incoming.rootPath;
  final pending = fromRequest ? _dec(s) : s.editor.pendingRequests;
  if (sameProject && incoming.revision < current.revision) {
    return Transition(s.copyWith(editor: s.editor.copyWith(pendingRequests: pending)));
  }
  final selection = _selectionStillValid(s.editor.selection, incoming)
      ? s.editor.selection
      : const NoSelection();
  return Transition(
    s.copyWith(
      project: incoming,
      editor: s.editor.copyWith(pendingRequests: pending, selection: selection),
    ),
    // A freshly opened project needs a subscription for pushed changes.
    sameProject ? const [] : const [SubscribeProject()],
  );
}

bool _selectionStillValid(Selection sel, pb.ProjectProjection p) => switch (sel) {
  NoSelection() => true,
  ConceptSelected(:final id) => p.concepts.any((c) => c.id.toInt() == id),
  MappingSelected(:final id) => p.mappings.any((m) => m.id.toInt() == id),
};

List<String> _appendLog(List<String> log, String line) {
  final next = [...log, line];
  return next.length > _maxLogLines ? next.sublist(next.length - _maxLogLines) : next;
}
