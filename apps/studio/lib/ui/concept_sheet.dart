/// The concept sheet (ADR-0041): a concept is created from a **value
/// category** — a Standard Library item: a value form (on / off, count, a
/// level, decide later) or a named physical quantity — and the **name**
/// it has in this product, given before anything is created.  *Angle* →
/// `LidAngle`; *Temperature* → `MotorTemperature`.  The Library provides
/// the category; the project provides the identity and its name.
///
/// What the sheet shows, ranked: the name (required; the one thing the
/// Library cannot know); the category, changeable in one pop-up; the
/// units the category is measured in — the compiler's registry for the
/// dimension, canonical first (`QuantityView.units`), a fact about what a
/// formula may write, never a choice: a concept stores its dimension and
/// no unit; the meaning; a live preview of the node and the declaration
/// the Code view will write.  Create dispatches one ordinary
/// [CreateConceptRequested]; Cancel dispatches [ConceptSheetDismissed] and
/// nothing is committed.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../l10n/l10n.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/node_canvas.dart' show NodePreview;
import 'library_panel.dart' show itemName, representationWord;
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'system_sheets.dart' show SheetScrim;

/// Whether [name] can be an identifier of the language: letters, digits
/// and `_`, not starting with a digit.  A hint before the daemon's own
/// refusal (`edit.invalid_name`), never a rule of its own.
bool looksLikeIdentifier(String name) => RegExp(r'^[A-Za-z_][A-Za-z0-9_]*$').hasMatch(name);

/// The compiler's units for a category, as their mathematical renderings
/// (`UnitExprView.display`: `rad/s`, `m/s²`), preferred first: the
/// registered atoms of the dimension, then the curated composites.  Empty
/// for a value form that has none, for *no unit*, and until the daemon
/// answered.
List<String> unitSymbolsFor(Iterable<pb.ValueCategoryView> categories, pb.Representation? r) {
  if (r == null || !r.hasQuantity()) return const [];
  final c = categories.where((c) => c.hasDim() && c.dim == r.quantity).firstOrNull;
  if (c == null) return const [];
  final preferred = c.hasPreferredUnit() ? c.preferredUnit.display : '';
  final rest = [
    for (final u in c.units)
      if (u.display != preferred) u.display,
  ];
  return [if (preferred.isNotEmpty) preferred, ...rest];
}

/// The display rendering of a category's preferred unit (`rad/s`), for a
/// dimension no unit of which is listed.
String displayUnitFor(Iterable<pb.ValueCategoryView> categories, pb.Representation? r) {
  if (r == null || !r.hasQuantity()) return '';
  final c = categories.where((c) => c.hasDim() && c.dim == r.quantity).firstOrNull;
  return c != null && c.hasPreferredUnit() ? c.preferredUnit.display : '';
}

/// The value category pop-up: every Concept item of the served libraries —
/// the value forms, then the quantities — with its unit symbol beside it.
/// Shared by the concept sheet and the Source sheet's new-concept path so
/// a category is chosen the same way everywhere.
class ValueCategoryField extends StatelessWidget {
  const ValueCategoryField({
    super.key,
    required this.items,
    required this.value,
    required this.onChanged,
    this.quantities = const [],
    this.dropdownKey,
  });

  /// The Concept items (`hasConcept()`), in library order, and the
  /// vocabulary a composite dimension's display symbol is read off.
  final List<pb.LibraryItemView> items;
  final Iterable<pb.QuantityView> quantities;
  final String? value;
  final void Function(String itemId) onChanged;
  final Key? dropdownKey;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final byId = {for (final i in items) i.id: i};
    return MacDropdown<String>(
      key: dropdownKey,
      value: value != null && byId.containsKey(value) ? value : null,
      hint: l10n.chooseACategory,
      items: [for (final i in items) i.id],
      labelOf: (id) => itemName(l10n, byId[id]!),
      // the unit column: the category's symbol; nothing for a form whose
      // name already says it (on / off, decide later)
      detailOf: (id) {
        final t = byId[id]!.concept;
        return t.hasRepresentation() && t.representation.hasQuantity()
            ? representationWord(l10n, t, quantities)
            : '';
      },
      onChanged: onChanged,
    );
  }
}

/// The row under the category: *Measured in* and the compiler's units as
/// cells (whitespace between facts, never a separator).  A fact about what
/// a formula may write, never a choice: a concept stores its dimension.
class MeasuredInRow extends StatelessWidget {
  const MeasuredInRow({super.key, required this.categories, required this.representation});
  final Iterable<pb.ValueCategoryView> categories;
  final pb.Representation? representation;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final r = representation;
    if (r == null || !r.hasQuantity()) return const SizedBox.shrink();
    final symbols = unitSymbolsFor(categories, r);
    final display = displayUnitFor(categories, r);
    final style = TextStyle(
      fontSize: MacType.secondary,
      color: t.textSecondary,
      fontFeatures: const [FontFeature.tabularFigures()],
    );
    return FormRow(
      label: l10n.measuredIn,
      child: Padding(
        padding: const EdgeInsets.only(top: 4),
        child: symbols.isEmpty
            ? Text(
                display.isEmpty ? l10n.noUnit : display,
                key: const ValueKey('concept-units'),
                style: style,
              )
            : Wrap(
                key: const ValueKey('concept-units'),
                spacing: MacMetrics.gapGroup,
                children: [for (final s in symbols) Text(s, style: style)],
              ),
      ),
    );
  }
}

