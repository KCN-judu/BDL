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
import '../protocol/versions.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
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
      StudioPage.deploy => const PlaceholderPage(
        title: 'Deploy',
        body:
            'Board selection, resource allocation, build and flash arrive with the '
            'hardware allocator (roadmap steps L–R).',
      ),
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
              padding: const EdgeInsets.only(left: 6),
              child: Text('— Edited', style: TextStyle(color: t.textTertiary)),
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
          OutlinedButton(
            onPressed: p != null && p.dirty ? () => dispatch(const SaveRequested()) : null,
            child: const Text('Save'),
          ),
          const SizedBox(width: 8),
          OutlinedButton(
            onPressed: p != null ? () => dispatch(const CloseProjectRequested()) : null,
            child: const Text('Close'),
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
          Icon(Icons.error_outline, size: 16, color: t.error),
          const SizedBox(width: 8),
          Expanded(child: Text(error.message)),
          Text(error.code, style: TextStyle(fontSize: 10, color: t.textTertiary)),
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
    final (Color dot, String connLabel) = switch (conn) {
      Disconnected() => (t.textTertiary, 'bdld not connected'),
      Connecting() => (t.open, 'connecting to bdld…'),
      Connected(:final handshake) => (
        t.settled,
        'bdld ${handshake.compilerVersion} · protocol ${formatVersion(handshake.protocolVersion)}',
      ),
      ConnectionFailed(:final reason) => (t.error, reason),
    };
    final undefined = p?.mappings.where((m) => !m.hasDefinition()).length ?? 0;
    final summary = p == null
        ? 'no project'
        : 'r${p.revision} · ${p.concepts.length} concept${p.concepts.length == 1 ? '' : 's'} · '
              '${p.mappings.length} mapping${p.mappings.length == 1 ? '' : 's'}'
              '${undefined > 0 ? ' · $undefined declared without definition' : ''}';
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    return Container(
      height: MacMetrics.statusHeight,
      color: t.window,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          Expanded(
            child: Text(summary, style: small, overflow: TextOverflow.ellipsis),
          ),
          if (state.editor.pendingRequests > 0)
            const Padding(
              padding: EdgeInsets.only(right: 8),
              child: SizedBox(
                width: 10,
                height: 10,
                child: CircularProgressIndicator(strokeWidth: 1.5),
              ),
            ),
          Icon(Icons.circle, size: 7, color: dot),
          const SizedBox(width: 6),
          Text(connLabel, style: small),
        ],
      ),
    );
  }
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
