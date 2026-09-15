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
}) {
  return showDialog<T>(
    context: context,
    barrierColor: Colors.black.withValues(alpha: 0.25),
    builder: (ctx) {
      final t = MacTokens.of(ctx);
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
                Text(title, style: Theme.of(ctx).textTheme.titleMedium),
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
                    for (var i = 0; i < actions.length; i++) ...[
                      if (i > 0) const SizedBox(width: 8),
                      actions[i],
                    ],
                  ],
                ),
              ],
            ),
          ),
        ),
      );
    },
  );
}

/// Typed-path fallback, used only when the OS dialog is unavailable.
Future<String?> showPathSheet(BuildContext context, {required String title}) {
  final path = TextEditingController();
  return showMacSheet<String>(
    context,
    title: title,
    content: FormRow(
      label: 'Folder',
      child: MacTextField(
        controller: path,
        autofocus: true,
        onSubmitted: (v) => Navigator.pop(context, v.trim()),
      ),
    ),
    actions: [
      MacButton(label: 'Cancel', onPressed: () => Navigator.pop(context)),
      MacButton.primary(label: 'Choose', onPressed: () => Navigator.pop(context, path.text.trim())),
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
    title: 'New concept',
    subtitle: 'Something the product senses, decides or shows.',
    content: _NewConceptForm(presets: presets ?? builtinUnitPresets),
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
          name: name.isEmpty ? 'Name' : name,
          representation: _representation,
        ),
      );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormRow(
          label: 'Name',
          child: MacTextField(
            controller: _name,
            autofocus: true,
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        FormRow(
          label: 'Value',
          child: MacSegmented<_Kind>(
            value: _kind,
            options: const {
              _Kind.quantity: 'Quantity',
              _Kind.boolean: 'On / off',
              _Kind.count: 'Count',
              _Kind.open: 'Decide later',
            },
            onChanged: (k) => setState(() => _kind = k),
          ),
        ),
        if (_kind == _Kind.quantity)
          FormRow(
            label: 'Unit',
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
          label: 'Meaning',
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
          _Kind.open =>
            'Hollow ring: the value can be decided later; relationships can already use it.',
          _Kind.quantity =>
            'Round socket: a measured quantity. Its unit is checked in every formula.',
          _Kind.boolean => 'Diamond socket: on or off — activates contexts, gates behaviour.',
          _Kind.count => 'Square socket: a whole number — occurrences, steps, items.',
        }, style: TextStyle(fontSize: 11, color: t.textSecondary)),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: 'Cancel', onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(label: 'Create', onPressed: name.isEmpty ? null : _submit),
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
    title: 'New mapping',
    subtitle: 'A relationship between concepts. It can exist before it is defined.',
    content: _NewMappingForm(concepts: concepts),
    actions: const [],
    width: 520,
  );
}

class _NewMappingForm extends StatefulWidget {
  const _NewMappingForm({required this.concepts});
  final List<pb.ConceptView> concepts;

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
          name: name.isEmpty ? 'Name' : name,
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
          label: 'Name',
          child: MacTextField(
            controller: _name,
            autofocus: true,
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        FormRow(
          label: 'Reads',
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
          label: 'Produces',
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
        Text(
          _output == null
              ? 'Choose what it produces: the output socket takes that concept’s colour and shape.'
              : 'Dashed: declared, not yet defined. Attach a formula from the inspector whenever '
                    'you are ready.',
          style: TextStyle(fontSize: 11, color: t.textSecondary),
        ),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: 'Cancel', onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(
              label: 'Create',
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
      label: 'read ${concept.name}',
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
    title: 'New timing domain',
    subtitle:
        'When a group of relationships and outputs update together. A name, not a rate: how '
        'often it activates is decided when the design runs.',
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
          label: 'Name',
          child: MacTextField(
            controller: _name,
            autofocus: true,
            hint: 'interaction, ambient, …',
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        if (_taken)
          Text(
            'A domain named $name already exists.',
            style: TextStyle(fontSize: 11, color: t.open),
          ),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            MacButton(label: 'Cancel', onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(label: 'Create', onPressed: name.isEmpty || _taken ? null : _submit),
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
    title: 'New output',
    subtitle: 'Where a value leaves the design for the world: a light, a motor, a display.',
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
          label: 'Name',
          child: MacTextField(
            controller: _name,
            autofocus: true,
            onChanged: (_) => setState(() {}),
            onSubmitted: (_) => _submit(),
          ),
        ),
        FormRow(
          label: 'Accepts',
          child: MacDropdown<int>(
            value: _accepts,
            hint: 'the concept this output takes',
            items: [for (final c in widget.concepts) c.id.toInt()],
            labelOf: (c) => concept(c).name,
            leadingOf: (c) => SocketGlyph.of(concept(c), t, size: 11),
            onChanged: (c) => setState(() => _accepts = c),
          ),
        ),
        FormRow(
          label: 'Updates in',
          child: MacDropdown<int>(
            value: _clock ?? -1,
            items: [-1, for (final c in widget.clocks) c.id.toInt()],
            labelOf: (c) =>
                c < 0 ? 'decide later' : widget.clocks.firstWhere((x) => x.id.toInt() == c).name,
            onChanged: (c) => setState(() => _clock = c < 0 ? null : c),
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
                  value: _required,
                  onChanged: (v) => setState(() => _required = v ?? false),
                ),
                Expanded(
                  child: Text(
                    'the design is incomplete until something drives it',
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
            MacButton(label: 'Cancel', onPressed: () => Navigator.pop(context)),
            const SizedBox(width: 8),
            MacButton.primary(
              label: 'Create',
              onPressed: name.isEmpty || _accepts == null ? null : _submit,
            ),
          ],
        ),
      ],
    );
  }
}
