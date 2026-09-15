/// The window: toolbar · banner · page content · status line · page bar.
/// Presentation only; every action goes through the store.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../app/store.dart';
import '../platform/desktop.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../protocol/versions.dart';
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'pages/deploy_page.dart';
import 'pages/design_page.dart';
import 'pages/placeholder_page.dart';
import 'welcome/welcome_page.dart';

class StudioShell extends ConsumerWidget {
  const StudioShell({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(appStoreProvider);
    final dispatch = ref.read(appStoreProvider.notifier).dispatch;
    final t = MacTokens.of(context);
    final project = state.project;

    final page = switch (state.editor.page) {
      StudioPage.design => DesignPage(state: state, dispatch: dispatch),
      StudioPage.simulate => const PlaceholderPage(
        title: 'Simulate',
        body:
            'Input traces, logical ticks and value plots arrive with the host simulator '
            '(roadmap step I).',
      ),
      StudioPage.deploy => DeployPage(state: state, dispatch: dispatch),
      StudioPage.monitor => const PlaceholderPage(
        title: 'Monitor',
        body: 'Live values on the canvas arrive with telemetry (roadmap step S).',
      ),
    };

    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.keyS, meta: true): () =>
            dispatch(const SaveRequested()),
        const SingleActivator(LogicalKeyboardKey.keyZ, meta: true): () =>
            dispatch(const UndoRequested()),
        const SingleActivator(LogicalKeyboardKey.keyZ, meta: true, shift: true): () =>
            dispatch(const RedoRequested()),
        const SingleActivator(LogicalKeyboardKey.digit1, meta: true): () =>
            dispatch(const PageSelected(StudioPage.design)),
        const SingleActivator(LogicalKeyboardKey.digit2, meta: true): () =>
            dispatch(const PageSelected(StudioPage.simulate)),
        const SingleActivator(LogicalKeyboardKey.digit3, meta: true): () =>
            dispatch(const PageSelected(StudioPage.deploy)),
        const SingleActivator(LogicalKeyboardKey.digit4, meta: true): () =>
            dispatch(const PageSelected(StudioPage.monitor)),
      },
      child: Focus(
        autofocus: true,
        child: Scaffold(
          backgroundColor: t.window,
          // Two screens, as in Resolve: the project manager until a project
          // is open, then the page workspace.
          body: project == null
              ? Column(
                  children: [
                    if (state.editor.lastError case final err?)
                      _Banner(error: err, onDismiss: () => dispatch(const ErrorDismissed())),
                    Expanded(
                      child: WelcomePage(state: state, dispatch: dispatch),
                    ),
                    Divider(color: t.hairline),
                    _StatusLine(state: state),
                  ],
                )
              : Column(
                  children: [
                    _Toolbar(state: state, dispatch: dispatch),
                    Divider(color: t.hairline),
                    if (state.editor.lastError case final err?)
                      _Banner(error: err, onDismiss: () => dispatch(const ErrorDismissed())),
                    Expanded(child: page),
                    Divider(color: t.hairline),
                    _StatusLine(state: state),
                    Divider(color: t.hairline),
                    _PageBar(
                      current: state.editor.page,
                      projectOpen: true,
                      connected: state.connection is Connected,
                      dispatch: dispatch,
                    ),
                  ],
                ),
        ),
      ),
    );
  }
}

