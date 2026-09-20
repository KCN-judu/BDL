/// Everything that can change application state.
///
/// [UserAction]s originate from widgets; [ResponseAction]s originate from the
/// effect executor (daemon responses, process events).  Both go through the
/// same pure reducer.
library;

import 'dart:ui' show Offset, Rect;

import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'state.dart';

@immutable
sealed class AppAction {
  const AppAction();
}

// ---------------------------------------------------------------------------
// User actions
// ---------------------------------------------------------------------------

sealed class UserAction extends AppAction {
  const UserAction();
}

class AppStarted extends UserAction {
  const AppStarted();
}

class ConnectRequested extends UserAction {
  const ConnectRequested();
}

/// User chose "Open Project…": ask the OS for a folder.
class OpenProjectPickRequested extends UserAction {
  const OpenProjectPickRequested();
}

/// User chose "New Project…": ask the OS where to create it.  There is
/// one kind of project (ADR-0023): sources under `src/`, a behaviour
/// system, shown as Design, Code or Split.
class NewProjectPickRequested extends UserAction {
  const NewProjectPickRequested();
}

class OpenProjectRequested extends UserAction {
  const OpenProjectRequested(this.rootPath);
  final String rootPath;
}

/// Create a project (docs/spec/project-format.md, ADR-0023).
class NewProjectRequested extends UserAction {
  const NewProjectRequested({required this.rootPath, required this.name});
  final String rootPath;
  final String name;
}

/// Save.  A text project whose sources changed on disk since they were
/// loaded refuses (`project.changed_on_disk`) unless [force]; the banner
/// offers reloading instead.
/// The designer answered *Save changes to “…”?*.
class CloseGuardAnswered extends UserAction {
  const CloseGuardAnswered(this.answer);
  final CloseGuardAnswer answer;
}

/// ⌘Q, the menu's Quit, the window's close button: leave the app — after
/// the open project has been unloaded through the guard.
class QuitRequested extends UserAction {
  const QuitRequested();
}

class SaveRequested extends UserAction {
  const SaveRequested({this.force = false});
  final bool force;
}

/// Re-read a text project from disk, dropping unsaved edits: the answer
/// to `project.changed_on_disk` when the other editor's version wins.
class ReloadProjectRequested extends UserAction {
  const ReloadProjectRequested();
}

class CloseProjectRequested extends UserAction {
  const CloseProjectRequested();
}

class UndoRequested extends UserAction {
  const UndoRequested();
}

class RedoRequested extends UserAction {
  const RedoRequested();
}

class CreateConceptRequested extends UserAction {
  const CreateConceptRequested({required this.name, this.description = '', this.representation});
  final String name;
  final String description;
  final pb.Representation? representation;
}

/// Insert a library item (right-click menu, a drag from the Library panel,
/// or its row): one request to the daemon, which creates the item's
/// ordinary objects — a concept, or a concept and its `() -> Value`
/// relationship — in one transaction with fresh identities.  [position] is
/// where the first node lands (scene coordinates); `null` auto-places.
/// Every entry point dispatches exactly this.
class InsertLibraryItemRequested extends UserAction {
  const InsertLibraryItemRequested(this.itemId, {this.position});
  final String itemId;
  final Offset? position;
}

/// Open the Source sheet: a Source is created over a concept the designer
/// chooses there.  [presetId] names a Source item of the library whose
/// preset prefills the sheet (names, a value form, a unit) and ranks the
/// existing concepts; empty for the generic *New Source…*.  [position] is
/// where the objects will land.  Nothing is committed by this action.
class NewSourceRequested extends UserAction {
  const NewSourceRequested({this.presetId = '', this.position});
  final String presetId;
  final Offset? position;
}

/// The Source sheet was closed without creating anything: the project is
/// untouched.
class SourceSheetDismissed extends UserAction {
  const SourceSheetDismissed();
}

/// Create the Source the sheet described, over the chosen concept: an
/// existing one by identity ([existingConcept]), or a new one
/// ([newConceptName] with its value form) created in the same transaction.
/// One revision, one undo, all or nothing — the daemon's `CreateSource`.
class CreateSourceRequested extends UserAction {
  const CreateSourceRequested({
    required this.sourceName,
    this.description = '',
    this.existingConcept,
    this.newConceptName,
    this.newConceptDescription = '',
    this.newConceptRepresentation,
  }) : assert((existingConcept == null) != (newConceptName == null));
  final String sourceName;
  final String description;
  final int? existingConcept;
  final String? newConceptName;
  final String newConceptDescription;
  final pb.Representation? newConceptRepresentation;
}

