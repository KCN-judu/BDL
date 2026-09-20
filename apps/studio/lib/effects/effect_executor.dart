/// The imperative shell: executes [Effect]s and feeds [ResponseAction]s back.
///
/// This is the only code that touches the daemon process.  It holds the
/// client handle (an asynchronous resource), nothing semantic.
library;

import 'dart:async';

import 'package:file_selector/file_selector.dart' as fs;
import 'package:fixnum/fixnum.dart';

import 'dart:ui' show AppExitType, PlatformDispatcher;

import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;

import '../app/actions.dart';
import '../app/effects.dart';
import '../daemon/daemon_client.dart';
import '../daemon/daemon_locator.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../protocol/versions.dart';
import '../l10n/diagnostics.dart';
import '../l10n/l10n.dart';
import 'preferences_store.dart';
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
    PreferencesStore? preferences,
    SpawnDaemon? spawn,
    this.draftDebounce = kDraftDebounce,
  }) : _locate = locate ?? locateDaemon,
       _recent = recent ?? RecentStore(),
       _preferences = preferences ?? PreferencesStore(),
       _spawn = spawn ?? DaemonClient.spawn;

  final Dispatch _dispatch;
  final String Function() _locate;
  final RecentStore _recent;
  final PreferencesStore _preferences;
  final SpawnDaemon _spawn;

  /// The language the OS dialogs are labelled in: what the preferences say,
  /// tracked here because the executor sees them load and save.
  LanguagePreference _language = LanguagePreference.system;
  AppLocalizations get _l10n => catalogFor(_language, PlatformDispatcher.instance.locale);
  final Duration draftDebounce;
  DaemonLink? _client;
  final List<StreamSubscription<Object?>> _subs = [];

  /// One pending (debounced) draft check per mapping; a newer one replaces it.
  final Map<int, Timer> _draftTimers = {};

  /// The debounced draft checks, by mapping, so a save or an unload can
  /// send them now instead of waiting: what is saved is what was typed.
  final Map<int, Future<void> Function()> _pendingDrafts = {};

  /// Send every debounced draft now and wait for the project to hold it.
  Future<void> _flushDrafts() async {
    for (final t in _draftTimers.values) {
      t.cancel();
    }
    _draftTimers.clear();
    final sends = _pendingDrafts.values.toList();
    _pendingDrafts.clear();
    for (final send in sends) {
      await send();
    }
  }

  Future<void> run(Effect effect) async {
    switch (effect) {
      case ConnectDaemon():
        await _connect();
      case LoadRecentProjects():
        _dispatch(RecentProjectsLoaded(await _recent.load()));
      case LoadPreferences():
        final loaded = await _preferences.load();
        _language = loaded.language;
        _dispatch(PreferencesLoaded(loaded));
      case SavePreferences(:final preferences):
        _language = preferences.language;
        try {
          await _preferences.save(preferences);
        } catch (e) {
          _dispatch(DaemonLogged('could not save preferences: $e'));
        }
      case SaveRecentProjects(:final recent):
        try {
          await _recent.save(recent);
        } catch (e) {
          _dispatch(DaemonLogged('could not save recent projects: $e'));
        }
      case PickProjectToOpen():
        final dir = await _picker(
          () => fs.getDirectoryPath(confirmButtonText: _l10n.dialogOpenProject),
        );
        if (dir != null) _dispatch(OpenProjectRequested(dir));
      case PickNewProjectLocation(:final template):
        // A save dialog names the new project directory — the native idiom
        // for creating a document on both macOS and Windows.
        final loc = await _picker(
          () => fs.getSaveLocation(
            suggestedName: template ?? _l10n.dialogUntitledProject,
            confirmButtonText: _l10n.dialogCreateProject,
          ),
        );
        if (loc != null) {
          _dispatch(
            NewProjectRequested(
              rootPath: loc.path,
              name: p.basenameWithoutExtension(loc.path),
              template: template,
            ),
          );
        }
      case OpenProject(:final rootPath):
        await _project(pb.ClientMessage(openProject: pb.OpenProjectRequest(rootPath: rootPath)));
      case InitProject(:final rootPath, :final name, :final template):
        await _project(
          pb.ClientMessage(
            initProject: pb.InitProjectRequest(rootPath: rootPath, name: name, template: template),
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
        // What is saved is what was typed: a debounced draft goes first.
        await _flushDrafts();
        await _project(pb.ClientMessage(saveProject: pb.SaveProjectRequest(force: force)));
      case GetProject():
        await _flushDrafts();
        await _project(pb.ClientMessage(getProject: pb.GetProjectRequest()));
      case QuitApplication():
        await _quit();
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
      case ListLibraryItems():
        await _call(
          pb.ClientMessage(listLibraryItems: pb.ListLibraryItemsRequest()),
          (r) => _dispatch(LibraryItemsReceived(r.libraryItems)),
          counted: false,
        );
      case ListValueCategories():
        await _call(
          pb.ClientMessage(listValueCategories: pb.ListValueCategoriesRequest()),
          (r) => _dispatch(ValueCategoriesReceived(r.valueCategories.categories)),
          counted: false,
        );
      case InstantiateLibraryItem(:final baseRevision, :final itemId, :final component):
        await _call(
          pb.ClientMessage(
            instantiateLibraryItem: pb.InstantiateLibraryItemRequest(
              baseRevision: Int64(baseRevision),
              itemId: itemId,
              component: component == null ? null : Int64(component),
            ),
          ),
          _onEditApplied,
        );
      case ListSourceCandidates(
        :final revision,
        :final itemId,
        :final generation,
        :final component,
      ):
        await _call(
          pb.ClientMessage(
            listSourceCandidates: pb.ListSourceCandidatesRequest(
              revision: Int64(revision),
              itemId: itemId,
              component: component == null ? null : Int64(component),
            ),
          ),
          (r) => _dispatch(
            SourceCandidatesReceived(generation: generation, response: r.sourceCandidates),
          ),
          counted: false,
        );
      case CreateSource(
        :final baseRevision,
        :final sourceName,
        :final description,
        :final existingConcept,
        :final newConcept,
        :final component,
      ):
        await _call(
          pb.ClientMessage(
            createSource: pb.CreateSourceRequest(
              baseRevision: Int64(baseRevision),
              sourceName: sourceName,
              sourceDescription: description,
              existingConcept: existingConcept == null ? null : Int64(existingConcept),
              newConcept: newConcept,
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
        _pendingDrafts[mappingId] = () =>
            _analyzeDraft(revision, mappingId, generation, source, component);
        _draftTimers[mappingId] = Timer(draftDebounce, () {
          _draftTimers.remove(mappingId);
          final send = _pendingDrafts.remove(mappingId);
          if (send != null) send();
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
      case FetchSemanticTokens(
        :final key,
        :final revision,
        :final generation,
        :final text,
        :final path,
        :final mappingId,
        :final component,
      ):
        final client = _client;
        if (client == null) {
          _dispatch(SemanticTokensFailed(key: key, generation: generation));
          return;
        }
        try {
          final r = await client.request(
            pb.ClientMessage(
              semanticTokens: pb.SemanticTokensRequest(
                revision: Int64(revision),
                generation: Int64(generation),
                text: text,
                path: path,
                formula: mappingId == null
                    ? null
                    : pb.FormulaDocument(
                        mappingId: Int64(mappingId),
                        component: component == null ? null : Int64(component),
                      ),
              ),
            ),
          );
          _dispatch(SemanticTokensReceived(key: key, result: r.semanticTokens));
        } catch (e) {
          _dispatch(SemanticTokensFailed(key: key, generation: generation));
        }
      case CompleteSource(
        :final revision,
        :final generation,
        :final path,
        :final text,
        :final offset,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            sourceCompletion: pb.SourceCompletionRequest(
              revision: Int64(revision),
              generation: Int64(generation),
              path: path,
              text: text,
              offset: offset,
            ),
          ),
          (r) => _dispatch(
            SourceCompletionReceived(generation: generation, result: r.sourceCompletion),
          ),
        );
      case HoverSource(:final revision, :final generation, :final path, :final text, :final offset):
        await _tooling(
          generation,
          pb.ClientMessage(
            sourceHover: pb.SourceHoverRequest(
              revision: Int64(revision),
              generation: Int64(generation),
              path: path,
              text: text,
              offset: offset,
            ),
          ),
          (r) => _dispatch(HoverReceived(generation: generation, result: r.draftHover)),
        );
      case DefineSource(
        :final revision,
        :final generation,
        :final path,
        :final text,
        :final offset,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            sourceDefinition: pb.SourceDefinitionRequest(
              revision: Int64(revision),
              generation: Int64(generation),
              path: path,
              text: text,
              offset: offset,
            ),
          ),
          (r) => _dispatch(
            SourceDefinitionReceived(generation: generation, result: r.sourceLocations),
          ),
        );
      case ReferencesSource(
        :final revision,
        :final generation,
        :final path,
        :final text,
        :final offset,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            sourceReferences: pb.SourceReferencesRequest(
              revision: Int64(revision),
              generation: Int64(generation),
              path: path,
              text: text,
              offset: offset,
              includeDeclaration: true,
            ),
          ),
          (r) => _dispatch(
            SourceReferencesReceived(generation: generation, result: r.sourceLocations),
          ),
        );
      case FormatSource(:final revision, :final generation, :final path, :final text):
        final client = _client;
        if (client == null) {
          _dispatch(
            FormatSourceReceived(
              generation: generation,
              path: path,
              result: pb.FormatSourceResponse(formatted: false, text: text),
            ),
          );
          return;
        }
        try {
          final r = await client.request(
            pb.ClientMessage(
              formatSource: pb.FormatSourceRequest(
                revision: Int64(revision),
                generation: Int64(generation),
                path: path,
                text: text,
              ),
            ),
          );
          _dispatch(
            FormatSourceReceived(generation: generation, path: path, result: r.formatSource),
          );
        } catch (e) {
          _dispatch(
            FormatSourceReceived(
              generation: generation,
              path: path,
              result: pb.FormatSourceResponse(formatted: false, text: text),
            ),
          );
        }
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
      case GetFormulaPreview(
        :final revision,
        :final mappingId,
        :final generation,
        :final component,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            getFormulaProjection: pb.GetFormulaProjectionRequest(
              revision: Int64(revision),
              mappingId: Int64(mappingId),
              component: component == null ? null : Int64(component),
            ),
          ),
          (r) => _dispatch(
            FormulaPreviewReceived(generation: generation, response: r.formulaProjection),
          ),
        );
      case GetFormulaProjection(
        :final revision,
        :final mappingId,
        :final generation,
        :final component,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            getFormulaProjection: pb.GetFormulaProjectionRequest(
              revision: Int64(revision),
              mappingId: Int64(mappingId),
              component: component == null ? null : Int64(component),
            ),
          ),
          (r) => _dispatch(
            FormulaProjectionReceived(generation: generation, result: r.formulaProjection),
          ),
        );
      case GetFormulaSlot(
        :final revision,
        :final mappingId,
        :final source,
        :final nodeId,
        :final generation,
        :final component,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            getFormulaSlot: pb.GetFormulaSlotRequest(
              revision: Int64(revision),
              mappingId: Int64(mappingId),
              source: source,
              nodeId: nodeId,
              component: component == null ? null : Int64(component),
            ),
          ),
          (r) => _dispatch(FormulaSlotReceived(generation: generation, result: r.formulaSlot)),
        );
      case ComposeFormula(
        :final revision,
        :final mappingId,
        :final source,
        :final action,
        :final generation,
        :final component,
      ):
        await _tooling(
          generation,
          pb.ClientMessage(
            composeFormula: pb.ComposeFormulaRequest(
              revision: Int64(revision),
              mappingId: Int64(mappingId),
              source: source,
              action: action,
              component: component == null ? null : Int64(component),
            ),
          ),
          (r) => _dispatch(ComposeReceived(generation: generation, result: r.composeFormula)),
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
      case BuildFirmware(:final targetId, :final revision):
        await _firmware(
          pb.ClientMessage(
            buildFirmware: pb.BuildFirmwareRequest(targetId: targetId, revision: Int64(revision)),
          ),
        );
      case CancelBuild():
        await _firmware(pb.ClientMessage(cancelBuild: pb.CancelBuildRequest()));
      case GetBuildStatus(:final targetId, :final generation):
        await _call(
          pb.ClientMessage(getBuildStatus: pb.GetBuildStatusRequest(targetId: targetId)),
          (r) =>
              _dispatch(BuildStatusReceived(generation: generation, status: r.buildStatus.status)),
          counted: false,
        );
      case ListFlashDevices(:final targetId):
        await _call(
          pb.ClientMessage(listFlashDevices: pb.ListFlashDevicesRequest(targetId: targetId)),
          (r) => _dispatch(
            FlashDevicesReceived(
              targetId: targetId,
              devices: r.flashDevices.devices,
              methods: r.flashDevices.methods,
            ),
          ),
          counted: false,
        );
      case FlashFirmware(:final targetId, :final deviceId):
        await _firmware(
          pb.ClientMessage(
            flashFirmware: pb.FlashFirmwareRequest(targetId: targetId, deviceId: deviceId),
          ),
        );
      case ListTemplates():
        await _call(
          pb.ClientMessage(listTemplates: pb.ListTemplatesRequest()),
          (r) => _dispatch(TemplatesReceived(r.templates.templates)),
          counted: false,
        );
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
      case pb.Event_Payload.buildProgress:
        _dispatch(BuildProgressReceived(event.buildProgress));
      case pb.Event_Payload.flashProgress:
        _dispatch(FlashProgressReceived(event.flashProgress));
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

  /// A build or flash request: acknowledged at once, then events; a
  /// refusal is the page's to show, never a banner.
  Future<void> _firmware(pb.ClientMessage m) async {
    final client = _client;
    if (client == null) {
      _dispatch(
        const FirmwareRequestFailed(
          code: 'studio.not_connected',
          message: 'The compiler service is not connected.',
        ),
      );
      return;
    }
    try {
      await client.request(m);
    } on DaemonError catch (e) {
      _dispatch(FirmwareRequestFailed(code: e.code, message: e.message));
    } catch (e) {
      _dispatch(FirmwareRequestFailed(code: 'studio.transport', message: '$e'));
    }
  }

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

  /// How the application is left once the guard has run: the framework's
  /// own exit, which the desktop embedders honour.  Replaceable for tests.
  Future<void> Function() quit = () async {
    await ServicesBinding.instance.exitApplication(AppExitType.required);
  };

  Future<void> _quit() => quit();

  Future<void> dispose() async {
    for (final t in _draftTimers.values) {
      t.cancel();
    }
    _draftTimers.clear();
    _pendingDrafts.clear();
    for (final s in _subs) {
      await s.cancel();
    }
    _subs.clear();
    final c = _client;
    _client = null;
    if (c != null) await c.shutdown();
  }
}
