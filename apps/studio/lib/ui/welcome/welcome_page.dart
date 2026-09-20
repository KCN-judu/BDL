/// The launcher shown when no project is open — DaVinci's Project Manager,
/// laid out like VS Code's welcome page: hero and Start on the left,
/// Recent on the right.  Presentation only.
library;

import 'dart:io';

import 'package:flutter/material.dart';

import '../../app/actions.dart';
import '../../app/state.dart';
import '../../platform/desktop.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../../protocol/versions.dart';
import '../../l10n/l10n.dart';
import '../dialogs.dart';
import '../preferences_sheet.dart';
import '../mac/interactive.dart';
import '../mac/tokens.dart';
import '../mac/widgets.dart';
import 'hero_mark.dart';

class WelcomePage extends StatelessWidget {
  const WelcomePage({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final connected = state.connection is Connected;
    return Container(
      color: t.window,
      alignment: Alignment.center,
      child: LayoutBuilder(
        builder: (context, c) {
          // Wide: hero + Start on the left, Recent on the right.
          // Narrow (< 760 pt): one column, Recent below.
          final wide = c.maxWidth >= 760;
          final pad = c.maxWidth < 600 ? 20.0 : 40.0;
          final left = Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            mainAxisSize: MainAxisSize.min,
            children: [
              // The hero keeps a fixed aspect ratio and fills the column.
              AspectRatio(aspectRatio: 1.15, child: HeroMark(version: kStudioVersion)),
              const SizedBox(height: 24),
              _Start(
                connected: connected,
                dispatch: dispatch,
                templates: state.editor.deploy.templates,
                pathFallback: state.editor.pickerUnavailable,
              ),
            ],
          );
          final recent = _Recent(state: state, dispatch: dispatch);
          if (wide) {
            return ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1360, maxHeight: 820),
              child: Padding(
                padding: EdgeInsets.all(pad),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Expanded(flex: 11, child: SingleChildScrollView(child: left)),
                    SizedBox(width: pad),
                    Expanded(flex: 9, child: recent),
                  ],
                ),
              ),
            );
          }
          return SingleChildScrollView(
            padding: EdgeInsets.all(pad),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 560),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                mainAxisSize: MainAxisSize.min,
                children: [
                  left,
                  const SizedBox(height: 28),
                  SizedBox(height: 260, child: recent),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

class _Start extends StatelessWidget {
  const _Start({
    required this.connected,
    required this.dispatch,
    this.templates = const [],
    this.pathFallback = false,
  });

  /// The designs a new project can start from, as bdld lists them.
  final List<pb.TemplateView> templates;
  final bool connected;
  final void Function(AppAction) dispatch;

  /// Offer typed paths when the OS dialog cannot be shown.
  final bool pathFallback;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(context.l10n.start, style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 8),
        MacLink(
          icon: Icons.add_box_outlined,
          label: context.l10n.newProject,
          shortcut: shortcut('N'),
          enabled: connected,
          onTap: () => dispatch(const NewProjectPickRequested()),
        ),
        MacLink(
          icon: Icons.folder_open_outlined,
          label: context.l10n.openProject,
          shortcut: shortcut('O'),
          enabled: connected,
          onTap: () => dispatch(const OpenProjectPickRequested()),
        ),
        if (pathFallback) ...[
          MacLink(
            icon: Icons.keyboard_outlined,
            label: context.l10n.openByPath,
            enabled: connected,
            onTap: () async {
              final path = await showPathSheet(context, title: context.l10n.openProjectByPath);
              if (path != null && path.isNotEmpty) dispatch(OpenProjectRequested(path));
            },
          ),
          MacLink(
            icon: Icons.keyboard_outlined,
            label: context.l10n.newAtPath,
            enabled: connected,
            onTap: () async {
              final path = await showPathSheet(context, title: context.l10n.createProjectAtPath);
              if (path != null && path.isNotEmpty) {
                dispatch(
                  NewProjectRequested(rootPath: path, name: path.split(RegExp(r'[/\\]')).last),
                );
              }
            },
          ),
        ],
        MacLink(
          icon: Icons.language_outlined,
          label: context.l10n.preferencesMenu,
          onTap: () => showPreferencesSheet(context, dispatch: dispatch),
        ),
        if (templates.isNotEmpty) ...[
          const SizedBox(height: MacMetrics.gapGroup),
          Text(context.l10n.demos, style: Theme.of(context).textTheme.titleSmall),
          const SizedBox(height: 8),
          // A demo is a template: the same New Project, with a design in
          // it (and, for the wired one, its deployment).
          for (final tpl in templates)
            Tooltip(
              message: tpl.description,
              waitDuration: const Duration(milliseconds: 600),
              child: MacLink(
                key: ValueKey('template-${tpl.id}'),
                icon: Icons.memory_outlined,
                label: tpl.displayName,
                enabled: connected,
                onTap: () => dispatch(NewProjectPickRequested(template: tpl.id)),
              ),
            ),
        ],
        if (!connected)
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: Text(
              context.l10n.waitingForTheCompilerTheStatusLine,
              style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
            ),
          ),
      ],
    );
  }
}