class SidebarTabSelected extends UserAction {
  const SidebarTabSelected(this.tab);
  final SidebarTab tab;
}

class LibrarySearchChanged extends UserAction {
  const LibrarySearchChanged(this.query);
  final String query;
}

/// Open a node's name for editing on the canvas (double-click the header,
/// or right after an insertion).
class InlineRenameStarted extends UserAction {
  const InlineRenameStarted(this.node);
  final NodeRef node;
}

/// The inline name editor closed: with a new name (a rename edit is sent)
/// or without one (Esc; the default name stays).
class InlineRenameFinished extends UserAction {
  const InlineRenameFinished(this.node, {this.name});
  final NodeRef node;
  final String? name;
}

/// Create a relationship; with [group], it joins that behaviour group once
/// the compiler has confirmed it ("+ Add relationship" in a group).
class CreateMappingRequested extends UserAction {
  const CreateMappingRequested({
    required this.name,
    required this.inputs,
    required this.output,
    this.group,
  });
  final String name;
  final List<int> inputs;
  final int output;
  final int? group;
}

// ---- system projects: context, components, instances, bindings ------------
//
// Every one of these is one `SystemEditOp`, sent against the revision Studio
// holds; the daemon answers with the system, the derived flat design and
// the outcome.  Nothing here decides a semantic fact.

/// Show the system's top level or one component's source on the canvas.
class ContextChanged extends UserAction {
  const ContextChanged(this.context);
  final DesignContext context;
}

class CreateComponentRequested extends UserAction {
  const CreateComponentRequested({required this.name, this.description = ''});
  final String name;
  final String description;
}

