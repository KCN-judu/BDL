---
id: ADR-0008
status: accepted
date: 2026-09-15
area: persistence
supersedes: []
superseded-by: []
related: []
fv: []
---
# ADR-0008: Stable identities are sequential per-project integers

**Status**: accepted (2026-09-15)

## Context
Identity must survive rename, layout changes, reopen, codegen, deployment
and telemetry, and must be deterministic. Candidates: UUIDs, content
hashes, sequential integers.

## Decision
Each sort (`SemanticId`, `DeclId`, `ClockId`, `OutputId`) is a distinct
newtype over `u64`, allocated from a per-project `IdAllocator` that is
persisted and never reuses a value. Display names are never identity.
This mirrors the Lean model (`⟨n : Nat⟩`).

## Consequences
* Deterministic, sortable, cheap; ordered maps give stable traversal.
* Importing/merging projects or shared component libraries will need id
  remapping; revisit (a namespace or UUID scheme) when that feature lands.
