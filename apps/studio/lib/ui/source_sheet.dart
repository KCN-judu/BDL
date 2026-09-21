/// The Source sheet: a Source — a value entering the behavior model from
/// the environment, `name : () -> C` with no definition (ADR-0032) — is
/// created over a concept the designer chooses here: an **existing**
/// concept of the design in view, by identity, or a **new** concept
/// created in the same transaction.  Nothing is committed until the
/// choice is complete; cancelling changes nothing; the `() -> ?` of an
/// unmade choice exists only on this sheet, never in the project
/// (docs/spec/concept-library.md).
///
/// A new concept is described as on the concept sheet (ADR-0041): a value
/// category of the Library — a value form or a named quantity — and the
/// name it has in this product; the units the category is measured in are
/// the compiler's, shown as a fact.  A served library's Source item is a
/// preset for this sheet: it prefills the names and the category, and the
/// daemon lists the existing concepts of that value form first
/// (`ListSourceCandidates`).  The preset decides no identity: the concept
/// is the designer's choice either way.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../l10n/l10n.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/concept_glyphs.dart';
import 'canvas/node_canvas.dart' show NodePreview;
import 'concept_sheet.dart' show MeasuredInRow, ValueCategoryField, looksLikeIdentifier;
import 'library_panel.dart' show itemName;
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'system_sheets.dart' show SheetScrim;
import 'units.dart';

/// Which concept the Source is created over.
enum SourceConceptChoice { existing, newConcept }

/// `RoomTemperature` → `roomTemperatureInput`, made free among [taken]
/// (`roomTemperatureInput2`, …).  A suggestion the designer may change;
/// what is typed is sent as typed.
String suggestedSourceName(String conceptName, Iterable<String> taken) {
  final trimmed = conceptName.trim();
  final base = trimmed.isEmpty
      ? 'input'
      : '${trimmed[0].toLowerCase()}${trimmed.substring(1)}Input';
  return freeName(base, taken);
}

/// [wanted] if no name in [taken] equals it, else `wanted2`, `wanted3`, …
String freeName(String wanted, Iterable<String> taken) {
  final names = taken.toSet();
  if (!names.contains(wanted)) return wanted;
  var i = 2;
  while (names.contains('$wanted$i')) {
    i++;
  }
  return '$wanted$i';
}

/// The Source sheet over the design page, while [SourceSheetState] is
/// open and its candidates have arrived.  Creating dispatches
/// [CreateSourceRequested]; cancelling dispatches [SourceSheetDismissed].
class SourceSheet extends StatelessWidget {
  const SourceSheet({super.key, required this.state, required this.sheet, required this.dispatch});
  final AppState state;
  final SourceSheetState sheet;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final item = sheet.presetId.isEmpty ? null : state.libraryItem(sheet.presetId);
    final project = state.project;
    // a block sheet: the concept is fixed, the block is named (ADR-0044)
    final fixed = sheet.conceptId == null
        ? null
        : project?.concepts.where((c) => c.id.toInt() == sheet.conceptId).firstOrNull;
    return SheetScrim(
      title: fixed != null
          ? context.l10n.newBlockOfConcept(fixed.name)
          : item == null
          ? context.l10n.newSource
          : itemName(context.l10n, item),
      subtitle: fixed != null
          ? context.l10n.aBlockOneValueOfConcept(fixed.name)
          : context.l10n.aSourceAValueTheEnvironmentProvides,
      width: 560,
      child: SourceSheetForm(
        key: ValueKey('source-sheet-${sheet.revision}-${sheet.presetId}-${sheet.conceptId}'),
        sheet: sheet,
        concepts: project?.concepts ?? const [],
        taken: [
          for (final c in project?.concepts ?? const <pb.ConceptView>[]) c.name,
          for (final m in project?.mappings ?? const <pb.MappingView>[]) m.name,
        ],
        unitPresets: unitPresetsFrom(state.library?.quantities ?? const [], context.l10n),
        categories: state.libraryItems.where((i) => i.hasConcept()).toList(),
        quantities: state.library?.quantities ?? const [],
        valueCategories: state.valueCategories,
        onCreate: dispatch,
        onCancel: () => dispatch(const SourceSheetDismissed()),
      ),
    );
  }
}

