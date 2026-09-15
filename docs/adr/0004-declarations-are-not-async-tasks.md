# ADR-0004: BDL declarations are not Embassy tasks

**Status**: accepted (2026-09-15)

## Context
The obvious Embassy mapping — one task per declaration, channels for wires —
makes BDL meaning depend on executor scheduling and breaks the synchronous
tick semantics (`delay` reading the previous instant; `sync` reading a
strictly-earlier committed activation).

## Decision
Code generation emits one deterministic step function per clock domain
over explicit previous/next state, with a separate output commit phase.
Runtime adapters (Embassy first) only turn timers/interrupts into domain
activations and commit outputs; they never own semantics.

## Consequences
* BDL semantics do not depend on Embassy; other backends are possible.
* The same generated core runs on host and device.
* Cross-domain reads are snapshot reads, never recursive step calls.