class _Recent extends StatelessWidget {
  const _Recent({required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final connected = state.connection is Connected;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(context.l10n.recent, style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 8),
        Expanded(
          child: state.recent.isEmpty
              ? Text(
                  context.l10n.projectsYouOpenWillAppearHere,
                  style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
                )
              : ListView(
                  children: [
                    for (final r in state.recent)
                      _RecentRow(
                        project: r,
                        enabled: connected,
                        onOpen: () => dispatch(OpenProjectRequested(r.path)),
                        onRemove: () => dispatch(RemoveRecentRequested(r.path)),
                      ),
                  ],
                ),
        ),
      ],
    );
  }
}

class _RecentRow extends StatefulWidget {
  const _RecentRow({
    required this.project,
    required this.enabled,
    required this.onOpen,
    required this.onRemove,
  });
  final RecentProject project;
  final bool enabled;
  final VoidCallback onOpen;
  final VoidCallback onRemove;

  @override
  State<_RecentRow> createState() => _RecentRowState();
}

class _RecentRowState extends State<_RecentRow> {
  bool _hover = false;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final r = widget.project;
    final missing = !Directory(r.path).existsSync();
    final canOpen = widget.enabled && !missing;
    return MacInteractive(
      onTap: canOpen ? widget.onOpen : null,
      radius: 6,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
      onHoverChanged: (v) => setState(() => _hover = v),
      child: Row(
        children: [
          Icon(
            missing ? Icons.folder_off_outlined : Icons.folder_outlined,
            size: 18,
            color: missing ? t.textTertiary : t.accent,
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  r.name,
                  style: TextStyle(
                    fontSize: MacType.body,
                    fontWeight: FontWeight.w500,
                    color: missing ? t.textTertiary : t.textPrimary,
                  ),
                ),
                Row(
                  spacing: MacMetrics.gapGroup,
                  children: [
                    Flexible(
                      child: Text(
                        r.path,
                        style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    if (missing)
                      Text(
                        context.l10n.notFound,
                        style: TextStyle(fontSize: MacType.secondary, color: t.open),
                      ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(width: MacMetrics.gutter),
          // the "when" column: fixed width, right-aligned, tabular figures
          SizedBox(
            width: 76,
            child: _hover || missing
                ? Align(
                    alignment: Alignment.centerRight,
                    child: IconButton(
                      icon: const Icon(Icons.close, size: 14),
                      tooltip: context.l10n.removeFromRecent,
                      onPressed: widget.onRemove,
                      constraints: const BoxConstraints.tightFor(width: 22, height: 22),
                    ),
                  )
                : Text(
                    relativeTime(r.lastOpened, l10n: context.l10n),
                    textAlign: TextAlign.right,
                    style: TextStyle(
                      fontSize: MacType.secondary,
                      color: t.textTertiary,
                      fontFeatures: kTabularFigures,
                    ),
                  ),
          ),
        ],
      ),
    );
  }
}

/// "just now", "3 h ago", "yesterday", "5 d ago", "2026-08-01".
String relativeTime(DateTime when, {DateTime? now, AppLocalizations? l10n}) {
  l10n ??= kEnglish;
  final d = (now ?? DateTime.now()).difference(when);
  if (d.inMinutes < 1) return l10n.justNow;
  if (d.inHours < 1) return l10n.minutesAgo(d.inMinutes);
  if (d.inDays < 1) return l10n.hoursAgo(d.inHours);
  if (d.inDays == 1) return l10n.yesterday;
  if (d.inDays < 30) return l10n.daysAgo(d.inDays);
  final local = when.toLocal();
  return '${local.year}-${local.month.toString().padLeft(2, '0')}-${local.day.toString().padLeft(2, '0')}';
}
