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

class SaveProject extends Effect {
  const SaveProject();
}

class CloseProject extends Effect {
  const CloseProject();
}

class SubscribeProject extends Effect {
  const SubscribeProject();
}

class ApplyEdit extends Effect {
  const ApplyEdit({required this.baseRevision, required this.op});
  final int baseRevision;
  final pb.EditOp op;
}

/// Ask the daemon for its concept libraries.  Not counted as pending.
class ListConceptTemplates extends Effect {
  const ListConceptTemplates();
}

/// The one instantiation request: the daemon builds and applies the
/// `CreateConcept` from the template's defaults.  Answered like an edit.
class InstantiateConceptTemplate extends Effect {
  const InstantiateConceptTemplate({required this.baseRevision, required this.templateId});
  final int baseRevision;
  final String templateId;
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
  });
  final int revision;
  final int mappingId;
  final int generation;
  final String source;
}

/// The draft is gone (revert, reload, detach, or the text returned to the
/// committed definition): tell the daemon so the overlay goes with it and
/// every later query, on every surface, sees the committed definition.
/// Read-only for the project; not counted.
class DiscardDraft extends Effect {
  const DiscardDraft(this.mappingId);
  final int mappingId;
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
  });
  final int revision;
  final int mappingId;
  final String source;
  final int offset;
  final int generation;
}

/// The hover card for a formula name (draft overlay) — uncounted.
class HoverDraft extends Effect {
  const HoverDraft({
    required this.revision,
    required this.mappingId,
    required this.source,
    required this.offset,
    required this.generation,
  });
  final int revision;
  final int mappingId;
  final String source;
  final int offset;
  final int generation;
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
