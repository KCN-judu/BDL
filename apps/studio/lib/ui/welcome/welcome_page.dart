/// The launcher shown when no project is open — DaVinci's Project Manager,
/// laid out like VS Code's welcome page: the hero and Start (New, Open,
/// Preferences) on the left; Recent and, beneath it, the Demos on the
/// right — where VS Code keeps its walkthroughs, so a first launch reads
/// *start here* → *or try one of these* and the left column never scrolls.
/// Presentation only.
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
          // Wide: hero + Start on the left, Recent and Demos on the right.
          // Narrow (< 760 pt): one column, Recent and Demos below.
          final wide = c.maxWidth >= 760;
          final pad = c.maxWidth < 600 ? 20.0 : 40.0;
          // The Start list (New, Open, Preferences) must stay above the
          // fold at the smallest supported window, and the left column
          // never scrolls: the hero keeps its aspect ratio but gives up
          // height before the list does, within the page's own height cap.
          final startHeight =
              24.0 +
              48 +
              33.0 * (3 + (state.editor.pickerUnavailable ? 2 : 0)) +
              (connected ? 0 : 24);
          final pageHeight = wide ? c.maxHeight.clamp(0.0, _maxPageHeight) : c.maxHeight;
          final heroMaxHeight = (pageHeight - 2 * pad - startHeight).clamp(240.0, 720.0);
          final left = Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            mainAxisSize: MainAxisSize.min,
            children: [
              // The hero keeps a fixed aspect ratio and fills the column,
              // shrinking when the window is short.
              Align(
                alignment: Alignment.topLeft,
                child: ConstrainedBox(
                  constraints: BoxConstraints(maxHeight: heroMaxHeight),
                  child: AspectRatio(aspectRatio: 1.15, child: HeroMark(version: kStudioVersion)),
                ),
              ),
              const SizedBox(height: 24),
              _Start(
                connected: connected,
                dispatch: dispatch,
                pathFallback: state.editor.pickerUnavailable,
              ),
            ],
          );
          if (wide) {
            return ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1360, maxHeight: _maxPageHeight),
              child: Padding(
                padding: EdgeInsets.all(pad),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Expanded(flex: 11, child: left),
                    SizedBox(width: pad),
                    // the right column is the one that may scroll: a long
                    // Recent list pushes the Demos down, never the Start
                    Expanded(
                      flex: 9,
                      child: _Aside(state: state, dispatch: dispatch),
                    ),
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
                  _Aside(state: state, dispatch: dispatch, scrolls: false),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

/// The page's height cap on a wide window: the hero is sized against it, so
/// the left column fits without scrolling.
const double _maxPageHeight = 820;

class _Start extends StatelessWidget {
  const _Start({required this.connected, required this.dispatch, this.pathFallback = false});

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

/// The right column: Recent, then the Demos — what a returning designer
/// reaches for first, then what a new one tries.  On a wide window it is
/// the one part of the page that scrolls.
class _Aside extends StatelessWidget {
  const _Aside({required this.state, required this.dispatch, this.scrolls = true});
  final AppState state;
  final void Function(AppAction) dispatch;
  final bool scrolls;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final connected = state.connection is Connected;
    final templates = state.editor.deploy.templates;
    final children = [
      Text(context.l10n.recent, style: Theme.of(context).textTheme.titleSmall),
      const SizedBox(height: 8),
      if (state.recent.isEmpty)
        Text(
          context.l10n.projectsYouOpenWillAppearHere,
          style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
        )
      else
        for (final r in state.recent)
          _RecentRow(
            project: r,
            enabled: connected,
            onOpen: () => dispatch(OpenProjectRequested(r.path)),
            onRemove: () => dispatch(RemoveRecentRequested(r.path)),
          ),
      if (templates.isNotEmpty) ...[
        const SizedBox(height: 28),
        Text(context.l10n.demos, style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 8),
        // A demo is a template: the same New Project, with a design in
        // it (and, for the wired one, its deployment).
        for (final tpl in templates)
          _DemoRow(
            key: ValueKey('template-${tpl.id}'),
            template: tpl,
            enabled: connected,
            onTap: () => dispatch(NewProjectPickRequested(template: tpl.id)),
          ),
      ],
    ];
    return ListView(
      shrinkWrap: !scrolls,
      physics: scrolls ? null : const NeverScrollableScrollPhysics(),
      children: children,
    );
  }
}

/// One demo: its name and what it is, laid out like a Recent row so the
/// two lists read as one column.
class _DemoRow extends StatelessWidget {
  const _DemoRow({super.key, required this.template, required this.enabled, required this.onTap});
  final pb.TemplateView template;
  final bool enabled;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return MacInteractive(
      onTap: enabled ? onTap : null,
      radius: 6,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.only(top: 1),
            child: Icon(
              Icons.memory_outlined,
              size: 18,
              color: enabled ? t.accent : t.textTertiary,
            ),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  template.displayName,
                  style: TextStyle(
                    fontSize: MacType.body,
                    fontWeight: FontWeight.w500,
                    color: enabled ? t.textPrimary : t.textTertiary,
                  ),
                ),
                if (template.description.isNotEmpty)
                  Text(
                    template.description,
                    style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
                  ),
              ],
            ),
          ),
        ],
      ),
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
