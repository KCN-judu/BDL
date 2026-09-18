---
id: ISS-0001
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-3", "ISS-0011", "ADR-0025"]
---

# ISS-0001: Cross-domain occurrence windows

## Problem

A value carried across timing domains by `sync` is _latest-value_ transport:
occurrences of the source between two activations of the destination are not
observable unless the design keeps a log. The kernel now has list data and the
lossless window can be written as five ordinary declarations, but there is no
surface form for "the occurrences since the last activation" and no capacity
check. The paper and the formal development call the general form a _window_
model.

## Why it matters

Event-like designs (a button pressed twice between two display ticks, a counter
of pulses) cannot be expressed without it; every multi-domain example today is a
sampled value.

## Current evidence

- Production: `docs/spec/runtime-semantics.md` (transport is `sync`, strictly
  before), `docs/archive/design-issues-ledger.md` DI-3 (the original entry),
  DI-32 (only a relationship without inputs can be transported).
- Formal: BDL_FV Phase 9a (`fad79d9`, `BDL/Core/ListData.lean`,
  `BDL/Validation/Buffer.lean`, `BDL/Validation/Capacity.lean`) adds list data
  and lossless buffered transport with capacity obligations to the kernel.
- Production since 2026-09-18 (ADR-0025): `bdl-ir` has `list τ` and the list
  operators; the five-declaration buffer (`log`, `logD`, `seen`, `cursor`,
  `window`) runs in all three engines — corpus case `buffer` in
  `crates/bdl-compiler/tests/support/mod.rs`,
  `the_lossless_buffer_window_is_the_source_activations_since_the_last_slow_tick`;
  `docs/spec/kernel.md` §9b transcribes Phase 9a–9c. Capacity checking does not
  exist (ISS-0011).

## Dependencies

- A surface form for a windowed read (a `window(source)` phrase that elaborates
  to the five declarations, or the designer writes them) and its Studio
  presentation (`docs/architecture/studio-ui.md` has no mark for it).
- Capacity validation — ISS-0011.

## Resolution

Open.
