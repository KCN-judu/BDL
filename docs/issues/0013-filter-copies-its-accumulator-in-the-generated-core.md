---
id: ISS-0013
state: open
area: codegen
opened: 2026-09-18
resolved-by: []
related: ["ADR-0024", "ADR-0016", "ADR-0027"]
---

# ISS-0013: `zip` copies its accumulator per element in the generated core (`filter` no longer does)

## Problem

As opened (2026-09-18): `filter` is
`fold (λx acc. ite (p x) (cons x acc) acc) [] xs`; `ite` is strict (DI-26), so
the generated core built both branches and cloned the accumulator `Vec` per
element — quadratic. The audit of the P9 hardening pass found the same cost in
`map` and `append`: the generator cloned _every_ local, so any fold that
`cons`ed onto its accumulator copied it per element.

**Narrowed** (2026-09-18, `496f1f2`): the generator moves a local referenced
once in its own scope and emits a strict `ite` whose branches are total as a
Rust `if` (not observable: neither branch can fail), so `map`, `filter`,
`append`, `sum`, `any`, `all`, `contains` are linear in the generated core
(measured 2 000 → 32 000 elements: ×15). What remains is **`zip`**: its step's
accumulator is the pair `(rest, out)`, and `rest` is used twice — `take 1 rest`
and `drop 1 rest` — so the pair is cloned per element and `zip` is quadratic (2
000 → 8 000: ×10).

## Why it matters

A long pair of collections zipped every tick costs more than it should on a
microcontroller; the semantics are unaffected, and a bounded design's cost is
bounded.

## Current evidence

- `crates/bdl-compiler/tests/golden/collections/src/lib.rs` (`pairs`):
  `list::drop(1.0_f64, l20.clone().0), … list::take_of(1.0_f64, &l20.0) … l20.clone().1`.
- `collections_cost_measurement` in
  `crates/bdl-compiler/tests/backend_differential.rs`;
  `docs/evidence/testing.md` §Collections cost.

## Dependencies

- A last-use analysis that orders the pair's construction so `take 1 rest` (a
  borrow) precedes `drop 1 rest` (a move) — sound only for total
  sub-expressions, which the step is; or a different but equivalent library
  expansion of `zip` (a change to the FV correspondence of `zipF`).

## Resolution

Open, narrowed to `zip`.
