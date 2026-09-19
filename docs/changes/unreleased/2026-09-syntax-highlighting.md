# Semantic highlighting: one classifier, the LSP vocabulary, colour in Studio (protocol 0.21)

- Date: 2026-09-20
- Area: ide, lsp, daemon, protocol, studio
- Affected: designers, protocol clients, developers
- Related: ADR-0035, ADR-0001, ADR-0032, ADR-0023

## What changed

- **The Code view and the formula editor are coloured.** A concept, a
  relationship (a rule, a value, a Source), an output, a device, an instance, a
  keyword, a number, a unit, an operator, a comment, a parameter and the `?`
  slot each read as what they are, in a restrained palette derived from the
  canvas's category tints, on both appearances. Declared names are heavier; a
  name bound in the formula itself is italic. Text that does not build keeps its
  keywords, numbers, comments and operators coloured; typing shifts the colours
  on show and asks again after a short pause, so nothing flickers and nothing
  stale is drawn over new text.
- **One classifier.** `bdl_ide::tokens` (`lexical_tokens`, `semantic_tokens`,
  `formula_tokens`, `legend`, `LEGEND_VERSION`, `tokens::encode`) is the only
  place a span of BDL text is classified — a lexical layer from the lossless
  tree, a semantic layer from the snapshot and the derived role (ADR-0032),
  merged. The vocabulary is the LSP semantic-token names (`type`, `function`,
  `variable`, `parameter`, `keyword`, `number`, `operator`, `comment`,
  `namespace`, `class`, `property`, `enumMember`) plus `unit` and `slot`;
  modifiers `declaration`, `defaultLibrary`, `source`, `output`, `device`,
  `instance`, `unresolved`. A Source is `variable` + `source`, a rule
  `function`, a value `variable`; an applied equation of the library is
  `function` + `defaultLibrary`.
- **The language server** serves the same legend and stream, encoded by the pure
  encoder in the client's position encoding (UTF-8 / UTF-16 / UTF-32). The old
  BDL-specific type names (`concept`, `mapping`, `output`, `clock`, `device`,
  `macro`) are gone from the legend.
- **The daemon** answers `SemanticTokens` for a source file (as an overlay on
  the sources as written back, so the other files resolve) or a relationship's
  draft (in its scope); the legend travels with every answer.

## Compatibility and migration

- Designers: nothing to do. No setting; the colours follow the appearance.
- Project files: nothing.
- Protocol clients: protocol **0.21**, additive — the `SemanticTokens` request
  and response, `FormulaDocument`, `SemanticTokenLegend`, `SemanticToken`. A
  0.20 client is unaffected.
- LSP clients: the legend's names changed to the standard ones; an editor that
  themed `concept` / `mapping` / `output` / `clock` / `device` / `macro` by name
  now gets the standard theme for `type` / `function` / `variable` / `namespace`
  and should map `unit` and `slot` if it wants them distinct (the VS Code
  extension does). A client that has not seen `LEGEND_VERSION` 1 must not assume
  the earlier index order.
- Developers: `bdl_ide::TokenKind` is gone; `bdl_ide::TokenType`,
  `TokenModifiers`, `Legend`, `SemanticToken` (`{ range, ty, modifiers }`),
  `legend`, `lexical_tokens`, `semantic_tokens`, `formula_tokens`,
  `LEGEND_VERSION` and `bdl_ide::tokens::encode` (`encode`, `encode_data`,
  `decode_data`, `byte_range`, `PositionEncoding`, `LineIndex`) replace it;
  `bdl_lsp::convert::{legend, semantic_tokens}` take the encoding; Studio gains
  `app/highlighting.dart` (`HighlightState`, `HighlightSpan`, `shiftSpans`,
  `spansOf`), `ui/code/{syntax_theme,highlighting_controller}.dart`, the actions
  `SemanticTokensRequested` / `SemanticTokensReceived` / `SemanticTokensFailed`
  and the effect `FetchSemanticTokens`; the definition editor's
  `_MarkedController` is the shared `HighlightingController`.

## Evidence

`docs/architecture/syntax-highlighting.md` § Tests and § Performance;
`crates/bdl-ide/tests/{tokens,tokens_property}.rs`,
`crates/bdl-daemon/tests/text_e2e.rs`, `crates/bdl-lsp/tests/e2e.rs`,
`apps/studio/test/highlighting_test.dart`, the `code-view` screenshot.
