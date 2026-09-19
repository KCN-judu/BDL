---
kind: architecture
area: ide
status: current
---

# Syntax and semantic highlighting

One classifier says what every span of BDL text _is_; every surface colours what
it is told. The classifier is `bdl-ide::tokens` over an `AnalysisSnapshot`; its
vocabulary is the LSP semantic-token vocabulary (types and modifiers, a
versioned legend) with two BDL additions; its answer is a sorted, disjoint
stream of byte-range tokens that survives any text. The language server encodes
that stream as `textDocument/semanticTokens`; the daemon sends it structured to
Studio (`SemanticTokens`, protocol 0.21); Studio maps the names to a theme and
draws. No colour crosses a boundary, and no Dart, TypeScript or grammar file
decides a class (ADR-0001, ADR-0035).

## Why one classifier

Highlighting looks lexical and is not: whether `tilt` is a Source or a value,
whether `deg` is a unit or a parameter, whether `all` is a keyword or a name,
whether `clamp` is the library's or the design's — these are facts of the
snapshot, and the same facts drive hover, Explain, completion and the canvas. A
second tokenizer in a client would answer them differently the day either
changed. So the rule is the one ADR-0001 set for semantics generally: Rust
classifies, clients render. The corollary for an editor is that _partial_
answers must still be right: a file mid-edit does not build, and the designer
must not lose the reading of the text because one parenthesis is missing.

## The two layers

```text
text ──parse (total)──▶ CST ──lexical layer──▶ tokens₁ ─┐
                                                        ├─ merge ─▶ sorted, disjoint stream
snapshot (committed + overlays) ──semantic layer─▶ tokens₂ ┘
```

**Lexical layer** — `lexical_tokens(root)`, from the lossless Rowan tree
(`bdl-syntax`). The parser is total: any text yields a tree, an error region is
a node like any other, and every token the lexer recognised is under it. The
layer classifies by the tree, never by spelling:

| Token                                                                                         | Class                                        |
| --------------------------------------------------------------------------------------------- | -------------------------------------------- |
| a keyword token (`concept`, `mapping`, `if`, `then`, `else`, `true`, `false`, `in`, `let`, …) | `keyword`                                    |
| a binder word (`all`, `any`, `sum`, `choose`, …) in head position (`BinderExpr`)              | `keyword`                                    |
| the same spelling as a name elsewhere                                                         | a name (the semantic layer says which)       |
| `Number`                                                                                      | `number`                                     |
| the identifier under `UnitSuffix` (`90 deg`)                                                  | `unit`                                       |
| `?`                                                                                           | `slot`                                       |
| `_` (a discarded pattern)                                                                     | `parameter`                                  |
| `+ - * / ! && \|\| == != < <= > >= = -> => ?? .. @ .`                                         | `operator`                                   |
| brackets, commas, colons, braces                                                              | nothing (punctuation is plain)               |
| comments                                                                                      | `comment` (a block comment may span lines)   |
| a name bound by a binder, a lambda, a match arm, a `let`, or a rule's parameter list          | `parameter` (+ `declaration` at the binding) |
| a constructor in a pattern                                                                    | `enumMember`                                 |

Units are tokenized by position — the identifier that follows a number inside a
`UnitSuffix` — never by a table of unit spellings: `deg` as a rule parameter is
`parameter`, `deg` after `90` is `unit`. Contextual words are keywords only
where the grammar makes them keywords: `all` at the head of a binder is a
keyword; `all` declared as a value is a value.

**Semantic layer** — `semantic_tokens(snapshot, document)` for a file,
`formula_tokens(snapshot, mapping)` for a definition draft. It walks the
snapshot's projection map (`ProjectionMap` anchors with `EntityRole::Name`, the
declaration, and `EntityRole::Reference`) and, for body names the projection did
not anchor, the snapshot's names by spelling — an index built once per call.
Each entity maps to a standard class with BDL's distinctions as modifiers:

| Entity                                                 | Type        | Modifiers                                           |
| ------------------------------------------------------ | ----------- | --------------------------------------------------- |
| concept                                                | `type`      | `unresolved` when it has no representation          |
| relationship, role **Rule** (ADR-0032)                 | `function`  | `unresolved` when declared without a definition     |
| relationship, role **Value**                           | `variable`  | —                                                   |
| relationship, role **Source**                          | `variable`  | `source` (having no definition is what a Source is) |
| clock                                                  | `namespace` | —                                                   |
| output                                                 | `variable`  | `output`                                            |
| device                                                 | `variable`  | `device`                                            |
| component                                              | `class`     | —                                                   |
| instance                                               | `variable`  | `instance`                                          |
| port, export                                           | `property`  | —                                                   |
| an applied equation of the library (`clamp`, `sum`, …) | `function`  | `defaultLibrary`                                    |
| a relationship's input named by concept in a formula   | `type`      | —                                                   |
| a relationship's input named by parameter in a formula | `parameter` | —                                                   |

