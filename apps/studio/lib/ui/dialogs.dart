import 'package:flutter/material.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

/// Path + name for a new project (no file picker yet: it needs platform
/// plugins, and v0.1 keeps the desktop build plugin-free).
Future<({String path, String name})?> showNewProjectDialog(BuildContext context) {
  final path = TextEditingController();
  final name = TextEditingController(text: 'lamp');
  return showDialog(
    context: context,
    builder: (ctx) => AlertDialog(
      title: const Text('New project'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: path,
            decoration: const InputDecoration(labelText: 'Directory'),
          ),
          TextField(
            controller: name,
            decoration: const InputDecoration(labelText: 'Name'),
          ),
        ],
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Cancel')),
        FilledButton(
          onPressed: () => Navigator.pop(ctx, (path: path.text.trim(), name: name.text.trim())),
          child: const Text('Create'),
        ),
      ],
    ),
  );
}

Future<String?> showOpenProjectDialog(BuildContext context) {
  final path = TextEditingController();
  return showDialog<String>(
    context: context,
    builder: (ctx) => AlertDialog(
      title: const Text('Open project'),
      content: TextField(
        controller: path,
        decoration: const InputDecoration(labelText: 'Project directory (contains bdl.toml)'),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Cancel')),
        FilledButton(
          onPressed: () => Navigator.pop(ctx, path.text.trim()),
          child: const Text('Open'),
        ),
      ],
    ),
  );
}

Future<String?> showNameDialog(BuildContext context, {required String title, String hint = ''}) {
  final name = TextEditingController();
  return showDialog<String>(
    context: context,
    builder: (ctx) => AlertDialog(
      title: Text(title),
      content: TextField(
        controller: name,
        autofocus: true,
        decoration: InputDecoration(labelText: 'Name', hintText: hint),
        onSubmitted: (v) => Navigator.pop(ctx, v.trim()),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Cancel')),
        FilledButton(
          onPressed: () => Navigator.pop(ctx, name.text.trim()),
          child: const Text('Create'),
        ),
      ],
    ),
  );
}

/// `name : (inputs) -> output` over existing concepts.
Future<({String name, List<int> inputs, int output})?> showNewMappingDialog(
  BuildContext context,
  List<pb.ConceptView> concepts,
) {
  final name = TextEditingController();
  final selectedInputs = <int>{};
  int? output = concepts.isNotEmpty ? concepts.first.id.toInt() : null;
  return showDialog(
    context: context,
    builder: (ctx) => StatefulBuilder(
      builder: (ctx, setState) => AlertDialog(
        title: const Text('New mapping'),
        content: SizedBox(
          width: 420,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TextField(
                controller: name,
                autofocus: true,
                decoration: const InputDecoration(labelText: 'Name', hintText: 'dimByTilt'),
              ),
              const SizedBox(height: 12),
              const Text('Inputs'),
              Wrap(
                spacing: 6,
                children: [
                  for (final c in concepts)
                    FilterChip(
                      label: Text(c.name),
                      selected: selectedInputs.contains(c.id.toInt()),
                      onSelected: (on) => setState(() {
                        on ? selectedInputs.add(c.id.toInt()) : selectedInputs.remove(c.id.toInt());
                      }),
                    ),
                ],
              ),
              const SizedBox(height: 12),
              DropdownButtonFormField<int>(
                initialValue: output,
                decoration: const InputDecoration(labelText: 'Output'),
                items: [
                  for (final c in concepts)
                    DropdownMenuItem(value: c.id.toInt(), child: Text(c.name)),
                ],
                onChanged: (v) => setState(() => output = v),
              ),
              const SizedBox(height: 8),
              Text(
                'The mapping is created without a definition. That is a legal state: '
                'others may depend on it before it is defined.',
                style: Theme.of(ctx).textTheme.bodySmall,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Cancel')),
          FilledButton(
            onPressed: output == null
                ? null
                : () => Navigator.pop(ctx, (
                    name: name.text.trim(),
                    inputs: selectedInputs.toList()..sort(),
                    output: output!,
                  )),
            child: const Text('Create'),
          ),
        ],
      ),
    ),
  );
}
