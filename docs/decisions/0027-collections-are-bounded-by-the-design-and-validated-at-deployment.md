---
id: ADR-0027
status: accepted
date: 2026-09-18
area: deployment
supersedes: []
superseded-by: []
related: ["ADR-0024", "ADR-0016", "ISS-0011", "ISS-0013"]
fv:
  [
    "informed by FV: BDL_FV Phase 9a `Validation/Capacity.lean` — capacity is
    a validation obligation (`CapacitySufficient`, `requiredCapacity`,
    `periodic_capacity_sufficient`); under sufficient capacity every overflow
    policy is the identity (`sufficient_capacity_preserves`,
    `bounded_buffer_agrees`), and only rejecting an insufficient deployment
    keeps the unbounded semantics",
  ]
---

# ADR-0027: A collection is bounded by what the design writes, the toolchain computes and validates the bound, and no bounded representation or overflow policy lives in the language or the generated code

## Status

Accepted (2026-09-18, the P9 hardening pass; closes ISS-0011).

## Context

ADR-0024 put collections in the generated core as `alloc::Vec` and left
capacity as an obligation "to be computed by the toolchain". The Phase-9a
window written as five declarations keeps a log that grows by one per source
activation — the formal construction, and as a deployed core an ever-growing
heap. The hardening brief asked how capacity validation enters production,
whether heap allocation is the right representation on RP2040/ESP32-S3, whether
a fixed-capacity or ring-buffer representation should replace `Vec` for
transport buffers, and how overflow is to be handled. Constraints: the list
semantics must not change; nothing may be dropped silently; capacity is
deployment data (rates are not in the design); the kernel gains no notion of
capacity, buffer or ring.

## Decision

1. **The design writes its bounds.** A remembered collection is bounded by
   the operators that bound it — `take cap …` — and by nothing else. The
   deployable window is the Phase-9a construction with `take cap` on the log
   and the count kept apart (`docs/spec/deployment-capacity.md` §5). Which
   values a bounded design keeps is what it says it keeps: `take cap (cons x …)`
   keeps the newest (FV `dropOldest`), and that is the overflow policy — in the
   design, explicit, semantic, differentially tested.
2. **The toolchain computes the bound and the requirement.** A sound static
   bound per declaration and cell (`bdl-exec-ir::bounds`), the required window
   capacity per crossing under the schedule (`bdl-reactive::capacity`, the
   production `Capacity.lean`), a report and diagnostics
   (`bdl-compiler::collections`): an unbounded state is reported on a host and
   **refuses the artefact on a bounded-memory target**; a synced list that
   keeps fewer values than the crossing produces is told so; an input-sized
   list is the platform adapter's to bound.
3. **The runtime representation stays `Vec`**, last element first, with the
   generator keeping the library's folds linear (moves, a lazy `if` on total
   branches, borrowed reads, commit-only writes). No fixed-capacity array, no
   bounded-vector abstraction, no ring buffer: under a validated bound a `Vec`
   grows to that bound and no further, which is the memory behaviour a bounded
   array would have, without a second representation, a second set of operators
   or a runtime failure the formal model does not have.
4. **Nothing is exposed.** Capacity, the backing storage and the allocator are
   not properties of a list value; the manifest and the report describe the
   program, not the language.

## Alternatives

- **A fixed-capacity array + length, or a bounded vector, for all lists**:
  rejected — a second representation with the same asymptotics as a `Vec` that
  has reached its bound, plus an overflow failure the kernel does not have.
  Reconsidered only if a target cannot provide an allocator at all.
- **A ring buffer specialised for transport buffers**: rejected for now — the
  window is not a construct the compiler can recognise without a surface form
  (ISS-0001), and its observable declarations (`log`, `logD`, `seen`) are the
  formal construction's; a bounded refinement must be visible in the design to
  be equivalent, and `take cap` is exactly that. Reconsidered with ISS-0001.
- **A capacity primitive or an overflow policy in the language or the core**:
  rejected — the kernel has no such notion; a core that drops a value changes
  the trace (FV `insufficient_capacity_counterexample`).
- **Recognising the five-declaration idiom and bounding it silently**:
  rejected — a hidden transformation of an observable declaration.

## Consequences

- A designer bounds state with `take`; the tool says where it is missing and
  what each crossing needs; a bounded-memory deployment is refused rather than
  degraded.
- The first platform adapter declares an allocator over an arena sized from
  the manifest's `state_bytes_max` + `tick_bytes_max` and states the most it
  will supply for each list input.
- ISS-0011 is resolved; ISS-0013 is narrowed to `zip`.
- Records changed: `docs/spec/deployment-capacity.md` (new),
  `docs/architecture/codegen-rust.md`, `docs/architecture/executable-ir.md`,
  `docs/architecture/compiler-pipeline.md`, `docs/project/status.md`,
  `docs/project/roadmap.md`, `docs/project/formal-correspondence.md`, the
  change fragment `2026-09-p9-hardening.md`.
