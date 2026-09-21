/// Left sidebar: the project's objects by kind, with "+" per section.
/// Selecting here selects on the canvas (one selection, one state).
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show HardwareKeyboard;

import '../platform/desktop.dart' show primaryModifierIsControl;

import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/state.dart';
import '../app/system.dart' show freshGroupName;
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/concept_glyphs.dart';
import 'library_panel.dart';
import 'dialogs.dart';
import 'mac/widgets.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';

/// Left sidebar, two tabs: **Project** — the project's objects by kind —
/// and **Library** — the concept libraries to insert from
/// (`concept_library_panel.dart`).
class Library extends StatelessWidget {
  const Library({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      color: t.sidebar,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(MacMetrics.gap, MacMetrics.gap, MacMetrics.gap, 0),
            child: MacSegmented<SidebarTab>(
              value: state.editor.sidebar,
              options: {
                SidebarTab.project: context.l10n.project,
                SidebarTab.library: context.l10n.library,
              },
              onChanged: (tab) => dispatch(SidebarTabSelected(tab)),
            ),
          ),
          Expanded(
            child: switch (state.editor.sidebar) {
              SidebarTab.project => _ProjectObjects(state: state, dispatch: dispatch),
              SidebarTab.library => LibraryPanel(state: state, dispatch: dispatch),
            },
          ),
        ],
      ),
    );
  }
}

