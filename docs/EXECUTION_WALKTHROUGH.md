# One complete execution, end to end

For a compiler engineer reading the repo: what happens between a designer's
formula and a `Brightness` sample. Every step is a pure function you can
call from a test; the numbers below are from
`crates/bdl-reactive/src/simulate.rs::the_lamp_executes_over_four_ticks`.

## 0. The design

```
concept Tilt        representation = quantity, angle          (sem#0 : q[rad])
concept Brightness  representation = quantity, dimensionless  (sem#1 : q[1])
mapping dimByTilt : Tilt -> Brightness   formula  Tilt / 90 deg
```
Plus, at Core level (the surface cannot say this yet, DI-17):
```
tilt       : Tilt         unresolved      → an input
brightness : Brightness   := dimByTilt tilt
```

## 1. Parse (`bdl-syntax`)

`Tilt / 90 deg` → `Binary { Div, Name("Tilt") @0..4, Number { 90, unit deg } @7..13 } @0..13`.

## 2. Elaborate (`bdl-elab`)

Signature → `Interface { expected_type: sem#0 → sem#1, commitments: [] }`.
Formula, under `Grant::of(sem#0 → sem#1) = {sem#1}`:

```
λ(sem#0). (mk sem#1 (div[rad,rad] (rep #0) 1.5707963[rad]))
```
`Tilt` became `rep (var 0)` (its representation, `q[rad]`); `90 deg` became a
literal in SI base units; the whole body was wrapped in `mk sem#1`, which the
signature grants. A path→span map remembers where each Core sub-term came
from.

## 3. Type-check (`bdl-check`)

`infer` derives `sem#0 → sem#1` — `div[rad,rad] : q[rad] → q[rad] → q[1]`,
`mk sem#1 : q[1] → sem#1` — and it equals the interface: **type-valid**.
Had the formula been `Tilt + 1 s`, `add[rad]` applied to `q[s]` fails the
ordinary application rule and the elaborator's diagnostic reads *"adds
values with different physical dimensions: an angle and a time"* at `0..10`.

## 4. Dependencies and causality (`bdl-reactive::{graph,causality}`)

```
brightness → {dimByTilt, tilt}   (all = instantaneous: no delay anywhere)
```
Tarjan finds no non-trivial SCC; Kahn gives ranks `tilt 0, dimByTilt 0,
brightness 1` and the order `[tilt, dimByTilt, brightness]`: **causal**.

## 5. Clock domains (`bdl-reactive::clocks`)

`tilt` and `brightness` are in `interaction`; `dimByTilt` is agnostic.
`brightness`'s reads are same-domain or agnostic: **clock-consistent**.
(Had `tilt` been in `ambient`, the diagnostic would read *"brightness
updates in a different timing domain from tilt. Choose how this
relationship should observe the source value."*)

## 6. Simulate (`bdl-reactive::{eval,simulate}`)

Schedule: `interaction` every tick. Input trace for `tilt`:
`Tilt(0°), Tilt(30°), Tilt(60°), Tilt(90°)`.

Tick 0, read phase: `brightness` → `app (declRef dimByTilt) (declRef tilt)`
→ closure applied to `Tilt(0 rad)` → `rep` gives `0 [rad]`, `div` gives
`0`, `mk` gives `Brightness(0)`. No temporal sites, so the write phase
commits nothing. Ticks 1–3 likewise.

```
tick  tilt            dimByTilt    brightness
0     Tilt(0 rad)     <function>   Brightness(0)
1     Tilt(0.524 rad) <function>   Brightness(0.3333…)
2     Tilt(1.047 rad) <function>   Brightness(0.6667…)
3     Tilt(1.571 rad) <function>   Brightness(1)
```

## 7. Memory (`prevTilt := delay Tilt(0) tilt`)

Cell `(prevTilt, [])`. Tick 0: read → never written → init `Tilt(0)`; write
→ `tilt` now = `Tilt(10°)` into the next state. Tick 1: read → `Tilt(10°)`;
write `Tilt(20°)`. The trace shows `0°, 10°, 20°` for inputs `10°, 20°,
30°` — the previous activation, never the current one.

## 8. Across domains

`y := sync interaction 0 x` in `ambient`, both domains active every tick:
`y` shows `0, x₀, x₁, …` — the source's *previous* activation — and a host
that evaluates `ambient` before `interaction` gets the same trace
(`step_in_order` test). With `ambient` active only at tick 2, `y` at tick 2
is `x₁`: the last source activation strictly before.

## 9. Over the wire

`RunAnalysis` returns the ladder status `ClockConsistent`, `causal =
true`, `evaluation_order = [dimByTilt]`; `StartSimulation` + `StepSimulation
{ticks: 3}` returns three `TickSample`s with `rendered` values; any commit
drops the run (it belongs to its revision).
