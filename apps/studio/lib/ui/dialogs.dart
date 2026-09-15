/// Sheets for creating objects.  Project open/new go through the OS's own
/// pickers (`file_selector`), see `effects/effect_executor.dart`.
library;

import 'package:flutter/material.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'mac/tokens.dart';
import 'mac/widgets.dart';

Future<T?> _sheet<T>(
  BuildContext context, {
  required String title,
  required Widget content,
  required List<Widget> actions,
}) {
  return showDialog<T>(
    context: context,
    builder: (ctx) => AlertDialog(
      title: Text(title),
      content: SizedBox(width: 420, child: content),
      actionsPadding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
      actions: actions,
    ),
  );
}

/// Typed-path fallback, used only when the OS dialog is unavailable.
Future<String?> showPathSheet(BuildContext context, {required String title}) {
  final path = TextEditingController();
  return _sheet<String>(
    context,
    title: title,
    content: FormRow(
      label: 'Folder',
      child: TextField(
        controller: path,
        autofocus: true,
        decoration: const InputDecoration(hintText: '/absolute/path/to/project'),
        onSubmitted: (v) => Navigator.pop(context, v.trim()),
      ),
    ),
    actions: [
      OutlinedButton(onPressed: () => Navigator.pop(context), child: const Text('Cancel')),
      FilledButton(
        onPressed: () => Navigator.pop(context, path.text.trim()),
        child: const Text('OK'),
      ),
    ],
  );
}

Future<String?> showNameSheet(BuildContext context, {required String title, String hint = ''}) {
  final name = TextEditingController();
  return _sheet<String>(
    context,
    title: title,
    content: FormRow(
      label: 'Name',
      child: TextField(
        controller: name,
        autofocus: true,
        decoration: InputDecoration(hintText: hint),
        onSubmitted: (v) => Navigator.pop(context, v.trim()),
      ),
    ),
    actions: [
      OutlinedButton(onPressed: () => Navigator.pop(context), child: const Text('Cancel')),
      FilledButton(
        onPressed: () => Navigator.pop(context, name.text.trim()),
        child: const Text('Create'),
      ),
    ],
  );
}

/// `name : (inputs) -> output` over existing concepts.
Future<({String name, List<int> inputs, int output})?> showNewMappingSheet(
  BuildContext context,
  List<pb.ConceptView> concepts,
) {
  final t = MacTokens.of(context);
  final name = TextEditingController();
  final selectedInputs = <int>{};
  int output = concepts.first.id.toInt();
  String nameOf(int id) => concepts.firstWhere((c) => c.id.toInt() == id).name;
  return _sheet(
    context,
    title: 'New mapping',
    content: StatefulBuilder(
      builder: (ctx, setState) => Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          FormRow(
            label: 'Name',
            child: TextField(
              controller: name,
              autofocus: true,
              decoration: const InputDecoration(hintText: 'dimByTilt'),
            ),
          ),
          FormRow(
            label: 'Inputs',
            child: Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                for (final c in concepts)
                  FilterChip(
                    label: Text(c.name, style: const TextStyle(fontSize: 12)),
                    avatar: Icon(Icons.circle, size: 8, color: t.conceptColor(c.id.toInt())),
                    selected: selectedInputs.contains(c.id.toInt()),
                    showCheckmark: false,
                    selectedColor: t.selection,
                    side: BorderSide(color: t.hairline),
                    visualDensity: VisualDensity.compact,
                    onSelected: (on) => setState(() {
                      on ? selectedInputs.add(c.id.toInt()) : selectedInputs.remove(c.id.toInt());
                    }),
                  ),
              ],
            ),
          ),
          FormRow(
            label: 'Output',
            child: MacDropdown<int>(
              value: output,
              items: [for (final c in concepts) c.id.toInt()],
              labelOf: nameOf,
              onChanged: (v) => setState(() => output = v),
            ),
          ),
          const SizedBox(height: 4),
          Text(
            'Created without a definition — a legal state. Others may depend on it before it '
            'is defined.',
            style: TextStyle(fontSize: 11, color: t.textSecondary),
          ),
        ],
      ),
    ),
    actions: [
      OutlinedButton(onPressed: () => Navigator.pop(context), child: const Text('Cancel')),
      FilledButton(
        onPressed: () => Navigator.pop(context, (
          name: name.text.trim(),
          inputs: selectedInputs.toList()..sort(),
          output: output,
        )),
        child: const Text('Create'),
      ),
    ],
  );
}