class _Toolbar extends StatelessWidget {
  const _Toolbar({required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project;
    return Container(
      height: MacMetrics.toolbarHeight,
      color: t.window,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          // Room for the macOS traffic lights when the title bar is hidden later.
          if (isMacOS) const SizedBox(width: 64),
          Text(p == null ? 'BDL Studio' : p.name, style: Theme.of(context).textTheme.titleMedium),
          if (p != null && p.dirty)
            Padding(
              padding: const EdgeInsets.only(left: MacMetrics.gap),
              child: Text('Edited', style: TextStyle(color: t.textTertiary)),
            ),
          const Spacer(),
          ToolbarButton(
            icon: Icons.undo,
            tooltip: 'Undo (${shortcut('Z')})',
            onPressed: p?.canUndo == true ? () => dispatch(const UndoRequested()) : null,
          ),
          ToolbarButton(
            icon: Icons.redo,
            tooltip: 'Redo (${shortcut('Z', shift: true)})',
            onPressed: p?.canRedo == true ? () => dispatch(const RedoRequested()) : null,
          ),
          const SizedBox(width: 12),
          MacButton(
            label: 'Save',
            onPressed: p != null && p.dirty ? () => dispatch(const SaveRequested()) : null,
          ),
          const SizedBox(width: 8),
          MacButton(
            label: 'Close',
            onPressed: p != null ? () => dispatch(const CloseProjectRequested()) : null,
          ),
        ],
      ),
    );
  }
}

class _Banner extends StatelessWidget {
  const _Banner({required this.error, required this.onDismiss});
  final UserFacingError error;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      color: t.sidebar,
      padding: const EdgeInsets.fromLTRB(12, 6, 6, 6),
      child: Row(
        children: [
          // The machine code is the explanation layer's: behind the icon.
          Tooltip(
            message: error.code,
            child: Icon(Icons.error_outline, size: 16, color: t.error),
          ),
          const SizedBox(width: 8),
          Expanded(child: Text(error.message)),
          const SizedBox(width: 8),
          TextButton(onPressed: onDismiss, child: const Text('Dismiss')),
        ],
      ),
    );
  }
}

class _StatusLine extends StatelessWidget {
  const _StatusLine({required this.state});
  final AppState state;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project;
    final conn = state.connection;
    final small = TextStyle(fontSize: 11, color: t.textSecondary, fontFeatures: kTabularFigures);
    final dim = TextStyle(fontSize: 11, color: t.textTertiary);

    // Facts are cells separated by whitespace, never by punctuation.  The
    // leading cell is what a glance is for: is my work saved.  Counts are an
    // overview of what the canvas already shows; the compiler's revision
    // counter and protocol version are not designer facts (Explain has the
    // revision).
    final facts = <Widget>[];
    if (p == null) {
      facts.add(Text('No project', style: small));
    } else {
      final declared = p.mappings.where((m) => !m.hasDefinition()).length;
      final wrong =
          state.analysis?.mappings
              .where((m) => m.status == pb.MappingStatus.MAPPING_STATUS_INVALID)
              .length ??
          0;
      facts.add(Text(p.dirty ? 'Edited' : 'Saved', style: small));
      facts.add(_Fact(count: p.concepts.length, noun: 'concept', style: small));
      facts.add(_Fact(count: p.mappings.length, noun: 'mapping', style: small));
      if (declared > 0) {
        facts.add(Text('$declared not yet defined', style: small.copyWith(color: t.open)));
      }
      if (wrong > 0) {
        facts.add(
          Text(
            wrong == 1 ? '1 definition does not check' : '$wrong definitions do not check',
            style: small.copyWith(color: t.error),
          ),
        );
      }
      // Unsaved definition drafts: Studio's, not the project's dirty flag.
      final drafts = state.editor.drafts.values
          .where((d) => d.dirtyAgainst(state.committedDefinition(d.mappingId)))
          .length;
      if (drafts > 0) {
        facts.add(
          Text(
            drafts == 1 ? '1 unsaved definition' : '$drafts unsaved definitions',
            style: small.copyWith(color: t.open),
          ),
        );
      }
      // Whole-design verdicts of the reactive and output passes, when the
      // analysis is the one for this revision (docs/STUDIO_COMPILER_INTEGRATION.md).
      if (state.analysis case final a? when a.revision == p.revision) {
        if (!a.causal) facts.add(Text('not causal', style: small.copyWith(color: t.error)));
        if (!a.clockConsistent) {
          facts.add(Text('reads across domains', style: small.copyWith(color: t.error)));
        }
        if (!a.outputComplete) {
          facts.add(Text('outputs incomplete', style: small.copyWith(color: t.open)));
        }
      }
    }