class RenameComponentRequested extends UserAction {
  const RenameComponentRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetComponentDescriptionRequested extends UserAction {
  const SetComponentDescriptionRequested({required this.id, required this.description});
  final int id;
  final String description;
}

class DeleteComponentRequested extends UserAction {
  const DeleteComponentRequested(this.id);
  final int id;
}

/// A new version of a component: same interface identities, a copied body.
class DuplicateComponentRequested extends UserAction {
  const DuplicateComponentRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class DeclarePortRequested extends UserAction {
  const DeclarePortRequested({
    required this.component,
    required this.decl,
    required this.kind,
    required this.name,
  });
  final int component;
  final int decl;
  final pb.PortKind kind;
  final String name;
}

class RenamePortRequested extends UserAction {
  const RenamePortRequested({required this.component, required this.port, required this.name});
  final int component;
  final int port;
  final String name;
}

class RetirePortRequested extends UserAction {
  const RetirePortRequested({required this.component, required this.port});
  final int component;
  final int port;
}

/// An explicit change of the promise (the backend classifies it).
class ChangePortContractRequested extends UserAction {
  const ChangePortContractRequested({
    required this.component,
    required this.port,
    required this.contract,
  });
  final int component;
  final int port;
  final pb.PortContractView contract;
}

/// Point the port at another body declaration; the promise is unchanged.
class RebindPortDeclarationRequested extends UserAction {
  const RebindPortDeclarationRequested({
    required this.component,
    required this.port,
    required this.decl,
  });
  final int component;
  final int port;
  final int decl;
}

class SetClockParameterRequested extends UserAction {
  const SetClockParameterRequested({
    required this.component,
    required this.clock,
    required this.parameter,
  });
  final int component;
  final int clock;
  final bool parameter;
}

class ShareConceptRequested extends UserAction {
  const ShareConceptRequested({required this.component, required this.local, this.system});
  final int component;
  final int local;
  final int? system;
}

class ExternalizeOutputRequested extends UserAction {
  const ExternalizeOutputRequested({required this.component, required this.local, this.system});
  final int component;
  final int local;
  final int? system;
}

/// Place an instance of a component; [position] is where the node lands.
class CreateInstanceRequested extends UserAction {
  const CreateInstanceRequested({required this.component, required this.name, this.position});
  final int component;
  final String name;
  final Offset? position;
}

class RenameInstanceRequested extends UserAction {
  const RenameInstanceRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class DeleteInstanceRequested extends UserAction {
  const DeleteInstanceRequested(this.id);
  final int id;
}

/// Swap the component behind an instance (the backend decides
/// substitutability on the interfaces and refuses otherwise).
class ReplaceInstanceComponentRequested extends UserAction {
  const ReplaceInstanceComponentRequested({required this.instance, required this.component});
  final int instance;
  final int component;
}

class SetClockArgumentRequested extends UserAction {
  const SetClockArgumentRequested({
    required this.instance,
    required this.parameter,
    required this.clock,
  });
  final int instance;
  final int parameter;
  final int? clock;
}

class SetParameterArgumentRequested extends UserAction {
  const SetParameterArgumentRequested({
    required this.instance,
    required this.port,
    required this.value,
  });
  final int instance;
  final int port;
  final String? value;
}

/// A link was drawn between two binding ends on the system canvas.  The
/// reducer sends it, or asks first when the destination is taken or the
/// domains differ ([PendingBind]).
class LinkEndsRequested extends UserAction {
  const LinkEndsRequested({required this.source, required this.destination});
  final pb.PortRefView source;
  final pb.PortRefView destination;
}

/// The designer answered the pending-bind question: disconnect and connect
/// (with a transport's initial value when one is needed).
class PendingBindConfirmed extends UserAction {
  const PendingBindConfirmed({this.transportInit});
  final String? transportInit;
}

class PendingBindCancelled extends UserAction {
  const PendingBindCancelled();
}

class BindRequested extends UserAction {
  const BindRequested({required this.source, required this.destination, this.transportInit});
  final pb.PortRefView source;
  final pb.PortRefView destination;
  final String? transportInit;
}

class UnbindRequested extends UserAction {
  const UnbindRequested(this.binding);
  final int binding;
}

// ---- behaviour groups (authoring metadata; never a revision) ---------------

/// A group in the design on screen (the system's own, or the open
/// component's); [renameAfter] opens its name for editing once it arrives.
class CreateGroupRequested extends UserAction {
  const CreateGroupRequested({
    required this.name,
    this.members = const [],
    this.description = '',
    this.renameAfter = false,
  });
  final String name;
  final String description;
  final List<int> members;
  final bool renameAfter;
}

/// "Group as Behavior" on a multi-selection: the relationships among the
/// selected nodes become a group named *Behavior*, then renamed inline.
class GroupSelectionRequested extends UserAction {
  const GroupSelectionRequested();
}

/// The canvas was panned or zoomed; where it stands is layout.
class ViewportChanged extends UserAction {
  const ViewportChanged({required this.pan, required this.zoom});
  final Offset pan;
  final double zoom;
}

class RenameGroupRequested extends UserAction {
  const RenameGroupRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetGroupDescriptionRequested extends UserAction {
  const SetGroupDescriptionRequested({required this.id, required this.description});
  final int id;
  final String description;
}

/// Dissolve the group; its relationships stay.
class UngroupRequested extends UserAction {
  const UngroupRequested(this.id);
  final int id;
}

/// Delete the group *and* its relationships (a revision).
class DeleteGroupWithMembersRequested extends UserAction {
  const DeleteGroupWithMembersRequested(this.id);
  final int id;
}

class AddGroupMemberRequested extends UserAction {
  const AddGroupMemberRequested({required this.group, required this.decl});
  final int group;
  final int decl;
}

class RemoveGroupMemberRequested extends UserAction {
  const RemoveGroupMemberRequested({required this.group, required this.decl});
  final int group;
  final int decl;
}

class MoveGroupMemberRequested extends UserAction {
  const MoveGroupMemberRequested({required this.decl, required this.to});
  final int decl;
  final int to;
}

class MergeGroupsRequested extends UserAction {
  const MergeGroupsRequested({required this.into, required this.from});
  final int into;
  final int from;
}

class SplitGroupRequested extends UserAction {
  const SplitGroupRequested({required this.id, required this.name, required this.members});
  final int id;
  final String name;
  final List<int> members;
}

/// Collapse or expand a group on the canvas (layout only).
class GroupCollapsedChanged extends UserAction {
  const GroupCollapsedChanged({required this.id, required this.collapsed});
  final int id;
  final bool collapsed;
}

/// A collapsed group's box was moved or resized (layout only).
class GroupBoxChanged extends UserAction {
  const GroupBoxChanged({required this.id, required this.rect});
  final int id;
  final Rect rect;
}

// ---- package as reusable component -------------------------------------------

class ExtractionSheetOpened extends UserAction {
  const ExtractionSheetOpened(this.group);
  final int group;
}

class ExtractionChoicesChanged extends UserAction {
  const ExtractionChoicesChanged({
    this.name,
    this.instanceName,
    this.keepInternal,
    this.internalizeSinks,
  });
  final String? name;
  final String? instanceName;
  final Set<int>? keepInternal;
  final Set<int>? internalizeSinks;
}

class ExtractionSheetClosed extends UserAction {
  const ExtractionSheetClosed();
}

/// Package: one atomic system edit from the sheet's choices.
class ExtractionConfirmed extends UserAction {
  const ExtractionConfirmed();
}

// ---- definition drafts -----------------------------------------------------
//
// The formula editor edits a draft owned by Studio; the project changes only
// on [CommitDefinitionRequested] / [DetachDefinitionRequested].

/// The designer typed in the definition editor.
class DefinitionDraftChanged extends UserAction {
  const DefinitionDraftChanged({required this.mappingId, required this.source});
  final int mappingId;
  final String source;
}

/// Discard the draft; the editor shows the committed definition again.
class DefinitionDraftReverted extends UserAction {
  const DefinitionDraftReverted(this.mappingId);
  final int mappingId;
}

/// After a conflict: take the definition committed meanwhile, drop the draft.
class DefinitionDraftReloaded extends UserAction {
  const DefinitionDraftReloaded(this.mappingId);
  final int mappingId;
}

/// After a conflict: keep the draft, rebase it on the current revision.
class DefinitionDraftKept extends UserAction {
  const DefinitionDraftKept(this.mappingId);
  final int mappingId;
}

/// Commit the draft: attach when the mapping has no definition, replace
/// when it has one — chosen from the committed projection, never by the
/// widget.
class CommitDefinitionRequested extends UserAction {
  const CommitDefinitionRequested(this.mappingId);
  final int mappingId;
}

/// Remove the committed definition; the mapping becomes unresolved again.
class DetachDefinitionRequested extends UserAction {
  const DetachDefinitionRequested(this.mappingId);
  final int mappingId;
}

// ---- semantic tooling in the definition field -------------------------------

/// Ask for completion at a byte [offset] into [source] (⌃Space, or typing
/// while the pop-up is open).
class CompletionRequested extends UserAction {
  const CompletionRequested({required this.mappingId, required this.source, required this.offset});
  final int mappingId;
  final String source;
  final int offset;
}

class CompletionDismissed extends UserAction {
  const CompletionDismissed();
}

/// Move the selection by [delta] rows (wraps).
class CompletionMoved extends UserAction {
  const CompletionMoved(this.delta);
  final int delta;
}

/// Hover a name in the definition field of [mappingId] at a byte [offset]
/// into [source]; `null` offset ends the hover.
class FormulaHoverRequested extends UserAction {
  const FormulaHoverRequested({
    required this.mappingId,
    required this.source,
    required this.offset,
  });
  final int mappingId;
  final String source;
  final int? offset;
}

// ---- the Formula Composer (app/composer.dart) --------------------------------

/// Formula (structured) or Text: which projection of the definition draft
/// the inspector edits.  Both edit the same draft.
class FormulaModeChanged extends UserAction {
  const FormulaModeChanged(this.formulaMode);
  final bool formulaMode;
}

/// Select a node of the mapping's projection (`null` clears); the
/// service is asked what the position expects and what fits.
class FormulaNodeSelected extends UserAction {
  const FormulaNodeSelected({required this.mappingId, required this.nodeId});
  final int mappingId;
  final String? nodeId;
}

/// A structured action on the draft, answered with the text it makes.
class ComposeRequested extends UserAction {
  const ComposeRequested({required this.mappingId, required this.action});
  final int mappingId;
  final pb.ComposeAction action;
}

/// Ask for the projection of the committed definition (no draft).
class FormulaProjectionRequested extends UserAction {
  const FormulaProjectionRequested(this.mappingId);
  final int mappingId;
}

/// Hover an entity (a canvas node, a library row); `null` ends the hover.
class EntityHoverRequested extends UserAction {
  const EntityHoverRequested(this.entity);
  final pb.EntityRef? entity;
}

class PageSelected extends UserAction {
  const PageSelected(this.page);
  final StudioPage page;
}

class RenameConceptRequested extends UserAction {
  const RenameConceptRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetConceptDescriptionRequested extends UserAction {
  const SetConceptDescriptionRequested({required this.id, required this.description});
  final int id;
  final String description;
}

class SetConceptRepresentationRequested extends UserAction {
  const SetConceptRepresentationRequested({required this.id, required this.representation});
  final int id;
  final pb.Representation? representation;
}

class SetConceptOrderedRequested extends UserAction {
  const SetConceptOrderedRequested({required this.id, required this.ordered});
  final int id;
  final bool ordered;
}

class DeleteConceptRequested extends UserAction {
  const DeleteConceptRequested(this.id);
  final int id;
}

class RenameMappingRequested extends UserAction {
  const RenameMappingRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetMappingDescriptionRequested extends UserAction {
  const SetMappingDescriptionRequested({required this.id, required this.description});
  final int id;
  final String description;
}

class SetMappingSignatureRequested extends UserAction {
  const SetMappingSignatureRequested({
    required this.id,
    required this.inputs,
    required this.output,
  });
  final int id;
  final List<int> inputs;
  final int output;
}

class DeleteMappingRequested extends UserAction {
  const DeleteMappingRequested(this.id);
  final int id;
}

// ---- timing domains, outputs, devices ---------------------------------------

class CreateClockDomainRequested extends UserAction {
  const CreateClockDomainRequested(this.name);
  final String name;
}

class RenameClockDomainRequested extends UserAction {
  const RenameClockDomainRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class DeleteClockDomainRequested extends UserAction {
  const DeleteClockDomainRequested(this.id);
  final int id;
}

/// `clockId == null` makes the mapping domain-agnostic (pure).
class SetMappingClockRequested extends UserAction {
  const SetMappingClockRequested({required this.mappingId, required this.clockId});
  final int mappingId;
  final int? clockId;
}

class CreateOutputRequested extends UserAction {
  const CreateOutputRequested({
    required this.name,
    required this.accepts,
    this.description = '',
    this.clockId,
    this.required = false,
  });
  final String name;
  final String description;
  final int accepts;
  final int? clockId;
  final bool required;
}

class RenameOutputRequested extends UserAction {
  const RenameOutputRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetOutputAcceptsRequested extends UserAction {
  const SetOutputAcceptsRequested({required this.id, required this.accepts});
  final int id;
  final int accepts;
}

class SetOutputClockRequested extends UserAction {
  const SetOutputClockRequested({required this.id, required this.clockId});
  final int id;
  final int? clockId;
}

class SetOutputRequiredRequested extends UserAction {
  const SetOutputRequiredRequested({required this.id, required this.required});
  final int id;
  final bool required;
}

class DeleteOutputRequested extends UserAction {
  const DeleteOutputRequested(this.id);
  final int id;
}

/// Connect a mapping as the driver of an output, or (`outputId == null`)
/// disconnect it from whatever it drives.
class SetMappingDriveRequested extends UserAction {
  const SetMappingDriveRequested({required this.mappingId, required this.outputId});
  final int mappingId;
  final int? outputId;
}

class CreateDeviceRequested extends UserAction {
  const CreateDeviceRequested({required this.name, required this.kind, this.outputId});
  final String name;
  final pb.DeviceKind kind;
  final int? outputId;
}

class RenameDeviceRequested extends UserAction {
  const RenameDeviceRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetDeviceKindRequested extends UserAction {
  const SetDeviceKindRequested({required this.id, required this.kind});
  final int id;
  final pb.DeviceKind kind;
}

class SetDeviceOutputRequested extends UserAction {
  const SetDeviceOutputRequested({required this.id, required this.outputId});
  final int id;
  final int? outputId;
}

/// Choose the realization profile of a device (null releases it), with the
/// device kind the profile prescribes — one deployment decision.
class SetDeviceRealizationRequested extends UserAction {
  const SetDeviceRealizationRequested({
    required this.id,
    required this.profileId,
    required this.kind,
  });
  final int id;
  final String? profileId;
  final pb.DeviceKind kind;
}

class SetDevicePinRequested extends UserAction {
  const SetDevicePinRequested({required this.id, required this.index, required this.resource});
  final int id;
  final int index;
  final String? resource;
}

class DeleteDeviceRequested extends UserAction {
  const DeleteDeviceRequested(this.id);
  final int id;
}

// ---- semantic actions ---------------------------------------------------------

/// Ask the service which fixes and context actions it offers for the
/// selected entity.
class SemanticActionsRequested extends UserAction {
  const SemanticActionsRequested(this.entity);
  final pb.EntityRef entity;
}

/// Apply a ready action's model edits (one revisioned edit each, in order),
/// or the chosen option of a needs-choice action.
class SemanticActionApplied extends UserAction {
  const SemanticActionApplied({required this.actionId, this.option});
  final String actionId;
  final int? option;
}

/// A Fix chosen where its finding is shown (the Simulate page's readiness
/// area) rather than in the inspector: select the object, ask the service
/// for its actions, and apply the one of [actionKind] as soon as it
/// arrives ready.  An action that needs a choice or is blocked is then on
/// show for the designer to complete.
class SemanticActionChosen extends UserAction {
  const SemanticActionChosen({required this.selection, required this.actionKind});
  final Selection selection;
  final String actionKind;
}

// ---- simulation ------------------------------------------------------------

/// The value an input takes from the next evaluated tick on.
class SimulationInputChanged extends UserAction {
  const SimulationInputChanged({required this.mappingId, required this.value});
  final int mappingId;
  final pb.Value value;
}

/// The activation period of a domain (applies to the whole run: the run
/// is re-created from tick 0 with the new schedule).
class SimulationPeriodChanged extends UserAction {
  const SimulationPeriodChanged({required this.clockId, required this.period});
  final int clockId;
  final int period;
}

/// Evaluate [ticks] more global ticks with the current inputs.
class SimulationStepRequested extends UserAction {
  const SimulationStepRequested([this.ticks = 1]);
  final int ticks;
}

/// Back to tick 0; the input trace is kept for editing.
class SimulationResetRequested extends UserAction {
  const SimulationResetRequested();
}

// ---- deployment ------------------------------------------------------------

/// Ask bdld which boards it knows (on first visit to Deploy).
class TargetsRequested extends UserAction {
  const TargetsRequested();
}

/// The designer chose a board; `null` clears the choice.
class TargetSelected extends UserAction {
  const TargetSelected(this.targetId);
  final String? targetId;
}

/// Ask again for the chosen board at the current revision.
class DeploymentRequested extends UserAction {
  const DeploymentRequested();
}

/// Delete whatever is selected.
class DeleteSelectionRequested extends UserAction {
  const DeleteSelectionRequested();
}

/// A node finished being dragged; commit its position (layout, not semantics).
class NodeMoved extends UserAction {
  const NodeMoved(this.node, this.position);
  final NodeRef node;
  final Offset position;
}

/// A link was drawn from a concept's value socket into a mapping's inputs.
class LinkConceptToMappingInput extends UserAction {
  const LinkConceptToMappingInput({required this.conceptId, required this.mappingId});
  final int conceptId;
  final int mappingId;
}

/// A link was drawn from a mapping's output socket to a concept.
class LinkMappingOutputToConcept extends UserAction {
  const LinkMappingOutputToConcept({required this.mappingId, required this.conceptId});
  final int mappingId;
  final int conceptId;
}

/// A concept was disconnected from a mapping's inputs.
class UnlinkMappingInput extends UserAction {
  const UnlinkMappingInput({required this.mappingId, required this.conceptId});
  final int mappingId;
  final int conceptId;
}

class RemoveRecentRequested extends UserAction {
  const RemoveRecentRequested(this.path);
  final String path;
}

class SelectionChanged extends UserAction {
  const SelectionChanged(this.selection);
  final Selection selection;
}

// ---- the Code view (ADR-0023 §3–§5) -------------------------------------

/// Design, Code or Split.
class DesignViewChanged extends UserAction {
  const DesignViewChanged(this.view);
  final DesignView view;
}

/// The editor shows another source file.
/// The text on screen wants its tokens (the Code view's file or a
/// formula draft): after a pause in typing, or when a text is first shown.
/// [component] scopes a formula document; [path] names a file document.
class SemanticTokensRequested extends UserAction {
  const SemanticTokensRequested.file({required this.path, required this.text})
    : mappingId = null,
      component = null;
  const SemanticTokensRequested.formula({
    required this.mappingId,
    required this.text,
    this.component,
  }) : path = null;
  final String? path;
  final int? mappingId;
  final int? component;
  final String text;
}

/// The Code view's IDE requests, each over the file's text exactly as
/// typed and a byte offset into it (`app/code_tooling.dart`).
class SourceCompletionRequested extends UserAction {
  const SourceCompletionRequested({required this.path, required this.text, required this.offset});
  final String path;
  final String text;
  final int offset;
}

/// A null [offset] ends the hover.
class SourceHoverRequested extends UserAction {
  const SourceHoverRequested({required this.path, required this.text, required this.offset});
  final String path;
  final String text;
  final int? offset;
}

class SourceDefinitionRequested extends UserAction {
  const SourceDefinitionRequested({required this.path, required this.text, required this.offset});
  final String path;
  final String text;
  final int offset;
}

class SourceReferencesRequested extends UserAction {
  const SourceReferencesRequested({required this.path, required this.text, required this.offset});
  final String path;
  final String text;
  final int offset;
}

class ReferencesDismissed extends UserAction {
  const ReferencesDismissed();
}

/// Format the file: the daemon's canonical layout of [text], applied as
/// one authored edit when it comes back for the text still on screen.
class FormatSourceRequested extends UserAction {
  const FormatSourceRequested({required this.path, required this.text});
  final String path;
  final String text;
}

class SourceFileOpened extends UserAction {
  const SourceFileOpened(this.path);
  final String path;
}

/// The designer typed in the editor: the text is the editor's until it is
/// sent (after a pause, or on leaving the field).
class SourceTextChanged extends UserAction {
  const SourceTextChanged(this.path, this.text);
  final String path;
  final String text;
}

/// Send the editor's text for [path] to the model.
class SourceEditRequested extends UserAction {
  const SourceEditRequested(this.path, this.text);
  final String path;
  final String text;
}

class ErrorDismissed extends UserAction {
  const ErrorDismissed();
}

// ---------------------------------------------------------------------------
// Response actions (from effects)
// ---------------------------------------------------------------------------

sealed class ResponseAction extends AppAction {
  const ResponseAction();
}

class DaemonConnected extends ResponseAction {
  const DaemonConnected({required this.executable, required this.handshake});
  final String executable;
  final pb.HandshakeResponse handshake;
}

class DaemonConnectionFailed extends ResponseAction {
  const DaemonConnectionFailed(this.reason);
  final String reason;
}

class DaemonExited extends ResponseAction {
  const DaemonExited(this.exitCode);
  final int exitCode;
}

class DaemonLogged extends ResponseAction {
  const DaemonLogged(this.line);
  final String line;
}

/// The authored system arrived (GetSystem, or a group edit's answer).
class SystemReceived extends ResponseAction {
  const SystemReceived(this.system, {this.fromRequest = true});
  final pb.SystemView system;
  final bool fromRequest;
}

/// A system edit was applied: the system, the derived flat design and the
/// outcome, in one answer.
class SystemEditApplied extends ResponseAction {
  const SystemEditApplied({required this.system, required this.project, this.outcome});
  final pb.SystemView system;
  final pb.ProjectProjection project;
  final pb.SystemEditOutcome? outcome;
}

/// The system-level analysis arrived.  Revision-tagged, like the flat one.
class SystemAnalysisReceived extends ResponseAction {
  const SystemAnalysisReceived(this.analysis);
  final pb.SystemAnalysisView analysis;
}

class ExtractionPreviewReceived extends ResponseAction {
  const ExtractionPreviewReceived({required this.generation, required this.preview});
  final int generation;
  final pb.ExtractionPreviewView preview;
}

class ExtractionPreviewFailed extends ResponseAction {
  const ExtractionPreviewFailed({
    required this.generation,
    required this.code,
    required this.message,
  });
  final int generation;
  final String code;
  final String message;
}

/// A project projection arrived (from open/init/save/get/edit responses or a
/// ProjectChanged event).  Carries the revision; stale ones are dropped.
class ProjectReceived extends ResponseAction {
  const ProjectReceived(this.project, {this.outcome, this.fromRequest = true});
  final pb.ProjectProjection project;
  final pb.EditOutcome? outcome;

