# Composite units after a number, the value categories, and the Formula view's structure (protocol 0.27)

- Date: 2026-09-21
- Area: language, textual, compiler, ide, protocol, daemon, library
- Affected: designers, protocol clients, developers
- Related: ADR-0040, ADR-0028, ISS-0004, FV Phase 10 (`Surface/Units.lean`),
  [2026-09 Formula Composer](2026-09-formula-composer.md)

## What changed

- **A unit after a number may be an expression**: `180 deg per s`,
  `9.81 m per s^2`, `1 N * m`, `1 kg * m per s^2`, `2 s^-1`. `per` is the unit
  quotient, `*` the unit product, `^n` a whole-number power; `/` stays the value
  division it always was (`Distance / Duration`; `10 m / s` divides by a
  reference named `s`). The unit's dimension and scale are computed from the
  registered atoms — `rad per s` is an angular rate (angle stays a base
  dimension), `deg per s` scales by π/180 against it, `36 km per h` is
  `10 m per s` — and no composite is a registry row. Studio may render `rad/s`,
  `m/s²`, `kg·m/s²`; the source stays `per`, `*`, `^n`.
- **The formatter** writes `m per s^2`, `N * m`, `kg * m per s^2` (one space
  around `per` and `*`, none around `^`).
- **Diagnostics in the designer's words**: _`furlong` in `m per furlong` is not
  a unit_, _`deg per s` is a unit of an angular rate, but this value must be a
  length_, _unit powers use a whole-number exponent, such as `s^2` or `s^-1`_,
  _a unit expression has one `per`: write a power instead, as in `m per s^2`_,
  _`°C` is an absolute temperature unit: it cannot be multiplied, divided or
  raised to a power_, _`m^1000` is beyond the unit powers this version
  represents (up to ±127)_. Derived dimensions are named by the vocabulary (_an
  acceleration_, _a force_, _a torque_).
- **The vocabulary's preferred units are canonical spellings** (`m per s`,
  `rad per s`, `N * m`) with a curated list of composite candidates per named
  quantity; `ListValueCategories` serves every category — the named quantities,
  the truth value, the count — with its display name, type name, value form,
  dimension, preferred unit and unit candidates as `UnitExprView` descriptors,
  so an authoring UI needs no concept preset and builds no unit string.
  `CreateSource` accepts `NewConcept.category_id`.
- **The Formula view's structure on the wire**: every projection node carries
  its `role` (`condition`, `numerator`, `argument 1`, `body`, …), the formula's
  `locals` in scope there and where a further child goes; `match`, `let` blocks,
  rules, collection and grouped literals, `delay` and `sync` are structured
  nodes (only `()` stays opaque); a composite unit literal carries its canonical
  spelling and its rendering. Structural carets (`NavigateFormula` — left,
  right, up, down, exit, next / previous slot), keyboard insertion at a caret
  (`ComposeAction.insert`: `+` → `node + ?`, `clamp` into a slot →
  `clamp(?, ?, ?)`, `deg` after `180` → `180 deg`, then `per`), completion at a
  caret with `structured_insert` and composite-unit suggestions (`10 d` → `deg`,
  `deg per s`; `10 deg per` → `s` first), signature help (`GetFormulaSignature`)
  and the saved-formula render (`GetFormulaRender`) — all from the one parser
  and elaboration; the daemon keeps no caret.

## Compatibility and migration

- Designers: every existing literal (`90 deg`, `2.5 s`, `300 lx`, `25.4 mm`) and
  every `/` keep their meaning; existing projects parse and elaborate unchanged
  (the parser continues a unit past `*` only before a registered atom, so
  `90 deg * gain` is untouched). A relationship named `per` is still legal.
- Protocol clients: 0.27 is additive; a 0.26 client sees new node kinds
  (`match`, `arm`, `block`, `let`, `rule`, `list`, `tuple`, `delay`, `sync`)
  where it saw `opaque` — it should render an unknown kind as its `text`.
  `QuantityView.unit` still carries the display rendering.
- Standard Library: schema 2 files and their `unit` symbols load unchanged; the
  presets stay served.
- Developers: the unit registry moved to `bdl_model::units` (re-exported from
  `bdl_elab::units`; every path compiles);
  `bdl_model::units::{UnitExpr, UnitFactor, UnitError, MAX_EXPONENT, candidates_for, preferred_for}`,
  `quantity::{preferred_units, QuantityDef::display_name}`,
  `bdl_syntax::{SurfaceUnitFactor, Unit::{numerator, denominator}}`,
  `SyntaxKind::{Caret, UnitFactor}`, `SyntaxErrorCode::MalformedUnit`,
  `bdl_ide::{Side, Caret, Motion, navigate, caret_offset, signature}`,
  `bdl_ide::{SignatureHelp, ParameterHelp, render, FormulaRender, Fragment}`,
  `ComposeOp::Insert`, `CompletionContext::FormulaCaret`,
  `SemanticCompletion::structured_insert`,
  `NodeKind::{Match, Arm, Block, Let, Rule, List, Tuple, Delay, Sync}`,
  `FormulaNode::{role, locals, append_at}`, `IdeHost::committed_snapshot`;
  `bdl_protocol::convert::{unit_expr_view, value_categories_response, representation_of_category}`.

## Evidence

`crates/bdl-elab/tests/dimension_algebra.rs` (the existing algebra through the
pipeline; the composite literals' dimensions and magnitudes; the faults),
`crates/bdl-model/tests/unit_expr.rs` (the properties, the conversion
identities, equivalence, normalisation, the affine and exponent boundaries),
`crates/bdl-syntax/tests/units.rs` (the grammar, the unit/arithmetic boundary,
the malformed forms, the formatter), `crates/bdl-ide/tests/formula_structure.rs`
(roles, locals, carets, insertion, completion, signature, render, a composite
unit switched), `crates/bdl-daemon/tests/{stdio_e2e,system_e2e}.rs` (the wire).
