---
id: ADR-0014
status: accepted
date: 2026-09-15
area: textual
supersedes: []
superseded-by: []
related: []
fv: []
---

# ADR-0014: Textual syntax infrastructure — Logos, hand-written event parser, Rowan

**Status**: accepted (2026-09-15)

## Context

The formula field was served by a 500-line hand lexer and Pratt parser producing
a small AST with `f64` literals. The textual authoring surface
(`docs/spec/textual-syntax.md`) needs whole-file parsing with declarations,
types, enums, `match`, blocks and patterns; error recovery that keeps parsing
after a mistake; a lossless tree for highlighting, formatting, rename and
visual/textual round-trip; and literals that keep their exact spelling for the
planned exact/symbolic numeric layer. There must be one expression grammar for
both the formula field and the file.

## Decisions

1. **Lexer: Logos 0.16.** Derive-generated, longest-match, byte-spanned, total
   (unmatched input becomes `Error` tokens covering whole UTF-8 characters).
   Keywords are split off identifiers in code so one regex defines what an
   identifier is. Numbers are tokens of text only.
2. **Parser: hand-written recursive descent with a Pratt loop for expressions**,
   emitting _events_ (`StartNode`/`Token`/`FinishNode`/ `Error`) with
   `Marker`/`CompletedMarker::precede` in the rust-analyzer style. A separate
   sink replays events into the green tree and re-inserts trivia. Recovery
   anchors per context are documented in `docs/spec/textual-syntax.md` §9.1;
   every loop consumes or stops at an anchor.
3. **Tree: Rowan 0.17.** Lossless: `tree.text() == source` for all input. One
   `SyntaxKind` covers tokens and nodes; `ErrorNode` wraps skipped tokens. Typed
   AST wrappers (`bdl_syntax::ast`) are views over the tree and store nothing.
4. **Literals are spellings.** `NumberLiteral { text }` with an exact `Decimal`
   reading; the only `f64` is made in `bdl-elab` at the elaboration boundary,
   marked as debt (DI-18).
5. **Lowering, not direct construction.** `bdl_syntax::lower` produces an
   id-free surface tree (`SurfaceModule`, `SurfaceExpr`, …); `bdl-elab` keeps
   consuming `SurfaceExpr`. Nothing semantic enters the parser.
6. **One grammar.** The formula field parses with the `Formula` root of the same
   parser; the old separate formula parser is gone.

## Rejected

- **Parser generators (LALRPOP, Pest)**: error recovery and lossless trees are
  afterthoughts in generated parsers; the grammar is small enough that a
  hand-written parser is shorter than the glue a generator needs, and
  diagnostics must be authored per context in product language.
- **Combinator frameworks (chumsky, nom, winnow)**: good for all-or-nothing
  parsing; the partial-tree-with-recovery requirement and the event/sink split
  fit them badly, and Pratt precedence with non-associative comparisons is
  simpler by hand.
- **Haskell layout rule**: significant whitespace adds a lexer state machine,
  makes copy-paste and formatting fragile, and buys nothing once declarations
  are keyword-delimited and calls use parentheses.
- **Juxtaposition application** (`f x`): would make `90 deg` ambiguous and every
  missing operator a silent call.

## Consequences

- `bdl-syntax` depends on `logos` and `rowan` (both `unsafe` internally; the
  crate itself stays `#![forbid(unsafe_code)]`).
- The tree carries `enum`, `match`, blocks, `let` and constructor patterns
  before the surface model or kernel do (DI-19); `bdl-elab` reports them as
  `formula.unsupported` until elaboration catches up.
- Golden corpus in `crates/bdl-syntax/test_data/{valid,invalid}` with `.snap`
  files; property tests assert no-panic and losslessness on arbitrary strings
  and token soups.
