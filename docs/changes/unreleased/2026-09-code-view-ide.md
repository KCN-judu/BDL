# The Code view is an IDE surface: completion, hover, navigation, references, format (protocol 0.22)

- Date: 2026-09-20
- Area: ide, lsp, daemon, protocol, studio
- Affected: designers, protocol clients, developers
- Related: ADR-0035, ADR-0023, ADR-0017, ADR-0001,
  [2026-09 syntax highlighting](2026-09-syntax-highlighting.md)

## What changed

- **Designers**: in the Code view, ⌃Space opens completion at the caret — the
  compiler service's candidates for this spot: a concept after `:`, a clock
  after `@`, the items allowed at an item start, and inside a formula the
  inputs, other relationships (a rule as a call, a Source or a value as a name),
  locals, the equation library, units after a number — ↑/↓, Return or Tab,
  Esc, re-asked as you type. Resting the pointer on a name shows its card
  (declaration, what it produces, its state, its role). ⌘-click or F12 goes to
  where a name is declared, in this file or another; ⇧F12 lists every place
  that names it under the editor. _Format_ in the file bar or ⌥⇧F lays the
  file out the canonical way as one edit, keeping the caret's line and column;
  a file that does not parse cleanly is left as it is. The formula field's
  hover now also names a relationship, an equation of the library and a
  parameter's concept.
- **One answer to _what is at a position_**: `bdl_ide::navigation` —
  `name_at`, `hover_at`, `definition_at`, `references_at` over a document and
  `formula_name_at`, `formula_hover_at` over a relationship's draft — resolved
  through the projection map and the elaborator's input environment, never by
  spelling. Among the entities a name site stands for, the authored one wins
  (a port over the flattened copies it backs) and a flattened copy is
  presented by its authored name. The language server's hover, definition and
  references, the daemon's formula hover and the Code view's queries all call
  these.
- **Completion** in a document finds the enclosing formula body by role
  (a name anchor inside it no longer hides it), delegates a definition line
  whose body has not built to the formula engine for the relationship the line
  defines, and offers no equation after a number.
- **Protocol 0.22**: `SourceCompletion`, `SourceHover`, `SourceDefinition`,
  `SourceReferences`, `FormatSource` over a source file's text as typed, on the
  overlay `SemanticTokens` set; `DraftHoverResponse.equation`. The daemon
  recomposes the workspace only when the file's text differs from the overlay
  already set.
- **Studio**: `app/code_tooling.dart` (the transitions, with the generation
  guards of the formula field's tooling); `ui/code/completion_popup.dart` and
  `ui/code/hover_card.dart` are the one pop-up and the one card for both text
  fields, anchored at the caret or the name in the Code view and flipped above
  when there is no room below; `SourceReveal`, `SourceReferencesState`,
  `SourcesState.formatGeneration` / `formatting` / `replaced`.

## Compatibility and migration

- Designers: nothing to do. The new keys are listed in the user guide's
  keyboard reference; nothing that worked changed meaning.
- Project files: nothing.
- Protocol clients: protocol **0.22**, additive — five requests, three
  responses, one field. A 0.21 client is unaffected.
- LSP clients: hover over whitespace, a keyword or a number now answers nothing
  (it answered the enclosing item's card before); hover over an equation of
  the library answers its shape and summary; every hover carries its range.
- Developers: `bdl_ide::{name_at, hover_at, definition_at, references_at,
  definition_sites, reference_sites, formula_name_at, formula_hover_at,
  NameAt, HoverAt, HoverContent}`; `bdl_lsp::convert::hover_at`;
  `Session::{source_completion, source_hover, source_definition,
  source_references, format_source, draft_hover → Option<HoverAt>}`; Studio's
  `CompletionState.mappingId` is nullable and `path` names a source pop-up,
  `HoverState.path` a source hover; the actions `SourceCompletionRequested`,
  `SourceHoverRequested`, `SourceDefinitionRequested`,
  `SourceReferencesRequested`, `ReferencesDismissed`, `FormatSourceRequested`
  and their `…Received` answers; the effects `CompleteSource`, `HoverSource`,
  `DefineSource`, `ReferencesSource`, `FormatSource`.

## Evidence

`crates/bdl-ide/tests/{navigation,source_completion}.rs`,
`crates/bdl-daemon/tests/text_e2e.rs::source_queries_answer_over_the_text_as_typed`,
`crates/bdl-lsp/tests/{e2e,text_workspace}.rs`,
`apps/studio/test/{code_tooling_test,code_ide_e2e_test}.dart`, the
`code-completion` and `code-hover` screenshots;
`docs/architecture/studio-compiler-integration.md` § The Code view's IDE
queries; `docs/architecture/ide-service.md` § Performance baseline.