    final (Color dot, List<Widget> connCells) = switch (conn) {
      Disconnected() => (t.textTertiary, [Text('Compiler not connected', style: small)]),
      Connecting() => (t.open, [Text('Connecting to the compiler', style: small)]),
      Connected(:final handshake) => (
        t.settled,
        [
          Text('Compiler ${handshake.compilerVersion}', style: small),
          if (!handshake.compatible)
            Text('protocol ${formatVersion(handshake.protocolVersion)}', style: dim),
        ],
      ),
      ConnectionFailed(:final reason) => (t.error, [Text(reason, style: small)]),
    };

    return Container(
      height: MacMetrics.statusHeight,
      color: t.window,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        spacing: MacMetrics.gapGroup,
        children: [
          Expanded(
            // Facts never wrap; when the window is too narrow for them all,
            // the row scrolls rather than clips or overflows.
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(spacing: MacMetrics.gapGroup, children: facts),
            ),
          ),
          if (state.editor.pendingRequests > 0)
            const SizedBox(
              width: 10,
              height: 10,
              child: CircularProgressIndicator(strokeWidth: 1.5),
            ),
          Row(
            spacing: MacMetrics.gapGroup,
            children: [
              Row(
                spacing: MacMetrics.gapTight + 2,
                children: [
                  Icon(Icons.circle, size: 7, color: dot),
                  connCells.first,
                ],
              ),
              ...connCells.skip(1),
            ],
          ),
        ],
      ),
    );
  }
}

/// "4 concepts": the number in tabular figures, the noun pluralised.
class _Fact extends StatelessWidget {
  const _Fact({required this.count, required this.noun, required this.style});
  final int count;
  final String noun;
  final TextStyle style;

  @override
  Widget build(BuildContext context) => Text('$count $noun${count == 1 ? '' : 's'}', style: style);
}

/// Resolve's page bar: pages in workflow order, project manager left,
/// settings right.
class _PageBar extends StatelessWidget {
  const _PageBar({
    required this.current,
    required this.projectOpen,
    required this.connected,
    required this.dispatch,
  });
  final StudioPage current;
  final bool projectOpen;
  final bool connected;
  final void Function(AppAction) dispatch;

  static const _pages = [
    (StudioPage.design, Icons.account_tree_outlined, 'Design'),
    (StudioPage.simulate, Icons.show_chart, 'Simulate'),
    (StudioPage.deploy, Icons.memory_outlined, 'Deploy'),
    (StudioPage.monitor, Icons.sensors, 'Monitor'),
  ];

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      height: MacMetrics.pageBarHeight,
      color: t.window,
      child: Row(
        children: [
          const SizedBox(width: 8),
          ToolbarButton(
            icon: Icons.grid_view_outlined,
            tooltip: 'Project manager (closes this project)',
            onPressed: projectOpen ? () => dispatch(const CloseProjectRequested()) : null,
          ),
          const Spacer(),
          for (final (page, icon, label) in _pages)
            _PageButton(
              icon: icon,
              label: label,
              active: page == current,
              onTap: () => dispatch(PageSelected(page)),
            ),
          const Spacer(),
          ToolbarButton(
            icon: Icons.settings_outlined,
            tooltip: 'Project settings',
            onPressed: connected && !projectOpen ? null : () => dispatch(const ConnectRequested()),
          ),
          const SizedBox(width: 8),
        ],
      ),
    );
  }
}

class _PageButton extends StatelessWidget {
  const _PageButton({
    required this.icon,
    required this.label,
    required this.active,
    required this.onTap,
  });
  final IconData icon;
  final String label;
  final bool active;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final color = active ? t.accent : t.textSecondary;
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(6),
      child: SizedBox(
        width: 96,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, size: 18, color: color),
            const SizedBox(height: 2),
            Text(
              label,
              style: TextStyle(
                fontSize: 10,
                color: color,
                fontWeight: active ? FontWeight.w600 : FontWeight.w400,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
