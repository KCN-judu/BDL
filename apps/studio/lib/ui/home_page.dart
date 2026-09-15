import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../app/store.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'dialogs.dart';
import 'status_bar.dart';

class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(appStoreProvider);
    final dispatch = ref.read(appStoreProvider.notifier).dispatch;
    final project = state.project;
    final connected = state.connection is Connected;

    return Scaffold(
      appBar: AppBar(
        title: Text(project == null ? 'BDL Studio' : '${project.name}${project.dirty ? ' •' : ''}'),
        actions: [
          IconButton(
            tooltip: 'New project',
            icon: const Icon(Icons.create_new_folder_outlined),
            onPressed: connected && project == null
                ? () async {
                    final r = await showNewProjectDialog(context);
                    if (r != null && r.path.isNotEmpty) {
                      dispatch(NewProjectRequested(rootPath: r.path, name: r.name));
                    }
                  }
                : null,
          ),
          IconButton(
            tooltip: 'Open project',
            icon: const Icon(Icons.folder_open_outlined),
            onPressed: connected && project == null
                ? () async {
                    final path = await showOpenProjectDialog(context);
                    if (path != null && path.isNotEmpty) dispatch(OpenProjectRequested(path));
                  }
                : null,
          ),
          IconButton(
            tooltip: 'Save',
            icon: const Icon(Icons.save_outlined),
            onPressed: project != null && project.dirty
                ? () => dispatch(const SaveRequested())
                : null,
          ),
          IconButton(
            tooltip: 'Undo',
            icon: const Icon(Icons.undo),
            onPressed: project?.canUndo == true ? () => dispatch(const UndoRequested()) : null,
          ),
          IconButton(
            tooltip: 'Redo',
            icon: const Icon(Icons.redo),
            onPressed: project?.canRedo == true ? () => dispatch(const RedoRequested()) : null,
          ),
          IconButton(
            tooltip: 'Close project',
            icon: const Icon(Icons.close),
            onPressed: project != null ? () => dispatch(const CloseProjectRequested()) : null,
          ),
        ],
      ),
      body: Column(
        children: [
          if (state.editor.lastError case final err?)
            MaterialBanner(
              content: Text(err.message),
              leading: const Icon(Icons.error_outline),
              actions: [
                TextButton(
                  onPressed: () => dispatch(const ErrorDismissed()),
                  child: const Text('Dismiss'),
                ),
              ],
            ),
          Expanded(
            child: project == null
                ? const _EmptyView()
                : Row(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Expanded(child: _ConceptsPanel(project: project)),
                      const VerticalDivider(width: 1),
                      Expanded(flex: 2, child: _MappingsPanel(project: project)),
                      const VerticalDivider(width: 1),
                      SizedBox(width: 300, child: _Inspector(state: state)),
                    ],
                  ),
          ),
          const StatusBar(),
        ],
      ),
    );
  }
}

class _EmptyView extends StatelessWidget {
  const _EmptyView();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text(
        'No project open.\nCreate or open one from the toolbar.',
        textAlign: TextAlign.center,
        style: Theme.of(context).textTheme.bodyLarge,
      ),
    );
  }
}

class _ConceptsPanel extends ConsumerWidget {
  const _ConceptsPanel({required this.project});
  final pb.ProjectProjection project;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final dispatch = ref.read(appStoreProvider.notifier).dispatch;
    final selection = ref.watch(appStoreProvider.select((s) => s.editor.selection));
    return _Panel(
      title: 'Concepts',
      onAdd: () async {
        final name = await showNameDialog(context, title: 'New concept', hint: 'Tilt');
        if (name != null && name.isNotEmpty) dispatch(CreateConceptRequested(name: name));
      },
      child: ListView(
        children: [
          for (final c in project.concepts)
            ListTile(
              dense: true,
              leading: const Icon(Icons.label_outline),
              title: Text(c.name),
              subtitle: c.hasRepresentation() ? null : const Text('representation not yet chosen'),
              selected: selection is ConceptSelected && selection.id == c.id.toInt(),
              onTap: () => dispatch(SelectionChanged(ConceptSelected(c.id.toInt()))),
            ),
        ],
      ),
    );
  }
}

