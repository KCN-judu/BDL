---
id: ADR-0011
status: accepted
date: 2026-09-15
area: language
supersedes: []
superseded-by: []
related: []
fv: ["engineering choice: deviation from the kernel's Nat, recorded in docs/DESIGN_ISSUES.md DI-1, DI-15"]
---
# ADR-0011: Production numerics are IEEE floats, recorded as a deviation

**Status**: accepted (2026-09-15)

## Context
The kernel evaluates over `Nat` (truncating subtraction, floor division),
which was enough to prove determinism and totality. Products measure
angles, temperatures and levels in `[0,1]`.

## Decision
IR literals are `Scalar(f64)` with bit-pattern equality (hashable,
deterministic). Generated code uses `f32` on device by default. Symbolic
normalization must record any numeric deviation as an obligation rather
than silently rewrite. Differential tests against Lean fixtures are
restricted to integer-valued cases.

## Consequences
* `Prim::compute` semantics for `sub`/`div` differ from Lean on negative
  results and non-integer quotients; documented in DESIGN_ISSUES DI-1.
* Range/limit validation (not modelled in the paper) becomes necessary
  engineering work.
