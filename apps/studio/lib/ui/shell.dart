/// The window: toolbar · banner · page content · status line · page bar.
/// Presentation only; every action goes through the store.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../app/store.dart';
import '../l10n/diagnostics.dart';
import '../l10n/l10n.dart';
import '../platform/desktop.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../protocol/versions.dart';
import 'mac/controls.dart';
import 'close_guard.dart';
import 'mac/tokens.dart';
import 'preferences_sheet.dart';
import 'mac/widgets.dart';
import 'pages/deploy_page.dart';
import 'pages/design_page.dart';
import 'pages/simulate_page.dart';
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
      StudioPage.simulate => SimulatePage(state: state, dispatch: dispatch),
      StudioPage.deploy => DeployPage(state: state, dispatch: dispatch),
      StudioPage.monitor => PlaceholderPage(
        title: context.l10n.monitor,
        body: context.l10n.liveValuesOnTheCanvasArriveWith,
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
        // Every way out of a project goes through the same guard.
        const SingleActivator(LogicalKeyboardKey.keyW, meta: true): () =>
            dispatch(const CloseProjectRequested()),
        const SingleActivator(LogicalKeyboardKey.keyQ, meta: true): () =>
            dispatch(const QuitRequested()),
      },
      child: Focus(
        autofocus: true,
        child: Scaffold(
          backgroundColor: t.window,
          // Two screens, as in Resolve: the project manager until a project
          // is open, then the page workspace.
          body: Stack(
            children: [
              _screen(context, state, dispatch, project, page),
              if (state.editor.closeGuard != null)
                CloseGuardSheet(state: state, dispatch: dispatch),
            ],
          ),
        ),
      ),
    );
  }

  Widget _screen(
    BuildContext context,
    AppState state,
    void Function(AppAction) dispatch,
    pb.ProjectProjection? project,
    Widget page,
  ) {
    final t = MacTokens.of(context);
    return project == null
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
                _Banner(
                  error: err,
                  onDismiss: () => dispatch(const ErrorDismissed()),
                  dispatch: dispatch,
                ),
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
          Text(
            p == null ? context.l10n.bdlStudio : p.name,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          if (p != null && p.dirty)
            Padding(
              padding: const EdgeInsets.only(left: MacMetrics.gap),
              child: Text(context.l10n.edited, style: TextStyle(color: t.textTertiary)),
            ),
          const Spacer(),
          ToolbarButton(
            icon: Icons.undo,
            tooltip: context.l10n.undoTooltip(shortcut('Z')),
            onPressed: p?.canUndo == true ? () => dispatch(const UndoRequested()) : null,
          ),
          ToolbarButton(
            icon: Icons.redo,
            tooltip: context.l10n.redoTooltip(shortcut('Z', shift: true)),
            onPressed: p?.canRedo == true ? () => dispatch(const RedoRequested()) : null,
          ),
          const SizedBox(width: 12),
          MacButton(
            label: context.l10n.save,
            onPressed: p != null && p.dirty ? () => dispatch(const SaveRequested()) : null,
          ),
          const SizedBox(width: 8),
          MacButton(
            label: context.l10n.close,
            onPressed: p != null ? () => dispatch(const CloseProjectRequested()) : null,
          ),
        ],
      ),
    );
  }
}

/// The error code a text project answers a save with when its sources
/// changed on disk since they were loaded (ADR-0020 §10).
const kChangedOnDisk = 'project.changed_on_disk';

