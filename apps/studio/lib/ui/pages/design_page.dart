import 'package:flutter/material.dart';

import '../../l10n/l10n.dart';
import '../../app/actions.dart';
import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../canvas/canvas_geometry.dart' show SystemSceneInput;
import '../canvas/node_canvas.dart';
import '../inspector.dart';
import '../library.dart';
import '../mac/controls.dart';
import '../mac/tokens.dart';
import '../mac/widgets.dart';
import 'code_pane.dart';
import '../system_inspector.dart' show portKindWord;
import '../system_sheets.dart';

/// Design page: library · node canvas · inspector (Fusion-page arrangement).
class DesignPage extends StatelessWidget {
  const DesignPage({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final project = state.project;
    return Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(
          width: MacMetrics.sidebarWidth,
          child: Library(state: state, dispatch: dispatch),
        ),
        VerticalDivider(width: 1, color: t.hairline),
        Expanded(
          child: project == null
              ? _Empty(dispatch: dispatch)
              : Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    if (state.isSystem)
                      // Keyed for the documentation screenshots (docs/user-guide/screenshots).
                      _ContextBar(
                        key: const ValueKey('context-bar'),
                        state: state,
                        dispatch: dispatch,
                      ),
                    Expanded(child: _views(context, project)),
                  ],
                ),
        ),
        VerticalDivider(width: 1, color: t.hairline),
        SizedBox(
          width: MacMetrics.inspectorWidth,
          child: Inspector(state: state, dispatch: dispatch),
        ),
      ],
    );
  }

  /// Design, Code or Split (ADR-0023 §3): the same project, one canvas
  /// and one editor; side by side they share the selection.
  Widget _views(BuildContext context, pb.ProjectProjection project) {
    final t = MacTokens.of(context);
    final canvas = _canvas(project);
    final code = CodePane(state: state, dispatch: dispatch);
    return switch (state.editor.view) {
      DesignView.design => canvas,
      DesignView.code => code,
      DesignView.split => Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(child: canvas),
          VerticalDivider(width: 1, color: t.hairline),
          Expanded(child: code),
        ],
      ),
    };
  }

  Widget _canvas(pb.ProjectProjection project) {
    final outOfSync = state.editor.sources.outOfSync;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // The text does not build: the graph is the last revision that did
        // (ADR-0023 §5).  Said once, at the top of the graph, in the words
        // the Code view uses.
        if (outOfSync) _OutOfSync(),
        Expanded(
          child: Stack(
            children: [
              NodeCanvas(
                project: project,
                layout: state.editor.layout,
                selection: state.editor.selection,
                dispatch: dispatch,
                statuses: {
                  for (final m in state.contextAnalysis?.mappings ?? const <pb.MappingAnalysis>[])
                    m.id.toInt(): m.status,
                },
                outputStates: {
                  for (final o in state.contextAnalysis?.outputs ?? const <pb.OutputAnalysis>[])
                    o.id.toInt(): o.state,
                },
                refs: {
                  for (final m in state.contextAnalysis?.mappings ?? const <pb.MappingAnalysis>[])
                    m.id.toInt(): [for (final d in m.references) d.toInt()],
                },
                templates: state.templates.toList(),
                sources: state.libraryItems.where((i) => i.category == 'source').toList(),
                recentTemplates: state.editor.recentTemplates,
                renaming: state.editor.renaming,
                canInsert: state.editor.pendingInsert == null,
                context: state.editor.context,
                system: _sceneInput(state),
                components: state.system?.components ?? const [],
                groups: state.groupsInView,
                viewport: state.editor.contextLayout.viewport,
                groupsEnabled: state.isSystem,
              ),
              if (state.editor.pendingBind case final b?)
                PendingBindSheet(state: state, bind: b, dispatch: dispatch),
              if (state.editor.extraction case final x?)
                ExtractionSheet(state: state, extraction: x, dispatch: dispatch),
            ],
          ),
        ),
      ],
    );
  }
}

