/// Sheets for creating objects.
///
/// The form is the explanation: what you choose is shown, live, as the node
/// it will become on the canvas (a hollow socket for a concept whose kind is
/// still open, the dashed outline of an undefined mapping, the inputs as
/// sockets).  No example text stands in for meaning.
///
/// Project open/new go through the OS's own pickers (`file_selector`), see
/// `effects/effect_executor.dart`.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';

import '../l10n/l10n.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/concept_glyphs.dart';
import 'canvas/node_canvas.dart' show NodePreview;
import 'mac/controls.dart';
import 'mac/interactive.dart';
import 'mac/theme.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'units.dart';

// ---------------------------------------------------------------------------
// Sheet chrome
// ---------------------------------------------------------------------------

Future<T?> showMacSheet<T>(
  BuildContext context, {
  required String title,
  String? subtitle,
  required Widget content,
  required List<Widget> actions,
  double width = 480,

  /// When given, the title and the actions are built inside the sheet
  /// (per frame) instead of taken from [title] and [actions] once — so a
  /// sheet that changes the language re-renders its own chrome.
  String Function(BuildContext)? titleOf,
  List<Widget> Function(BuildContext)? actionsOf,
}) {
  return showDialog<T>(
    context: context,
    barrierColor: Colors.black.withValues(alpha: 0.25),
    builder: (ctx) => Builder(
      builder: (ctx) {
        final t = MacTokens.of(ctx);
        final title_ = titleOf?.call(ctx) ?? title;
        final actions_ = actionsOf?.call(ctx) ?? actions;
        return Dialog(
          backgroundColor: t.window,
          surfaceTintColor: Colors.transparent,
          insetPadding: const EdgeInsets.all(24),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(10),
            side: BorderSide(color: t.hairline),
          ),
          child: SizedBox(
            width: width,
            child: Padding(
              padding: const EdgeInsets.fromLTRB(20, 18, 20, 16),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(title_, style: Theme.of(ctx).textTheme.titleMedium),
                  if (subtitle != null) ...[
                    const SizedBox(height: 2),
                    Text(subtitle, style: TextStyle(fontSize: 11, color: t.textSecondary)),
                  ],
                  const SizedBox(height: 14),
                  content,
                  const SizedBox(height: 18),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      for (var i = 0; i < actions_.length; i++) ...[
                        if (i > 0) const SizedBox(width: 8),
                        actions_[i],
                      ],
                    ],
                  ),
                ],
              ),
            ),
          ),
        );
      },
    ),
  );
}

/// Typed-path fallback, used only when the OS dialog is unavailable.
Future<String?> showPathSheet(BuildContext context, {required String title}) {
  final path = TextEditingController();
  return showMacSheet<String>(
    context,
    title: title,
    content: FormRow(
      label: context.l10n.folder,
      child: MacTextField(
        controller: path,
        autofocus: true,
        onSubmitted: (v) => Navigator.pop(context, v.trim()),
      ),
    ),
    actions: [
      MacButton(label: context.l10n.cancel, onPressed: () => Navigator.pop(context)),
      MacButton.primary(
        label: context.l10n.choose,
        onPressed: () => Navigator.pop(context, path.text.trim()),
      ),
    ],
  );
}

/// A one-field sheet: a name for a new named thing (a component).
Future<String?> showNameSheet(
  BuildContext context, {
  required String title,
  String? subtitle,
  String? hint,
}) {
  final name = TextEditingController();
  return showMacSheet<String>(
    context,
    title: title,
    subtitle: subtitle,
    content: FormRow(
      label: context.l10n.name,
      child: MacTextField(
        controller: name,
        hint: hint,
        autofocus: true,
        onSubmitted: (v) => Navigator.pop(context, v.trim()),
      ),
    ),
    actions: [
      MacButton(label: context.l10n.cancel, onPressed: () => Navigator.pop(context)),
      MacButton.primary(
        label: context.l10n.create,
        onPressed: () => Navigator.pop(context, name.text.trim()),
      ),
    ],
  );
}

// ---------------------------------------------------------------------------
// New concept
// ---------------------------------------------------------------------------

typedef NewConceptResult = ({String name, String description, pb.Representation? representation});

Future<NewConceptResult?> showNewConceptSheet(BuildContext context, {List<UnitPreset>? presets}) {
  return showMacSheet<NewConceptResult>(
    context,
    title: context.l10n.newConcept,
    subtitle: context.l10n.somethingTheProductSensesDecidesOrShows,
    content: _NewConceptForm(presets: presets ?? builtinUnitPresets(context.l10n)),
    actions: const [],
  );
}

