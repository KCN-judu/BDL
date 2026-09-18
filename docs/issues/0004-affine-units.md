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
  id and a `Chart` onto the canonical magnitude, `Linear { factor }` or
  `Affine { scale, offset }`; `convert(x, from, to)` is the one conversion
  operation for either shape, and the Phase-10b laws are property-tested on
  `f64` including the Celsius/Fahrenheit charts (`AFFINE_CHARTS`), which are
  infrastructure only — not registered, so no formula, picker or text completion
  offers them (ADR-0028 amendment). `docs/spec/textual-syntax.md` §6. DI-7.
- Formal: BDL_FV Phase 10 (`Surface/Affine.lean`) shows °C/°F literals and
  coordinates elaborate exactly with no kernel change (`celsius_not_linear`,
  `affLitE_typed`, `affine_roundtrip_ev`); Phase 10b (`Surface/Charts.lean`,
  `bd87b66`) proves conversion between charts is a groupoid of affine maps
  (`convert_compose`, `convert_inverse`, `display_switch_preserves_quantity`)
  and that a point/difference sort is optional validation information,
  orthogonal to conversion (`sort_orthogonal_to_conversion`). Production
  consumed 10b's conversion model (P10b); what is not consumed is exposure.

## Dependencies

- Exposing °C/°F: registering the affine charts so a literal `20 °C` elaborates
  (`reconstruct` at the literal, exact per Phase 10 `affLitE`) and the
  Composer's pop-up offers them for a temperature — together with the decision
  on arithmetic over absolute temperatures (the point/difference validation of
  Phase 10 §10), since a registered unit is usable in any arithmetic.

## Resolution

Open.
