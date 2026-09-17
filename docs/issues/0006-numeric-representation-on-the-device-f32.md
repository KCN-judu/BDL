---
id: ISS-0006
state: deferred
area: runtime
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-28"]
---

# ISS-0006: Numeric representation on the device (`f32`)

## Problem

Generated code computes in `f64` everywhere, matching the reference evaluator
bit for bit; an embedded target may want `f32`, which the differential tests
cannot hide.

## Why it matters

The first platform adapter will meet this on the first constrained board.

## Current evidence

- Production: `docs/architecture/codegen-rust.md`,
  `docs/spec/runtime-semantics.md` (numeric policy, DI-15); DI-1, DI-28.

## Dependencies

- The first embedded platform adapter (roadmap priority 1).

## Resolution

Deferred: see _Dependencies_; nothing is decided by this entry.
