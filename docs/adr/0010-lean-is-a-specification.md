---
id: ADR-0010
status: accepted
date: 2026-09-15
area: formal
supersedes: []
superseded-by: []
related: []
fv: ["informed by FV: the Lean development is the semantic authority (docs/02-kernel-spec.md)"]
---
# ADR-0010: The Lean development is a specification, not a dependency

**Status**: accepted (2026-09-15)

## Context
BDL_FV defines and checks the semantics. Invoking Lean at edit time would be
slow and would make the tool depend on a proof assistant.

## Decision
Lean provides semantics, theorem-backed design decisions, reference
examples and counterexamples. Rust is the production implementation.
`docs/02-kernel-spec.md` transcribes the definitions; deviations are
recorded in `docs/IR.md` and `docs/DESIGN_ISSUES.md`. Lean examples become
Rust golden/differential tests where practical.

## Consequences
* Documentation says "follows the formally developed semantics", never
  "formally verified".
* A future formal verification of the Rust compiler is a separate project.
