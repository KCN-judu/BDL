/// The inspector: every editable fact about the selected object, as a
/// macOS-style form.  Each control commits one `EditOp`; the classification
/// the compiler returns (refinement / edit) is shown underneath.
library;

import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/canvas_geometry.dart' show dimLabel;
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';

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
          usedBy: project.mappings
              .where(
                (m) =>
                    m.signature.inputs.any((i) => i.toInt() == id) ||
                    m.signature.output.toInt() == id,
              )
              .toList(),
          dispatch: dispatch,
        ),
        MappingSelected(:final id) => _MappingInspector(
          key: ValueKey('m$id'),
          mapping: project.mappings.firstWhere((m) => m.id.toInt() == id),
          concepts: project.concepts,
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
          if (state.editor.lastOutcome case final o?) _OutcomeNote(outcome: o),
        ],
      ),
    );
  }
}

class _OutcomeNote extends StatelessWidget {
  const _OutcomeNote({required this.outcome});
  final pb.EditOutcome outcome;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final (label, color) = switch (outcome.kind) {
      pb.EditKind.EDIT_KIND_REFINEMENT => (
        'refinement — nothing established elsewhere is reopened',
        t.settled,
      ),
      pb.EditKind.EDIT_KIND_EDIT => (
        'edit — reopens: ${outcome.invalidates.map(_invalidationWord).join(', ')}',
        t.open,
      ),
      _ => ('', t.textTertiary),
    };
    if (label.isEmpty) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.fromLTRB(12, 6, 12, 8),
      decoration: BoxDecoration(
        border: Border(top: BorderSide(color: t.hairline)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Icon(Icons.circle, size: 7, color: color),
          ),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              'last change: $label',
              style: TextStyle(fontSize: 11, color: t.textSecondary),
            ),
          ),
        ],
      ),
    );
  }
}

String _invalidationWord(pb.Invalidation i) => switch (i) {
  pb.Invalidation.INVALIDATION_INTERFACE => 'typing of dependents',
  pb.Invalidation.INVALIDATION_REALIZATION => 'this definition',
  pb.Invalidation.INVALIDATION_SEMANTIC => 'semantic checks',
  pb.Invalidation.INVALIDATION_REACTIVE => 'simulation',
  pb.Invalidation.INVALIDATION_CLOCK => 'clock domains',
  pb.Invalidation.INVALIDATION_OUTPUT => 'outputs',
  pb.Invalidation.INVALIDATION_DEPLOYMENT => 'deployment',
  _ => 'unknown',
};

// ---------------------------------------------------------------------------
// Concept
// ---------------------------------------------------------------------------

class _ConceptInspector extends StatelessWidget {
  const _ConceptInspector({
    super.key,
    required this.concept,
    required this.usedBy,
    required this.dispatch,
  });
  final pb.ConceptView concept;
  final List<pb.MappingView> usedBy;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = concept.id.toInt();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Concept',
          trailing: Text(
            'identity ${concept.id}',
            style: TextStyle(fontSize: 10, color: t.textTertiary),
          ),
          children: [
            FormRow(
              label: 'Name',
              child: CommitTextField(
                value: concept.name,
                onCommit: (v) => dispatch(RenameConceptRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: 'Description',
              child: CommitTextField(
                value: concept.description,
                hint: 'what this means for the product',
                maxLines: 3,
                onCommit: (v) => dispatch(SetConceptDescriptionRequested(id: id, description: v)),
              ),
            ),
          ],
        ),
        InspectorSection(
          title: 'Representation',
          children: [
            _RepresentationEditor(
              current: concept.hasRepresentation() ? concept.representation : null,
              onChanged: (r) =>
                  dispatch(SetConceptRepresentationRequested(id: id, representation: r)),
            ),
            if (concept.hasRepresentation())
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: Text(
                  'Changing a chosen representation is an edit: every mapping typed against '
                  'it is re-checked.',
                  style: TextStyle(fontSize: 11, color: t.textSecondary),
                ),
              )
            else
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: Text(
                  'Open. Mappings may already use this concept; the representation can be '
                  'chosen later.',
                  style: TextStyle(fontSize: 11, color: t.textSecondary),
                ),
              ),
          ],
        ),
        InspectorSection(
          title: 'Used by',
          children: [
            if (usedBy.isEmpty)
              Text('no mapping yet', style: TextStyle(color: t.textTertiary))
            else
              for (final m in usedBy)
                Text(
                  '${m.name}  ·  ${m.signature.output.toInt() == id ? 'produces' : 'reads'}',
                  style: TextStyle(color: t.textPrimary),
                ),
          ],
        ),
        Padding(
          padding: const EdgeInsets.all(12),
          child: Align(
            alignment: Alignment.centerLeft,
            child: DestructiveButton(
              label: 'Delete ${concept.name}',
              enabled: usedBy.isEmpty,
              tooltip: usedBy.isEmpty
                  ? null
                  : 'Still used by ${usedBy.map((m) => m.name).join(', ')}',
              onPressed: () => dispatch(DeleteConceptRequested(id)),
            ),
          ),
        ),
      ],
    );
  }
}

/// Representation chooser: none / quantity (+ dimension preset) / boolean / count.
class _RepresentationEditor extends StatelessWidget {
  const _RepresentationEditor({required this.current, required this.onChanged});
  final pb.Representation? current;
  final void Function(pb.Representation?) onChanged;

