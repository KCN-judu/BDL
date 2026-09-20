---
id: ISS-0019
state: open
area: persistence
opened: 2026-09-21
resolved-by: []
related: [ADR-0041, ADR-0040, ADR-0033, ISS-0004]
---

# ISS-0019: A concept has no display unit — the unit a designer thinks in for `LidAngle` is nowhere to keep

## Problem

A concept stores its representation — for a physical quantity, a dimension
(`Representation::Quantity { dim }`) — and no unit. That is right for meaning:
every unit of the dimension is legal in every formula over the concept, a
literal carries its own unit, and the evaluator renders a value once, in the
canonical unit (ADR-0033). But a designer thinks of `LidAngle` in degrees and of
`CycleTime` in milliseconds, and nothing in the project model can say so:

- the concept sheet (ADR-0041) shows the compiler's unit candidates for the
  category as a fact and lets nothing be chosen — the honest answer, and the one
  designers reach for a pop-up on;
- the Simulate page and the inspector show `1.5707963 rad` where the designer
  wrote `90 deg`;
- a formula's slot panel offers the units of the dimension with the registry's
  preferred one first (`rad`), not the one this concept's other literals use.

## Why it matters

The first thing a designer asks of an angle is _in degrees?_ Every place that
renders a value or offers a unit answers the same for every concept of a
dimension, so a design in degrees is read back in radians; the workaround
(reading a literal's unit off the formula) does not survive to the Simulate
page.

## Current evidence

`docs/spec/kernel.md` (a concept's representation is a dimension);
`docs/spec/project-format.md` (no unit field on a concept);
`bdl_model::units::preferred_for` (one preferred unit per dimension, from the
vocabulary); `apps/studio/lib/ui/concept_sheet.dart` (`MeasuredInRow`: the
candidates as a fact); `docs/spec/protocol.md` § The value categories (0.27).
The Lean kernel has no such notion: it would be presentation, not semantics,
which is exactly why no record has claimed it.

## Dependencies

A decision on **where a display unit lives**: on the concept in the project file
(a presentation fact persisted with the design, like a layout position —
`PROJECT_SCHEMA_VERSION` bump, `persist::check_schema` migration with a default
of the dimension's preferred unit), or in Studio's per-project preferences
(never in the file, lost with the preferences), or inferred by the daemon from
the formulas that mention the concept (a heuristic, and the only option that
needs no storage). Then: what the evaluator's one rendering (ADR-0033) does with
it — the trace stays canonical and a conversion for display is the compiler's
(`bdl_model::units::convert`, `UnitExpr::scale`), never a client's arithmetic;
and whether the textual syntax gets a spelling for it
(`concept LidAngle : Angle in deg`), which is a language change.

## Resolution

Open.