/// The concept sheet over the design page while [ConceptSheetState] is
/// open.
class ConceptSheet extends StatelessWidget {
  const ConceptSheet({super.key, required this.state, required this.sheet, required this.dispatch});
  final AppState state;
  final ConceptSheetState sheet;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final project = state.project;
    return SheetScrim(
      title: context.l10n.newConcept,
      subtitle: context.l10n.conceptSheetSubtitle,
      width: 520,
      child: ConceptSheetForm(
        key: ValueKey('concept-sheet-${sheet.presetId}'),
        sheet: sheet,
        items: state.libraryItems.where((i) => i.hasConcept()).toList(),
        quantities: state.library?.quantities ?? const [],
        categories: state.valueCategories,
        taken: [
          for (final c in project?.concepts ?? const <pb.ConceptView>[]) c.name,
          for (final m in project?.mappings ?? const <pb.MappingView>[]) m.name,
        ],
        onCreate: dispatch,
        onCancel: () => dispatch(const ConceptSheetDismissed()),
      ),
    );
  }
}

class ConceptSheetForm extends StatefulWidget {
  const ConceptSheetForm({
    super.key,
    required this.sheet,
    required this.items,
    required this.quantities,
    this.categories = const [],
    required this.taken,
    required this.onCreate,
    required this.onCancel,
  });
  final ConceptSheetState sheet;
  final List<pb.LibraryItemView> items;

  /// The vocabulary (a composite dimension's display symbol) and the
  /// compiler's categories (the units each is measured in).
  final List<pb.QuantityView> quantities;
  final List<pb.ValueCategoryView> categories;

  /// Every name of the design in view: a taken name is said before the
  /// daemon refuses it.
  final List<String> taken;
  final void Function(CreateConceptRequested) onCreate;
  final VoidCallback onCancel;

  @override
  State<ConceptSheetForm> createState() => ConceptSheetFormState();
}

class ConceptSheetFormState extends State<ConceptSheetForm> {
  final TextEditingController _name = TextEditingController();
  final TextEditingController _meaning = TextEditingController();
  late String? _category = widget.items.any((i) => i.id == widget.sheet.presetId)
      ? widget.sheet.presetId
      : null;

  pb.LibraryItemView? get _item => widget.items.where((i) => i.id == _category).firstOrNull;

  pb.Representation? get _representation {
    final t = _item?.concept;
    return t != null && t.hasRepresentation() ? t.representation : null;
  }

  String get _typeName => _item?.concept.typeName ?? '';

  /// Why the name cannot be used yet, in the sheet's words; `null` when
  /// it can.  An empty field says nothing until Create is tried.
  String? _nameProblem(AppLocalizations l10n) {
    final name = _name.text.trim();
    if (name.isEmpty) return null;
    if (!looksLikeIdentifier(name)) return l10n.nameNotIdentifier;
    if (widget.taken.contains(name)) return l10n.nameTaken(name);
    return null;
  }

  bool get _complete =>
      _name.text.trim().isNotEmpty &&
      looksLikeIdentifier(_name.text.trim()) &&
      !widget.taken.contains(_name.text.trim()) &&
      _category != null;

  void _submit() {
    if (!_complete) return;
    widget.onCreate(
      CreateConceptRequested(
        name: _name.text.trim(),
        description: _meaning.text.trim(),
        representation: _representation,
        position: widget.sheet.position,
        presetId: _category ?? '',
      ),
    );
  }

  @override
  void dispose() {
    _name.dispose();
    _meaning.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final mono = TextStyle(fontSize: MacType.code, fontFamily: 'Menlo', color: t.textPrimary);
    final name = _name.text.trim();
    final problem = _nameProblem(l10n);
    // the preview: the node the entries become, at canvas fidelity
    const previewId = 1 << 40;
    final preview = pb.ProjectProjection(name: 'preview')
      ..concepts.add(
        pb.ConceptView(
          id: Int64(previewId),
          name: name.isEmpty ? l10n.name : name,
          representation: _representation,
        ),
      );
    final declaration =
        'concept ${name.isEmpty ? '…' : name}${_typeName.isEmpty ? '' : ' : $_typeName'}';
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormRow(
          label: l10n.name,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              MacTextField(
                key: const ValueKey('concept-name'),
                controller: _name,
                autofocus: true,
                onChanged: (_) => setState(() {}),
                onSubmitted: (_) => _submit(),
              ),
              if (problem != null)
                Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Text(
                    problem,
                    key: const ValueKey('concept-name-problem'),
                    style: TextStyle(fontSize: MacType.secondary, color: t.error),
                  ),
                ),
            ],
          ),
        ),
        FormRow(
          label: l10n.valueCategory,
          child: ValueCategoryField(
            dropdownKey: const ValueKey('concept-category'),
            items: widget.items,
            quantities: widget.quantities,
            value: _category,
            onChanged: (id) => setState(() => _category = id),
          ),
        ),
        MeasuredInRow(categories: widget.categories, representation: _representation),
        FormRow(
          label: l10n.meaning,
          child: MacTextField(
            key: const ValueKey('concept-meaning'),
            controller: _meaning,
            onSubmitted: (_) => _submit(),
          ),
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
            node: const NodeRef.concept(previewId),
            height: 56,
            neutralConcepts: const {previewId},
          ),
        ),
        const SizedBox(height: 6),
        Text(declaration, key: const ValueKey('concept-preview'), style: mono),
        const SizedBox(height: 4),
        Text(l10n.conceptSheetCaption, style: small),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: l10n.cancel, onPressed: widget.onCancel),
            const SizedBox(width: 8),
            MacButton.primary(
              key: const ValueKey('concept-create'),
              label: l10n.createConcept,
              onPressed: _complete ? _submit : null,
            ),
          ],
        ),
      ],
    );
  }
}