class SourceSheetForm extends StatefulWidget {
  const SourceSheetForm({
    super.key,
    required this.sheet,
    required this.concepts,
    required this.taken,
    required this.unitPresets,
    this.categories = const [],
    this.quantities = const [],
    this.valueCategories = const [],
    required this.onCreate,
    required this.onCancel,
  });

  final SourceSheetState sheet;

  /// The value categories a new concept is created from (the Library's
  /// Concept items), and the vocabulary their units are read off.
  final List<pb.LibraryItemView> categories;
  final List<pb.QuantityView> quantities;
  final List<pb.ValueCategoryView> valueCategories;

  /// The concepts of the design in view (the candidates name them by id).
  final List<pb.ConceptView> concepts;

  /// Every name of the design in view, for free suggestions.
  final List<String> taken;
  final List<UnitPreset> unitPresets;
  final void Function(CreateSourceRequested) onCreate;
  final VoidCallback onCancel;

  @override
  State<SourceSheetForm> createState() => SourceSheetFormState();
}

class SourceSheetFormState extends State<SourceSheetForm> {
  late final pb.SourceCandidatesResponse _cands = widget.sheet.candidates!;
  late final pb.SourcePresetView? _preset = _cands.hasPreset() ? _cands.preset : null;

  /// The daemon's ranked candidates, restricted to concepts the view shows.
  late final List<pb.ConceptView> _ranked = [
    for (final c in _cands.candidates)
      ?widget.concepts.where((x) => x.id == c.conceptId).firstOrNull,
  ];

  /// A block sheet: the concept is decided (ADR-0044) — no choice on the
  /// sheet, the name is the question.
  int? get _fixed => widget.sheet.conceptId;

  late SourceConceptChoice _choice = _fixed != null || _ranked.isNotEmpty
      ? SourceConceptChoice.existing
      : SourceConceptChoice.newConcept;
  late int? _existing = _fixed ?? _ranked.firstOrNull?.id.toInt();

  late final TextEditingController _conceptName = TextEditingController(
    text: _cands.suggestedConceptName,
  );
  final TextEditingController _conceptMeaning = TextEditingController();

  /// The value category of a new concept: the library item whose value
  /// form and unit the preset suggests, else *decide later*.
  late String? _category = _presetCategory();
  late final TextEditingController _sourceName = TextEditingController(text: _suggestedName());
  final TextEditingController _sourceMeaning = TextEditingController();

  /// Once the designer edits the Source name, no suggestion replaces it.
  bool _nameTouched = false;

  /// The category matching the preset's value form (the same
  /// representation), else the open one, else none.
  String? _presetCategory() {
    final p = _preset;
    final rep = p != null && p.hasRepresentation() ? p.representation : null;
    bool same(pb.LibraryItemView i) {
      final t = i.concept;
      if (rep == null) return !t.hasRepresentation();
      return t.hasRepresentation() && t.representation == rep;
    }

    return widget.categories.where(same).firstOrNull?.id ??
        widget.categories.where((i) => !i.concept.hasRepresentation()).firstOrNull?.id ??
        widget.categories.firstOrNull?.id;
  }

  String _conceptNameChosen() => switch (_choice) {
    SourceConceptChoice.existing =>
      widget.concepts.where((c) => c.id.toInt() == _existing).firstOrNull?.name ?? '',
    SourceConceptChoice.newConcept => _conceptName.text.trim(),
  };

  /// The suggested Source name for the current choice: the preset's (made
  /// free by the daemon) while the concept is the preset's own new one,
  /// else `<concept>Input` made free here.
  String _suggestedName() {
    final concept = _conceptNameChosen();
    // a block of a concept is named after it: `tilt`, `tilt2`
    if (_fixed != null) return blockNameFor(concept, widget.taken);
    if (_choice == SourceConceptChoice.newConcept &&
        _cands.suggestedSourceName.isNotEmpty &&
        concept == _cands.suggestedConceptName) {
      return _cands.suggestedSourceName;
    }
    return suggestedSourceName(concept, widget.taken);
  }

  void _resuggest() {
    if (!_nameTouched) _sourceName.text = _suggestedName();
  }

  pb.Representation? get _representation {
    final t = widget.categories.where((i) => i.id == _category).firstOrNull?.concept;
    return t != null && t.hasRepresentation() ? t.representation : null;
  }