class _ProjectObjects extends StatelessWidget {
  const _ProjectObjects({required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  /// The canvas nodes the list shows, in list order: the range a ⇧-click
  /// selects is contiguous in this order.
  List<NodeRef> _order() {
    final p = state.project!;
    return [
      for (final c in p.concepts) NodeRef.concept(c.id.toInt()),
      for (final m in p.mappings) NodeRef.mapping(m.id.toInt()),
      for (final o in p.outputs) NodeRef.output(o.id.toInt()),
      if (state.isSystem && state.editor.context is SystemContext)
        for (final i in state.system!.instances) NodeRef.instance(i.id.toInt()),
      if (state.isSystem)
        for (final g in state.groupsInView) NodeRef.group(g.id.toInt()),
    ];
  }

  /// Desktop list selection: a plain click selects the row alone; the
  /// primary modifier (⌘ / Ctrl) toggles it; ⇧ selects the contiguous range
  /// from the active object (the anchor) to it.  The anchor is the object
  /// the last plain or modifier click named, never the set's order.
  void _rowTap(NodeRef ref) {
    final sel = state.editor.selection;
    final current = selectedNodes(sel);
    final anchor = activeNode(sel);
    final primary = primaryModifierIsControl
        ? HardwareKeyboard.instance.isControlPressed
        : HardwareKeyboard.instance.isMetaPressed;
    final shift = HardwareKeyboard.instance.isShiftPressed;
    if (shift && anchor != null) {
      final order = _order();
      final a = order.indexOf(anchor);
      final b = order.indexOf(ref);
      if (a >= 0 && b >= 0) {
        final range = order.sublist(a < b ? a : b, (a < b ? b : a) + 1).toSet();
        final next = primary ? current.union(range) : range;
        dispatch(SelectionChanged(selectionOfNodes(next, active: anchor)));
        return;
      }
    }
    if (primary) {
      final next = {...current};
      if (next.remove(ref)) {
        dispatch(SelectionChanged(selectionOfNodes(next, active: anchor == ref ? null : anchor)));
      } else {
        dispatch(SelectionChanged(selectionOfNodes(next..add(ref), active: ref)));
      }
      return;
    }
    dispatch(SelectionChanged(singleSelection(ref)));
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project;
    final sel = state.editor.selection;
    final selectedSet = selectedNodes(sel);
    final active = activeNode(sel);
    return Container(
      color: t.sidebar,
      child: p == null
          ? const SizedBox.shrink()
          : ListView(
              padding: const EdgeInsets.symmetric(vertical: 4),
              children: [
                _Section(
                  title: context.l10n.concepts,
                  // the concept sheet, with the category to choose there
                  onAdd: () => dispatch(const NewConceptRequested()),
                ),
                // a concept is a template: its row drags onto the canvas to
                // make a block of it there (ADR-0044)
                for (final c in p.concepts)
                  _Row(
                    glyph: SocketGlyph.of(c, t),
                    title: c.name,
                    selected: selectedSet.contains(NodeRef.concept(c.id.toInt())),
                    active: active == NodeRef.concept(c.id.toInt()) && selectedSet.length > 1,
                    onTap: () => _rowTap(NodeRef.concept(c.id.toInt())),
                    drag: ConceptDrag(c.id.toInt()),
                  ),
                _Section(
                  title: context.l10n.mappings,
                  onAdd: p.concepts.isEmpty
                      ? null
                      : () async {
                          final r = await showNewMappingSheet(context, p.concepts);
                          if (r != null && r.name.isNotEmpty) {
                            dispatch(
                              CreateMappingRequested(
                                name: r.name,
                                inputs: r.inputs,
                                output: r.output,
                              ),
                            );
                          }
                        },
                ),
                for (final m in p.mappings)
                  _Row(
                    glyph: MappingGlyph(
                      declared: state.isDeclared(m),
                      wrong:
                          state.mappingAnalysis(m.id.toInt())?.status ==
                          pb.MappingStatus.MAPPING_STATUS_INVALID,
                      source: state.isSource(m),
                    ),
                    title: m.name,
                    selected: selectedSet.contains(NodeRef.mapping(m.id.toInt())),
                    active: active == NodeRef.mapping(m.id.toInt()) && selectedSet.length > 1,
                    onTap: () => _rowTap(NodeRef.mapping(m.id.toInt())),
                  ),
                _Section(
                  title: context.l10n.timingDomains,
                  onAdd: () async {
                    final name = await showNewClockSheet(context, p.clocks);
                    if (name != null) dispatch(CreateClockDomainRequested(name));
                  },
                ),
                for (final c in p.clocks)
                  _ClockRow(
                    clock: c,
                    inUse:
                        p.mappings.any((m) => m.hasClockId() && m.clockId == c.id) ||
                        p.outputs.any((o) => o.hasClockId() && o.clockId == c.id),
                    dispatch: dispatch,
                  ),
                _Section(
                  title: context.l10n.outputs,
                  onAdd: p.concepts.isEmpty
                      ? null
                      : () async {
                          final r = await showNewOutputSheet(context, p.concepts, p.clocks);
                          if (r != null) {
                            dispatch(
                              CreateOutputRequested(
                                name: r.name,
                                accepts: r.accepts,
                                clockId: r.clockId,
                                required: r.required,
                              ),
                            );
                          }
                        },
                ),
                for (final o in p.outputs)
                  _Row(
                    glyph: OutputGlyph(
                      state: state.outputAnalysis(o.id.toInt())?.state,
                      open: !o.hasClockId(),
                    ),
                    title: o.name,
                    selected: selectedSet.contains(NodeRef.output(o.id.toInt())),
                    active: active == NodeRef.output(o.id.toInt()) && selectedSet.length > 1,
                    onTap: () => _rowTap(NodeRef.output(o.id.toInt())),
                  ),
                _Section(title: context.l10n.contexts),
                if (state.isSystem) ...[
                  // ---- the system's own objects (docs/architecture/studio-ui.md §11) ----
                  _Section(
                    title: context.l10n.components,
                    onAdd: () async {
                      final name = await showNameSheet(
                        context,
                        title: context.l10n.newComponent,
                        subtitle: context.l10n.aReusableBehaviourWithAPromiseIts,
                        hint: context.l10n.adaptivelamp,
                      );
                      if (name != null && name.isNotEmpty) {
                        dispatch(CreateComponentRequested(name: name));
                      }
                    },
                  ),
                  for (final c in state.system!.components)
                    _Row(
                      glyph: _ComponentGlyph(
                        broken:
                            state.systemAnalysis?.components
                                .where((x) => x.id == c.id)
                                .firstOrNull
                                ?.realizes ==
                            false,
                        editing: state.editor.context == ComponentContext(c.id.toInt()),
                      ),
                      title: c.name,
                      selected: sel is ComponentSelected && sel.id == c.id.toInt(),
                      onTap: () => dispatch(SelectionChanged(ComponentSelected(c.id.toInt()))),
                    ),
                  if (state.editor.context is SystemContext) ...[
                    _Section(title: context.l10n.instances),
                    for (final i in state.system!.instances)
                      _Row(
                        glyph: _InstanceGlyph(),
                        title: i.name,
                        selected: selectedSet.contains(NodeRef.instance(i.id.toInt())),
                        active: active == NodeRef.instance(i.id.toInt()) && selectedSet.length > 1,
                        onTap: () => _rowTap(NodeRef.instance(i.id.toInt())),
                      ),
                  ],
                  // Behaviours of the design on screen: the system's own,
                  // or the open component's.
                  _Section(
                    title: context.l10n.behaviors,
                    onAdd: () => dispatch(
                      CreateGroupRequested(name: freshGroupName(state), renameAfter: true),
                    ),
                  ),
                  for (final g in state.groupsInView)
                    _Row(
                      glyph: _GroupGlyph(
                        collapsed:
                            state.editor.contextLayout.groups[g.id.toInt()]?.collapsed ?? false,
                      ),
                      title: g.name,
                      selected: selectedSet.contains(NodeRef.group(g.id.toInt())),
                      active: active == NodeRef.group(g.id.toInt()) && selectedSet.length > 1,
                      onTap: () => _rowTap(NodeRef.group(g.id.toInt())),
                    ),
                ] else
                  _Section(title: context.l10n.components),
              ],
            ),
    );
  }
}

class _Section extends StatelessWidget {
  const _Section({required this.title, this.onAdd});
  final String title;
  final VoidCallback? onAdd;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 10, 4, 2),
      child: Row(
        children: [
          Text(title, style: Theme.of(context).textTheme.titleSmall),
          const Spacer(),
          IconButton(
            icon: const Icon(Icons.add, size: 14),
            onPressed: onAdd,
            tooltip: context.l10n.add,
          ),
        ],
      ),
    );
  }
}