class _NewConceptForm extends StatefulWidget {
  const _NewConceptForm({required this.presets});
  final List<UnitPreset> presets;

  @override
  State<_NewConceptForm> createState() => _NewConceptFormState();
}

enum _Kind { open, quantity, boolean, count }

class _NewConceptFormState extends State<_NewConceptForm> {
  final _name = TextEditingController();
  final _description = TextEditingController();
  _Kind _kind = _Kind.open;
  late UnitPreset _unit = widget.presets.first;

  pb.Representation? get _representation => switch (_kind) {
    _Kind.open => null,
    _Kind.quantity => pb.Representation(quantity: _unit.dim),
    _Kind.boolean => pb.Representation(boolean: pb.Unit()),
    _Kind.count => pb.Representation(count: pb.Unit()),
  };

  void _submit() {
    final name = _name.text.trim();
    if (name.isEmpty) return;
    Navigator.pop(context, (
      name: name,
      description: _description.text.trim(),
      representation: _representation,
    ));
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final name = _name.text.trim();
    // The preview is a real node built by the real geometry.
    final preview = pb.ProjectProjection(name: 'preview')
      ..concepts.add(
        pb.ConceptView(
          id: Int64(0),
          name: name.isEmpty ? context.l10n.name : name,
          representation: _representation,
        ),
      );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormRow(
          label: context.l10n.name,
          child: MacTextField(
            controller: _name,
            autofocus: true,
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        FormRow(
          label: context.l10n.value,
          child: MacSegmented<_Kind>(
            value: _kind,
            options: {
              _Kind.quantity: context.l10n.quantity,
              _Kind.boolean: context.l10n.onOff,
              _Kind.count: context.l10n.count,
              _Kind.open: context.l10n.decideLater,
            },
            onChanged: (k) => setState(() => _kind = k),
          ),
        ),
        if (_kind == _Kind.quantity)
          FormRow(
            label: context.l10n.unit,
            child: MacDropdown<UnitPreset>(
              value: _unit,
              items: widget.presets,
              labelOf: (p) => p.name,
              // the unit symbol sits in its own column, secondary colour
              detailOf: (p) => p.symbol,
              onChanged: (p) => setState(() => _unit = p),
            ),
          ),
        FormRow(
          label: context.l10n.meaning,
          child: MacTextField(controller: _description, maxLines: 2, onSubmitted: (_) => _submit()),
        ),
        const SizedBox(height: 6),
        _PreviewBox(
          child: NodePreview(
            projection: preview,
            node: const NodeRef.concept(0),
            height: 72,
            neutralConcepts: const {0},
          ),
        ),
        const SizedBox(height: 4),
        // The caption names the mark the designer will meet on the canvas.
        Text(switch (_kind) {
          _Kind.open => context.l10n.hollowRingTheValueCanBeDecided,
          _Kind.quantity => context.l10n.roundSocketAMeasuredQuantityItsUnit,
          _Kind.boolean => context.l10n.diamondSocketOnOrOffActivatesContexts,
          _Kind.count => context.l10n.squareSocketAWholeNumberOccurrencesSteps,
        }, style: TextStyle(fontSize: 11, color: t.textSecondary)),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: context.l10n.cancel, onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(label: context.l10n.create, onPressed: name.isEmpty ? null : _submit),
          ],
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// New mapping
// ---------------------------------------------------------------------------

typedef NewMappingResult = ({String name, List<int> inputs, int output});

Future<NewMappingResult?> showNewMappingSheet(BuildContext context, List<pb.ConceptView> concepts) {
  return showMacSheet<NewMappingResult>(
    context,
    title: context.l10n.newMapping,
    subtitle: context.l10n.aRelationshipBetweenConceptsItCanExist,
    content: _NewMappingForm(concepts: concepts),
    actions: const [],
    width: 520,
  );
}

/// A Source: `name : () -> concept` with no definition — the same form
/// without the *Reads* row, because the environment provides the value
/// (ADR-0032).  What comes back is an ordinary [NewMappingResult] with no
/// inputs; there is one creation path.
Future<NewMappingResult?> showNewSourceSheet(BuildContext context, List<pb.ConceptView> concepts) {
  return showMacSheet<NewMappingResult>(
    context,
    title: context.l10n.newSource,
    subtitle: context.l10n.aSourceAValueTheEnvironmentProvides,
    content: _NewMappingForm(concepts: concepts, source: true),
    actions: const [],
    width: 520,
  );
}

class _NewMappingForm extends StatefulWidget {
  const _NewMappingForm({required this.concepts, this.source = false});
  final List<pb.ConceptView> concepts;
  final bool source;

  @override
  State<_NewMappingForm> createState() => _NewMappingFormState();
}

class _NewMappingFormState extends State<_NewMappingForm> {
  final _name = TextEditingController();
  final _inputs = <int>[];

  /// What the mapping produces — a choice, never a silent default: a
  /// mapping that "produces" the first concept in the list would be created
  /// without anyone deciding so.
  int? _output;

  /// Stands in for the not-yet-chosen output in the preview: a hollow,
  /// neutral socket with no name.
  static const _unchosen = 1 << 40;

  void _submit() {
    final name = _name.text.trim();
    final output = _output;
    if (name.isEmpty || output == null) return;
    Navigator.pop(context, (name: name, inputs: List<int>.of(_inputs), output: output));
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final name = _name.text.trim();
    final preview = pb.ProjectProjection(name: 'preview')
      ..concepts.addAll(widget.concepts)
      ..concepts.add(pb.ConceptView(id: Int64(_unchosen), name: ''))
      ..mappings.add(
        pb.MappingView(
          id: Int64(0),
          name: name.isEmpty ? context.l10n.name : name,
          signature: pb.Signature(
            inputs: _inputs.map(Int64.new),
            output: Int64(_output ?? _unchosen),
          ),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED,
        ),
      );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormRow(
          label: context.l10n.name,
          child: MacTextField(
            controller: _name,
            autofocus: true,
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        if (!widget.source)
          FormRow(
            label: context.l10n.reads,
            child: Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                for (final c in widget.concepts)
                  _ConceptToggle(
                    concept: c,
                    selected: _inputs.contains(c.id.toInt()),
                    onChanged: (on) => setState(() {
                      on ? _inputs.add(c.id.toInt()) : _inputs.remove(c.id.toInt());
                    }),
                  ),
              ],
            ),
          ),
        FormRow(
          label: widget.source ? context.l10n.provides : context.l10n.produces,
          child: MacDropdown<int>(
            value: _output,
            hint: 'choose',
            items: [for (final c in widget.concepts) c.id.toInt()],
            labelOf: (id) => widget.concepts.firstWhere((c) => c.id.toInt() == id).name,
            leadingOf: (id) =>
                SocketGlyph.of(widget.concepts.firstWhere((c) => c.id.toInt() == id), t, size: 11),
            onChanged: (v) => setState(() => _output = v),
          ),
        ),
        const SizedBox(height: 6),
        _PreviewBox(
          child: NodePreview(
            projection: preview,
            node: const NodeRef.mapping(0),
            height: 72 + 22.0 * (_inputs.isEmpty ? 0 : _inputs.length - 1),
            neutralConcepts: const {_unchosen},
          ),
        ),
        const SizedBox(height: 4),
        // The preview derives the role like the canvas: with no reads the
        // node is a Source, and the hint says so instead of "dashed".
        Text(
          _output == null
              ? widget.source
                    ? context.l10n.chooseWhatItProvidesTheOutputSocket
                    : context.l10n.chooseWhatItProducesTheOutputSocket
              : _inputs.isEmpty
              ? context.l10n.theEnvironmentProvidesItNoInputSockets
              : context.l10n.dashedDeclaredNotYetDefinedAttachA,
          style: TextStyle(fontSize: 11, color: t.textSecondary),
        ),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: context.l10n.cancel, onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(
              label: context.l10n.create,
              onPressed: name.isEmpty || _output == null ? null : _submit,
            ),
          ],
        ),
      ],
    );
  }
}