class _MappingsPanel extends ConsumerWidget {
  const _MappingsPanel({required this.project});
  final pb.ProjectProjection project;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final dispatch = ref.read(appStoreProvider.notifier).dispatch;
    final selection = ref.watch(appStoreProvider.select((s) => s.editor.selection));
    String conceptName(Int64 id) =>
        project.concepts.where((c) => c.id == id).map((c) => c.name).firstOrNull ?? '?';
    return _Panel(
      title: 'Mappings',
      onAdd: project.concepts.isEmpty
          ? null
          : () async {
              final r = await showNewMappingDialog(context, project.concepts);
              if (r != null && r.name.isNotEmpty) {
                dispatch(CreateMappingRequested(name: r.name, inputs: r.inputs, output: r.output));
              }
            },
      child: ListView(
        children: [
          for (final m in project.mappings)
            ListTile(
              dense: true,
              leading: Icon(
                m.state == pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED
                    ? Icons.radio_button_unchecked
                    : Icons.radio_button_checked,
              ),
              title: Text(
                '${m.name} : (${m.signature.inputs.map(conceptName).join(', ')}) '
                '-> ${conceptName(m.signature.output)}',
              ),
              subtitle: Text(_stateLabel(m.state)),
              selected: selection is MappingSelected && selection.id == m.id.toInt(),
              onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
            ),
        ],
      ),
    );
  }
}

String _stateLabel(pb.AcceptanceState s) => switch (s) {
  pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED => 'declared — named and typed; no definition yet',
  pb.AcceptanceState.ACCEPTANCE_STATE_DEFINED => 'defined — a definition is attached',
  pb.AcceptanceState.ACCEPTANCE_STATE_TYPE_VALID => 'type-valid',
  pb.AcceptanceState.ACCEPTANCE_STATE_TEMPORALLY_VALID => 'temporally valid',
  pb.AcceptanceState.ACCEPTANCE_STATE_CLOCK_CONSISTENT => 'clock-consistent',
  pb.AcceptanceState.ACCEPTANCE_STATE_OUTPUT_COMPLETE => 'output-complete',
  pb.AcceptanceState.ACCEPTANCE_STATE_HARDWARE_FEASIBLE => 'hardware-feasible',
  _ => '',
};

class _Inspector extends ConsumerWidget {
  const _Inspector({required this.state});
  final AppState state;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final project = state.project!;
    final sel = state.editor.selection;
    final dispatch = ref.read(appStoreProvider.notifier).dispatch;

    final body = switch (sel) {
      NoSelection() => const Text('Select a concept or a mapping.'),
      ConceptSelected(:final id) => () {
        final c = project.concepts.firstWhere((c) => c.id.toInt() == id);
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(c.name, style: theme.textTheme.titleMedium),
            Text('identity ${c.id}', style: theme.textTheme.bodySmall),
            const SizedBox(height: 8),
            Text(c.hasRepresentation() ? 'representation: bound' : 'representation: open'),
          ],
        );
      }(),
      MappingSelected(:final id) => () {
        final m = project.mappings.firstWhere((m) => m.id.toInt() == id);
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(m.name, style: theme.textTheme.titleMedium),
            Text('identity ${m.id}', style: theme.textTheme.bodySmall),
            const SizedBox(height: 8),
            Text(_stateLabel(m.state)),
            const SizedBox(height: 8),
            if (m.hasDefinition())
              Text('formula: ${m.definition.formula}')
            else
              FilledButton.tonal(
                onPressed: () async {
                  final src = await showNameDialog(
                    context,
                    title: 'Attach formula',
                    hint: 'clamp(0.2 + 0.8 * Tilt / 60deg, 0, 1)',
                  );
                  if (src != null && src.isNotEmpty) {
                    dispatch(AttachFormulaRequested(mappingId: id, source: src));
                  }
                },
                child: const Text('Attach formula'),
              ),
          ],
        );
      }(),
    };

    return Padding(
      padding: const EdgeInsets.all(12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Inspector', style: theme.textTheme.labelLarge),
          const Divider(),
          body,
          const Spacer(),
          Text('revision r${project.revision}', style: theme.textTheme.bodySmall),
          Text(project.rootPath, style: theme.textTheme.bodySmall, overflow: TextOverflow.ellipsis),
        ],
      ),
    );
  }
}

class _Panel extends StatelessWidget {
  const _Panel({required this.title, required this.child, this.onAdd});
  final String title;
  final Widget child;
  final VoidCallback? onAdd;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 8, 4, 0),
          child: Row(
            children: [
              Text(title, style: Theme.of(context).textTheme.labelLarge),
              const Spacer(),
              IconButton(icon: const Icon(Icons.add), onPressed: onAdd, tooltip: 'Add'),
            ],
          ),
        ),
        Expanded(child: child),
      ],
    );
  }
}
