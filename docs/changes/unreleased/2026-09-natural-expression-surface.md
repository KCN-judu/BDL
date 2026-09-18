# The natural expression surface (P11)

- Date: 2026-09-18
- Area: language, ide, studio, protocol
- Affected: designers, developers, protocol clients
- Related: ADR-0028 (second amendment), ADR-0025, ISS-0001, ISS-0010

## What changed

- **Three natural forms**, each a spelling of an equation of the library and
  nothing more: `all reading in readings: reading < limit` (also `any`, `map`,
  `filter`) is `all(readings, reading => reading < limit)`;
  `angle in -45 deg .. 45 deg` is `inRange(angle, -45 deg, 45 deg)` — the closed
  range, never a value; `x ?? d` is `getOrElse(x, d)`. The elaborator lowers
  them once; the parser keeps them as their own nodes and the formatter as
  written (`docs/spec/textual-syntax.md` §17). The four binder words are
  contextual: a relationship called `map` stays callable. `..` binds between
  `in` and `??`, `??` between `..` and `+ −`; neither chains.
- **Locals are the formula's own.** A binder's local is visible in its body
  only, shadows lexically, and is never a reference to the design — rename of a
  concept leaves it alone, references never count it, tokens class it as a
  parameter, completion offers it inside the body (kind `local`) above the
  design's names. An unknown name inside a binder lists the locals in scope.
- **Diagnostics in the form's words**: _all expects a collection after 'in'._,
  _The body of 'filter' must be true or false._, _This range endpoint must be an
  angle._ / _Both ends of the range must be comparable with angle._, _A range is
  written after `in`._, _The value before `??` must be one that may be absent._
- **The Formula Composer** draws a binder as a head over an indented body with
  italic local chips and a range as two ends around `..`; a range end expects
  the subject's kind and offers its units; a `map` body expects one element of
  the result. Two new actions: **Each element** (all / any / map / filter, with
  a fresh readable local the compiler chooses) and **Range**. Completion offers
  `all item in collection: …` templates only when a collection is in sight.

## Compatibility and migration

- Designers: every existing formula parses and means what it did; `all`, `any`,
  `map`, `filter` as names keep working. Nothing to do.
- Project files: nothing.
- Protocol clients: protocol **0.13**, additive — `FormulaNode.kind` gains
  `binder` and `range`, `FormulaNode.local` / `param` / `param_type` and
  `TypeView.element` are new, `ComposeAction` gains `binder { form }` and
  `range`, `DraftCompletionItem.kind` may be `local`. A 0.12 client remains
  compatible (equal major); it sees the new kinds as unknown strings.
- Developers: `bdl_syntax::ExprKind::{Binder, Range}`, `BinaryOp::Coalesce`,
  `BinderForm`, `ast::NameExpr::local_binding`;
  `bdl_ide::NodeKind::{Binder, Range}`, `NodeKind::Reference.local`,
  `TypeView.element`, `ComposeOp::{Binder, Range}`, `CompletionKind::Local`. `?`
  and `??` are distinct tokens: `? ?? ?` is a default between two slots.

## Evidence

`crates/bdl-syntax/src/parser/tests.rs`, `crates/bdl-elab/tests/equations.rs`,
`crates/bdl-ide/tests/formula_composer.rs`,
`crates/bdl-ide/tests/acceptance.rs`,
`apps/studio/test/formula_composer_test.dart`,
`apps/studio/test/formula_composer_e2e_test.dart`; FV Phase 11 `3b4f11b`.