Every declaration carries `declaration`. The role comes from
`bdl_ide::role::relationship_role` — the committed design's answer, a draft
being no realization — so the class of a name follows the derived role and
nothing else: a Source given a definition becomes a value at the next revision,
a rule stays a rule whether or not anything applies it
(`reactive.rule_unapplied` is a diagnostic, never a class).

**Merge** — `merge(semantic, lexical)`: one linear sweep over both sorted
streams; a semantic token wins over any lexical token it overlaps; the result is
sorted, disjoint, non-empty, on character boundaries (`finish`). When the
document did not build, the semantic layer is empty and the lexical layer is the
whole answer; when it built, names gain their class and everything else keeps
its lexical one. Nothing is ever dropped to plain because a later token failed
to classify.

## The vocabulary

`bdl_ide::tokens::{TokenType, TokenModifiers, Legend, LEGEND_VERSION}`:

- **Types** (LSP standard names where one fits): `namespace`, `type`, `class`,
  `function`, `variable`, `parameter`, `property`, `enumMember`, `keyword`,
  `number`, `operator`, `comment`; BDL's own: `unit`, `slot`.
- **Modifiers**: standard `declaration`, `defaultLibrary`; BDL's own `source`,
  `output`, `device`, `instance`, `unresolved`.
- **Legend**: `legend()` — `{ version, types[], modifiers[] }`, the index order
  the wire uses. `LEGEND_VERSION` moves when a name is added, removed or
  renumbered. A client colours **by name**, never by index, and leaves a name it
  does not know plain, so an older client renders a newer daemon's stream
  without a wrong colour.

Why not a BDL-specific vocabulary: an editor that speaks LSP already has a theme
for `type`, `function`, `variable`, `keyword`; a Source as `variable` + `source`
gets a sensible colour with no configuration and a distinct one with a line of
theme. The two additions are what LSP has no word for — a unit beside a number,
and the `?` slot, which is neither an operator nor a name but the text form of
the Composer's dashed hollow chip (a decision still to be made).

## Two wire forms of one stream

**LSP** — `bdl_ide::tokens::encode`: pure `encode(tokens, text, encoding)` →
`[deltaLine, deltaStart, length, type, modifiers]` rows in the client's
negotiated position encoding (UTF-8 / UTF-16 / UTF-32, LSP 3.17
`positionEncodings`); a multi-line token is split per line; a range past the
text is clamped, never a panic. `decode_data` and `byte_range` are its inverse
for tests. `bdl-lsp` builds its `SemanticTokensLegend` from `legend()` and calls
`encode`; it classifies nothing.

**Studio** —
`SemanticTokensRequest { revision, generation, text, document: path | formula { mapping_id, component? } }`
→
`SemanticTokensResponse { revision, generation, legend, text_len, tokens[] { start, end, token_type, token_modifiers } }`
(`docs/spec/protocol.md`). Structured rather than the LSP array because Studio
holds byte offsets already (its diagnostics and anchors are byte spans) and the
conversion to the LSP form is lossless (`encode` is a function of the same
stream); the legend travels with every answer because it is small and a client
must never assume a legend it has not seen.

The daemon classifies the **text as typed**, whole: the same text
`ApplySourceEdit` or `AnalyzeDefinitionDraft` carries. A file is an overlay
(`IdeHost::set_text_document`) on a text-workspace host over the sources as
written back (`SystemState::text_ide`, seeded on first use and dropped on every
commit), so the other files' declarations resolve while this one is mid-edit; a
formula is the relationship's draft overlay in its own scope
(`Session::draft_snapshot`). A stale `revision` in the request is not an error —
tokens are presentation — the daemon answers over its current project and states
the revision it classified at.

## Studio: colour, but never classify

`apps/studio/lib/app/highlighting.dart` holds the state and the reducers,
`apps/studio/lib/ui/code/` the theme and the one text controller both BDL text
fields use — the Code view's file and the definition editor's formula.

- **Generation guards.** One `HighlightState` per document key (`file:<path>`,
  `formula:<component>:<mapping>`) carries the generation of the latest request
  and its text. Only the answer to the latest generation lands; an answer whose
  `text_len` is not the requested text's is dropped; a failure clears _pending_
  and keeps what is on show. The service's byte ranges become UTF-16 code-unit
  ranges once, in the reducer, and the legend's indices become names
  (`HighlightSpan { start, end, type, modifiers }`).
- **No flicker.** The spans are kept by text, not by revision, so a pushed
  projection does not drop them. While the editor's text differs from the text
  the spans are over, the controller applies `shiftSpans(from, to, spans)`:
  spans strictly inside the unchanged prefix stay, spans strictly inside the
  unchanged suffix move by the change in length, spans touching the edit point
  go (a word being typed into may have become another word). The changed middle
  is plain for at most one round trip; nothing stale is ever drawn over new
  text. Tokens are asked for when a text is first shown and 150 ms after typing
  pauses (`kHighlightPause`; the source send waits 400 ms).
