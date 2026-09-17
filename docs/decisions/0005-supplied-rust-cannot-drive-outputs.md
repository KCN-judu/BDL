---
id: ADR-0005
status: accepted
date: 2026-09-15
area: runtime
supersedes: []
superseded-by: []
related: []
fv: []
---

# ADR-0005: Supplied Rust cannot access outputs

**Status**: accepted (2026-09-15)

## Context

Expert-supplied Rust computation is necessary (filters, estimators). If a
component could receive a PWM handle or an `OutputId` writer, it would bypass
single-driver, semantic identity, clock discipline and allocation.

## Decision

Supplied components are pure (`I → O`) or stateful (`S × I → S × O`) values in,
values out. They never receive GPIO/PWM handles, output writers, HAL devices or
output capabilities. They are classified as _trusted implementation code_ with a
constrained API — not sandboxed — and carry a manifest of obligations with
evidence origins.

## Consequences

- Physical effect always flows through one final `OutputId` driver.
- Documentation must not claim a sandbox; a stronger one is future work.
