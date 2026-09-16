/// The inspector: what the selected object means, what can be changed, and
/// what a change will affect — in the designer's words (docs/STUDIO_UI.md
/// §7, level 2).  Each control commits one `EditOp`.  Formal vocabulary
/// (ids, kernel types, invalidation categories, core terms) lives in one
/// collapsed *Explain* disclosure at the end (level 3).
library;

import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/canvas_geometry.dart' show dimLabel, socketKind, statusWord, SocketKind;
import 'canvas/concept_glyphs.dart';
import 'definition_editor.dart';
import 'mac/controls.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'semantic_actions.dart';
import 'system_inspector.dart';
import 'units.dart';

class Inspector extends StatelessWidget {
  const Inspector({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final project = state.project;
    final sel = state.editor.selection;

    Widget body;
    if (project == null) {
      body = const SizedBox.shrink();
    } else {
      body = switch (sel) {
        NoSelection() => Padding(
          padding: const EdgeInsets.all(12),
          child: Text(
            state.isSystem
                ? 'Select a concept, a relationship, an output, an instance or a group.'
                : 'Select a concept, a mapping or an output.',
            style: TextStyle(color: t.textTertiary),
          ),
        ),
        ConceptSelected(:final id) => _ConceptInspector(
          key: ValueKey('c$id'),
          concept: project.concepts.firstWhere((c) => c.id.toInt() == id),
          readers: project.mappings
              .where((m) => m.signature.inputs.any((i) => i.toInt() == id))
              .toList(),
          producers: project.mappings.where((m) => m.signature.output.toInt() == id).toList(),
          revision: project.revision.toInt(),
          outcome: state.editor.lastOutcome,
          presets: unitPresetsFrom(state.library?.quantities ?? const []),
          dispatch: dispatch,
        ),
        MappingSelected(:final id) => _MappingInspector(
          key: ValueKey('m$id'),
          mapping: project.mappings.firstWhere((m) => m.id.toInt() == id),
          concepts: project.concepts,
          analysis: state.mappingAnalysis(id),
          draft: state.draft(id),
          completion: state.editor.completion?.mappingId == id ? state.editor.completion : null,
          hover: state.editor.hover?.mappingId == id ? state.editor.hover : null,
          revision: project.revision.toInt(),
          outcome: state.editor.lastOutcome,
          clocks: project.clocks,
          outputs: project.outputs,
          actions: state.editor.actions,
          dispatch: dispatch,
          // A base relationship the system realised by a binding, or a
          // port-backed relationship of a component's source.
          boundTo: state.system?.bindings
              .where((b) => b.destination.hasBaseDecl() && b.destination.baseDecl.toInt() == id)
              .firstOrNull,
          boundLabel: (b) => endLabel(state, b.source),
          portWord: state.openComponent?.ports
              .where((p) => p.decl.toInt() == id)
              .map((p) => '${portKindWord(p.kind)} ${p.name}')
              .firstOrNull,
          group: state.editor.context is SystemContext ? state.groupOf(id) : null,
        ),
        OutputSelected(:final id) => _OutputInspector(
          key: ValueKey('o$id'),
          output: project.outputs.firstWhere((o) => o.id.toInt() == id),
          state: state,
          dispatch: dispatch,
        ),
        ComponentSelected(:final id) => ComponentInspector(
          key: ValueKey('comp$id'),
          state: state,
          id: id,
          dispatch: dispatch,
        ),
        InstanceSelected(:final id) => InstanceInspector(
          key: ValueKey('inst$id'),
          state: state,
          id: id,
          dispatch: dispatch,
        ),
        PortSelected(:final instance, :final port) => PortInspector(
          key: ValueKey('port$instance/$port'),
          state: state,
          instance: instance,
          port: port,
          dispatch: dispatch,
        ),
        BindingSelected(:final id) => BindingInspector(
          key: ValueKey('bind$id'),
          state: state,
          id: id,
          dispatch: dispatch,
        ),
        GroupSelected(:final id) => GroupInspector(
          key: ValueKey('group$id'),
          state: state,
          id: id,
          dispatch: dispatch,
        ),
      };
    }

    return Container(
      color: t.sidebar,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const PanelHeader('Inspector'),
          Expanded(child: SingleChildScrollView(child: body)),
          if (state.editor.lastOutcome case final o?)
            if (project != null) _ChangeNote(outcome: o, project: project, dispatch: dispatch),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Consequence of the last change
// ---------------------------------------------------------------------------

/// What the last change did to the rest of the design, as a sentence about
/// other objects — never as a classification word.  The formal kind and the
/// invalidation categories are in Explain.
class _ChangeNote extends StatelessWidget {
  const _ChangeNote({required this.outcome, required this.project, required this.dispatch});
  final pb.EditOutcome outcome;
  final pb.ProjectProjection project;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    if (outcome.kind == pb.EditKind.EDIT_KIND_UNSPECIFIED) return const SizedBox.shrink();
    final affected = [
      for (final d in outcome.originDecls) ...project.mappings.where((m) => m.id == d),
    ];
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final Widget sentence;
    if (outcome.kind == pb.EditKind.EDIT_KIND_REFINEMENT) {
      sentence = Text('Nothing else needs rechecking.', style: small);
    } else if (affected.isEmpty) {
      sentence = Text('This will be checked again.', style: small);
    } else {
      sentence = Wrap(
        crossAxisAlignment: WrapCrossAlignment.center,
        spacing: MacMetrics.gapTight,
        children: [
          Text(affected.length == 1 ? 'This change affects' : 'This change affects', style: small),
          for (final m in affected)
            MacLink(
              label: m.name,
              onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
            ),
          Text(
            affected.length == 1 ? '— it will be checked again.' : '— they will be checked again.',
            style: small,
          ),
        ],
      );
    }
    return Container(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 10),
      decoration: BoxDecoration(
        border: Border(top: BorderSide(color: t.hairline)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gapTight,
        children: [
          Text('Last change', style: Theme.of(context).textTheme.titleSmall),
          sentence,
        ],
      ),
    );
  }
}

String _kindWord(pb.EditKind k) => switch (k) {
  pb.EditKind.EDIT_KIND_REFINEMENT => 'refinement',
  pb.EditKind.EDIT_KIND_EDIT => 'edit',
  _ => 'unspecified',
};

/// The last change in the kernel's terms: refinement or edit, and which
/// invalidation categories it raised.
String _outcomeNotation(pb.EditOutcome o) {
  final cats = o.invalidates.map(_invalidationWord).join(', ');
  return 'last change: ${_kindWord(o.kind)}${cats.isEmpty ? '' : ' · invalidates $cats'}';
}

String _invalidationWord(pb.Invalidation i) => switch (i) {
  pb.Invalidation.INVALIDATION_INTERFACE => 'Interface',
  pb.Invalidation.INVALIDATION_REALIZATION => 'Realization',
  pb.Invalidation.INVALIDATION_SEMANTIC => 'Semantic',
  pb.Invalidation.INVALIDATION_REACTIVE => 'Reactive',
  pb.Invalidation.INVALIDATION_CLOCK => 'Clock',
  pb.Invalidation.INVALIDATION_OUTPUT => 'Output',
  pb.Invalidation.INVALIDATION_DEPLOYMENT => 'Deployment',
  _ => 'Unspecified',
};

/// Names as links, in a form row: "Used by  dimByTilt  warmPulse".
class _NameLinks extends StatelessWidget {
  const _NameLinks({required this.mappings, required this.dispatch, required this.empty});
  final List<pb.MappingView> mappings;
  final void Function(AppAction) dispatch;
  final String empty;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    if (mappings.isEmpty) {
      return Padding(
        padding: const EdgeInsets.only(top: 4),
        child: Text(empty, style: TextStyle(fontSize: 13, color: t.textTertiary)),
      );
    }
    return Wrap(
      spacing: MacMetrics.gap,
      runSpacing: MacMetrics.gapTight,
      children: [
        for (final m in mappings)
          MacLink(
            label: m.name,
            onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
          ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Concept
// ---------------------------------------------------------------------------

/// A concept is a product meaning: what it means, what form its value
/// takes, who reads it, who produces it.
class _ConceptInspector extends StatelessWidget {
  const _ConceptInspector({
    super.key,
    required this.concept,
    required this.readers,
    required this.producers,
    required this.revision,
    required this.outcome,
    required this.presets,
    required this.dispatch,
  });
  final pb.ConceptView concept;
  final List<pb.MappingView> readers;
  final List<pb.MappingView> producers;
  final int revision;

  /// The unit picker's quantities (the daemon's shared vocabulary).
  final List<UnitPreset> presets;

  /// The last change, for its formal classification in Explain.
  final pb.EditOutcome? outcome;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = concept.id.toInt();
    final users = [...producers, ...readers];
    final bound = concept.hasRepresentation();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Meaning',
          trailing: SocketGlyph.of(concept, t),
          children: [
            FormRow(
              label: 'Name',
              child: CommitTextField(
                value: concept.name,
                onCommit: (v) => dispatch(RenameConceptRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: 'Meaning',
              child: CommitTextField(
                value: concept.description,
                maxLines: 3,
                onCommit: (v) => dispatch(SetConceptDescriptionRequested(id: id, description: v)),
              ),
            ),
          ],
        ),
        InspectorSection(
          title: 'Value',
          children: [
            _ValueEditor(
              current: bound ? concept.representation : null,
              presets: presets,
              onChanged: (r) =>
                  dispatch(SetConceptRepresentationRequested(id: id, representation: r)),
            ),
            // The consequence, only when there is one: rebinding a chosen
            // value form reopens every relationship typed against it.
            if (bound && users.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(
                  'Changing this re-checks ${_names(users)}.',
                  style: TextStyle(fontSize: 11, color: t.textSecondary),
                ),
              ),
          ],
        ),
        InspectorSection(
          title: 'Relationships',
          children: [
            FormRow(
              label: 'Produced by',
              child: _NameLinks(mappings: producers, dispatch: dispatch, empty: 'nothing yet'),
            ),
            FormRow(
              label: 'Used by',
              child: _NameLinks(mappings: readers, dispatch: dispatch, empty: 'nothing yet'),
            ),
          ],
        ),
        Padding(
          padding: const EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: MacMetrics.gapTight,
            children: [
              DestructiveButton(
                label: 'Delete ${concept.name}',
                enabled: users.isEmpty,
                onPressed: () => dispatch(DeleteConceptRequested(id)),
              ),
              // A disabled control says why, at rest.
              if (users.isNotEmpty)
                Text(
                  'Still used by ${_names(users)}.',
                  style: TextStyle(fontSize: 11, color: t.textSecondary),
                ),
            ],
          ),
        ),
        MacDisclosure(
          title: 'Explain',
          children: [
            ExplainLine('SemanticId ${concept.id}'),
            ExplainLine(
              'Θ(${concept.id}) = ${bound ? _tyNotation(concept.representation) : 'none'}',
            ),
            ExplainLine('revision $revision'),
            if (outcome case final o?) ExplainLine(_outcomeNotation(o)),
          ],
        ),
      ],
    );
  }

  static String _names(List<pb.MappingView> ms) => ms.map((m) => m.name).join(', ');
}

String _tyNotation(pb.Representation r) => switch (r.whichKind()) {
  pb.Representation_Kind.quantity => 'q[${dimLabel(r.quantity)}]',
  pb.Representation_Kind.boolean => 'bool',
  pb.Representation_Kind.count => 'nat',
  pb.Representation_Kind.notSet => 'none',
};

/// The value form, in the same words and order as the creation sheet:
/// Quantity / On–off / Count / Decide later, then a quantity's unit.
class _ValueEditor extends StatelessWidget {
  const _ValueEditor({required this.current, required this.presets, required this.onChanged});
  final pb.Representation? current;
  final List<UnitPreset> presets;
  final void Function(pb.Representation?) onChanged;

  @override
  Widget build(BuildContext context) {
    final kind = current?.whichKind() ?? pb.Representation_Kind.notSet;
    final dim = kind == pb.Representation_Kind.quantity ? current!.quantity : null;
    final preset = dim == null ? null : unitPresetFor(presets, dim);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        MacSegmented<pb.Representation_Kind>(
          value: kind,
          options: const {
            pb.Representation_Kind.quantity: 'Quantity',
            pb.Representation_Kind.boolean: 'On / off',
            pb.Representation_Kind.count: 'Count',
            pb.Representation_Kind.notSet: 'Decide later',
          },
          onChanged: (k) => onChanged(switch (k) {
            pb.Representation_Kind.notSet => null,
            pb.Representation_Kind.quantity => pb.Representation(quantity: pb.Dim()),
            pb.Representation_Kind.boolean => pb.Representation(boolean: pb.Unit()),
            pb.Representation_Kind.count => pb.Representation(count: pb.Unit()),
          }),
        ),
        if (kind == pb.Representation_Kind.quantity) ...[
          const SizedBox(height: 8),
          FormRow(
            label: 'Unit',
            child: MacDropdown<UnitPreset>(
              value: preset,
              // a dimension outside the presets is still shown, as its symbol
              hint: dim == null ? null : dimLabel(dim),
              items: presets,
              labelOf: (p) => p.name,
              detailOf: (p) => p.symbol,
              onChanged: (p) => onChanged(pb.Representation(quantity: p.dim)),
            ),
          ),
        ],
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

/// A mapping is a relationship between concepts: what it means, what it
/// reads, what it produces, and the relationship itself (its definition).
class _MappingInspector extends StatelessWidget {
  const _MappingInspector({
    super.key,
    required this.mapping,
    required this.concepts,
    required this.analysis,
    required this.draft,
    required this.completion,
    required this.hover,
    required this.revision,
    required this.outcome,
    required this.clocks,
    required this.outputs,
    required this.actions,
    required this.dispatch,
    this.boundTo,
    this.boundLabel,
    this.portWord,
    this.group,
  });
  final pb.MappingView mapping;
  final List<pb.ConceptView> concepts;
  final List<pb.ClockView> clocks;
  final List<pb.OutputView> outputs;
  final SemanticActionsState? actions;

  /// The binding that realises this (open) base relationship, if any: the
  /// definition is then the system's, not the designer's — shown, never
  /// edited here.
  final pb.BindingView? boundTo;
  final String Function(pb.BindingView)? boundLabel;

  /// "provides brightness": the port this relationship backs, in a
  /// component's source.
  final String? portWord;

  /// The behaviour group the relationship belongs to (system canvas).
  final pb.BehaviorGroupView? group;

  /// The compiler's verdict for the current revision; `null` while pending.
  final pb.MappingAnalysis? analysis;

  /// Studio's uncommitted definition text for this mapping, if any.
  final DefinitionDraft? draft;
  final CompletionState? completion;
  final HoverState? hover;
  final int revision;

  /// The last change, for its formal classification in Explain.
  final pb.EditOutcome? outcome;
  final void Function(AppAction) dispatch;

  pb.ConceptView? _concept(int id) => concepts.where((c) => c.id.toInt() == id).firstOrNull;
  String _name(int id) => _concept(id)?.name ?? '?';

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = mapping.id.toInt();
    final inputs = mapping.signature.inputs.map((i) => i.toInt()).toList();
    final output = mapping.signature.output.toInt();
    final declared = !mapping.hasDefinition();
    final committed = mapping.hasDefinition() ? mapping.definition.formula : null;
    final a = analysis;
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    // Findings without a span, by where they belong: timing ones under
    // *Updates in*, drive ones under *Drives*, the rest (causality) with the
    // relationship itself.
    final spanless = a?.diagnostics.where((d) => !d.hasSpan()).toList() ?? const <pb.Diagnostic>[];
    final timingIssues = spanless.where((d) => d.code.startsWith('clock.')).toList();
    final driveIssues = spanless.where((d) => d.code.startsWith('output.')).toList();
    final broader = spanless
        .where((d) => !d.code.startsWith('clock.') && !d.code.startsWith('output.'))
        .toList();
    final clockId = mapping.hasClockId() ? mapping.clockId.toInt() : null;
    final drives = mapping.hasDrivesOutputId() ? mapping.drivesOutputId.toInt() : null;
    String clockName(int c) =>
        clocks.where((x) => x.id.toInt() == c).map((x) => x.name).firstOrNull ?? '?';
    String outputName(int o) =>
        outputs.where((x) => x.id.toInt() == o).map((x) => x.name).firstOrNull ?? '?';

    // Reads whose value form is still open: the reason a definition cannot
    // be checked yet.  Read off the projection, not computed.
    final waitingOn = [
      for (final i in inputs)
        if (_concept(i) case final c? when socketKind(c) == SocketKind.open) c.name,
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Meaning',
          trailing: MappingGlyph(
            declared: declared,
            wrong: a?.status == pb.MappingStatus.MAPPING_STATUS_INVALID,
          ),
          children: [
            FormRow(
              label: 'Name',
              child: CommitTextField(
                value: mapping.name,
                onCommit: (v) => dispatch(RenameMappingRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: 'Meaning',
              child: CommitTextField(
                value: mapping.description,
                maxLines: 3,
                onCommit: (v) => dispatch(SetMappingDescriptionRequested(id: id, description: v)),
              ),
            ),
          ],
        ),
        InspectorSection(
          title: 'Reads',
          children: [
            Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                for (final c in inputs)
                  if (_concept(c) case final concept?)
                    ConceptChip(
                      concept: concept,
                      onRemove: () => dispatch(UnlinkMappingInput(mappingId: id, conceptId: c)),
                    ),
                MacDropdown<int>(
                  value: null,
                  hint: '+',
                  compact: true,
                  items: [
                    for (final c in concepts)
                      if (!inputs.contains(c.id.toInt())) c.id.toInt(),
                  ],
                  labelOf: _name,
                  leadingOf: (c) => SocketGlyph.of(_concept(c)!, t, size: 11),
                  onChanged: (c) =>
                      dispatch(LinkConceptToMappingInput(conceptId: c, mappingId: id)),
                ),
              ],
            ),
          ],
        ),
        InspectorSection(
          title: 'Produces',
          children: [
            MacDropdown<int>(
              value: output,
              items: [for (final c in concepts) c.id.toInt()],
              labelOf: _name,
              leadingOf: (c) => SocketGlyph.of(_concept(c)!, t, size: 11),
              onChanged: (c) => dispatch(LinkMappingOutputToConcept(mappingId: id, conceptId: c)),
            ),
          ],
        ),
        if (portWord != null || group != null)
          InspectorSection(
            title: 'Place',
            children: [
              if (portWord case final w?)
                Text(
                  'Backs the port "$w": what instances see of it is the promise, kept on the '
                  'component, not this definition.',
                  style: small,
                ),
              if (group case final g?)
                Row(
                  children: [
                    Expanded(child: Text('In group ${g.name}.', style: small)),
                    MacButton(
                      label: 'Show Group',
                      onPressed: () => dispatch(SelectionChanged(GroupSelected(g.id.toInt()))),
                    ),
                  ],
                ),
            ],
          ),
        InspectorSection(
          title: 'Relationship',
          // The one state word, only while there is nothing to show; an
          // unsaved draft is the editor's state, not the mapping's.
          trailing: boundTo != null
              ? Text('bound', style: small)
              : draft != null && draft!.dirtyAgainst(committed)
              ? Text('unsaved', style: small)
              : declared
              ? Text('declared', style: small)
              : null,
          children: [
            // A relationship realised by a binding has the system's
            // definition: a reference by identity, generated on every
            // commit.  It is shown and traced, never typed into.
            if (boundTo case final b?) ...[
              Text(
                'Takes its value from ${boundLabel?.call(b) ?? '?'}'
                '${b.hasTransportInit() ? ', carried across timing domains starting at ${b.transportInit}' : ''}.',
                style: TextStyle(fontSize: 12, color: t.textPrimary),
              ),
              const SizedBox(height: 6),
              Row(
                children: [
                  MacButton(
                    label: 'Show Binding',
                    onPressed: () => dispatch(SelectionChanged(BindingSelected(b.id.toInt()))),
                  ),
                  const SizedBox(width: 8),
                  MacButton(
                    label: 'Disconnect',
                    onPressed: () => dispatch(UnbindRequested(b.id.toInt())),
                  ),
                ],
              ),
              const SizedBox(height: 6),
              Text('Disconnect it to define the relationship yourself.', style: small),
            ] else
              // The editor shows formula-local findings under the text they
              // point into.  What remains here is about the mapping's place
              // in the design (causality, domains, outputs) — attached to the
              // relationship, in product language; the rule is in Explain.
              DefinitionEditor(
                mappingId: id,
                committed: committed,
                draft: draft,
                committedAnalysis: analysis,
                inputNames: inputs.map(_name).toList(),
                completion: completion,
                hover: hover,
                dispatch: dispatch,
              ),
            for (final d in broader)
              Padding(
                padding: const EdgeInsets.only(top: MacMetrics.gap),
                child: DiagnosticCard(diagnostic: d, source: committed ?? ''),
              ),
            if (a?.status == pb.MappingStatus.MAPPING_STATUS_OPEN && waitingOn.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: Text(
                  "Checked once ${waitingOn.join(' and ')}'s value is decided.",
                  style: small,
                ),
              ),
          ],
        ),
        InspectorSection(
          title: 'Timing',
          children: [
            FormRow(
              label: 'Updates in',
              child: MacDropdown<int>(
                value: clockId ?? -1,
                items: [-1, for (final c in clocks) c.id.toInt()],
                labelOf: (c) => c < 0 ? 'any domain' : clockName(c),
                detailOf: (c) => c < 0 ? 'pure' : '',
                onChanged: (c) =>
                    dispatch(SetMappingClockRequested(mappingId: id, clockId: c < 0 ? null : c)),
              ),
            ),
            Text(
              clockId == null
                  ? 'A relationship in no domain is pure: it is evaluated wherever it is read.'
                  : 'Evaluated at each activation of ${clockName(clockId)}; a value read from '
                        'another domain needs an explicit transport.',
              style: small,
            ),
            for (final d in timingIssues)
              Padding(
                padding: const EdgeInsets.only(top: MacMetrics.gap),
                child: DiagnosticCard(diagnostic: d, source: ''),
              ),
          ],
        ),
        InspectorSection(
          title: 'Drives',
          children: [
            FormRow(
              label: 'Output',
              child: MacDropdown<int>(
                value: drives ?? -1,
                items: [-1, for (final o in outputs) o.id.toInt()],
                labelOf: (o) => o < 0 ? 'nothing' : outputName(o),
                onChanged: (o) =>
                    dispatch(SetMappingDriveRequested(mappingId: id, outputId: o < 0 ? null : o)),
              ),
            ),
            Text(
              drives == null
                  ? inputs.isEmpty
                        ? 'This value can commit to a physical output.'
                        : 'Only a relationship without inputs can drive an output: connect the '
                              'one that combines the sources.'
                  : 'Each activation commits this value to ${outputName(drives)}. One driver per '
                        'output: a second one is a conflict, never a priority.',
              style: small,
            ),
            for (final d in driveIssues)
              Padding(
                padding: const EdgeInsets.only(top: MacMetrics.gap),
                child: DiagnosticCard(diagnostic: d, source: ''),
              ),
          ],
        ),
        FixList(actions: actions, dispatch: dispatch),
        Padding(
          padding: const EdgeInsets.all(12),
          child: Align(
            alignment: Alignment.centerLeft,
            child: DestructiveButton(
              label: 'Delete ${mapping.name}',
              onPressed: () => dispatch(DeleteMappingRequested(id)),
            ),
          ),
        ),
        MacDisclosure(
          title: 'Explain',
          children: [
            ExplainLine('DeclId ${mapping.id}'),
            if (clockId != null) ExplainLine('Κ = ClockId $clockId (${clockName(clockId)})'),
            if (drives != null) ExplainLine('β: drives OutputId $drives (${outputName(drives)})'),
            ExplainLine(
              'Interface: ${a?.interface.isNotEmpty == true ? a!.interface : '${inputs.map((i) => 'sem#$i').join(' → ')}${inputs.isEmpty ? '' : ' → '}sem#$output'}',
            ),
            if (a != null) ...[
              ExplainLine('status: ${statusWord(a.status)}'),
              if (a.inferredType.isNotEmpty) ExplainLine('⊢ ${a.inferredType}'),
              if (a.coreExpr.isNotEmpty) ExplainLine('core: ${a.coreExpr}'),
              for (final d in a.diagnostics)
                ExplainLine('${d.code}${d.technical.isEmpty ? '' : ': ${d.technical}'}'),
            ] else
              ExplainLine('status: ${declared ? 'declared' : 'pending analysis'}'),
            ExplainLine('revision $revision'),
            if (outcome case final o?) ExplainLine(_outcomeNotation(o)),
          ],
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Physical output
// ---------------------------------------------------------------------------

/// A physical output is the boundary to the world: what value it accepts,
/// when it updates, whether the design must drive it, and who drives it.
/// The output pass's verdict is shown as the sink's state; every claimant
/// is listed so a conflict is local and obvious, never arbitrated.
class _OutputInspector extends StatelessWidget {
  const _OutputInspector({
    super.key,
    required this.output,
    required this.state,
    required this.dispatch,
  });
  final pb.OutputView output;
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project!;
    final id = output.id.toInt();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final a = state.outputAnalysis(id);
    final clockId = output.hasClockId() ? output.clockId.toInt() : null;
    pb.ConceptView? concept(int c) => p.concepts.where((x) => x.id.toInt() == c).firstOrNull;
    String conceptName(int c) => concept(c)?.name ?? '?';
    String mappingName(int m) =>
        p.mappings.where((x) => x.id.toInt() == m).map((x) => x.name).firstOrNull ?? '?';
    final claimants = [
      for (final m in p.mappings)
        if (m.hasDrivesOutputId() && m.drivesOutputId.toInt() == id) m,
    ];
    final driver = a != null && a.hasDriver() ? a.driver.toInt() : null;
    // Faults of the drive edges are reported on the drivers; they belong
    // here too, where the sink is looked at.
    final driveIssues = [
      for (final m in claimants)
        ...?state
            .mappingAnalysis(m.id.toInt())
            ?.diagnostics
            .where((d) => d.code.startsWith('output.')),
    ];
    final (String stateText, Color stateColor) = clockId == null
        ? (
            'No timing domain yet: not part of the design\'s commitment until one is chosen.',
            t.open,
          )
        : switch (a?.state) {
            pb.OutputState.OUTPUT_STATE_DRIVEN => ('Driven by ${mappingName(driver!)}.', t.settled),
            pb.OutputState.OUTPUT_STATE_UNDRIVEN => (
              output.required
                  ? 'Undriven — the design is incomplete without a driver.'
                  : 'Undriven.',
              t.open,
            ),
            pb.OutputState.OUTPUT_STATE_CONFLICT => (
              '${output.name} already has a final target: ${claimants.map((m) => m.name).join(' and ')} both claim it.',
              t.error,
            ),
            pb.OutputState.OUTPUT_STATE_ILL_FORMED => (
              'The connection does not fit: see the driver\'s findings below.',
              t.error,
            ),
            _ => ('Checking…', t.textTertiary),
          };

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Output',
          trailing: Text(output.required ? 'required' : 'optional', style: small),
          children: [
            FormRow(
              label: 'Name',
              child: CommitTextField(
                value: output.name,
                onCommit: (v) => dispatch(RenameOutputRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: 'Accepts',
              child: MacDropdown<int>(
                value: output.accepts.toInt(),
                items: [for (final c in p.concepts) c.id.toInt()],
                labelOf: conceptName,
                leadingOf: (c) => SocketGlyph.of(concept(c)!, t, size: 11),
                onChanged: (c) => dispatch(SetOutputAcceptsRequested(id: id, accepts: c)),
              ),
            ),
            FormRow(
              label: 'Updates in',
              child: MacDropdown<int>(
                value: clockId ?? -1,
                items: [-1, for (final c in p.clocks) c.id.toInt()],
                labelOf: (c) => c < 0 ? 'no domain yet' : state.clockName(c) ?? '?',
                onChanged: (c) =>
                    dispatch(SetOutputClockRequested(id: id, clockId: c < 0 ? null : c)),
              ),
            ),
            FormRow(
              label: 'Required',
              child: Align(
                alignment: Alignment.centerLeft,
                child: Row(
                  spacing: MacMetrics.gap,
                  children: [
                    Checkbox(
                      value: output.required,
                      onChanged: (v) =>
                          dispatch(SetOutputRequiredRequested(id: id, required: v ?? false)),
                    ),
                    Expanded(
                      child: Text('the design is incomplete until this is driven', style: small),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
        InspectorSection(
          title: 'Driver',
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: MacMetrics.gap,
              children: [
                Padding(
                  padding: const EdgeInsets.only(top: 5),
                  child: Icon(Icons.circle, size: 7, color: stateColor),
                ),
                Expanded(
                  child: Text(
                    stateText,
                    key: const ValueKey('output-state'),
                    style: TextStyle(fontSize: 11, color: stateColor),
                  ),
                ),
              ],
            ),
            const SizedBox(height: MacMetrics.gap),
            for (final m in claimants)
              Row(
                children: [
                  Expanded(
                    child: MacLink(
                      label: m.name,
                      onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
                    ),
                  ),
                  MacLink(
                    label: 'disconnect',
                    onTap: () =>
                        dispatch(SetMappingDriveRequested(mappingId: m.id.toInt(), outputId: null)),
                  ),
                ],
              ),
            FormRow(
              label: 'Connect',
              child: MacDropdown<int>(
                value: null,
                hint: 'a relationship…',
                items: [
                  for (final m in p.mappings)
                    if (!claimants.contains(m)) m.id.toInt(),
                ],
                labelOf: mappingName,
                detailOf: (m) =>
                    p.mappings.firstWhere((x) => x.id.toInt() == m).signature.inputs.isEmpty
                    ? ''
                    : 'has inputs',
                onChanged: (m) => dispatch(SetMappingDriveRequested(mappingId: m, outputId: id)),
              ),
            ),
            for (final d in driveIssues)
              Padding(
                padding: const EdgeInsets.only(top: MacMetrics.gap),
                child: DiagnosticCard(diagnostic: d, source: ''),
              ),
          ],
        ),
        FixList(actions: state.editor.actions, dispatch: dispatch),
        Padding(
          padding: const EdgeInsets.all(12),
          child: Align(
            alignment: Alignment.centerLeft,
            child: DestructiveButton(
              label: 'Delete ${output.name}',
              enabled: claimants.isEmpty,
              tooltip: claimants.isEmpty
                  ? null
                  : 'Still driven by ${claimants.map((m) => m.name).join(', ')}',
              onPressed: () => dispatch(DeleteOutputRequested(id)),
            ),
          ),
        ),
        MacDisclosure(
          title: 'Explain',
          children: [
            ExplainLine('OutputId ${output.id}'),
            ExplainLine('Ω accepts = sem#${output.accepts}'),
            ExplainLine(clockId == null ? 'Ω clock = none (open)' : 'Ω clock = ClockId $clockId'),
            if (a != null) ExplainLine('state: ${a.state.name.toLowerCase()}'),
            for (final m in claimants) ExplainLine('β ${m.id} → ${output.id}'),
          ],
        ),
      ],
    );
  }
}
