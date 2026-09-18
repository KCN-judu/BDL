---
id: ISS-0004
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-7", "ADR-0028"]
---

# ISS-0004: Affine units

## Problem

Units are linear scalings of a dimension; affine units (°C against K) have no
model, so a temperature can only be written in kelvin.

## Why it matters

Sensors and product vocabularies speak Celsius; a designer typing `20 °C` gets
_not a unit_.

## Current evidence

- Production: `crates/bdl-elab/src/units.rs` — a registry of units with a stable
  id and a `Chart` onto the canonical magnitude, `Linear { factor }` only;
  `convert(x, from, to)` is the one conversion operation and hides the chart's
  shape, so an affine chart is a new variant, not a new API (ADR-0028).
  `docs/spec/textual-syntax.md` §6. DI-7.
- Formal: BDL_FV Phase 10 (`Surface/Affine.lean`) shows °C/°F literals and
  coordinates elaborate exactly with no kernel change (`celsius_not_linear`,
  `affLitE_typed`, `affine_roundtrip_ev`); Phase 10b (`Surface/Charts.lean`,
  `bd87b66`) proves conversion between charts is a groupoid of affine maps
  (`convert_compose`, `convert_inverse`, `display_switch_preserves_quantity`)
  and that a point/difference sort is optional validation information,
  orthogonal to conversion (`sort_orthogonal_to_conversion`). Production has not
  consumed 10b yet.

## Dependencies

- Consuming Phase 10b: an affine `Chart` variant in the registry with the
  property tests the note prescribes (identity, composition, inverse and
  difference laws within a few ulps), °C/°F literals and the Composer's unit
  pop-up over them; arithmetic on absolute temperatures stays a separate
  decision (the point/difference validation of Phase 10 §10).

## Resolution

Open.