class _Banner extends StatelessWidget {
  const _Banner({required this.error, required this.onDismiss, this.dispatch});
  final UserFacingError error;
  final VoidCallback onDismiss;
  final void Function(AppAction)? dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final conflict = error.code == kChangedOnDisk && dispatch != null;
    return Container(
      color: t.sidebar,
      padding: const EdgeInsets.fromLTRB(12, 6, 6, 6),
      child: Row(
        children: [
          // The machine code is the explanation layer's: behind the icon.
          Tooltip(
            message: isStudioWorded(error.code) && error.message.isNotEmpty
                ? '${error.code}\n${error.message}'
                : error.code,
            child: Icon(Icons.error_outline, size: 16, color: t.error),
          ),
          const SizedBox(width: 8),
          Expanded(child: Text(localizedMessage(context.l10n, error.code, error.message))),
          const SizedBox(width: 8),
          // A conflict has two honest answers: take the disk's version
          // (dropping the edits here) or keep this one (writing over the
          // other editor's).  Neither is silent.
          if (conflict) ...[
            TextButton(
              onPressed: () => dispatch!(const ReloadProjectRequested()),
              child: Text(context.l10n.reloadFromDisk),
            ),
            TextButton(
              onPressed: () => dispatch!(const SaveRequested(force: true)),
              child: Text(context.l10n.overwrite),
            ),
          ],
          TextButton(onPressed: onDismiss, child: Text(context.l10n.dismiss)),
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
      facts.add(Text(context.l10n.noProject, style: small));
    } else {
      // *Not yet defined* is exactly what the canvas draws dashed: a Source
      // reads nothing and has no formula, and nothing is missing from it
      // (docs/user-guide/concepts/incomplete-designs.md); it is a count of
      // the design, not an open item.
      final declared = p.mappings.where(state.isDeclared).length;
      final sources = p.mappings.where(state.isSource).length;
      final wrong =
          state.analysis?.mappings
              .where((m) => m.status == pb.MappingStatus.MAPPING_STATUS_INVALID)
              .length ??
          0;
      facts.add(Text(p.dirty ? context.l10n.edited : context.l10n.saved, style: small));
      facts.add(Text(context.l10n.conceptsCount(p.concepts.length), style: small));
      facts.add(Text(context.l10n.mappingsCount(p.mappings.length), style: small));
      if (sources > 0) {
        facts.add(Text(context.l10n.sourcesCount(sources), style: small));
      }
      if (declared > 0) {
        facts.add(
          Text(context.l10n.notYetDefinedCount(declared), style: small.copyWith(color: t.open)),
        );
      }
      if (wrong > 0) {
        facts.add(
          Text(
            context.l10n.definitionsDoNotCheckCount(wrong),
            style: small.copyWith(color: t.error),
          ),
        );
      }
      // Definition drafts: saved with the project, not yet added to it.
      final drafts = state.editor.drafts.values
          .where((d) => d.dirtyAgainst(state.committedDefinition(d.mappingId)))
          .length;
      if (drafts > 0) {
        facts.add(
          Text(context.l10n.definitionsNotAddedCount(drafts), style: small.copyWith(color: t.open)),
        );
      }
      // Whole-design verdicts of the reactive and output passes, when the
      // analysis is the one for this revision (docs/architecture/studio-compiler-integration.md).
      if (state.analysis case final a? when a.revision == p.revision) {
        if (!a.causal) {
          facts.add(Text(context.l10n.notCausal, style: small.copyWith(color: t.error)));
        }
        if (!a.clockConsistent) {
          facts.add(Text(context.l10n.readsAcrossDomains, style: small.copyWith(color: t.error)));
        }
        if (!a.outputComplete) {
          facts.add(Text(context.l10n.outputsIncomplete, style: small.copyWith(color: t.open)));
        }
      }
      // Deployment is target-relative: named with its board, never folded
      // into the design's own state.
      final deploy = state.editor.deploy;
      if (deploy.analysis case final d? when d.revision == p.revision) {
        final board = deploy.target?.name ?? d.target;
        facts.add(switch (d.status) {
          pb.DeploymentStatus.DEPLOYMENT_STATUS_FEASIBLE => Text(
            context.l10n.feasibleOnBoard(board),
            style: small.copyWith(color: t.settled),
          ),
          pb.DeploymentStatus.DEPLOYMENT_STATUS_INFEASIBLE => Text(
            context.l10n.notFeasibleOnBoard(board),
            style: small.copyWith(color: t.error),
          ),
          _ => Text(context.l10n.incompleteOnBoard(board), style: small.copyWith(color: t.open)),
        });
      }
    }

    final (Color dot, List<Widget> connCells) = switch (conn) {
      Disconnected() => (t.textTertiary, [Text(context.l10n.compilerNotConnected, style: small)]),
      Connecting() => (t.open, [Text(context.l10n.connectingToTheCompiler, style: small)]),
      Connected(:final handshake) => (
        t.settled,
        [
          Text(context.l10n.compilerVersion(handshake.compilerVersion), style: small),
          if (!handshake.compatible)
            Text(
              context.l10n.protocolVersion(formatVersion(handshake.protocolVersion)),
              style: dim,
            ),
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

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final pages = [
      (StudioPage.design, Icons.account_tree_outlined, l10n.design),
      (StudioPage.simulate, Icons.show_chart, l10n.simulate),
      (StudioPage.deploy, Icons.memory_outlined, l10n.deploy),
      (StudioPage.monitor, Icons.sensors, l10n.monitor),
    ];
    return Container(
      height: MacMetrics.pageBarHeight,
      color: t.window,
      child: Row(
        children: [
          const SizedBox(width: 8),
          ToolbarButton(
            icon: Icons.grid_view_outlined,
            tooltip: context.l10n.projectManagerClosesThisProject,
            onPressed: projectOpen ? () => dispatch(const CloseProjectRequested()) : null,
          ),
          const Spacer(),
          for (final (page, icon, label) in pages)
            _PageButton(
              icon: icon,
              label: label,
              active: page == current,
              onTap: () => dispatch(PageSelected(page)),
            ),
          const Spacer(),
          ToolbarButton(
            icon: Icons.settings_outlined,
            tooltip: context.l10n.preferencesMenu,
            onPressed: () => showPreferencesSheet(context, dispatch: dispatch),
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
      // At least the English width; a longer label (a translation) widens
      // the tab rather than wrapping or clipping.
      child: ConstrainedBox(
        constraints: const BoxConstraints(minWidth: 96),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(icon, size: 18, color: color),
              const SizedBox(height: 2),
              Text(
                label,
                softWrap: false,
                style: TextStyle(
                  fontSize: 10,
                  color: color,
                  fontWeight: active ? FontWeight.w600 : FontWeight.w400,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
