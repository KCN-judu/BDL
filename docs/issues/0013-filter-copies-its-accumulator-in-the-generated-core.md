---
id: ISS-0013
state: open
area: codegen
opened: 2026-09-18
resolved-by: []
related: ["ADR-0024", "ADR-0016"]
---

# ISS-0013: `filter` copies its accumulator per element in the generated core

## Problem

`filter` is `fold (λx acc. ite (p x) (cons x acc) acc) [] xs`; `ite` is strict
(DI-26), so the generated core builds both branches and must clone the
accumulator `Vec` for the `cons` branch on every element — quadratic in the
list's length. The reference evaluator and the interpreter share the list and
stay linear.

## Why it matters

A long collection filtered every tick costs more than it should on a
microcontroller; the semantics are unaffected.

## Current evidence

- `crates/bdl-compiler/tests/golden/collections/src/lib.rs` (`kept`):
  `prim::ite(…, list::cons(l43.clone(), l44.clone()), l44.clone())`.
- `docs/architecture/codegen-rust.md` records the cost.

## Dependencies

- Either a lazy conditional in the executable IR (a semantic change for `ite`'s
  strictness, DI-26) or a shared list representation in the core (ADR-0024's
  rejected alternative), or peephole emission of `ite` over `cons`/`acc` as a
  Rust `if`.

## Resolution

Open.
