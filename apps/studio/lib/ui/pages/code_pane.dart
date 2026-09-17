/// The Code view: the project's source files, editable (ADR-0023 §3–§5).
///
/// Task: change the design as text and see the graph follow — or see, at
/// once, that what was typed does not build yet and where.  The text is
/// the primary object; the one fact that must be found without reading
/// is whether the graph is in step with it.  That fact is a document-
/// level condition, so it is a banner (the platform's shape for one),
/// never a colour on the text: the graph shows the last revision that
/// built, and the diagnostics below the editor say why the draft is not
/// it.  Open faults (incompleteness) are listed with the hollow ring the
/// canvas uses for the undecided, never red.
///
/// Studio never parses the text: the file list, the draft state, the
/// anchors and every diagnostic come from the daemon.
library;

import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../app/actions.dart';
import '../../app/sources.dart';
import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../mac/interactive.dart';
import '../mac/tokens.dart';
import '../mac/widgets.dart';

/// How long typing pauses before the file is sent.
const Duration kSourceEditPause = Duration(milliseconds: 400);

class CodePane extends StatefulWidget {
  const CodePane({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  State<CodePane> createState() => _CodePaneState();
}

class _CodePaneState extends State<CodePane> {
  final TextEditingController _c = TextEditingController();
  final FocusNode _focus = FocusNode();
  final ScrollController _scroll = ScrollController();
  Timer? _pause;

  /// The path the controller's text belongs to.
  String? _path;

  /// The last text put into the controller from the state, so a state
  /// change that repeats it does not move the caret.
  String _shown = '';

  SourcesState get _sources => widget.state.editor.sources;

  @override
  void initState() {
    super.initState();
    _focus.addListener(() {
      if (!_focus.hasFocus) _send();
    });
    _show();
  }

  @override
  void didUpdateWidget(CodePane old) {
    super.didUpdateWidget(old);
    _show();
    final sel = widget.state.editor.selection;
    if (sel != old.state.editor.selection && !_focus.hasFocus) _reveal(sel);
  }

  /// Put the state's text in the editor when it is not the designer's
  /// own typing: another file, a new revision from the graph, a reload.
  void _show() {
    final text = _sources.text;
    final path = _sources.openPath;
    if (path == _path && text == _shown) return;
    final switching = path != _path;
    _path = path;
    _shown = text;
    if (switching || _sources.buffer == null) {
      final caret = switching ? 0 : _c.selection.baseOffset.clamp(0, text.length);
      _c.value = TextEditingValue(
        text: text,
        selection: TextSelection.collapsed(offset: caret.clamp(0, text.length)),
      );
    }
  }

  void _typed(String text) {
    final path = _path;
    if (path == null) return;
    _shown = text;
    widget.dispatch(SourceTextChanged(path, text));
    _pause?.cancel();
    _pause = Timer(kSourceEditPause, _send);
  }

  void _send() {
    _pause?.cancel();
    final path = _path;
    if (path == null) return;
    if (_sources.buffer == null && _c.text == _sources.open?.text) return;
    widget.dispatch(SourceEditRequested(path, _c.text));
  }

  /// Scroll the editor to the item the canvas selected (Split sync).
  void _reveal(Selection selection) {
    final file = _sources.open;
    if (file == null) return;
    final anchor = anchorOf(widget.state, selection, file);
    if (anchor == null) return;
    final offset = _charOffset(file.text, anchor.start);
    _c.selection = TextSelection.collapsed(offset: offset);
    final line = '\n'.allMatches(file.text.substring(0, offset)).length;
    if (_scroll.hasClients) {
      final y = (line * _lineHeight - 48).clamp(0.0, _scroll.position.maxScrollExtent);
      _scroll.animateTo(y, duration: const Duration(milliseconds: 160), curve: Curves.easeOut);
    }
  }

  /// The caret moved into an item: select its node (Split sync, the other
  /// way).  Only on a click or arrow move, never while typing.
  void _caretMoved() {
    final file = _sources.open;
    if (file == null || file.draft || _c.text != file.text) return;
    final byte = _byteOffset(file.text, _c.selection.baseOffset);
    for (final a in file.anchors) {
      if (byte >= a.start && byte < a.end) {
        final sel = selectionOf(widget.state, a);
        if (sel != null && sel != widget.state.editor.selection) {
          widget.dispatch(SelectionChanged(sel));
        }
        return;
      }
    }
  }

  @override
  void dispose() {
    _pause?.cancel();
    _c.dispose();
    _focus.dispose();
    _scroll.dispose();
    super.dispose();
  }

  static const double _lineHeight = 18;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final sources = _sources;
    final file = sources.open;
    final connected = widget.state.connection is Connected;
    if (sources.revision < 0 || file == null) {
      return Container(
        color: t.content,
        alignment: Alignment.center,
        child: Text('Reading the sources…', style: TextStyle(color: t.textTertiary)),
      );
    }
    final diagnostics = sources.diagnosticsOf(file.path);
    final errors = diagnostics.where((d) => !d.open).length;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _FileBar(
          sources: sources,
          onOpen: (p) {
            _send();
            widget.dispatch(SourceFileOpened(p));
          },
        ),
        if (file.draft)
          _Banner(
            icon: Icons.sync_problem_outlined,
            text: errors == 1
                ? 'This file does not build yet: the design shows the last version that did.'
                : 'This file does not build yet ($errors problems): the design shows the last '
                      'version that did.',
          ),
        Expanded(
          child: Container(
            color: t.content,
            child: Semantics(
              label: 'Source of ${file.path}',
              child: TextField(
                controller: _c,
                focusNode: _focus,
                scrollController: _scroll,
                readOnly: !connected,
                maxLines: null,
                expands: true,
                keyboardType: TextInputType.multiline,
                textAlignVertical: TextAlignVertical.top,
                style: TextStyle(
                  fontSize: 12,
                  fontFamily: 'Menlo',
                  height: _lineHeight / 12,
                  color: t.textPrimary,
                ),
                decoration: const InputDecoration(
                  border: InputBorder.none,
                  isDense: true,
                  contentPadding: EdgeInsets.fromLTRB(16, 12, 16, 12),
                ),
                inputFormatters: const [_Lf()],
                onChanged: _typed,
                onTap: _caretMoved,
              ),
            ),
          ),
        ),
        if (diagnostics.isNotEmpty) ...[
          Divider(height: 1, color: t.hairline),
          _Diagnostics(
            text: file.text,
            diagnostics: diagnostics,
            onGo: (d) {
              final offset = _charOffset(file.text, d.start);
              _focus.requestFocus();
              _c.selection = TextSelection(
                baseOffset: offset,
                extentOffset: _charOffset(file.text, d.end).clamp(offset, file.text.length),
              );
            },
          ),
        ],
      ],
    );
  }
}

