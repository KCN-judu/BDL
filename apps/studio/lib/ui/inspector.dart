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
import 'mac/interactive.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
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
          child: Text('Select a concept or a mapping.', style: TextStyle(color: t.textTertiary)),
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
    required this.dispatch,
  });
  final pb.ConceptView concept;
  final List<pb.MappingView> readers;
  final List<pb.MappingView> producers;
  final int revision;

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
  const _ValueEditor({required this.current, required this.onChanged});
  final pb.Representation? current;
  final void Function(pb.Representation?) onChanged;

  @override
  Widget build(BuildContext context) {
    final kind = current?.whichKind() ?? pb.Representation_Kind.notSet;
    final dim = kind == pb.Representation_Kind.quantity ? current!.quantity : null;
    final preset = dim == null ? null : unitPresetFor(dim);
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
              items: unitPresets,
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
    required this.dispatch,
  });
  final pb.MappingView mapping;
  final List<pb.ConceptView> concepts;

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
    final broader = a?.diagnostics.where((d) => !d.hasSpan()).toList() ?? const [];

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
        InspectorSection(
          title: 'Relationship',
          // The one state word, only while there is nothing to show; an
          // unsaved draft is the editor's state, not the mapping's.
          trailing: draft != null && draft!.dirtyAgainst(committed)
              ? Text('unsaved', style: small)
              : declared
              ? Text('declared', style: small)
              : null,
          children: [
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
