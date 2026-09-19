---
id: ADR-0033
status: accepted
date: 2026-09-20
area: runtime
supersedes: []
superseded-by: []
related: [ADR-0001, ADR-0031, ADR-0032]
fv: []
---

# ADR-0033: A value is rendered once, by the evaluator, in product words — and a fed input is echoed like any other declaration

## Status

Accepted (Simulate page consistency pass, after the Source role).

## Context

A simulation shows values in three places — the trace table, the probe, the
Explain panel — and the same truth value read three ways: a fed boolean Source
rendered as _on_ / _off_ by Studio, a computed boolean rendered as
`SwitchState(true)` by `bdld` (`bdl_reactive::Value::render`), and the value
form itself called _On / off_ in every sheet while BDL text writes `true` /
`false`. The two renderings existed because the evaluator recorded computed
declarations only: an unresolved declaration's value came from the input trace
and was never written into the tick's sample, so Studio kept its own copy of
the fed values and rendered them itself, with its own number format
(`0.785398` beside `Brightness(0.5)` in one row of one table).

Semantic truth is Rust-only (ADR-0001); Studio renders projections. A value's
text is part of that projection. Two renderers of one value form is a defect of
the same kind ADR-0031 forbids for terminology: one thing, two words.

## Decision

1. **The evaluator renders every value, once.** `Value::render` is the
   designer-facing text of a value and every host — Studio, the CLI's trace
   output, a diagnostic — shows it as it is. Studio never turns a `Value` into
   text; a `DeclarationSample.rendered` is the only text a value has on screen.
2. **Product words.** A truth value renders as `on` / `off` — the value form is
   called _On / off_ — never `true` / `false` (syntax, kept for formulas and
   input files) and never a kernel constructor. A number renders with six
   significant digits, an integer as an integer (`0.785398`, `2`, `1.5e-7`); the
   serialised trace keeps the exact `f64`. Concept, unit and collection notation
   is unchanged: `Tilt(0.785398 [rad])`, `Held(on)`, `[1, 2]`, `none`.
3. **A fed input is echoed.** An unresolved declaration due at a tick is
   memoised like a computed one, so `TickSample.values` holds every declaration
   due at the tick — a Source's echoed input in the ticks its domain activated,
   nothing at the others. Whether an input's cell shows a value is the
   evaluator's fact, not a Studio calculation over `active_clock_ids`.
4. **Rendering is notation, not locale.** The words `on`, `off`, `none`, `some`
   and the unit symbols are notation in the sense of ADR-0031 and do not change
   with the locale; the control beside an _On / off_ switch says the same
   `on` / `off`.

## Alternatives

- **Studio renders, with bdld's words copied.** Rejected: two implementations
  of one format drift — they had.
- **Drop Studio's fed-value rendering and show nothing for inputs.** Rejected:
  a trace whose input columns are empty is not a trace.
- **`true` / `false` as the rendering.** Rejected: the designer chose a value
  form named _On / off_ and reads a switch, not a formula; `true` / `false`
  stays where it is syntax.
- **Full-precision numbers.** Rejected for the designer-facing text; the JSON
  trace keeps them.

## Consequences

- `bdl-reactive`: `render` (booleans, `render_number`), `eval::decl_value`
  memoises the input; `TickOutcome.values` and `TickSample.values` documented as
  every declaration due. Differential tests compare computed declarations only
  and are unaffected.
- Studio: `sampleOf(tick, mapping)` is the evaluator's sample; `_fedValue`,
  `_plainText` and the probe's duplicate Explain line are gone; the trace, the
  probe's _now_ and _over the run_ show `rendered`. `SimulationBlocker` carries
  a kind and names; the page words it through the catalog (the one page that
  was English-only is localised).
- Records: `docs/spec/runtime-semantics.md` (Traces), `docs/spec/protocol.md`
  (StepSimulation), the user guide's Simulate page and screenshot; fragment
  `docs/changes/unreleased/2026-09-one-value-rendering.md`.
