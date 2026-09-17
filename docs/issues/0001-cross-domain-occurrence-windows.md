---
id: ISS-0001
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-3"]
---
# ISS-0001: Cross-domain occurrence windows

## Problem

The production kernel (`bdl-ir`) has no list type, so a value carried across
timing domains by `sync` is *latest-value* transport: occurrences of the source
between two activations of the destination are not observable. The paper and
the formal development call the general form a *window* model.

## Why it matters

Event-like designs (a button pressed twice between two display ticks, a
counter of pulses) cannot be expressed without it; every multi-domain example
today is a sampled value.

## Current evidence

* Production: `docs/spec/runtime-semantics.md` (transport is `sync`, strictly
  before), `docs/archive/design-issues-ledger.md` DI-3 (the original entry), DI-32 (only a
  relationship without inputs can be transported).
* Formal: BDL_FV Phase 9a (`fad79d9`, `BDL/Core/ListData.lean`,
  `BDL/Validation/Buffer.lean`, `BDL/Validation/Capacity.lean`) adds list
  data and lossless buffered transport with capacity obligations to the
  kernel. Production has not mirrored it; `docs/spec/kernel.md` still
  transcribes the pre-9a kernel.

## Dependencies

* A production mirror of Phase 9a (`bdl-ir` list type, buffer primitives,
  capacity checking) — a kernel extension, so Lean first (ADR-0010).
* A surface form for a windowed read and its Studio presentation
  (`docs/architecture/studio-ui.md` has no mark for it).

## Resolution

Open.