/// A concept as a toggle: its socket glyph — the same mark it has on the
/// canvas — and its name.  A chip filled in the concept's hue means *read by
/// this mapping*.  A real control: focusable, Space toggles.
class _ConceptToggle extends StatelessWidget {
  const _ConceptToggle({required this.concept, required this.selected, required this.onChanged});
  final pb.ConceptView concept;
  final bool selected;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final color = t.conceptColor(concept.id.toInt());
    return Semantics(
      toggled: selected,
      label: context.l10n.readConcept(concept.name),
      child: MacInteractive(
        onTap: () => onChanged(!selected),
        child: AnimatedContainer(
          duration: MacStates.duration,
          curve: MacStates.curve,
          height: MacMetrics.controlHeight,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(
            color: selected ? color.withValues(alpha: 0.16) : t.control,
            borderRadius: BorderRadius.circular(5),
            border: Border.all(color: selected ? color : t.hairline),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            spacing: 6,
            children: [
              SocketGlyph.of(concept, t, size: 11),
              Text(concept.name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
            ],
          ),
        ),
      ),
    );
  }
}

class _PreviewBox extends StatelessWidget {
  const _PreviewBox({required this.child});
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      decoration: BoxDecoration(
        color: t.canvas,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: t.hairline),
      ),
      child: child,
    );
  }
}