  bool get _complete =>
      _sourceName.text.trim().isNotEmpty &&
      switch (_choice) {
        SourceConceptChoice.existing => _existing != null,
        SourceConceptChoice.newConcept =>
          _conceptName.text.trim().isNotEmpty && looksLikeIdentifier(_conceptName.text.trim()),
      };

  void _submit() {
    if (!_complete) return;
    widget.onCreate(switch (_choice) {
      SourceConceptChoice.existing => CreateSourceRequested(
        sourceName: _sourceName.text.trim(),
        description: _sourceMeaning.text.trim(),
        existingConcept: _existing,
      ),
      SourceConceptChoice.newConcept => CreateSourceRequested(
        sourceName: _sourceName.text.trim(),
        description: _sourceMeaning.text.trim(),
        newConceptName: _conceptName.text.trim(),
        newConceptDescription: _conceptMeaning.text.trim(),
        newConceptRepresentation: _representation,
      ),
    });
  }

  @override
  void dispose() {
    _conceptName.dispose();
    _conceptMeaning.dispose();
    _sourceName.dispose();
    _sourceMeaning.dispose();
    super.dispose();
  }

  /// The value form as a designer's word beside a concept in the pop-up:
  /// the quantity kind, *on / off*, *count*, or nothing while open.
  String _kindWord(AppLocalizations l10n, pb.Representation? r) => switch (r?.whichKind()) {
    pb.Representation_Kind.quantity =>
      widget.unitPresets.where((u) => u.dim == r!.quantity).firstOrNull?.name ?? l10n.quantity,
    pb.Representation_Kind.boolean => l10n.onOff,
    pb.Representation_Kind.count => l10n.count,
    _ => '',
  };

