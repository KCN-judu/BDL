/// The Code view's IDE transitions: completion, hover, definition,
/// references and formatting over a source file's text as typed
/// (`docs/architecture/ide-service.md` § Studio integration, protocol
/// 0.22).
///
/// The same rules as the formula field's tooling (`app/tooling.dart`):
/// every request carries a generation and only the latest answer is
/// applied; an answer is about the exact text and byte offset it was asked
/// for, so a reply to an older text never acts on a newer one; a failure
/// leaves what is on show.  Studio owns which row is selected, where a
/// card sits and when to ask; the IDE service owns every candidate, every
/// card, every site and every formatted byte.  No name, keyword, unit or
/// syntax table exists here.
library;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition;
import 'sources.dart' show sourceEditRequested;
import 'state.dart';

// ---- completion --------------------------------------------------------------

Transition sourceCompletionRequested(AppState s, String path, String text, int offset) {
  if (s.project == null) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  final previous = s.editor.completion;
  // Re-asking for the same file keeps the rows on screen (no flicker)
  // until the new candidates arrive; another document starts empty.
  final completion = previous != null && previous.path == path
      ? previous.copyWith(generation: generation, source: text, offset: offset, pending: true)
      : CompletionState(path: path, generation: generation, source: text, offset: offset);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(completion: completion, toolingGeneration: generation),
    ),
    [
      CompleteSource(
        revision: s.revision,
        generation: generation,
        path: path,
        text: text,
        offset: offset,
      ),
    ],
  );
}

Transition sourceCompletionReceived(AppState s, int generation, pb.SourceCompletionResponse r) {
  final c = s.editor.completion;
  if (c == null || c.path == null || generation != c.generation) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        completion: c.copyWith(
          items: r.items,
          pending: false,
          selected: r.items.isEmpty ? 0 : c.selected.clamp(0, r.items.length - 1),
        ),
      ),
    ),
  );
}

// ---- hover ---------------------------------------------------------------------

Transition sourceHoverRequested(AppState s, String path, String text, int? offset) {
  if (offset == null) return Transition(s.copyWith(editor: s.editor.copyWith(clearHover: true)));
  if (s.project == null) return Transition(s);
  final h = s.editor.hover;
  // The same spot, or a spot inside the card on show: nothing to ask.
  if (h != null && h.path == path && h.offset == offset) return Transition(s);
  final card = h?.card;
  if (h != null &&
      h.path == path &&
      card != null &&
      card.found &&
      card.hasSpan() &&
      offset >= card.span.start &&
      offset < card.span.end) {
    return Transition(s);
  }
  final generation = s.editor.toolingGeneration + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        hover: HoverState(generation: generation, path: path, offset: offset),
        toolingGeneration: generation,
      ),
    ),
    [
      HoverSource(
        revision: s.revision,
        generation: generation,
        path: path,
        text: text,
        offset: offset,
      ),
    ],
  );
}

// ---- navigation ------------------------------------------------------------------

Transition sourceDefinitionRequested(AppState s, String path, String text, int offset) {
  if (s.project == null) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  return Transition(s.copyWith(editor: s.editor.copyWith(toolingGeneration: generation)), [
    DefineSource(
      revision: s.revision,
      generation: generation,
      path: path,
      text: text,
      offset: offset,
    ),
  ]);
}

/// The definition arrived: the first site becomes the reveal the pane
/// acts on (opening the file when it is another).  Only the latest
/// navigation request's answer moves the caret.
Transition sourceDefinitionReceived(AppState s, int generation, pb.SourceLocationsResponse r) {
  if (generation != s.editor.toolingGeneration) return Transition(s);
  final first = r.locations.firstOrNull;
  if (first == null) return Transition(s);
  final location = SourceLocation(path: first.path, start: first.start, end: first.end);
  var editor = s.editor.copyWith(
    reveal: SourceReveal(generation: generation, location: location),
  );
  if (location.path != editor.sources.openPath && editor.sources.file(location.path) != null) {
    editor = editor.copyWith(
      sources: editor.sources.copyWith(openPath: location.path, clearBuffer: true),
    );
  }
  return Transition(s.copyWith(editor: editor));
}

Transition sourceReferencesRequested(AppState s, String path, String text, int offset) {
  if (s.project == null) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        references: SourceReferencesState(generation: generation, path: path, offset: offset),
        toolingGeneration: generation,
      ),
    ),
    [
      ReferencesSource(
        revision: s.revision,
        generation: generation,
        path: path,
        text: text,
        offset: offset,
      ),
    ],
  );
}

Transition sourceReferencesReceived(AppState s, int generation, pb.SourceLocationsResponse r) {
  final refs = s.editor.references;
  if (refs == null || generation != refs.generation) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        references: refs.copyWith(
          title: r.title,
          pending: false,
          locations: [
            for (final l in r.locations) SourceLocation(path: l.path, start: l.start, end: l.end),
          ],
        ),
      ),
    ),
  );
}

Transition referencesDismissed(AppState s) =>
    Transition(s.copyWith(editor: s.editor.copyWith(clearReferences: true)));

// ---- format ----------------------------------------------------------------------

Transition formatSourceRequested(AppState s, String path, String text) {
  if (s.project == null || s.editor.sources.openPath != path) return Transition(s);
  final sources = s.editor.sources;
  final generation = sources.formatGeneration + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        sources: sources.copyWith(formatGeneration: generation, formatting: text),
      ),
    ),
    [FormatSource(revision: s.revision, generation: generation, path: path, text: text)],
  );
}

/// The canonical text arrived.  It replaces the editor's text only when
/// it answers the latest format request and the designer has not typed
/// since; then it is one authored edit, sent at once.
Transition formatSourceReceived(
  AppState s,
  int generation,
  String path,
  pb.FormatSourceResponse r,
) {
  final sources = s.editor.sources;
  final asked = sources.formatting;
  if (generation != sources.formatGeneration || asked == null) return Transition(s);
  final settled = s.copyWith(
    editor: s.editor.copyWith(sources: sources.copyWith(clearFormatting: true)),
  );
  if (!r.formatted || sources.openPath != path || sources.text != asked) return Transition(settled);
  final replaced = settled.copyWith(
    editor: settled.editor.copyWith(
      sources: settled.editor.sources.copyWith(buffer: r.text, replaced: sources.replaced + 1),
    ),
  );
  return sourceEditRequested(replaced, path, r.text);
}

/// A failed request under a generation the Code view's states hold.
EditorState codeToolingFailed(EditorState e, int generation) {
  var next = e;
  if (e.references?.generation == generation) next = next.copyWith(clearReferences: true);
  return next;
}

/// A source-file failure by the format request's own generation.
Transition formatSourceFailed(AppState s, int generation) {
  final sources = s.editor.sources;
  if (generation != sources.formatGeneration) return Transition(s);
  return Transition(
    s.copyWith(editor: s.editor.copyWith(sources: sources.copyWith(clearFormatting: true))),
  );
}
