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

/// How long the executor waits after the last keystroke before asking the
/// compiler about a draft.  Short enough to feel immediate, long enough
/// that a burst of typing costs one request.  Commits never wait on it.
const Duration kDraftDebounce = Duration(milliseconds: 150);

typedef SpawnDaemon = Future<DaemonLink> Function(String executable);

class EffectExecutor {
  EffectExecutor(
    this._dispatch, {
    String Function()? locate,
    RecentStore? recent,
    SpawnDaemon? spawn,
    this.draftDebounce = kDraftDebounce,
  }) : _locate = locate ?? locateDaemon,
       _recent = recent ?? RecentStore(),
       _spawn = spawn ?? DaemonClient.spawn;

  final Dispatch _dispatch;
  final String Function() _locate;
  final RecentStore _recent;
  final SpawnDaemon _spawn;
  final Duration draftDebounce;
  DaemonLink? _client;
  final List<StreamSubscription<Object?>> _subs = [];

  /// One pending (debounced) draft check per mapping; a newer one replaces it.
  final Map<int, Timer> _draftTimers = {};

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
      case GetSources():
        await _call(
          pb.ClientMessage(getSources: pb.GetSourcesRequest()),
          (r) => _dispatch(SourcesReceived(r.sources.sources)),
          counted: false,
        );
      case ApplySourceEdit(:final baseRevision, :final path, :final text):
        await _call(
          pb.ClientMessage(
            applySourceEdit: pb.ApplySourceEditRequest(
              baseRevision: Int64(baseRevision),
              path: path,
              text: text,
            ),
          ),
          (r) => _dispatch(SourceEditApplied(r.sourceEditApplied)),
        );
      case SaveProject(:final force):
        await _project(pb.ClientMessage(saveProject: pb.SaveProjectRequest(force: force)));
      case ReloadProject():
        await _project(pb.ClientMessage(reloadProject: pb.ReloadProjectRequest()));
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
      case ApplySystemEdit(:final baseRevision, :final op):
        await _call(
          pb.ClientMessage(
            applySystemEdit: pb.ApplySystemEditRequest(baseRevision: Int64(baseRevision), op: op),
          ),
          (r) {
            final e = r.systemEditApplied;
            _dispatch(
              SystemEditApplied(
                system: e.system,
                project: e.project,
                outcome: e.hasOutcome() ? e.outcome : null,
              ),
            );
          },
        );
      case ApplyGroupEdit(:final op, :final baseGeneration):
        await _call(
          pb.ClientMessage(
            applyGroupEdit: pb.ApplyGroupEditRequest(
              op: op,
              baseGeneration: baseGeneration == null ? null : Int64(baseGeneration),
            ),
          ),
          (r) => _dispatch(SystemReceived(r.system.system)),
        );
      case GetSystem():
        // Counted: a project is not "here" until its system is (the
        // reducer registers it when it asks).
        await _call(
          pb.ClientMessage(getSystem: pb.GetSystemRequest()),
          (r) => _dispatch(SystemReceived(r.system.system)),
        );
      case RunSystemAnalysis():
        await _call(
          pb.ClientMessage(runSystemAnalysis: pb.RunSystemAnalysisRequest()),
          (r) => _dispatch(SystemAnalysisReceived(r.systemAnalysis.analysis)),
          counted: false,
        );
      case PreviewExtraction(:final group, :final choices, :final generation):
        final client = _client;
        if (client == null) {
          _dispatch(
            ExtractionPreviewFailed(
              generation: generation,
              code: 'studio.not_connected',
              message: 'The compiler service is not connected.',
            ),
          );
          return;
        }
        try {
          final r = await client.request(
            pb.ClientMessage(
              previewComponentExtraction: pb.PreviewComponentExtractionRequest(
                group: Int64(group),
                choices: choices,
              ),
            ),
          );
          _dispatch(
            ExtractionPreviewReceived(generation: generation, preview: r.extractionPreview.preview),
          );
        } on DaemonError catch (e) {
          _dispatch(
            ExtractionPreviewFailed(generation: generation, code: e.code, message: e.message),
          );
        } catch (e) {
          _dispatch(
            ExtractionPreviewFailed(
              generation: generation,
              code: 'studio.transport',
              message: '$e',
            ),
          );
        }
      case ListConceptTemplates():
        await _call(
          pb.ClientMessage(listConceptTemplates: pb.ListConceptTemplatesRequest()),
          (r) => _dispatch(ConceptTemplatesReceived(r.conceptTemplates)),
          counted: false,
        );
      case InstantiateConceptTemplate(:final baseRevision, :final templateId, :final component):
        await _call(
          pb.ClientMessage(
            instantiateConceptTemplate: pb.InstantiateConceptTemplateRequest(
              baseRevision: Int64(baseRevision),
              templateId: templateId,
              component: component == null ? null : Int64(component),
            ),
          ),
          _onEditApplied,
        );
      case RunAnalysis():
        await _call(
          pb.ClientMessage(runAnalysis: pb.RunAnalysisRequest()),
          (r) => _dispatch(AnalysisReceived(r.analysis.analysis)),
          counted: false,
        );
      case AnalyzeDraft(
        :final revision,
        :final mappingId,
        :final generation,
        :final source,
        :final component,
      ):
        _draftTimers.remove(mappingId)?.cancel();
        _draftTimers[mappingId] = Timer(draftDebounce, () {
          _draftTimers.remove(mappingId);
          _analyzeDraft(revision, mappingId, generation, source, component);
        });
      case CompleteDraft(
        :final revision,
        :final mappingId,
        :final source,
        :final offset,
        :final generation,
        :final component,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            completeDefinitionDraft: pb.CompleteDefinitionDraftRequest(
              revision: Int64(revision),
              mappingId: Int64(mappingId),
              source: source,
              offset: offset,
              component: component == null ? null : Int64(component),
            ),
          ),
          (r) => _dispatch(CompletionReceived(generation: generation, result: r.draftCompletion)),
        );
      case HoverDraft(
        :final revision,
        :final mappingId,
        :final source,
        :final offset,
        :final generation,
        :final component,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            hoverDefinitionDraft: pb.HoverDefinitionDraftRequest(
              revision: Int64(revision),
              mappingId: Int64(mappingId),
              source: source,
              offset: offset,
              component: component == null ? null : Int64(component),
            ),
          ),
          (r) => _dispatch(HoverReceived(generation: generation, result: r.draftHover)),
        );
      case HoverEntity(:final revision, :final entity, :final generation):
        await _tooling(
          generation,
          pb.ClientMessage(
            hoverEntity: pb.HoverEntityRequest(revision: Int64(revision), entity: entity),
          ),
          (r) => _dispatch(HoverReceived(generation: generation, result: r.draftHover)),
        );
      case ListSemanticActions(:final revision, :final entity, :final generation):
        await _tooling(
          generation,
          pb.ClientMessage(
            listSemanticActions: pb.ListSemanticActionsRequest(
              revision: Int64(revision),
              entity: entity,
            ),
          ),
          (r) =>
              _dispatch(SemanticActionsReceived(generation: generation, result: r.semanticActions)),
        );
      case RunSimulation(:final inputs, :final schedule, :final ticks, :final generation):
        final client = _client;
        if (client == null) {
          _dispatch(
            SimulationFailed(
              generation: generation,
              code: 'studio.not_connected',
              message: 'The compiler service is not connected.',
            ),
          );
          return;
        }
        try {
          // A run's inputs are fixed at its start: re-create it with the
          // whole trace, then step to the wanted tick.  Sequential, never
          // merged with another step.
          final started = await client.request(
            pb.ClientMessage(
              startSimulation: pb.StartSimulationRequest(inputs: inputs, schedule: schedule),
            ),
          );
          if (ticks == 0) {
            _dispatch(SimulationReceived(generation: generation, response: started.simulation));
            return;
          }
          final stepped = await client.request(
            pb.ClientMessage(stepSimulation: pb.StepSimulationRequest(ticks: Int64(ticks))),
          );
          _dispatch(SimulationReceived(generation: generation, response: stepped.simulation));
        } on DaemonError catch (e) {
          _dispatch(SimulationFailed(generation: generation, code: e.code, message: e.message));
        } catch (e) {
          _dispatch(
            SimulationFailed(generation: generation, code: 'studio.transport', message: '$e'),
          );
        }
      case ListTargets():
        await _call(
          pb.ClientMessage(listTargets: pb.ListTargetsRequest()),
          (r) => _dispatch(TargetsReceived(r.targets.targets)),
          counted: false,
        );
      case AnalyzeDeployment(:final targetId, :final generation):
        final client = _client;
        if (client == null) {
          _dispatch(
            DeploymentFailed(
              generation: generation,
              code: 'studio.not_connected',
              message: 'The compiler service is not connected.',
            ),
          );
          return;
        }
        try {
          final r = await client.request(
            pb.ClientMessage(analyzeDeployment: pb.AnalyzeDeploymentRequest(targetId: targetId)),
          );
          _dispatch(DeploymentReceived(generation: generation, analysis: r.deployment.deployment));
        } on DaemonError catch (e) {
          _dispatch(DeploymentFailed(generation: generation, code: e.code, message: e.message));
        } catch (e) {
          _dispatch(
            DeploymentFailed(generation: generation, code: 'studio.transport', message: '$e'),
          );
        }
      case DiscardDraft(:final mappingId, :final component):
        // A check still debounced for this draft would resurrect the overlay.
        _draftTimers.remove(mappingId)?.cancel();
        await _call(
          pb.ClientMessage(
            discardDefinitionDraft: pb.DiscardDefinitionDraftRequest(
              mappingId: Int64(mappingId),
              component: component == null ? null : Int64(component),
            ),
          ),
          (_) {},
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

  /// Completion and hover: uncounted, answered by generation; a failure
  /// (stale revision, daemon gone) only means no pop-up or card.
  Future<void> _tooling(int generation, pb.ClientMessage m, void Function(pb.Response) onOk) async {
    final client = _client;
    if (client == null) {
      _dispatch(ToolingFailed(generation: generation, code: 'studio.not_connected', message: ''));
      return;
    }
    try {
      onOk(await client.request(m));
    } on DaemonError catch (e) {
      _dispatch(ToolingFailed(generation: generation, code: e.code, message: e.message));
    } catch (e) {
      _dispatch(ToolingFailed(generation: generation, code: 'studio.transport', message: '$e'));
    }
  }

  /// Read-only and uncounted: the app is not "busy" while a draft is
  /// checked.  Every outcome is tagged with the generation it answers so the
  /// reducer can drop what a newer draft has superseded.
  Future<void> _analyzeDraft(
    int revision,
    int mappingId,
    int generation,
    String source,
    int? component,
  ) async {
    final client = _client;
    if (client == null) {
      _dispatch(
        DraftAnalysisFailed(
          mappingId: mappingId,
          generation: generation,
          code: 'studio.not_connected',
          message: 'The compiler service is not connected.',
        ),
      );
      return;
    }
    try {
      final r = await client.request(
        pb.ClientMessage(
          analyzeDefinitionDraft: pb.AnalyzeDefinitionDraftRequest(
            revision: Int64(revision),
            mappingId: Int64(mappingId),
            generation: Int64(generation),
            source: source,
            component: component == null ? null : Int64(component),
          ),
        ),
      );
      _dispatch(DraftAnalysisReceived(r.definitionDraft));
    } on DaemonError catch (e) {
      _dispatch(
        DraftAnalysisFailed(
          mappingId: mappingId,
          generation: generation,
          code: e.code,
          message: e.message,
        ),
      );
    } catch (e) {
      _dispatch(
        DraftAnalysisFailed(
          mappingId: mappingId,
          generation: generation,
          code: 'studio.transport',
          message: 'The compiler service could not be reached: $e',
        ),
      );
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
      final client = await _spawn(executable);
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

  /// An edit's answer: a flat `EditApplied`, or — on a system project —
  /// a `SystemEditApplied` carrying the system alongside.
  void _onEditApplied(pb.Response r) {
    if (r.whichPayload() == pb.Response_Payload.systemEditApplied) {
      final e = r.systemEditApplied;
      _dispatch(
        SystemEditApplied(
          system: e.system,
          project: e.project,
          outcome: e.hasOutcome() ? e.outcome : null,
        ),
      );
      return;
    }
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
    for (final t in _draftTimers.values) {
      t.cancel();
    }
    _draftTimers.clear();
    for (final s in _subs) {
      await s.cancel();
    }
    _subs.clear();
    final c = _client;
    _client = null;
    if (c != null) await c.shutdown();
  }
}
