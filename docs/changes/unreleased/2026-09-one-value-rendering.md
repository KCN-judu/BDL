# One rendering of a value, the evaluator's; inputs echoed in the trace (ADR-0033)

- Date: 2026-09-20
- Area: runtime, studio, protocol
- Affected: designers, protocol clients, developers
- Related: ADR-0033, ADR-0031, ADR-0001

## What changed

- **A truth value reads `on` / `off`** everywhere a value is shown — the trace,
  the probe, the CLI's trace output — fed or computed: `Held(on)`, never
  `Held(true)`. The value form is called _On / off_; `true` / `false` remain the
  formula's syntax.
- **Numbers read as a designer writes them**: six significant digits, an integer
  as an integer (`Tilt(0.785398 [rad])`, `Brightness(2)`); the JSON trace still
  carries the exact value.
- **A fed input is echoed in the trace**: each tick's samples hold every
  declaration due at the tick, a Source's value included, in the ticks its
  domain activated. The Simulate page's input column, the probe's _now_ and
  _over the run_ all show the evaluator's text; the probe no longer strips the
  concept from the value (`Brightness(0.5)`, as in the table) and its Explain
  panel no longer repeats it.
- **The Simulate page is localised**: the readiness sentences (_tilt needs a
  value before simulation can step._ and the others), the trace headers and the
  switch's word come from the catalog in all three locales.

## Compatibility and migration

- Designers: nothing to do; the same values in fewer spellings.
- Project files: nothing.
- Protocol clients: no version change. `StepSimulation` samples now include the
  unresolved declarations due at the tick (`DeclarationSample` with the fed
  `value` and its `rendered` text); a client that indexed `values[0]` expecting
  the first computed declaration reads by `mapping_id`. `rendered` strings
  changed for booleans and long fractions.
- Developers: `bdl_reactive::value::render_number`; `TickOutcome.values` holds
  inputs; Studio `sampleOf(pb.TickSample, int)`, `SimulationBlockerKind`,
  `blockerSentence`.

## Evidence

`crates/bdl-reactive/src/value.rs` (`render_tests`:
`truth_values_render_in_product_words`,
`numbers_read_as_a_designer_writes_them`),
`apps/studio/test/simulation_test.dart`
(`a truth value reads on / off, fed or computed — one rendering, bdld's`; the
lamp e2e's echoed `Tilt(0.523599 [rad])`;
`an input's value is the evaluator's echo, never Studio's own rendering`; the
readiness tests worded through `blockerSentence`).