  /// True when this answers a request Studio sent (and counted as pending);
  /// false for unsolicited `ProjectChanged` events.
  final bool fromRequest;
}

/// The OS picker returned instantly with nothing — it was refused, not
/// cancelled (view-bridge failure under a sandboxed host).
class PickerUnavailable extends ResponseAction {
  const PickerUnavailable();
}

/// A compiler analysis arrived (response or pushed event).  Revision-tagged;
/// the reducer keeps it only if it matches the project it holds.
class AnalysisReceived extends ResponseAction {
  const AnalysisReceived(this.analysis, {this.fromRequest = false});
  final pb.ProjectAnalysis analysis;
  final bool fromRequest;
}

/// The compiler's verdict on a draft.  Tagged with revision, mapping and
/// generation; the reducer keeps it only if all three still match.
class DraftAnalysisReceived extends ResponseAction {
  const DraftAnalysisReceived(this.result);
  final pb.DefinitionDraftAnalysis result;
}

/// A draft check could not be completed.  `draft.stale_revision` means a
/// newer projection is on its way and will re-ask; anything else leaves the
/// draft unchecked (never discarded).
class DraftAnalysisFailed extends ResponseAction {
  const DraftAnalysisFailed({
    required this.mappingId,
    required this.generation,
    required this.code,
    required this.message,
  });
  final int mappingId;
  final int generation;
  final String code;
  final String message;
}

class CompletionReceived extends ResponseAction {
  const CompletionReceived({required this.generation, required this.result});
  final int generation;
  final pb.DraftCompletionResponse result;
}

class SemanticTokensReceived extends ResponseAction {
  const SemanticTokensReceived({required this.key, required this.result});
  final String key;
  final pb.SemanticTokensResponse result;
}

/// The request failed or could not be sent: the tokens on show stay.
class SemanticTokensFailed extends ResponseAction {
  const SemanticTokensFailed({required this.key, required this.generation});
  final String key;
  final int generation;
}

class SourceCompletionReceived extends ResponseAction {
  const SourceCompletionReceived({required this.generation, required this.result});
  final int generation;
  final pb.SourceCompletionResponse result;
}

/// The definition site(s) of the name asked about.
class SourceDefinitionReceived extends ResponseAction {
  const SourceDefinitionReceived({required this.generation, required this.result});
  final int generation;
  final pb.SourceLocationsResponse result;
}

class SourceReferencesReceived extends ResponseAction {
  const SourceReferencesReceived({required this.generation, required this.result});
  final int generation;
  final pb.SourceLocationsResponse result;
}

class FormatSourceReceived extends ResponseAction {
  const FormatSourceReceived({required this.generation, required this.path, required this.result});
  final int generation;
  final String path;
  final pb.FormatSourceResponse result;
}

class HoverReceived extends ResponseAction {
  const HoverReceived({required this.generation, required this.result});
  final int generation;
  final pb.DraftHoverResponse result;
}

class FormulaSlotReceived extends ResponseAction {
  const FormulaSlotReceived({required this.generation, required this.result});
  final int generation;
  final pb.FormulaSlotResponse result;
}

class FormulaProjectionReceived extends ResponseAction {
  const FormulaProjectionReceived({required this.generation, required this.result});
  final int generation;
  final pb.FormulaProjectionResponse result;
}

class ComposeReceived extends ResponseAction {
  const ComposeReceived({required this.generation, required this.result});
  final int generation;
  final pb.ComposeFormulaResponse result;
}

/// A tooling request (completion, hover) failed; the pop-up or card just
/// does not appear.  Never a banner.
class ToolingFailed extends ResponseAction {
  const ToolingFailed({required this.generation, required this.code, required this.message});
  final int generation;
  final String code;
  final String message;
}

class SemanticActionsReceived extends ResponseAction {
  const SemanticActionsReceived({required this.generation, required this.result});
  final int generation;
  final pb.SemanticActionsResponse result;
}

class SimulationReceived extends ResponseAction {
  const SimulationReceived({required this.generation, required this.response});
  final int generation;
  final pb.SimulationResponse response;
}

class SimulationFailed extends ResponseAction {
  const SimulationFailed({required this.generation, required this.code, required this.message});
  final int generation;
  final String code;
  final String message;
}

class TargetsReceived extends ResponseAction {
  const TargetsReceived(this.targets);
  final List<pb.TargetView> targets;
}

class DeploymentReceived extends ResponseAction {
  const DeploymentReceived({required this.generation, required this.analysis});
  final int generation;
  final pb.DeploymentAnalysis analysis;
}

class DeploymentFailed extends ResponseAction {
  const DeploymentFailed({required this.generation, required this.code, required this.message});
  final int generation;
  final String code;
  final String message;
}

/// The daemon's libraries arrived, as items (asked once per connection).
class LibraryItemsReceived extends ResponseAction {
  const LibraryItemsReceived(this.library);
  final pb.LibraryItemsResponse library;
}

/// The daemon's ranked concepts for the Source sheet ([generation] ties
/// the answer to the request that opened the sheet).
class SourceCandidatesReceived extends ResponseAction {
  const SourceCandidatesReceived({required this.generation, required this.response});
  final int generation;
  final pb.SourceCandidatesResponse response;
}

/// The user chose a display language in Preferences.  An application
/// preference: the UI re-renders, nothing in the project changes.
class LanguageChanged extends UserAction {
  const LanguageChanged(this.language);
  final LanguagePreference language;
}

class PreferencesLoaded extends ResponseAction {
  const PreferencesLoaded(this.preferences);
  final AppPreferences preferences;
}

class RecentProjectsLoaded extends ResponseAction {
  const RecentProjectsLoaded(this.recent);
  final List<RecentProject> recent;
}

class ProjectClosed extends ResponseAction {
  const ProjectClosed();
}

class RequestSucceeded extends ResponseAction {
  const RequestSucceeded();
}

/// GetSources answered.
class SourcesReceived extends ResponseAction {
  const SourcesReceived(this.sources);
  final pb.SourcesView sources;
}

/// ApplySourceEdit answered: accepted (a new revision) or refused with
/// the draft's diagnostics; the projection and sources either way.
class SourceEditApplied extends ResponseAction {
  const SourceEditApplied(this.applied);
  final pb.SourceEditApplied applied;
}

class RequestFailed extends ResponseAction {
  const RequestFailed({required this.code, required this.message, this.details = ''});
  final String code;
  final String message;
  final String details;
}