// ---------------------------------------------------------------------------
// New timing domain / new output
// ---------------------------------------------------------------------------

/// A timing domain is a name for *when* things happen — never a rate.
Future<String?> showNewClockSheet(BuildContext context, List<pb.ClockView> existing) {
  return showMacSheet<String>(
    context,
    title: context.l10n.newTimingDomain,
    subtitle: context.l10n.whenAGroupOfRelationshipsAndOutputs,
    content: _NewClockForm(existing: existing),
    actions: const [],
    width: 440,
  );
}

class _NewClockForm extends StatefulWidget {
  const _NewClockForm({required this.existing});
  final List<pb.ClockView> existing;
  @override
  State<_NewClockForm> createState() => _NewClockFormState();
}

class _NewClockFormState extends State<_NewClockForm> {
  final _name = TextEditingController();

  bool get _taken => widget.existing.any((c) => c.name == _name.text.trim());

  void _submit() {
    final name = _name.text.trim();
    if (name.isEmpty || _taken) return;
    Navigator.pop(context, name);
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final name = _name.text.trim();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormRow(
          label: context.l10n.name,
          child: MacTextField(
            controller: _name,
            autofocus: true,
            hint: context.l10n.interactionAmbient,
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        if (_taken)
          Text(
            context.l10n.domainAlreadyExists(name),
            style: TextStyle(fontSize: 11, color: t.open),
          ),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: context.l10n.cancel, onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(
              label: context.l10n.create,
              onPressed: name.isEmpty || _taken ? null : _submit,
            ),
          ],
        ),
      ],
    );
  }
}

typedef NewOutputResult = ({String name, int accepts, int? clockId, bool required});

/// A physical output: the boundary where a value leaves the design.
Future<NewOutputResult?> showNewOutputSheet(
  BuildContext context,
  List<pb.ConceptView> concepts,
  List<pb.ClockView> clocks,
) {
  return showMacSheet<NewOutputResult>(
    context,
    title: context.l10n.newOutput,
    subtitle: context.l10n.whereAValueLeavesTheDesignFor,
    content: _NewOutputForm(concepts: concepts, clocks: clocks),
    actions: const [],
    width: 480,
  );
}

class _NewOutputForm extends StatefulWidget {
  const _NewOutputForm({required this.concepts, required this.clocks});
  final List<pb.ConceptView> concepts;
  final List<pb.ClockView> clocks;
  @override
  State<_NewOutputForm> createState() => _NewOutputFormState();
}

class _NewOutputFormState extends State<_NewOutputForm> {
  final _name = TextEditingController();
  int? _accepts;
  int? _clock;
  bool _required = true;

  void _submit() {
    final name = _name.text.trim();
    final accepts = _accepts;
    if (name.isEmpty || accepts == null) return;
    Navigator.pop(context, (name: name, accepts: accepts, clockId: _clock, required: _required));
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final name = _name.text.trim();
    pb.ConceptView concept(int id) => widget.concepts.firstWhere((c) => c.id.toInt() == id);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormRow(
          label: context.l10n.name,
          child: MacTextField(
            controller: _name,
            autofocus: true,
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        FormRow(
          label: context.l10n.accepts,
          child: MacDropdown<int>(
            value: _accepts,
            hint: context.l10n.theConceptThisOutputTakes,
            items: [for (final c in widget.concepts) c.id.toInt()],
            labelOf: (c) => concept(c).name,
            leadingOf: (c) => SocketGlyph.of(concept(c), t, size: 11),
            onChanged: (c) => setState(() => _accepts = c),
          ),
        ),
        FormRow(
          label: context.l10n.updatesIn,
          child: MacDropdown<int>(
            value: _clock ?? -1,
            items: [-1, for (final c in widget.clocks) c.id.toInt()],
            labelOf: (c) => c < 0
                ? context.l10n.decideLaterLower
                : widget.clocks.firstWhere((x) => x.id.toInt() == c).name,
            onChanged: (c) => setState(() => _clock = c < 0 ? null : c),
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
                  value: _required,
                  onChanged: (v) => setState(() => _required = v ?? false),
                ),
                Expanded(
                  child: Text(
                    context.l10n.theDesignIsIncompleteUntilSomethingDrives,
                    style: TextStyle(fontSize: 11, color: t.textSecondary),
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: context.l10n.cancel, onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(
              label: context.l10n.create,
              onPressed: name.isEmpty || _accepts == null ? null : _submit,
            ),
          ],
        ),
      ],
    );
  }
}
