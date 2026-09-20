---
id: ISS-0011
state: resolved
area: deployment
opened: 2026-09-18
resolved-by: ["ADR-0027"]
related: ["ISS-0001", "ADR-0024", "ADR-0027"]
---

# ISS-0011: Capacity validation for collections and the lossless buffer

## Problem

The kernel's collections are unbounded and the Phase-9a buffer is written as
ordinary declarations; a deployment has finite memory, and nothing in the
toolchain yet computes whether a design's collections — in particular a
cross-domain window — fit a target.

## Why it matters

Without a capacity check a list-carrying design generates a core that allocates
without bound; the only overflow policy that preserves the kernel's semantics is
to refuse the deployment (FVD-0086), and today nothing refuses.

## Current evidence

- Production: `list τ` values, `fold`, and the five-declaration buffer run in
  the reference evaluator, the executable-IR interpreter and the generated core
  (corpus cases `collections`, `buffer`); the manifest records
  `requires_allocator`; no `CapacitySufficient` exists.
- Formal: `BDL/Validation/Capacity.lean` — `CapacitySufficient S src dst cap T`
  decidable for a finite horizon, `requiredCapacity` least sufficient,
  `periodic_capacity_sufficient` (one destination period suffices for periodic
  schedules), `dropOldest`/`dropNewest` identity under sufficient capacity.

## Dependencies

- Schedules as deployment data (rates are validation data, never in the design);
  a target memory model in `docs/spec/hardware-model.md`; the first embedded
  platform adapter's allocator.

## Resolution

Resolved by ADR-0027 (2026-09-18): the design writes its bounds (`take cap`),
the toolchain computes a sound static bound per declaration and cell
(`bdl-exec-ir::bounds`), the required window capacity per crossing under the
deployment schedule (`bdl-reactive::capacity`, the production `Capacity.lean`),
and a report with diagnostics (`bdl-compiler::collections`); an unbounded state
refuses a bounded-memory deployment. Specification:
`docs/spec/deployment-capacity.md`. What remains — a target memory model beyond
the byte estimates, the adapter's arena and input bounds — is roadmap
priority 1.