/// One object in the library: its glyph (the same mark as on the canvas)
/// and its name.  State is in the glyph — hollow, dashed, red mark — not in
/// a trailing word.
class _Row extends StatelessWidget {
  const _Row({
    required this.glyph,
    required this.title,
    required this.selected,
    required this.onTap,
    this.active = false,
    this.drag,
  });
  final Widget glyph;
  final String title;
  final bool selected;

  /// The active object of a multi-selection: semibold, the same tint —
  /// weight ranks, colour does not.
  final bool active;
  final VoidCallback onTap;

  /// What the row is when dragged onto the canvas (a concept: a block of
  /// it at the drop point); `null` for a row that stays put.
  final ConceptDrag? drag;

  @override
  Widget build(BuildContext context) {
    final row = _rowBody(context);
    final d = drag;
    if (d == null) return row;
    return Draggable<ConceptDrag>(
      data: d,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: DragFeedback(
        child: Row(
          spacing: MacMetrics.gap,
          children: [
            SizedBox(width: 14, child: Center(child: glyph)),
            Text(title),
          ],
        ),
      ),
      child: row,
    );
  }

  Widget _rowBody(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 6),
      child: MacInteractive(
        onTap: onTap,
        selected: selected,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: SizedBox(
          height: MacMetrics.rowHeight,
          child: Row(
            spacing: MacMetrics.gap,
            children: [
              SizedBox(width: 14, child: Center(child: glyph)),
              Expanded(
                child: Text(
                  title,
                  overflow: TextOverflow.ellipsis,
                  style: active ? const TextStyle(fontWeight: FontWeight.w600) : null,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// A timing domain: a name, renamed in place, deleted only while nothing
/// updates in it.  A domain is when things happen, never how often.
class _ClockRow extends StatelessWidget {
  const _ClockRow({required this.clock, required this.inUse, required this.dispatch});
  final pb.ClockView clock;
  final bool inUse;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = clock.id.toInt();
    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 0, 6, 2),
      child: Row(
        spacing: MacMetrics.gapTight,
        children: [
          Text(
            '↻',
            style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
          ),
          Expanded(
            child: CommitTextField(
              value: clock.name,
              onCommit: (v) => dispatch(RenameClockDomainRequested(id: id, name: v)),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.close, size: 12),
            onPressed: inUse ? null : () => dispatch(DeleteClockDomainRequested(id)),
            tooltip: inUse ? context.l10n.stillInUse : context.l10n.deleteNamed(clock.name),
            constraints: const BoxConstraints.tightFor(width: 20, height: 20),
            padding: EdgeInsets.zero,
          ),
        ],
      ),
    );
  }
}

/// A component: a rounded box with two port ticks; a red mark when its
/// source no longer keeps its promise; filled when its source is open.
class _ComponentGlyph extends StatelessWidget {
  const _ComponentGlyph({required this.broken, required this.editing});
  final bool broken;
  final bool editing;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return SizedBox(
      width: 14,
      height: 12,
      child: CustomPaint(
        painter: _BoxGlyphPainter(
          fill: editing ? t.textSecondary : null,
          stroke: t.textSecondary,
          mark: broken ? t.error : null,
          ticks: true,
        ),
      ),
    );
  }
}

class _InstanceGlyph extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return SizedBox(
      width: 14,
      height: 12,
      child: CustomPaint(
        painter: _BoxGlyphPainter(fill: null, stroke: t.textSecondary, mark: null, ticks: true),
      ),
    );
  }
}

/// A group: a dashed region (expanded) or a solid box (collapsed).
class _GroupGlyph extends StatelessWidget {
  const _GroupGlyph({required this.collapsed});
  final bool collapsed;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return SizedBox(
      width: 14,
      height: 12,
      child: CustomPaint(
        painter: _BoxGlyphPainter(
          fill: null,
          stroke: t.textSecondary,
          mark: null,
          ticks: false,
          dashed: !collapsed,
        ),
      ),
    );
  }
}

