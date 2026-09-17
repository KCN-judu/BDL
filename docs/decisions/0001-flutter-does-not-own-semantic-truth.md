---
id: ADR-0001
status: accepted
date: 2026-09-15
area: studio
supersedes: []
superseded-by: []
related: []
fv: []
---

# ADR-0001: Flutter does not own semantic truth

**Status**: accepted (2026-09-15)

## Context

A visual editor is tempted to compute "is this wire valid?" locally for
responsiveness. Doing so creates a second implementation of the language that
drifts from the compiler, and moves semantic decisions into widgets.

## Decision

Every semantic judgment — type validity, semantic identity, dimensions,
causality, clock compatibility, output ownership, hardware feasibility,
simulation — is answered by the Rust compiler and delivered to Studio as a
projection. Studio may render an edit optimistically but never derives a
semantic fact. Studio state is split into semantic projection / editor state /
rendering state; only the first comes from Rust and Studio never mutates it.

## Consequences

- Studio has no model classes for BDL beyond the protobuf projection.
- Latency of an edit round-trip is a UX cost to be engineered (incremental
  analysis, deltas), never worked around by local semantics.
- A CLI, CI, or textual editor shares exactly the same semantics.
