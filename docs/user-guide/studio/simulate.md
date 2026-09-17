# Simulate

The Simulate page (⌘2) answers *what does the design do over time*, one
tick at a time. Every value comes from the compiler service's reference
evaluator — the executable definition of what a BDL design means. Studio
holds the inputs you typed, the periods you chose and the samples that
came back; it computes nothing itself.

```
┌───────────────┬────────────────────────────────────────┬───────────────────┐
│ Inputs        │ [Step] [Step ×10] [Reset]   tick 3     │ Probe             │
│ ○ tilt  Tilt  │ ● held needs a value before simulation │ brightness   ○    │
│   [0.7854] rad│   can step.  Show                      │   Now   Brightness(0.5)
│ ◇ held  Held  │  tick active       tilt  brightness …  │   Over the run    │
│   [off]       │    0  interaction  0.785 Brightness(…) │    0  Brightness(0.5)
│ Timing domains│    1  interaction  0.785 Brightness(…) │ ▸ Explain         │
│ ↻ interaction │                                        │                   │
│   every [1]   │                                        │                   │
└───────────────┴────────────────────────────────────────┴───────────────────┘
```

## Inputs (left)

One control for every value that comes from outside — every relationship
that reads nothing and has no formula. The control follows the concept's
value form, never its name: a number with the unit beside it for a
quantity (in the base unit: radians, metres, seconds, kelvin…), a switch
for on / off, a whole number for a count. The row carries the concept's
glyph and colour, and clicking it selects the object — on this page and on
Design.

Under the inputs, **Timing domains**: a period per domain — *every N*
ticks — the schedule the evaluator activates by. A period, never a rate.
Changing a period starts the run over.

In a system project, an instance's **open required ports** are inputs too,
and an instance's values are listed as `lampA.brightness`.

## Readiness (above the trace)

Before anything runs, the page lists what would stop a step, as
sentences about named objects, each with a *Show* link that selects it:

* *tilt needs a value before simulation can step.* — an input without a
  value
* *Tilt needs a value form (Quantity, On / off or Count) before tilt can be
  given a value.* — a concept still on *decide later*
* *dimByTilt has no definition.* / *level has no valid definition.*
* *These relationships depend on each other in the same instant: a, b.*

While any is listed **Step** is disabled and does nothing. While the
analysis for the current design has not arrived the list says *Checking
the design…* and nothing is wrong. Opening the page never starts a run.

<!-- figure F9 -->

## Step, Step ×10, Reset

**Step** evaluates the next tick with the inputs on screen; **Step ×10**
ten of them. Every step is a **replay**: the evaluator restarts at tick 0
with every tick's inputs so far and the schedule, then runs to the new
tick — so the same design, inputs and periods always give the same trace.
**Reset** returns to tick 0; inputs and periods stay.

A tick that **fails** — a division by zero, a value that is not a number
— stops there, and the failure is written on the controls' line about the
object (*bad divided by zero.*), never as a banner.

## The trace (centre)

Rows are ticks. Columns are the design's **values** — relationships
without inputs — and its **driven outputs**; a rule (a relationship with
inputs) has no column, because it is a function, not a value. The
*active* column names the domains that ticked. A cell is the evaluator's
own rendering, always with the concept: `Brightness(0.5)`, `Held(true)`.

An empty cell means the value's domain did not activate at that tick.
An input's cell shows the value you fed, only at ticks where its domain
activated. Column order is by identity, not by time; click a column
header to select that relationship.

Memory and transports show up as values: `acc = delay(0, acc + x)` reads
`0` at tick 0 and last tick's sum after; `y = sync(fast, -1, x)` in a
slower domain reads the source's last activation strictly before its own,
so a source value produced at the same instant is not yet visible.

## The probe (right)

The selected object's value **now** and **over the run**, with its glyph;
for an output, its driver. **Explain** under it holds the identity number,
the run's revision, the rendered value and, for a failure, the code and
technical text.

## What a new revision does

Editing the design drops the samples and any answer still in flight,
keeps the fed values for inputs that still exist, and re-checks
readiness. A trace always belongs to one revision of the design.

## Not on this page

Values on the canvas (the trace and probe are the only views), plots (the
trace is a table), and telemetry from a real device (that is Monitor,
which does not exist yet).

## Related

[Your first simulation](../getting-started/first-simulation.md) ·
[Timing](../concepts/timing.md) ·
[Troubleshooting: incomplete design](../troubleshooting/incomplete-design.md)
