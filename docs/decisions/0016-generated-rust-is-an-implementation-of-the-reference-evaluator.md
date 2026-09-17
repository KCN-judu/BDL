---
id: ADR-0016
status: accepted
date: 2026-09-15
area: codegen
supersedes: []
superseded-by: []
related: []
fv:
  [
    "production-tested: differential tests against the reference evaluator
    (docs/architecture/codegen-rust.md)",
  ]
---

# ADR-0016: Generated Rust is an implementation of the reference evaluator

## Status

Accepted (backend milestone).

## Context

`bdl-reactive::eval` is the executable definition of BDL runtime behaviour:
two-phase ticks over state cells keyed by `StateCellId`, strictly-before `sync`,
agnostic declarations, structured runtime errors, IEEE `f64`. Firmware needs the
same behaviour without a tree-walking interpreter, maps, or allocation. The
temptation is either to ship the interpreter, or to let a code generator grow
its own reading of the semantics.

## Decision

- An explicit **executable IR** (`bdl-exec-ir`) sits between the Design IR and
  any backend: dense clock/input/state/output slots, first-order expressions, an
  evaluation plan. `bdl-lower` produces it from the _checked_ Design IR and the
  validated drive edges; it decides slots, writers, inlining and order once.
  Codegen consumes the plan and rediscovers nothing.
- Higher-order forms are lowered away (inlined) or refused; there is no runtime
  closure system (DI-24).
- Generated Rust goes through a small owned AST and one deterministic printer,
  never scattered `format!`. Symbols derive from stable ids; `bdl-manifest.json`
  (versioned) maps them back.
- The core is `no_std`, `forbid(unsafe_code)`, statically laid out, and
  target-independent; `runtime/bdl-runtime-core` is its only dependency and
  knows no board, device kind, transport or editor.
- The reference evaluator is **kept**, unchanged, as the oracle. The generated
  program is compared with it trace for trace — values, state, outputs, errors —
  over a corpus, golden files, and property-based designs (in process via the
  exec-IR interpreter; compiled via the host bridge). Exact equality is the
  policy; the backend performs no optimisation that would justify tolerance.
- `compile(snapshot, options)` is the one entry; readiness is decided on the
  authoritative analysis (`backend.not_ready`), never re-derived.

## Consequences

- A semantic question (e.g. strictness of `if`, DI-26) is settled in the kernel
  and the reference first; the backend follows.
- Optimisation is future work and must keep the differential tests green; any
  rewrite that changes floating-point results is a recorded numeric obligation
  (DI-1).
- The platform adapter (Embassy, peripherals from `DeploymentAnalysis`) is a
  separate artefact layered on the core; the core never sees it.
