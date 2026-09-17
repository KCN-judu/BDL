---
id: ISS-0008
state: deferred
area: protocol
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-29"]
---

# ISS-0008: A structural diagnostic entity for outputs

## Problem

`bdl_diagnostics::Entity` has `Project`, `Concept` and `Mapping` only, so a
finding about a physical output (`output.missing_driver`, `output.clock_unset`)
names the sink in its message and the IDE lift recovers the entity by matching
the name.

## Why it matters

Structural anchoring is what every other entity has; name matching breaks on a
rename between analysis and display and cannot address a device.

## Current evidence

- Production: `crates/bdl-diagnostics/src/lib.rs` (`Entity`),
  `crates/bdl-ide/src/diagnostics.rs` (the lift); DI-29 (first entry).
- Protocol 0.9 bumped the minor version without adding the variant; it is
  additive and can land in any minor.

## Dependencies

- None; deferred until the next protocol change that touches `Diagnostic`.

## Resolution

Deferred: see _Dependencies_; nothing is decided by this entry.
