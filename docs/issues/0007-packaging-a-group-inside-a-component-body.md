---
id: ISS-0007
state: deferred
area: behavior-systems
opened: 2026-09-16
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-44"]
---
# ISS-0007: Packaging a group inside a component body

## Problem

Extracting a group that lives inside a component body would create a
component and an instance *inside* the parent, but a body is a flat design
and cannot hold an instance.

## Why it matters

Nested reuse (a component built from components) is the natural next step
of packaging; without it a body is flat.

## Current evidence

* Production: `preview_extraction` refuses a component-scoped group with
  `extract.not_a_base_group`; `docs/architecture/behavior-systems.md` §13
  (option A); DI-44.
* Formal: `BDL/Behavior/Extract.lean` extracts from a `GroupedDesign`, not
  from a body.

## Dependencies

* A nested component model (body as a system) in the formal development
  first, or a lowering that re-flattens without a second semantic truth.

## Resolution

Deferred: see *Dependencies*; nothing is decided by this entry.