class _BoxGlyphPainter extends CustomPainter {
  _BoxGlyphPainter({
    required this.fill,
    required this.stroke,
    required this.mark,
    required this.ticks,
    this.dashed = false,
  });
  final Color? fill;
  final Color stroke;
  final Color? mark;
  final bool ticks;
  final bool dashed;

  @override
  void paint(Canvas canvas, Size size) {
    final r = RRect.fromRectAndRadius(
      Rect.fromLTWH(2, 1, size.width - 4, size.height - 2),
      const Radius.circular(2.5),
    );
    if (fill != null) canvas.drawRRect(r, Paint()..color = fill!.withValues(alpha: 0.35));
    final p = Paint()
      ..color = stroke
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    if (dashed) {
      final path = Path()..addRRect(r);
      for (final m in path.computeMetrics()) {
        var d = 0.0;
        while (d < m.length) {
          canvas.drawPath(m.extractPath(d, d + 2.5), p);
          d += 4.5;
        }
      }
    } else {
      canvas.drawRRect(r, p);
    }
    if (ticks) {
      canvas.drawLine(Offset(0, size.height / 2), Offset(2, size.height / 2), p);
      canvas.drawLine(
        Offset(size.width - 2, size.height / 2),
        Offset(size.width, size.height / 2),
        p,
      );
    }
    if (mark != null) {
      canvas.drawCircle(Offset(size.width - 2, 2), 2.2, Paint()..color = mark!);
    }
  }

  @override
  bool shouldRepaint(_BoxGlyphPainter old) =>
      old.fill != fill || old.stroke != stroke || old.mark != mark || old.dashed != dashed;
}
