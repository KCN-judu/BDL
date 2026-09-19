/// Side effects the reducer asks for.  Pure data; executed by
/// `effects/effect_executor.dart`.
library;

import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'state.dart';

@immutable
sealed class Effect {
  const Effect();
}

class ConnectDaemon extends Effect {
  const ConnectDaemon();
}

class LoadRecentProjects extends Effect {
  const LoadRecentProjects();
}

class SaveRecentProjects extends Effect {
  const SaveRecentProjects(this.recent);
  final List<RecentProject> recent;
}

class LoadPreferences extends Effect {
  const LoadPreferences();
}

class SavePreferences extends Effect {
  const SavePreferences(this.preferences);
  final AppPreferences preferences;
}

/// Show the OS folder picker; the executor dispatches `OpenProjectRequested`
/// with the choice, or nothing when cancelled.
class PickProjectToOpen extends Effect {
  const PickProjectToOpen();
}

/// Show the OS save dialog to choose where a new project directory goes;
/// the executor dispatches `NewProjectRequested(rootPath, name)`.
class PickNewProjectLocation extends Effect {
  const PickNewProjectLocation();
}

class OpenProject extends Effect {
  const OpenProject(this.rootPath);
  final String rootPath;
}

class InitProject extends Effect {
  const InitProject({required this.rootPath, required this.name});
  final String rootPath;
  final String name;
}

/// The Code view's sources (uncounted: a view fetch, answered by
/// `SourcesReceived`).
class GetSources extends Effect {
  const GetSources();
}

/// A text edit against [baseRevision]; counted, answered by
/// `SourceEditApplied` or `RequestFailed`.
class ApplySourceEdit extends Effect {
  const ApplySourceEdit({required this.baseRevision, required this.path, required this.text});
  final int baseRevision;
  final String path;
  final String text;
}

class SaveProject extends Effect {
  const SaveProject({this.force = false});
  final bool force;
}

/// `ReloadProject`: a text project re-read from disk.
class ReloadProject extends Effect {
  const ReloadProject();
}

class CloseProject extends Effect {
  const CloseProject();
}

/// Ask the project whether it differs from what is saved, after every
/// unsent edit (typing that has not been sent yet) has reached it.
/// Answered by `ProjectReceived`.
class GetProject extends Effect {
  const GetProject();
}

/// Leave the application: the guard has run, nothing unsaved is open.
class QuitApplication extends Effect {
  const QuitApplication();
}

class SubscribeProject extends Effect {
  const SubscribeProject();
}

class ApplyEdit extends Effect {
  const ApplyEdit({required this.baseRevision, required this.op});
  final int baseRevision;
  final pb.EditOp op;
}

/// One system edit against the revision Studio holds; answered like an
/// edit, with the system alongside (`SystemEditApplied`).
class ApplySystemEdit extends Effect {
  const ApplySystemEdit({required this.baseRevision, required this.op});
  final int baseRevision;
  final pb.SystemEditOp op;
}

/// A group edit: authoring metadata, never a revision.  Counted (it is
/// the designer's act), answered with the system at a new authoring
/// generation.
class ApplyGroupEdit extends Effect {
  const ApplyGroupEdit(this.op, {this.baseGeneration});
  final pb.GroupEditOp op;

  /// The authoring generation Studio holds; the daemon refuses the edit
  /// if the table moved (`group_edit.stale_generation`).
  final int? baseGeneration;
}

/// The authored system of the open system project — uncounted.
class GetSystem extends Effect {
  const GetSystem();
}

/// The system-level analysis at the current revision — uncounted.
class RunSystemAnalysis extends Effect {
  const RunSystemAnalysis();
}

/// What packaging a group would do — uncounted, tagged.
class PreviewExtraction extends Effect {
  const PreviewExtraction({required this.group, required this.choices, required this.generation});
  final int group;
  final pb.ExtractionChoices choices;
  final int generation;
}

/// Ask the daemon for its libraries, as items.  Not counted as pending.
class ListLibraryItems extends Effect {
  const ListLibraryItems();
}

/// The one instantiation request: the daemon plans the item's fragment
/// against the target design and applies every step in one transaction.
/// Answered like an edit.
class InstantiateLibraryItem extends Effect {
  const InstantiateLibraryItem({required this.baseRevision, required this.itemId, this.component});
  final int baseRevision;
  final String itemId;

  /// On a system project: the component body to insert into (`null`: the
  /// system's own design).
  final int? component;
}