  /// The type the Code view will write for a value form (`Temperature`,
  /// `Bool`, `Count`; nothing while open) — never localized.
  String _typeName(pb.Representation? r) => switch (r?.whichKind()) {
    pb.Representation_Kind.quantity =>
      widget.categories
              .where((i) => i.id == _category)
              .map((i) => i.concept.typeName)
              .where((n) => n.isNotEmpty)
              .firstOrNull ??
          widget.unitPresets.where((u) => u.dim == r!.quantity).firstOrNull?.typeName ??
          'Scalar',
    pb.Representation_Kind.boolean => 'Bool',
    pb.Representation_Kind.count => 'Count',
    _ => '',
  };

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final mono = TextStyle(fontSize: MacType.code, fontFamily: 'Menlo', color: t.textPrimary);
    final sourceName = _sourceName.text.trim();
    // The preview is the objects that will be committed, nothing else: a
    // concept only on the new-concept path; the Source over the chosen
    // concept, or over a hollow, unnamed one while nothing is chosen.
    const unchosen = 1 << 40;
    final chosen = switch (_choice) {
      SourceConceptChoice.existing =>
        widget.concepts.where((c) => c.id.toInt() == _existing).firstOrNull,
      SourceConceptChoice.newConcept => pb.ConceptView(
        id: Int64(unchosen + 1),
        name: _conceptName.text.trim().isEmpty ? l10n.name : _conceptName.text.trim(),
        representation: _representation,
      ),
    };
    final previewConcept = chosen ?? pb.ConceptView(id: Int64(unchosen), name: '');
    final preview = pb.ProjectProjection(name: 'preview')
      ..concepts.addAll(widget.concepts)
      ..concepts.add(previewConcept)
      ..mappings.add(
        pb.MappingView(
          id: Int64(0),
          name: sourceName.isEmpty ? l10n.sourceName : sourceName,
          signature: pb.Signature(output: previewConcept.id),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED,
          role: pb.RelationshipRole.RELATIONSHIP_ROLE_SOURCE,
        ),
      );
    final conceptWord = chosen == null ? '?' : chosen.name;
    final lines = <String>[
      if (_choice == SourceConceptChoice.newConcept && chosen != null)
        'concept ${chosen.name}${_typeName(_representation).isEmpty ? '' : ' : ${_typeName(_representation)}'}',
      'mapping ${sourceName.isEmpty ? '…' : sourceName} : () -> $conceptWord',
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // The concept: the one decision the sheet exists for — or, on a
        // block sheet, the decided template the block is made from.
        if (_fixed != null && chosen != null)
          FormRow(
            label: l10n.conceptLabel,
            child: Row(
              spacing: 6,
              children: [
                SocketGlyph.of(chosen, t),
                Text(chosen.name, key: const ValueKey('block-concept')),
                Text(
                  _kindWord(l10n, chosen.hasRepresentation() ? chosen.representation : null),
                  style: small,
                ),
              ],
            ),
          )
        else
          FormRow(
            label: l10n.conceptLabel,
            child: MacSegmented<SourceConceptChoice>(
              value: _choice,
              options: {
                SourceConceptChoice.existing: l10n.existingConcept,
                SourceConceptChoice.newConcept: l10n.newConcept,
              },
              onChanged: (c) => setState(() {
                _choice = c;
                _resuggest();
              }),
            ),
          ),
        if (_fixed == null && _choice == SourceConceptChoice.existing)
          FormRow(
            label: '',
            child: _ranked.isEmpty
                ? Text(l10n.noConceptsYetCreateOne, style: small)
                : MacDropdown<int>(
                    key: const ValueKey('source-concept'),
                    value: _existing,
                    items: [for (final c in _ranked) c.id.toInt()],
                    labelOf: (id) => widget.concepts.firstWhere((c) => c.id.toInt() == id).name,
                    leadingOf: (id) =>
                        SocketGlyph.of(widget.concepts.firstWhere((c) => c.id.toInt() == id), t),
                    // the value form, so two concepts of one form are told
                    // apart by name alone — never merged
                    detailOf: (id) => _kindWord(
                      l10n,
                      widget.concepts.firstWhere((c) => c.id.toInt() == id).let((c) {
                        return c.hasRepresentation() ? c.representation : null;
                      }),
                    ),
                    onChanged: (id) => setState(() {
                      _existing = id;
                      _resuggest();
                    }),
                  ),
          )
        else ...[
          FormRow(
            label: l10n.name,
            child: MacTextField(
              key: const ValueKey('source-concept-name'),
              controller: _conceptName,
              autofocus: true,
              onChanged: (_) => setState(_resuggest),
              onSubmitted: (_) => _submit(),
            ),
          ),
          FormRow(
            label: l10n.valueCategory,
            child: ValueCategoryField(
              dropdownKey: const ValueKey('source-concept-category'),
              items: widget.categories,
              quantities: widget.quantities,
              value: _category,
              onChanged: (id) => setState(() => _category = id),
            ),
          ),
          MeasuredInRow(categories: widget.valueCategories, representation: _representation),
          FormRow(
            label: l10n.meaning,
            child: MacTextField(controller: _conceptMeaning, onSubmitted: (_) => _submit()),
          ),
        ],
        const SizedBox(height: 10),
        Divider(height: 1, color: t.hairline),
        const SizedBox(height: 10),
        FormRow(
          label: _fixed != null ? l10n.blockName : l10n.sourceName,
          child: MacTextField(
            key: const ValueKey('source-name'),
            controller: _sourceName,
            autofocus: _choice == SourceConceptChoice.existing,
            onChanged: (_) => setState(() => _nameTouched = true),
            onSubmitted: (_) => _submit(),
          ),
        ),
        FormRow(
          label: l10n.meaning,
          child: MacTextField(controller: _sourceMeaning, onSubmitted: (_) => _submit()),
        ),
        const SizedBox(height: 6),
        Container(
          decoration: BoxDecoration(
            color: t.canvas,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: t.hairline),
          ),
          child: NodePreview(
            projection: preview,
            node: const NodeRef.mapping(0),
            height: 72,
            neutralConcepts: {unchosen, unchosen + 1},
          ),
        ),
        const SizedBox(height: 6),
        // What will be committed, as the Code view will show it.
        for (final line in lines)
          Text(line, key: ValueKey('source-preview-${lines.indexOf(line)}'), style: mono),
        const SizedBox(height: 4),
        Text(
          _fixed != null
              ? l10n.blockOfConceptCaption
              : switch (_choice) {
                  SourceConceptChoice.existing => l10n.sourceOverExistingConceptCaption,
                  SourceConceptChoice.newConcept => l10n.sourceWithNewConceptCaption,
                },
          style: small,
        ),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: l10n.cancel, onPressed: widget.onCancel),
            const SizedBox(width: 8),
            MacButton.primary(
              key: const ValueKey('source-create'),
              label: _fixed != null ? l10n.createBlock : l10n.createSource,
              onPressed: _complete ? _submit : null,
            ),
          ],
        ),
      ],
    );
  }
}

extension<T> on T {
  R let<R>(R Function(T) f) => f(this);
}
