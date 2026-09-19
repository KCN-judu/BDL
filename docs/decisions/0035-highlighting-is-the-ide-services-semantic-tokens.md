---
id: ADR-0035
status: accepted
date: 2026-09-20
area: ide
supersedes: []
superseded-by: []
related: [ADR-0001, ADR-0002, ADR-0007, ADR-0023, ADR-0032]
fv: []
---

# ADR-0035: Highlighting is the IDE service's semantic tokens, on the LSP vocabulary, and no client classifies text

## Status

Accepted (the syntax-highlighting milestone, after the IDE service of ADR-0023
and the derived role of ADR-0032).

## Context

`bdl-ide` served semantic tokens to the language server with a BDL-specific
vocabulary (`concept`, `mapping`, `output`, `clock`, `device`, `unit`), and
Studio's Code view and formula editor showed plain monospace text with wavy
diagnostic marks. Adding colour to Studio put a choice on the table: a Dart
tokenizer for `.bdl` (fast to write, always available, never in step) or the
service's tokens over the wire (one classifier, one more request per text, and
an answer that arrives after the keystroke). The facts a highlighter must state
are not lexical — the role of a relationship, whether a word after a number is a
unit, whether `all` is a binder word here, whether `clamp` is the library's —
and every one of them is already a query over the `AnalysisSnapshot` that hover,
Explain, completion and the canvas read. Two classifiers would diverge the day
either changed, and the second one would live in the layer ADR-0001 forbids from
owning semantic truth. The language server, meanwhile, spoke a vocabulary every
LSP editor had to be configured for, name by name, while the standard names
(`type`, `function`, `variable`, `parameter`, `keyword`, …) are themed
everywhere without configuration.

## Decision

1. **One classifier.** `bdl-ide::tokens` is the only place a span of BDL text is
   classified: a lexical layer from the lossless tree (total on any text) and a
   semantic layer from the snapshot's projections and the derived role
   (ADR-0032), merged with the semantic layer winning, into one sorted, disjoint
   stream of byte-range tokens. No client — Studio, the VS Code extension, a
   future editor — tokenizes BDL, keeps a keyword or unit table, or decides a
   class from a spelling.
2. **The LSP vocabulary.** Token types are the LSP standard names where one
   fits, plus `unit` and `slot`; modifiers are the standard `declaration` and
   `defaultLibrary` plus `source`, `output`, `device`, `instance`, `unresolved`.
   BDL's distinctions are modifiers on standard types, never new types: a Source
   is `variable` + `source`. The legend is versioned (`LEGEND_VERSION`); a
   client colours by name and leaves an unknown name plain.
3. **Two wire forms, one stream.** The language server encodes the stream as
   `textDocument/semanticTokens` in the client's position encoding through a
   pure encoder; the daemon sends it to Studio structured (`SemanticTokens`,
   protocol 0.21: byte ranges, legend indices, the legend, the text length, the
   client's generation echoed). Conversion between the two is lossless and lives
   in `bdl-ide`, not in an adapter.
4. **Colour is theme policy.** No colour, font or theme word crosses a boundary.
   Studio's `SyntaxTheme` maps names to styles from the window's tokens, in the
   canvas's own category encoding; the language server sends none.
5. **Partial answers are still right.** A text that does not build keeps its
   lexical classes; a client keeps the last answer through edits by shifting the
   spans of the unchanged prefix and suffix and never draws a stale span over
   changed text; only the latest generation's answer lands.

## Alternatives

- **A Dart tokenizer in Studio** (a regex or hand-written lexer over `.bdl`).
  Always available and instant, but a second definition of the language in the
  presentation layer — the thing ADR-0001 exists to forbid — and blind to every
  fact that matters (role, unit by position, contextual keyword, library
  function). Rejected.
- **A TextMate grammar as the Studio highlighter.** The same second definition
  in a third language; the extension keeps one only as a fallback before the
  server answers, which is what TextMate grammars are for.
- **Keep the BDL-specific vocabulary** (`concept`, `mapping`, …). Every LSP
  client would need a mapping for every name; the standard names are themed out
  of the box and the distinctions fit modifiers. Rejected.
- **Send the LSP data array to Studio.** Studio holds byte offsets already and
  would decode line/character deltas back into them; a structured message is the
  same information without the round trip, and the encoder proves the forms
  equivalent.
- **Highlight only in Studio, leave the language server as it was.** The two
  would have disagreed on `unit`, `parameter` and the role from the first day.

## Consequences

- `crates/bdl-ide/src/tokens/` (the classifier, the legend, the encoder) is the
  authority; `bdl-lsp` builds its legend from it and encodes; the daemon serves
  `SemanticTokens`; Studio's `app/highlighting.dart` and `ui/code/` render.
- Adding a class is a change to `TokenType` / `TokenModifiers`, a bump of
  `LEGEND_VERSION`, a row in `docs/architecture/syntax-highlighting.md`, a
  golden test, and — if the class should look different — a line in
  `SyntaxTheme`. Nothing else.
- Studio pays one uncounted request per text shown and per pause in typing; at
  the large baseline the whole document classifies in 1.5 ms.
- Records: `docs/architecture/syntax-highlighting.md` (new), `ide-service.md`,
  `studio-compiler-integration.md`, `studio-ui.md` §12, `docs/spec/protocol.md`,
  `docs/project/status.md`, protocol history 0.21, the change record
  `2026-09-syntax-highlighting.md`.
