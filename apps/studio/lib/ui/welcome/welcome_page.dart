/// The launcher shown when no project is open — DaVinci's Project Manager,
/// laid out like VS Code's welcome page: hero and Start on the left,
/// Recent on the right.  Presentation only.
library;

import 'dart:io';

import 'package:flutter/material.dart';

import '../../app/actions.dart';
import '../../app/state.dart';
import '../../platform/desktop.dart';
import '../../protocol/versions.dart';
import '../dialogs.dart';
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
        Text('Start', style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 8),
        MacLink(
          icon: Icons.add_box_outlined,
          label: 'New Project…',
          shortcut: shortcut('N'),
          enabled: connected,
          onTap: () => dispatch(const NewProjectPickRequested()),
        ),
        MacLink(
          icon: Icons.account_tree_outlined,
          label: 'New System…',
          enabled: connected,
          onTap: () => dispatch(const NewProjectPickRequested(kind: NewProjectKind.system)),
        ),
        MacLink(
          icon: Icons.code_outlined,
          label: 'New Text Project…',
          enabled: connected,
          onTap: () => dispatch(const NewProjectPickRequested(kind: NewProjectKind.text)),
        ),
        MacLink(
          icon: Icons.folder_open_outlined,
          label: 'Open Project…',
          shortcut: shortcut('O'),
          enabled: connected,
          onTap: () => dispatch(const OpenProjectPickRequested()),
        ),
        if (pathFallback) ...[
          MacLink(
            icon: Icons.keyboard_outlined,
            label: 'Open by path…',
            enabled: connected,
            onTap: () async {
              final path = await showPathSheet(context, title: 'Open project by path');
              if (path != null && path.isNotEmpty) dispatch(OpenProjectRequested(path));
            },
          ),
          MacLink(
            icon: Icons.keyboard_outlined,
            label: 'New at path…',
            enabled: connected,
            onTap: () async {
              final path = await showPathSheet(context, title: 'Create project at path');
              if (path != null && path.isNotEmpty) {
                dispatch(
                  NewProjectRequested(rootPath: path, name: path.split(RegExp(r'[/\\]')).last),
                );
              }
            },
          ),
          MacLink(
            icon: Icons.keyboard_outlined,
            label: 'New system at path…',
            enabled: connected,
            onTap: () async {
              final path = await showPathSheet(context, title: 'Create system at path');
              if (path != null && path.isNotEmpty) {
                dispatch(
                  NewProjectRequested(
                    rootPath: path,
                    name: path.split(RegExp(r'[/\\]')).last,
                    kind: NewProjectKind.system,
                  ),
                );
              }
            },
          ),
        ],
        if (!connected)
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: Text(
              'Waiting for the compiler. The status line shows the connection.',
              style: TextStyle(fontSize: 11, color: t.textTertiary),
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
        Text('Recent', style: Theme.of(context).textTheme.titleSmall),
        const SizedBox(height: 8),
        Expanded(
          child: state.recent.isEmpty
              ? Text(
                  'Projects you open will appear here.',
                  style: TextStyle(fontSize: 12, color: t.textTertiary),
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
                    fontSize: 13,
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
                        style: TextStyle(fontSize: 11, color: t.textTertiary),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    if (missing) Text('not found', style: TextStyle(fontSize: 11, color: t.open)),
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
                      tooltip: 'Remove from Recent',
                      onPressed: widget.onRemove,
                      constraints: const BoxConstraints.tightFor(width: 22, height: 22),
                    ),
                  )
                : Text(
                    relativeTime(r.lastOpened),
                    textAlign: TextAlign.right,
                    style: TextStyle(
                      fontSize: 11,
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
String relativeTime(DateTime when, {DateTime? now}) {
  final d = (now ?? DateTime.now()).difference(when);
  if (d.inMinutes < 1) return 'just now';
  if (d.inHours < 1) return '${d.inMinutes} min ago';
  if (d.inDays < 1) return '${d.inHours} h ago';
  if (d.inDays == 1) return 'yesterday';
  if (d.inDays < 30) return '${d.inDays} d ago';
  final local = when.toLocal();
  return '${local.year}-${local.month.toString().padLeft(2, '0')}-${local.day.toString().padLeft(2, '0')}';
}
