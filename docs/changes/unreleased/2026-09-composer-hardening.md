# Composer hardening and the affine-ready chart model (P10b)

- Date: 2026-09-18
- Area: ide, studio, language
- Affected: designers, developers
- Related: ADR-0028 (amendment), ISS-0004

## What changed

- **One chart model.** `UnitDef { id, symbol, dim, chart }`; the chart (`Linear`
  or `Affine`) owns the conversion, `convert` is the one operation. The
  Phase-10b laws are property-tested on `f64`; the Celsius/Fahrenheit charts
  exist as tested infrastructure and are **not** offered anywhere.
- **Stale structure never edits current text.** Studio offers structured actions
  only over a projection of exactly the text on screen; it refuses an action
  while out of sync or while one is in flight, drops the selection when the text
  changes, and discards a compose answer for text that moved on. Unreadable text
  shows no tree; the notice says _Waiting for the compiler to read the formula…_
  or _The text cannot be read as a formula._
- **Nominal positions.** A relationship's input or an equation's argument bound
  to a concept offers only that concept's references; an equation's written
  arguments bind its variables before the expected result does.
- **Grouping by position.** Composed text is parenthesised as its position
  demands (`? / (a / (b / c))`, `((a + b) - ?) * c`, `-(Tilt + ?)`); the next
  selection after an action is the new slot, or the next slot in Tab order.

## Compatibility and migration

- Designers: the Formula view waits for the compiler after every change before
  offering the next action (tens to hundreds of microseconds of compiler time
  plus the round trip); nothing else changes.
- Developers: `UnitDef.name`/`factor` are gone (`symbol`, `chart`);
  `TypeView.nominal` is new (serialised only when true); protocol unchanged.
- Project files: nothing.

## Evidence

`crates/bdl-elab/src/units.rs`, `crates/bdl-ide/tests/formula_composer.rs`,
`apps/studio/test/formula_composer_test.dart`,
`apps/studio/test/formula_composer_e2e_test.dart`.