class _OutOfSync extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      color: t.sidebar,
      padding: const EdgeInsets.fromLTRB(12, 6, 12, 6),
      child: Row(
        children: [
          Icon(Icons.sync_problem_outlined, size: 16, color: t.textSecondary),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              context.l10n.showingTheLastVersionThatBuiltThe,
              style: TextStyle(fontSize: 12, color: t.textPrimary),
            ),
          ),
        ],
      ),
    );
  }
}

class _Empty extends StatelessWidget {
  const _Empty({required this.dispatch});
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      color: t.canvas,
      alignment: Alignment.center,
      child: Text(
        context.l10n.noProjectOpen,
        textAlign: TextAlign.center,
        style: TextStyle(color: t.textTertiary),
      ),
    );
  }
}

/// What the canvas needs of the system: on the system canvas, the
/// instances, bindings, groups and their verdicts; in a component's source,
/// the words for its port-backed relationships.
SystemSceneInput _sceneInput(AppState state) {
  final sys = state.system;
  if (sys == null) return const SystemSceneInput();
  final groups = state.groupsInView;
  final boundaries = [for (final g in groups) ?state.boundary(g.id.toInt())];
  return switch (state.editor.context) {
    SystemContext() => SystemSceneInput(
      system: sys,
      analysis: state.systemAnalysis,
      groups: groups,
      boundaries: boundaries,
      groupBoxes: state.editor.contextLayout.groups,
    ),
    ComponentContext(:final id) => SystemSceneInput(
      groups: groups,
      boundaries: boundaries,
      groupBoxes: state.editor.contextLayout.groups,
      portWords: {
        for (final p in state.component(id)?.ports ?? const <pb.PortView>[])
          p.decl.toInt(): portKindWord(p.kind),
      },
    ),
  };
}

/// Where the designer is: the system, or one component's source ("Editing
/// AdaptiveLamp · used by 3 instances"), with the way back.
class _ContextBar extends StatelessWidget {
  const _ContextBar({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final comp = state.openComponent;
    final used = comp == null
        ? 0
        : (state.system?.instances.where((i) => i.component == comp.id).length ?? 0);
    return Container(
      height: 30,
      color: t.window,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          if (comp == null) ...[
            Text(
              context.l10n.system,
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: t.textPrimary),
            ),
            const SizedBox(width: 8),
            Text(
              '${context.l10n.instancesCount(state.system?.instances.length ?? 0)} · '
              '${context.l10n.componentsCount(state.system?.components.length ?? 0)}',
              style: TextStyle(fontSize: 11, color: t.textTertiary),
            ),
          ] else ...[
            MacButton(
              label: context.l10n.backToSystem,
              onPressed: () => dispatch(const ContextChanged(SystemContext())),
            ),
            const SizedBox(width: 10),
            Text(
              context.l10n.editingComponent(comp.name),
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: t.textPrimary),
            ),
            const SizedBox(width: 8),
            Text(
              used == 0 ? context.l10n.notPlacedYet : context.l10n.usedByInstances(used),
              style: TextStyle(fontSize: 11, color: t.textTertiary),
            ),
            const SizedBox(width: 8),
            // The one sentence that may give way when the bar is narrow.
            Flexible(
              child: Text(
                context.l10n.itsPromiseIsWhatInstancesSeeEdits,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(fontSize: 11, color: t.textTertiary),
              ),
            ),
          ],
          const SizedBox(width: 12),
          const Spacer(),
          // Views of the one project, as Xcode switches editors: a
          // segmented control, the platform's shape for a mode.
          SizedBox(
            width: 200,
            child: MacSegmented<DesignView>(
              value: state.editor.view,
              options: {
                DesignView.design: context.l10n.design,
                DesignView.code: context.l10n.code,
                DesignView.split: context.l10n.split,
              },
              onChanged: (v) => dispatch(DesignViewChanged(v)),
            ),
          ),
        ],
      ),
    );
  }
}
