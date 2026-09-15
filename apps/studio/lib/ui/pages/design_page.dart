import 'package:flutter/material.dart';

import '../../app/actions.dart';
import '../../app/state.dart';
import '../canvas/node_canvas.dart';
import '../inspector.dart';
import '../library.dart';
import '../../platform/desktop.dart';
import '../mac/tokens.dart';

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
              : NodeCanvas(
                  project: project,
                  layout: state.editor.layout,
                  selection: state.editor.selection,
                  dispatch: dispatch,
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
        'No project open.\nUse the project manager (⊞) in the page bar, or ${shortcut('O')} / ${shortcut('N')}.',
        textAlign: TextAlign.center,
        style: TextStyle(color: t.textTertiary),
      ),
    );
  }
}