- **Theme.** `SyntaxTheme.of(MacTokens)` maps names to styles and is the only
  place a colour is chosen. The palette is the canvas's own encoding: a name's
  ink carries the _category_ of what it names, in the hue family of that
  category's node header tint — concept, relationship, Source, output/device,
  instance — so text and graph agree without a legend; keywords, operators and
  units recede into the secondary ink, comments into the tertiary; weight marks
  a declaration, italic a name bound in the formula itself (a parameter or
  binder), the _open_ colour the `?` slot. Every colour has a non-colour partner
  (the declaring keyword, the shape of the use, the hover card's word, the
  weight or slant), and both appearances are checked for contrast
  (`test/highlighting_test.dart`). The `unresolved` modifier is not coloured in
  the default theme: the canvas draws it as a dashed boundary and the Code
  pane's fault list names it; a second cue on the same word would be noise.
- **Marks.** Diagnostic underlines (wavy, by severity) are drawn on top of the
  spans by the same controller; they were the definition editor's only
  decoration before and keep their meaning.

## Invalid and incomplete text

The parser is total and the lexical layer classifies whatever the tree holds:
every keyword, number, comment and operator keeps its class in a file that does
not build. Inside an error region the parser's recovery decides what is still a
`UnitSuffix` or a parameter list — a unit after a number inside an unclosed call
may come back plain until the parenthesis is closed — but nothing outside the
region is affected, and a semantic class is never invented from a spelling to
fill the gap. Studio's document-level condition (_This file does not build yet_)
stays a banner, never a colour on the text.

## Tests

- `crates/bdl-ide/tests/tokens.rs` — golden tests on the language: the lamp
  (exact classes on both layers, LSP encode/decode round-trip), expression forms
  (`if`/`then`/`else`, `&& ! > || .. ??`, the `?` slot, binder words, `in`,
  units after numbers, an applied equation), contextual words (`all` as a
  value), units by position (`deg` as a parameter and as a unit), incomplete and
  invalid text, garbage and Unicode across all three encodings, the derived role
  changing the class (Source ⇄ Value on one relationship; a rule stays
  `function` with or without a definition), nominal identity over
  representation, a formula draft relative to its own text, components (class,
  instance, ports, output, clock, device, export).
- `crates/bdl-ide/tests/tokens_property.rs` — property tests: on any text the
  lexical stream is well formed and round-trips in every encoding; on text drawn
  from the language's alphabet every keyword, number and comment is classified.
- `crates/bdl-ide/src/tokens/encode.rs` — unit tests: non-ASCII round-trips,
  multi-line split, clamping.
- `crates/bdl-daemon/tests/text_e2e.rs::semantic_tokens_are_served_for_sources_and_formula_drafts`
  — over the wire: legend, generation echo, a source file's classes, a broken
  edit keeping its lexical layer without moving the project, a component body's
  draft in its own scope, a Source named in a system draft, an invalid path
  refused.
- `crates/bdl-lsp/tests/e2e.rs` — the LSP legend and data array.
- `apps/studio/test/highlighting_test.dart` — the reducer (generations, stale
  and mismatched answers, failures, byte → code-unit, survival across a pushed
  projection), `shiftSpans`, the theme's mapping and contrast on both
  appearances, the controller's rendering with marks and while typing, the
  definition editor asking for its tokens and colouring the answer.
- `docs/user-guide/screenshots/manifest.json` `code-view` — the Code view of the
  component system captured against the real daemon (`just docs-shots`).

## Performance

`cargo run --release -p bdl-ide --example perf_baseline` (best of N, one core,
2026-09-20, Apple silicon; the table in `ide-service.md` has the other queries):

| query                               | small (3/3/1) | medium (60/80/10) | large (300/400/40) |
| ----------------------------------- | ------------- | ----------------- | ------------------ |
| semantic tokens (whole document)    | 0.03 ms       | 0.50 ms           | 1.5 ms             |
| semantic tokens → LSP data (UTF-16) | <0.01 ms      | 0.07 ms           | 0.21 ms            |
| formula tokens (one draft)          | 0.01 ms       | 0.01 ms           | 0.01 ms            |

Before this change (2026-09-15) the whole-document query measured 0.03 / 0.41 /
4.3 ms; the large case is faster because the semantic layer resolves names
through an index built once per call and the merge is one linear sweep.

## Not in this design

An LSP server change beyond the legend and the encoder (the server already
served tokens); editor extensions (the VS Code extension maps only `unit` and
`slot` to TextMate scopes, the standard names need no mapping); delta tokens
(`semanticTokens/full/delta`) and range requests — the whole document is
classified in under two milliseconds at the large size; a user-editable theme; a
Studio-side tokenizer of any kind.

## Related

ADR-0001, ADR-0035, ADR-0032 (the role), ADR-0023 (the Code view);
`ide-service.md` (the service, the LSP adapter, the text workspace);
`studio-compiler-integration.md` (the request flow); `studio-ui.md` §12 (the
Code view); `docs/spec/protocol.md` (`SemanticTokens`).
