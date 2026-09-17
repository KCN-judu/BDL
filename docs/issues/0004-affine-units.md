---
id: ISS-0004
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/DESIGN_ISSUES.md#di-7"]
---
# ISS-0004: Affine units

## Problem

Units are linear scalings of a dimension; affine units (°C against K) have
no model, so a temperature can only be written in kelvin.

## Why it matters

Sensors and product vocabularies speak Celsius; a designer typing `20 °C`
gets *not a unit*.

## Current evidence

* Production: `crates/bdl-elab/src/units.rs` (`UNITS`, linear only);
  `docs/TEXTUAL_SYNTAX.md` §6. DI-7.
* Formal: `BDL/Core/Typing.lean` dimensions are linear; nothing affine.

## Dependencies

* A surface-level conversion rule that keeps the kernel's linear dimensions
  (offset applied at the literal), or a decision to keep rejecting affine
  units permanently.

## Resolution

Open.