class RunAnalysis extends Effect {
  const RunAnalysis();
}

/// Ask the compiler about an uncommitted definition.  Read-only for the
/// project; not counted as a pending request.  The executor debounces
/// these per mapping (`EffectExecutor.draftDebounce`), so only the last of
/// a burst is sent.
class AnalyzeDraft extends Effect {
  const AnalyzeDraft({
    required this.revision,
    required this.mappingId,
    required this.generation,
    required this.source,
    this.component,
  });
  final int revision;
  final int mappingId;
  final int generation;
  final String source;

  /// The component whose body the mapping belongs to (its source is open);
  /// `null` for the flat design.
  final int? component;
}

/// The draft is gone (revert, reload, detach, or the text returned to the
/// committed definition): tell the daemon so the overlay goes with it and
/// every later query, on every surface, sees the committed definition.
/// Read-only for the project; not counted.
class DiscardDraft extends Effect {
  const DiscardDraft(this.mappingId, {this.component});
  final int mappingId;
  final int? component;
}

/// Completion candidates from the IDE service, over the draft overlay.
/// Uncounted; answered by `CompletionReceived` / `ToolingFailed` tagged
/// with [generation].
class CompleteDraft extends Effect {
  const CompleteDraft({
    required this.revision,
    required this.mappingId,
    required this.source,
    required this.offset,
    required this.generation,
    this.component,
  });
  final int revision;
  final int mappingId;
  final String source;
  final int offset;
  final int generation;
  final int? component;
}

/// The hover card for a formula name (draft overlay) — uncounted.
/// The Formula Composer's requests (protocol 0.12), tagged like the other
/// tooling requests.
class GetFormulaProjection extends Effect {
  const GetFormulaProjection({
    required this.revision,
    required this.mappingId,
    required this.generation,
    this.component,
  });
  final int revision;
  final int mappingId;
  final int generation;
  final int? component;
}

class GetFormulaSlot extends Effect {
  const GetFormulaSlot({
    required this.revision,
    required this.mappingId,
    required this.source,
    required this.nodeId,
    required this.generation,
    this.component,
  });
  final int revision;
  final int mappingId;
  final String source;
  final String nodeId;
  final int generation;
  final int? component;
}

class ComposeFormula extends Effect {
  const ComposeFormula({
    required this.revision,
    required this.mappingId,
    required this.source,
    required this.action,
    required this.generation,
    this.component,
  });
  final int revision;
  final int mappingId;
  final String source;
  final pb.ComposeAction action;
  final int generation;
  final int? component;
}

class HoverDraft extends Effect {
  const HoverDraft({
    required this.revision,
    required this.mappingId,
    required this.source,
    required this.offset,
    required this.generation,
    this.component,
  });
  final int revision;
  final int mappingId;
  final String source;
  final int offset;
  final int generation;
  final int? component;
}

/// The hover card for an entity — uncounted.
class HoverEntity extends Effect {
  const HoverEntity({required this.revision, required this.entity, required this.generation});
  final int revision;
  final pb.EntityRef entity;
  final int generation;
}

/// The service's semantic actions for an entity — uncounted.
class ListSemanticActions extends Effect {
  const ListSemanticActions({
    required this.revision,
    required this.entity,
    required this.generation,
  });
  final int revision;
  final pb.EntityRef entity;
  final int generation;
}

/// Run the reference evaluator on bdld: start a run at the current
/// revision with the whole input trace and schedule, then step [ticks].
/// Sequential and uncounted; answered by `SimulationReceived` /
/// `SimulationFailed` tagged with [generation].
class RunSimulation extends Effect {
  const RunSimulation({
    required this.inputs,
    required this.schedule,
    required this.ticks,
    required this.generation,
  });
  final List<pb.SimulationInput> inputs;
  final List<pb.SchedulePeriod> schedule;
  final int ticks;
  final int generation;
}

/// The board registry — uncounted.
class ListTargets extends Effect {
  const ListTargets();
}

/// Target-relative analysis of the current revision — uncounted, tagged.
class AnalyzeDeployment extends Effect {
  const AnalyzeDeployment({required this.targetId, required this.generation});
  final String targetId;
  final int generation;
}

class SetLayout extends Effect {
  const SetLayout(this.layout);
  final pb.Layout layout;
}

class Undo extends Effect {
  const Undo();
}

class Redo extends Effect {
  const Redo();
}
