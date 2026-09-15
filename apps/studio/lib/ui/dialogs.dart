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
import 'canvas/canvas_geometry.dart' show dimLabel;
import 'canvas/node_canvas.dart' show NodePreview;
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';

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

Future<NewConceptResult?> showNewConceptSheet(BuildContext context) {
  return showMacSheet<NewConceptResult>(
    context,
    title: 'New concept',
    subtitle: 'Something the product senses, decides or shows.',
    content: const _NewConceptForm(),
    actions: const [],
  );
}

class _NewConceptForm extends StatefulWidget {
  const _NewConceptForm();

  @override
  State<_NewConceptForm> createState() => _NewConceptFormState();
}

enum _Kind { open, quantity, boolean, count }

class _NewConceptFormState extends State<_NewConceptForm> {
  final _name = TextEditingController();
  final _description = TextEditingController();
  _Kind _kind = _Kind.open;
  String _dim = 'dimensionless';

  static final dims = <String, pb.Dim>{
    'dimensionless': pb.Dim(),
    'angle': pb.Dim(angle: 1),
    'length': pb.Dim(length: 1),
    'time': pb.Dim(time: 1),
    'temperature': pb.Dim(temperature: 1),
    'mass': pb.Dim(mass: 1),
    'current': pb.Dim(current: 1),
    'angular rate': pb.Dim(angle: 1, time: -1),
    'speed': pb.Dim(length: 1, time: -1),
  };

  pb.Representation? get _representation => switch (_kind) {
    _Kind.open => null,
    _Kind.quantity => pb.Representation(quantity: dims[_dim]!),
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
          label: 'Kind',
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
            label: 'Dimension',
            child: MacDropdown<String>(
              value: _dim,
              items: dims.keys.toList(),
              labelOf: (k) => k,
              // the unit symbol sits in its own column, secondary colour
              detailOf: (k) => dimLabel(dims[k]!),
              onChanged: (k) => setState(() => _dim = k),
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
        Text(switch (_kind) {
          _Kind.open =>
            'Open socket: the kind can be chosen later; relationships can already use it.',
          _Kind.quantity =>
            'Filled socket: a measured value. Its dimension is checked in every formula.',
          _Kind.boolean => 'Filled socket: true or false — activates contexts, gates behaviour.',
          _Kind.count => 'Filled socket: a whole number — occurrences, steps, items.',
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
  late int _output = widget.concepts.first.id.toInt();

  void _submit() {
    final name = _name.text.trim();
    if (name.isEmpty) return;
    Navigator.pop(context, (name: name, inputs: List<int>.of(_inputs), output: _output));
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final name = _name.text.trim();
    final preview = pb.ProjectProjection(name: 'preview')
      ..concepts.addAll(widget.concepts)
      ..mappings.add(
        pb.MappingView(
          id: Int64(0),
          name: name.isEmpty ? 'Name' : name,
          signature: pb.Signature(inputs: _inputs.map(Int64.new), output: Int64(_output)),
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
            items: [for (final c in widget.concepts) c.id.toInt()],
            labelOf: (id) => widget.concepts.firstWhere((c) => c.id.toInt() == id).name,
            onChanged: (v) => setState(() => _output = v),
          ),
        ),
        const SizedBox(height: 6),
        _PreviewBox(
          child: NodePreview(
            projection: preview,
            node: const NodeRef.mapping(0),
            height: 72 + 22.0 * (_inputs.isEmpty ? 0 : _inputs.length - 1),
          ),
        ),
        const SizedBox(height: 4),
        Text(
          'Dashed: declared, not yet defined. Attach a formula, curve or component from the '
          'inspector whenever you are ready.',
          style: TextStyle(fontSize: 11, color: t.textSecondary),
        ),
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

/// A concept as a toggle: its socket dot in its colour, filled when chosen —
/// the same visual it has on the canvas.
class _ConceptToggle extends StatelessWidget {
  const _ConceptToggle({required this.concept, required this.selected, required this.onChanged});
  final pb.ConceptView concept;
  final bool selected;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final color = t.conceptColor(concept.id.toInt());
    return GestureDetector(
      onTap: () => onChanged(!selected),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 120),
        height: MacMetrics.controlHeight,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(
          color: selected ? color.withValues(alpha: 0.16) : t.control,
          borderRadius: BorderRadius.circular(5),
          border: Border.all(color: selected ? color : t.hairline),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 9,
              height: 9,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: selected ? color : Colors.transparent,
                border: Border.all(color: color, width: 1.5),
              ),
            ),
            const SizedBox(width: 6),
            Text(concept.name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
          ],
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
