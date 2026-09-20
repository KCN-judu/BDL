---
id: ISS-0006
state: deferred
area: runtime
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-28", ADR-0037]
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
- Since ADR-0037 the RP2040 firmware runs the core's `f64` arithmetic as
  generated (the Cortex-M0+ has no FPU; software floating point) and converts
  only at the raw command boundary through one explicit policy
  (`bdl_runtime_adapter::duty8`, `docs/architecture/embedded-adapter.md`).
  Whether the core itself should compute in `f32` on such a target — and what
  the differential tests would then compare — is this issue, unchanged.

## Dependencies

- The first embedded platform adapter (roadmap priority 1).

## Resolution

Deferred: see _Dependencies_; nothing is decided by this entry.