/// Newlines only: the daemon's text is LF.
class _Lf extends TextInputFormatter {
  const _Lf();
  @override
  TextEditingValue formatEditUpdate(TextEditingValue old, TextEditingValue value) =>
      value.text.contains('\r') ? value.copyWith(text: value.text.replaceAll('\r\n', '\n')) : value;
}

/// The file on screen and the others, when there are others.
class _FileBar extends StatelessWidget {
  const _FileBar({required this.sources, required this.onOpen});
  final SourcesState sources;
  final void Function(String) onOpen;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final files = sources.files;
    return Container(
      height: 30,
      color: t.window,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          if (files.length <= 1)
            Text(
              sources.openPath ?? '',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: t.textPrimary),
            )
          else
            SizedBox(
              width: 240,
              child: MacDropdown<String>(
                value: sources.openPath ?? files.first.path,
                items: [for (final f in files) f.path],
                onChanged: onOpen,
                labelOf: (p) => sources.file(p)?.draft == true ? '$p — not built' : p,
              ),
            ),
        ],
      ),
    );
  }
}

/// A document-level condition, as the window's banners are drawn.
class _Banner extends StatelessWidget {
  const _Banner({required this.icon, required this.text});
  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      color: t.sidebar,
      padding: const EdgeInsets.fromLTRB(12, 6, 12, 6),
      child: Row(
        children: [
          Icon(icon, size: 16, color: t.textSecondary),
          const SizedBox(width: 8),
          Expanded(
            child: Text(text, style: TextStyle(fontSize: 12, color: t.textPrimary)),
          ),
        ],
      ),
    );
  }
}

/// The daemon's reasons, one row each: a mark that survives without
/// colour (× for an error, a hollow ring for something still open), the
/// line, the message.  Activating a row puts the caret there.
class _Diagnostics extends StatelessWidget {
  const _Diagnostics({required this.text, required this.diagnostics, required this.onGo});
  final String text;
  final List<pb.SourceDiagnostic> diagnostics;
  final void Function(pb.SourceDiagnostic) onGo;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return ConstrainedBox(
      constraints: const BoxConstraints(maxHeight: 6 * 24 + 8),
      child: Container(
        color: t.window,
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: ListView(
          shrinkWrap: true,
          children: [
            for (final d in diagnostics)
              MacInteractive(
                onTap: () => onGo(d),
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: SizedBox(
                  height: 24,
                  child: Row(
                    children: [
                      Semantics(
                        label: d.open ? 'open' : 'error',
                        child: d.open
                            ? Container(
                                width: 10,
                                height: 10,
                                decoration: BoxDecoration(
                                  shape: BoxShape.circle,
                                  border: Border.all(color: t.open, width: 1.5),
                                ),
                              )
                            : Icon(Icons.close, size: 12, color: t.error),
                      ),
                      const SizedBox(width: 8),
                      SizedBox(
                        width: 40,
                        child: Text(
                          '${_lineOf(text, d.start)}',
                          textAlign: TextAlign.right,
                          style: TextStyle(
                            fontSize: 11,
                            color: t.textSecondary,
                            fontFeatures: const [FontFeature.tabularFigures()],
                          ),
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          d.message,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(fontSize: 12, color: t.textPrimary),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

/// 1-based line of a byte offset.
int _lineOf(String text, int byte) =>
    '\n'.allMatches(text.substring(0, _charOffset(text, byte))).length + 1;

/// UTF-16 code-unit offset of a UTF-8 byte offset (the daemon's spans).
int _charOffset(String text, int byte) {
  if (byte <= 0) return 0;
  final bytes = utf8.encode(text);
  if (byte >= bytes.length) return text.length;
  return utf8.decode(bytes.sublist(0, byte), allowMalformed: true).length;
}

int _byteOffset(String text, int char) {
  if (char <= 0) return 0;
  if (char >= text.length) return utf8.encode(text).length;
  return utf8.encode(text.substring(0, char)).length;
}
