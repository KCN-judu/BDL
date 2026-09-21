/// The inspector: what the selected object means, what can be changed, and
/// what a change will affect — in the designer's words (docs/architecture/studio-ui.md
/// §7, level 2).  Each control commits one `EditOp`.  Formal vocabulary
/// (ids, kernel types, invalidation categories, core terms) lives in one
/// collapsed *Explain* disclosure at the end (level 3).
library;

import 'package:flutter/material.dart';

import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/composer.dart' show composerProjection;
import '../app/simulation.dart' show kApplyRuleActionKind, kRuleUnappliedCode;
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
                ? context.l10n.selectAConceptARelationshipAnOutput
                : context.l10n.selectAConceptAMappingOrAn,
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
          presets: unitPresetsFrom(state.library?.quantities ?? const [], context.l10n),
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
          highlight:
              state.editor.highlights[HighlightState.formulaKey(id, state.editor.componentScope)],
          component: state.editor.componentScope,
          composer: state.editor.composer,
          projection: composerProjection(state, id),
          revision: project.revision.toInt(),
          outcome: state.editor.lastOutcome,
          definitionFocus: state.editor.definitionFocus,
          clocks: project.clocks,
          outputs: project.outputs,
          appliedBy: valuesApplying(state, id),
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
          dependsOn: [
            for (final d in state.refsOf(id))
              if (d != id) ?project.mappings.where((m) => m.id.toInt() == d).firstOrNull,
          ],
          namedIn: [
            for (final r in state.referrersOf(id))
              ?project.mappings.where((m) => m.id.toInt() == r).firstOrNull,
          ],
          // The Source's provision on the chosen board, when the Deploy
          // page has asked for one at this revision.
          provision: state.editor.deploy.analysis?.revision.toInt() == project.revision.toInt()
              ? state.editor.deploy.analysis?.provisions
                    .where((p) => p.sourceId.toInt() == id)
                    .firstOrNull
              : null,
          board: state.editor.deploy.target?.name,
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
        LinkSelected(:final link) => LinkInspector(
          key: ValueKey('link$link'),
          state: state,
          link: link,
          dispatch: dispatch,
        ),
        GroupSelected(:final id) => GroupInspector(
          key: ValueKey('group$id'),
          state: state,
          id: id,
          dispatch: dispatch,
        ),
        MultiSelected(:final nodes, :final active) => MultiInspector(
          key: ValueKey('multi${nodes.length}'),
          state: state,
          nodes: nodes,
          active: active,
          dispatch: dispatch,
        ),
      };
    }

    return Container(
      color: t.sidebar,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          PanelHeader(context.l10n.inspector),
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
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final Widget sentence;
    if (outcome.kind == pb.EditKind.EDIT_KIND_REFINEMENT) {
      sentence = Text(context.l10n.nothingElseNeedsRechecking, style: small);
    } else if (affected.isEmpty) {
      sentence = Text(context.l10n.thisWillBeCheckedAgain, style: small);
    } else {
      sentence = Wrap(
        crossAxisAlignment: WrapCrossAlignment.center,
        spacing: MacMetrics.gapTight,
        children: [
          Text(
            affected.length == 1 ? context.l10n.thisChangeAffects : context.l10n.thisChangeAffects,
            style: small,
          ),
          for (final m in affected)
            MacLink(
              label: m.name,
              onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
            ),
          Text(
            affected.length == 1
                ? context.l10n.itWillBeCheckedAgain
                : context.l10n.theyWillBeCheckedAgain,
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
          Text(context.l10n.lastChange, style: Theme.of(context).textTheme.titleSmall),
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

// Explain's formal vocabulary (kernel names): not localized by design
// (docs/project/localization-style.md).
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

/// The values whose definition applies relationship [id] — the compiler's
/// `applied_by` (its direct reverse dependency edges), restricted to the
/// values among them; never a name match on formula text.  In projection
/// order.
List<pb.MappingView> valuesApplying(AppState state, int id) {
  final appliers = state.referrersOf(id).toSet();
  return [
    for (final m in state.project?.mappings ?? const <pb.MappingView>[])
      if (appliers.contains(m.id.toInt()) && relationshipRole(m) == RelationshipRole.value) m,
  ];
}

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
        child: Text(
          empty,
          style: TextStyle(fontSize: MacType.body, color: t.textTertiary),
        ),
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
          title: context.l10n.meaning,
          trailing: SocketGlyph.of(concept, t),
          children: [
            FormRow(
              label: context.l10n.name,
              child: CommitTextField(
                value: concept.name,
                onCommit: (v) => dispatch(RenameConceptRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: context.l10n.meaning,
              child: CommitTextField(
                value: concept.description,
                maxLines: 3,
                onCommit: (v) => dispatch(SetConceptDescriptionRequested(id: id, description: v)),
              ),
            ),
          ],
        ),
        InspectorSection(
          title: context.l10n.value,
          children: [
            _ValueEditor(
              current: bound ? concept.representation : null,
              presets: presets,
              onChanged: (r) =>
                  dispatch(SetConceptRepresentationRequested(id: id, representation: r)),
            ),
            // Whether two values of this concept can be put in order: what
            // `<`, `min`, `max`, `clamp` and `inRange` between them need.
            // Never inferred from the value form — a Mode encoded as a
            // number is not a magnitude.  Only a quantity can carry it.
            if (bound && concept.representation.hasQuantity())
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: FormRow(
                  label: context.l10n.order,
                  child: Row(
                    spacing: MacMetrics.gap,
                    children: [
                      Checkbox(
                        value: concept.ordered,
                        onChanged: (v) =>
                            dispatch(SetConceptOrderedRequested(id: id, ordered: v ?? false)),
                      ),
                      Expanded(
                        child: Text(
                          concept.ordered
                              ? context.l10n.valuesAreMagnitudesSmallestLargestClampIn
                              : context.l10n.valuesAreComparedForEqualityOnly,
                          style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            // The consequence, only when there is one: rebinding a chosen
            // value form reopens every relationship typed against it.
            if (bound && users.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(
                  context.l10n.changingThisRechecks(_names(users)),
                  style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
                ),
              ),
          ],
        ),
        InspectorSection(
          title: context.l10n.relationships,
          children: [
            FormRow(
              label: context.l10n.producedBy,
              child: _NameLinks(
                mappings: producers,
                dispatch: dispatch,
                empty: context.l10n.nothingYet,
              ),
            ),
            FormRow(
              label: context.l10n.usedBy,
              child: _NameLinks(
                mappings: readers,
                dispatch: dispatch,
                empty: context.l10n.nothingYet,
              ),
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
                label: context.l10n.deleteNamed(concept.name),
                enabled: users.isEmpty,
                onPressed: () => dispatch(DeleteConceptRequested(id)),
              ),
              // A disabled control says why, at rest.
              if (users.isNotEmpty)
                Text(
                  context.l10n.stillUsedBy(_names(users)),
                  style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
                ),
            ],
          ),
        ),
        MacDisclosure(
          title: context.l10n.explain,
          children: [
            ExplainLine('ConceptId ${concept.id}'),
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
  pb.Representation_Kind.list => 'list ${_tyNotation(r.list)}',
  pb.Representation_Kind.pair => '(${_tyNotation(r.pair.first)} × ${_tyNotation(r.pair.second)})',
  pb.Representation_Kind.optional => 'opt ${_tyNotation(r.optional)}',
  pb.Representation_Kind.notSet => 'none',
};

/// A value form as the pop-up offers it.  The structured forms wrap a
/// further form, chosen in the row below.
enum _Form { quantity, onOff, count, collection, grouped, optional, later }

_Form _formOf(pb.Representation? r) => switch (r?.whichKind()) {
  pb.Representation_Kind.quantity => _Form.quantity,
  pb.Representation_Kind.boolean => _Form.onOff,
  pb.Representation_Kind.count => _Form.count,
  pb.Representation_Kind.list => _Form.collection,
  pb.Representation_Kind.pair => _Form.grouped,
  pb.Representation_Kind.optional => _Form.optional,
  _ => _Form.later,
};

/// The default value of a form when it is first chosen: a dimensionless
/// quantity, and structured forms holding one.
pb.Representation? _defaultOf(_Form f) => switch (f) {
  _Form.later => null,
  _Form.quantity => pb.Representation(quantity: pb.Dim()),
  _Form.onOff => pb.Representation(boolean: pb.Unit()),
  _Form.count => pb.Representation(count: pb.Unit()),
  _Form.collection => pb.Representation(list: pb.Representation(quantity: pb.Dim())),
  _Form.optional => pb.Representation(optional: pb.Representation(quantity: pb.Dim())),
  _Form.grouped => pb.Representation(
    pair: pb.PairRepresentation(
      first: pb.Representation(quantity: pb.Dim()),
      second: pb.Representation(quantity: pb.Dim()),
    ),
  ),
};

Map<_Form, String> _formWords(AppLocalizations l10n) => {
  _Form.quantity: l10n.quantity,
  _Form.onOff: l10n.onOff,
  _Form.count: l10n.count,
  _Form.collection: l10n.collectionOf,
  _Form.grouped: l10n.groupedValueTitle,
  _Form.optional: l10n.optional,
  _Form.later: l10n.decideLater,
};

/// The value form: a pop-up (seven choices are too many for a segmented
/// control), then what the form needs — a quantity's unit, a collection's
/// or optional value's element form, a grouped value's two parts.  Nested
/// forms are the same editor one level in, without "Decide later": a part
/// of a chosen form is chosen.
class _ValueEditor extends StatelessWidget {
  const _ValueEditor({
    required this.current,
    required this.presets,
    required this.onChanged,
    this.nested = false,
  });
  final pb.Representation? current;
  final List<UnitPreset> presets;
  final void Function(pb.Representation?) onChanged;
  final bool nested;

  @override
  Widget build(BuildContext context) {
    final form = _formOf(current);
    final forms = [
      for (final f in _Form.values)
        if (!nested || f != _Form.later) f,
    ];
    Widget part(String label, pb.Representation r, void Function(pb.Representation) set) => Padding(
      padding: const EdgeInsets.only(top: 8),
      child: FormRow(
        label: label,
        child: _ValueEditor(
          current: r,
          presets: presets,
          nested: true,
          onChanged: (v) => set(v ?? pb.Representation(quantity: pb.Dim())),
        ),
      ),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        MacDropdown<_Form>(
          value: form,
          items: forms,
          labelOf: (f) => _formWords(context.l10n)[f]!,
          onChanged: (f) {
            if (f != form) onChanged(_defaultOf(f));
          },
        ),
        if (form == _Form.quantity) ...[
          const SizedBox(height: 8),
          FormRow(
            label: context.l10n.unit,
            child: MacDropdown<UnitPreset>(
              value: unitPresetFor(presets, current!.quantity),
              // a dimension outside the presets is still shown, as its symbol
              hint: dimLabel(current!.quantity),
              items: presets,
              labelOf: (p) => p.name,
              detailOf: (p) => p.symbol,
              onChanged: (p) => onChanged(pb.Representation(quantity: p.dim)),
            ),
          ),
        ],
        if (form == _Form.collection)
          part(context.l10n.each, current!.list, (v) => onChanged(pb.Representation(list: v))),
        if (form == _Form.optional)
          part(
            context.l10n.whenPresent,
            current!.optional,
            (v) => onChanged(pb.Representation(optional: v)),
          ),
        if (form == _Form.grouped) ...[
          part(
            context.l10n.first,
            current!.pair.first,
            (v) => onChanged(
              pb.Representation(
                pair: pb.PairRepresentation(first: v, second: current!.pair.second),
              ),
            ),
          ),
          part(
            context.l10n.second,
            current!.pair.second,
            (v) => onChanged(
              pb.Representation(
                pair: pb.PairRepresentation(first: current!.pair.first, second: v),
              ),
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
    this.highlight,
    this.component,
    required this.revision,
    required this.outcome,
    this.definitionFocus = 0,
    required this.clocks,
    required this.outputs,
    this.appliedBy = const [],
    required this.actions,
    required this.dispatch,
    this.composer = const ComposerState(),
    this.projection,
    this.boundTo,
    this.boundLabel,
    this.portWord,
    this.group,
    this.dependsOn = const [],
    this.namedIn = const [],
    this.provision,
    this.board,
  });
  final pb.MappingView mapping;
  final List<pb.ConceptView> concepts;

  /// For a Source: its provision on the Deploy page's board at this
  /// revision, and that board's name — the deployment's projection,
  /// never a judgment made here.  Absent until a board is chosen.
  final pb.ProvisionView? provision;
  final String? board;

  /// The definition's semantic tokens, and the component whose body the
  /// relationship belongs to (the scope of a formula request).
  final HighlightState? highlight;
  final int? component;

  /// The relationships this one's definition references, and those whose
  /// definitions reference it — the analysis's `refs`, the canvas's
  /// reference edges (ADR-0034).  Empty while the analysis is pending.
  final List<pb.MappingView> dependsOn;
  final List<pb.MappingView> namedIn;
  final List<pb.ClockView> clocks;
  final List<pb.OutputView> outputs;

  /// The values whose definition applies this relationship (a rule): the
  /// ones that could drive an output in its place.
  final List<pb.MappingView> appliedBy;
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

  /// The Formula Composer's state and the projection it draws.
  final ComposerState composer;
  final pb.FormulaProjection? projection;
  final int revision;

  /// *Edit Definition* bumps this: the definition editor takes focus.
  final int definitionFocus;

  /// The last change, for its formal classification in Explain.
  final pb.EditOutcome? outcome;
  final void Function(AppAction) dispatch;

  pb.ConceptView? _concept(int id) => concepts.where((c) => c.id.toInt() == id).firstOrNull;
  String _name(int id) => _concept(id)?.name ?? '?';

  /// The Source's realization row: what the deployment says on the chosen
  /// board, or the environment when no board is chosen.
  String _sourceRealization(BuildContext context) {
    final l10n = context.l10n;
    final p = provision;
    final b = board;
    if (p == null || b == null) return l10n.realizationEnvironment;
    if (!p.hasDeviceId()) return l10n.realizationNoDeviceOn(b);
    if (!p.hasProfileId() || p.profileId.isEmpty) {
      return l10n.realizationProvidedBy(p.deviceName, b);
    }
    final profile = p.candidates.where((c) => c.id == p.profileId).firstOrNull;
    return l10n.realizationProvidedByAs(p.deviceName, profile?.displayName ?? p.profileId, b);
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = mapping.id.toInt();
    final inputs = mapping.signature.inputs.map((i) => i.toInt()).toList();
    final output = mapping.signature.output.toInt();
    final committed = mapping.hasDefinition() ? mapping.definition.formula : null;
    final a = analysis;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    // The role is the daemon's (ADR-0032, `MappingView.role`); a bound
    // base relationship arrives as a Value, a port-backed Source of an
    // open component wears its port's word.  *Declared* — the one hole a
    // designer fills — is a rule with no formula.
    final role = relationshipRole(mapping);
    final source = role == RelationshipRole.source && portWord == null;
    final declared = role == RelationshipRole.rule && !mapping.hasDefinition();
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
          title: context.l10n.meaning,
          trailing: MappingGlyph(
            declared: declared,
            wrong: a?.status == pb.MappingStatus.MAPPING_STATUS_INVALID,
            source: source,
          ),
          children: [
            FormRow(
              label: context.l10n.name,
              child: CommitTextField(
                value: mapping.name,
                onCommit: (v) => dispatch(RenameMappingRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: context.l10n.meaning,
              child: CommitTextField(
                value: mapping.description,
                maxLines: 3,
                onCommit: (v) => dispatch(SetMappingDescriptionRequested(id: id, description: v)),
              ),
            ),
            // The three shapes (ADR-0034): a Source reads nothing and has no
            // formula; a rule reads something; a value reads nothing and is
            // defined.  A port-backed relationship's role is its port's.
            FormRow(
              label: context.l10n.role,
              child: Text(
                portWord ??
                    switch (role) {
                      RelationshipRole.source => context.l10n.roleSource,
                      RelationshipRole.rule => context.l10n.roleRule,
                      RelationshipRole.value => context.l10n.roleValue,
                    },
                style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
              ),
            ),
            if (portWord == null)
              Text(switch (role) {
                RelationshipRole.source => context.l10n.sourceExplanation,
                RelationshipRole.rule => context.l10n.ruleExplanation,
                RelationshipRole.value => context.l10n.valueExplanation,
              }, style: small),
          ],
        ),
        InspectorSection(
          title: context.l10n.reads,
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
          // A Source *provides* its concept to the model; a relationship
          // *produces* it from what it reads.
          title: source ? context.l10n.provides : context.l10n.produces,
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
            title: context.l10n.place,
            children: [
              if (portWord case final w?) Text(context.l10n.backsThePort(w), style: small),
              if (group case final g?)
                Row(
                  children: [
                    Expanded(child: Text(context.l10n.inGroup(g.name), style: small)),
                    MacButton(
                      label: context.l10n.showGroup,
                      onPressed: () => dispatch(SelectionChanged(GroupSelected(g.id.toInt()))),
                    ),
                  ],
                ),
            ],
          ),
        InspectorSection(
          title: context.l10n.relationship,
          // The one state word, only while there is nothing to show; an
          // unsaved draft is the editor's state, not the mapping's.
          trailing: boundTo != null
              ? Text('bound', style: small)
              : draft != null && draft!.dirtyAgainst(committed)
              ? Text('unsaved', style: small)
              : source
              ? Text(context.l10n.roleSource, style: small)
              : declared
              ? Text('declared', style: small)
              : null,
          children: [
            // What realizes a Source is deployment's: the environment
            // provides it, and on a chosen board a device may — the Deploy
            // page's provision, projected; the graph is complete either way.
            if (source) ...[
              FormRow(
                label: context.l10n.realization,
                child: Text(_sourceRealization(context), style: small),
              ),
              const SizedBox(height: 6),
            ],
            // A relationship realised by a binding has the system's
            // definition: a reference by identity, generated on every
            // commit.  It is shown and traced, never typed into.
            if (boundTo case final b?) ...[
              Text(
                b.hasTransportInit()
                    ? context.l10n.takesItsValueFromTransported(
                        boundLabel?.call(b) ?? '?',
                        b.transportInit,
                      )
                    : context.l10n.takesItsValueFrom(boundLabel?.call(b) ?? '?'),
                style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
              ),
              const SizedBox(height: 6),
              Row(
                children: [
                  MacButton(
                    label: context.l10n.showBinding,
                    onPressed: () => dispatch(SelectionChanged(BindingSelected(b.id.toInt()))),
                  ),
                  const SizedBox(width: 8),
                  MacButton(
                    label: context.l10n.disconnect,
                    onPressed: () => dispatch(UnbindRequested(b.id.toInt())),
                  ),
                ],
              ),
              const SizedBox(height: 6),
              Text(context.l10n.disconnectItToDefineTheRelationshipYourself, style: small),
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
                highlight: highlight,
                component: component,
                composer: composer,
                projection: projection,
                concepts: {for (final c in concepts) c.id.toInt(): c},
                focusGeneration: definitionFocus,
                dispatch: dispatch,
              ),
            // What the formula references and who references this: the
            // reference edges of the canvas, as rows — the detail behind
            // the neutral link into the formula line.
            if (mapping.hasDefinition() && boundTo == null)
              FormRow(
                label: context.l10n.dependsOn,
                child: _NameLinks(
                  mappings: dependsOn,
                  dispatch: dispatch,
                  empty: context.l10n.nothingYet,
                ),
              ),
            if (!source && boundTo == null)
              FormRow(
                label: context.l10n.namedIn,
                child: _NameLinks(
                  mappings: namedIn,
                  dispatch: dispatch,
                  empty: context.l10n.nothingYet,
                ),
              ),
            for (final d in broader) ...[
              Padding(
                padding: const EdgeInsets.only(top: MacMetrics.gap),
                child: DiagnosticCard(diagnostic: d, source: committed ?? ''),
              ),
              // A rule nothing applies: its fix sits with the finding —
              // the value that would apply it — not in a list below.
              if (d.code == kRuleUnappliedCode)
                if (actions?.ofKind(kApplyRuleActionKind) case final fix?)
                  Padding(
                    padding: const EdgeInsets.only(left: 15, bottom: MacMetrics.gap),
                    child: FixItem(action: fix, dispatch: dispatch),
                  ),
            ],
            if (a?.status == pb.MappingStatus.MAPPING_STATUS_OPEN && waitingOn.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: Text(
                  context.l10n.checkedOnceValueDecided(waitingOn.join(' and ')),
                  style: small,
                ),
              ),
          ],
        ),
        InspectorSection(
          title: context.l10n.timing,
          children: [
            FormRow(
              label: context.l10n.updatesIn,
              child: MacDropdown<int>(
                value: clockId ?? -1,
                items: [-1, for (final c in clocks) c.id.toInt()],
                labelOf: (c) => c < 0 ? context.l10n.anyDomain : clockName(c),
                detailOf: (c) => c < 0 ? 'pure' : '',
                onChanged: (c) =>
                    dispatch(SetMappingClockRequested(mappingId: id, clockId: c < 0 ? null : c)),
              ),
            ),
            Text(
              clockId == null
                  ? context.l10n.aRelationshipInNoDomainIsPure
                  : context.l10n.evaluatedAtEachActivationOf(clockName(clockId)),
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
          title: context.l10n.drives,
          children: [
            // Only a Source or a value — a relationship that reads nothing
            // — can be *the* value an output commits at a tick (the output
            // pass's DriveWF).  A rule is not offered the pop-up: the
            // caption says what would make the connection possible, and
            // names the value when there is one.
            if (role != RelationshipRole.rule) ...[
              FormRow(
                label: context.l10n.output,
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
                    ? context.l10n.thisValueCanCommitToAPhysical
                    : context.l10n.eachActivationCommitsThisValueTo(outputName(drives)),
                style: small,
              ),
            ] else ...[
              Text(
                context.l10n.onlyARelationshipWithoutInputsCanDrive,
                key: const ValueKey('drives-caption'),
                style: small,
              ),
              if (appliedBy case [final v])
                Row(
                  children: [
                    Expanded(child: Text(context.l10n.appliedByValue(v.name), style: small)),
                    MacLink(
                      label: context.l10n.show,
                      onTap: () => dispatch(SelectionChanged(MappingSelected(v.id.toInt()))),
                    ),
                  ],
                ),
              // A text-authored design may record the edge anyway; the
              // model keeps it and the output pass reports it (card below).
              if (drives case final o?)
                Row(
                  children: [
                    Expanded(
                      child: Text(context.l10n.recordedAsDriving(outputName(o)), style: small),
                    ),
                    MacLink(
                      label: context.l10n.disconnect,
                      onTap: () =>
                          dispatch(SetMappingDriveRequested(mappingId: id, outputId: null)),
                    ),
                  ],
                ),
            ],
            for (final d in driveIssues)
              Padding(
                padding: const EdgeInsets.only(top: MacMetrics.gap),
                child: DiagnosticCard(diagnostic: d, source: ''),
              ),
          ],
        ),
        FixList(actions: actions, dispatch: dispatch, excludeKinds: const {kApplyRuleActionKind}),
        Padding(
          padding: const EdgeInsets.all(12),
          child: Align(
            alignment: Alignment.centerLeft,
            child: DestructiveButton(
              label: context.l10n.deleteNamed(mapping.name),
              onPressed: () => dispatch(DeleteMappingRequested(id)),
            ),
          ),
        ),
        MacDisclosure(
          title: context.l10n.explain,
          children: [
            ExplainLine('DeclId ${mapping.id}'),
            // the canonical type: the domain of no inputs is (), the empty
            // product — the canvas draws that domain as no socket
            ExplainLine(
              'type: ${switch (inputs.length) {
                0 => '()',
                1 => _concept(inputs.single)?.name ?? '?',
                _ => '(${inputs.map((i) => _concept(i)?.name ?? '?').join(', ')})',
              }} -> ${_concept(output)?.name ?? '?'}',
            ),
            if (inputs.isEmpty) ExplainLine(context.l10n.noExplicitInputsTheCanonicalDomainIs),
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
              // The kernel's rung: a declaration without a realization,
              // Source or not, is *declared* there.
              ExplainLine(
                'status: ${mapping.hasDefinition() ? context.l10n.pendingAnalysis : 'declared'}',
              ),
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
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
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
    // What can drive the sink is the output pass's rule (DriveWF), read off
    // the projection by `driveCandidates` — the same list the canvas offers
    // when a concept is dropped on the sink.
    final candidates = driveCandidates(p, output);
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
        ? (context.l10n.noTimingDomainYetNotPartOf, t.open)
        : switch (a?.state) {
            pb.OutputState.OUTPUT_STATE_DRIVEN => (
              context.l10n.drivenBy(mappingName(driver!)),
              t.settled,
            ),
            pb.OutputState.OUTPUT_STATE_UNDRIVEN => (
              output.required
                  ? context.l10n.undrivenTheDesignIsIncompleteWithoutA
                  : context.l10n.undriven,
              t.open,
            ),
            pb.OutputState.OUTPUT_STATE_CONFLICT => (
              context.l10n.contestedOutput(output.name, claimants.map((m) => m.name).join(' and ')),
              t.error,
            ),
            pb.OutputState.OUTPUT_STATE_ILL_FORMED => (
              context.l10n.theConnectionDoesNotFitSeeThe,
              t.error,
            ),
            _ => (context.l10n.checking, t.textTertiary),
          };

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: context.l10n.output,
          trailing: Text(output.required ? 'required' : 'optional', style: small),
          children: [
            FormRow(
              label: context.l10n.name,
              child: CommitTextField(
                value: output.name,
                onCommit: (v) => dispatch(RenameOutputRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: context.l10n.accepts,
              child: MacDropdown<int>(
                value: output.accepts.toInt(),
                items: [for (final c in p.concepts) c.id.toInt()],
                labelOf: conceptName,
                leadingOf: (c) => SocketGlyph.of(concept(c)!, t, size: 11),
                onChanged: (c) => dispatch(SetOutputAcceptsRequested(id: id, accepts: c)),
              ),
            ),
            FormRow(
              label: context.l10n.updatesIn,
              child: MacDropdown<int>(
                value: clockId ?? -1,
                items: [-1, for (final c in p.clocks) c.id.toInt()],
                labelOf: (c) => c < 0 ? context.l10n.noDomainYet : state.clockName(c) ?? '?',
                onChanged: (c) =>
                    dispatch(SetOutputClockRequested(id: id, clockId: c < 0 ? null : c)),
              ),
            ),
            FormRow(
              label: context.l10n.required,
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
                      child: Text(context.l10n.theDesignIsIncompleteUntilThisIs, style: small),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
        InspectorSection(
          title: context.l10n.driver,
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
                    style: TextStyle(fontSize: MacType.secondary, color: stateColor),
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
            // Candidates are what the canvas lets land on the sink: values
            // (relationships that read nothing) producing the accepted
            // concept.  A rule is never offered; the output pass stays the
            // verdict for a text-authored edge.
            if (candidates.isEmpty)
              Text(
                context.l10n.noValueOfConceptYet(conceptName(output.accepts.toInt())),
                key: const ValueKey('no-candidate'),
                style: small,
              )
            else
              FormRow(
                label: context.l10n.connect,
                child: MacDropdown<int>(
                  value: null,
                  hint: context.l10n.aValue,
                  items: [for (final m in candidates) m.id.toInt()],
                  labelOf: mappingName,
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
              label: context.l10n.deleteNamed(output.name),
              enabled: claimants.isEmpty,
              tooltip: claimants.isEmpty
                  ? null
                  : context.l10n.stillDrivenBy(claimants.map((m) => m.name).join(', ')),
              onPressed: () => dispatch(DeleteOutputRequested(id)),
            ),
          ),
        ),
        MacDisclosure(
          title: context.l10n.explain,
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