  static final _dims = <String, pb.Dim>{
    'dimensionless': pb.Dim(),
    'angle (rad)': pb.Dim(angle: 1),
    'length (m)': pb.Dim(length: 1),
    'time (s)': pb.Dim(time: 1),
    'temperature (K)': pb.Dim(temperature: 1),
    'mass (kg)': pb.Dim(mass: 1),
    'current (A)': pb.Dim(current: 1),
    'angular rate (rad/s)': pb.Dim(angle: 1, time: -1),
    'speed (m/s)': pb.Dim(length: 1, time: -1),
  };

  @override
  Widget build(BuildContext context) {
    final kind = current?.whichKind() ?? pb.Representation_Kind.notSet;
    final dim = kind == pb.Representation_Kind.quantity ? current!.quantity : null;
    final preset = dim == null
        ? null
        : _dims.entries.where((e) => e.value == dim).map((e) => e.key).firstOrNull ?? dimLabel(dim);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        MacSegmented<pb.Representation_Kind>(
          value: kind,
          options: const {
            pb.Representation_Kind.notSet: 'none',
            pb.Representation_Kind.quantity: 'quantity',
            pb.Representation_Kind.boolean: 'boolean',
            pb.Representation_Kind.count: 'count',
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
            label: 'Dimension',
            child: MacDropdown<String>(
              value: _dims.containsKey(preset) ? preset : null,
              hint: preset ?? 'choose',
              items: _dims.keys.toList(),
              onChanged: (k) => onChanged(pb.Representation(quantity: _dims[k]!)),
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

class _MappingInspector extends StatelessWidget {
  const _MappingInspector({
    super.key,
    required this.mapping,
    required this.concepts,
    required this.dispatch,
  });
  final pb.MappingView mapping;
  final List<pb.ConceptView> concepts;
  final void Function(AppAction) dispatch;

  String _name(int id) =>
      concepts.where((c) => c.id.toInt() == id).map((c) => c.name).firstOrNull ?? '?';

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = mapping.id.toInt();
    final inputs = mapping.signature.inputs.map((i) => i.toInt()).toList();
    final output = mapping.signature.output.toInt();
    final unresolved = !mapping.hasDefinition();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Mapping',
          trailing: Text(
            'identity ${mapping.id}',
            style: TextStyle(fontSize: 10, color: t.textTertiary),
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
              label: 'Description',
              child: CommitTextField(
                value: mapping.description,
                hint: 'the relationship, in product words',
                maxLines: 3,
                onCommit: (v) => dispatch(SetMappingDescriptionRequested(id: id, description: v)),
              ),
            ),
            FormRow(
              label: 'State',
              child: StatePill(
                word: switch (mapping.state) {
                  pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED => 'declared',
                  pb.AcceptanceState.ACCEPTANCE_STATE_DEFINED => 'defined',
                  _ => mapping.state.name.toLowerCase(),
                },
                settled: !unresolved,
              ),
            ),
          ],
        ),
        InspectorSection(
          title: 'Signature',
          children: [
            FormRow(
              label: 'Inputs',
              child: Wrap(
                spacing: 4,
                runSpacing: 4,
                children: [
                  for (final c in inputs)
                    ConceptChip(
                      label: _name(c),
                      color: t.conceptColor(c),
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
                    onChanged: (c) =>
                        dispatch(LinkConceptToMappingInput(conceptId: c, mappingId: id)),
                  ),
                ],
              ),
            ),
            FormRow(
              label: 'Output',
              child: MacDropdown<int>(
                value: output,
                items: [for (final c in concepts) c.id.toInt()],
                labelOf: _name,
                onChanged: (c) => dispatch(LinkMappingOutputToConcept(mappingId: id, conceptId: c)),
              ),
            ),
            Padding(
              padding: const EdgeInsets.only(top: 6),
              child: Text(
                'Changing the signature is an edit: the expected type of this relationship '
                'changes and everything depending on it is re-checked.',
                style: TextStyle(fontSize: 11, color: t.textSecondary),
              ),
            ),
          ],
        ),
        InspectorSection(
          title: 'Definition',
          children: [
            if (unresolved) ...[
              Text(
                'No definition yet. This is a legal state: other relationships may already '
                'depend on the signature.',
                style: TextStyle(fontSize: 11, color: t.textSecondary),
              ),
              const SizedBox(height: 8),
              CommitTextField(
                value: '',
                hint:
                    'formula, e.g. clamp(0.2 + 0.8 * ${inputs.isEmpty ? 'x' : _name(inputs.first)} / 60deg, 0, 1)',
                maxLines: 3,
                commitLabel: 'Attach',
                onCommit: (v) {
                  if (v.trim().isNotEmpty) {
                    dispatch(AttachFormulaRequested(mappingId: id, source: v.trim()));
                  }
                },
              ),
            ] else ...[
              CommitTextField(
                value: mapping.definition.formula,
                maxLines: 4,
                monospace: true,
                onCommit: (v) => dispatch(ReplaceDefinitionRequested(mappingId: id, source: v)),
              ),
              const SizedBox(height: 6),
              Row(
                children: [
                  MacButton(
                    label: 'Detach definition',
                    onPressed: () =>
                        dispatch(ReplaceDefinitionRequested(mappingId: id, source: null)),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Replacing or detaching is an edit.',
                      style: TextStyle(fontSize: 11, color: t.textSecondary),
                    ),
                  ),
                ],
              ),
            ],
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
      ],
    );
  }
}
