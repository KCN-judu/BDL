# Your first simulation

**Goal.** Tilt the lamp from the [first tutorial](first-behavior.md) and watch
its brightness, one tick at a time.

**Time.** Ten minutes.

## 1. Open the Simulate page

Click **Simulate** in the page bar at the bottom (or press **⌘2**).

The page has three parts:

- **Inputs**, on the left — one control for every value that comes from outside.
  Here that is `tilt`, with a number field and the unit `rad` beside it. Under
  the inputs, **Timing domains** lists `interaction` with a period: _every 1_
  tick.
- the **trace**, in the middle — a table that fills as you step: one row per
  tick, a column for every computed value and every driven output. Above it,
  **Step**, **Step ×10** and **Reset**, and the tick counter.
- the **Probe**, on the right — the value of whatever is selected, now and over
  the run.

Nothing runs until you step. If anything in the design would stop a step, it is
listed above the trace as a sentence about the object — _tilt needs a value
before simulation can step._ — with a _Show_ link that selects it. Step is
disabled while such a line is there.

## 2. Give the tilt a value

In the `tilt` field enter `0.7854` (45°; the field takes the base unit shown
beside it — radians for an angle).

## 3. Step

Click **Step**.

The trace gains its first row: tick 0, the active domain _interaction_, and the
values the design computed — `brightness` as `Brightness(0.5)` (or within a few
digits of it, depending on what you typed), and the _light_ column showing the
same value, because `brightness` drives it. A value is always written with the
concept it belongs to. Click **Step** twice more: three rows, the same values,
because the input has not changed.

Change the field to `1.5708` (90°) and **Step**: about `Brightness(1)`. Set `0`
and step: `Brightness(0)`. Upright is off; flat is full.

**What happened.** Each tick, every value in the _interaction_ domain was
recomputed from the inputs you supplied. The evaluator that produced these
numbers is the one that defines what a BDL design means; the generated code for
a device is held to it.

![The Simulate page: on the left an input control for tilt showing 0.785398 rad and the interaction domain's period of every 1 ticks; in the middle the Step, Step ×10 and Reset buttons, tick 3, and a trace with three rows whose tilt, brightness and light columns read 0.785398, Brightness(0.5) and Brightness(0.5); on the right the probe for brightness with 0.5 now and at each tick over the run.](../assets/studio/simulate-page.png)

_The Simulate page after three steps with the tilt at 45°: inputs on the left,
the trace in the middle, the probe on the right._

## 4. Look at one value

Click the `brightness` column header, or select `brightness` on the Design page
and come back. The **Probe** shows its value now and its values over the run,
with the concept's glyph. Open **Explain** under it for the formal detail — the
identity number, the run's revision, the raw rendered value.

## 5. Reset and try the rhythm

**Reset** returns to tick 0 and keeps your inputs.

Under _Timing domains_, set _interaction_ to _every 2_. **Step ×10**: the
_active_ column now names _interaction_ only on every second tick, and `tilt`'s
cell is shown only on those ticks — on the others the domain did not activate,
so no value was read. A period is a simulation choice; the design itself only
says _which_ domain a value belongs to.

## What a step actually does

Every step is a replay: the simulator restarts at tick 0 with all the inputs you
have entered so far and runs to the new tick. The same design, the same inputs
and the same periods give the same trace, every time. Changing the design on the
Design page drops the trace; your input values stay.

## If something does not work

- **Step is disabled and a sentence names a relationship.** Read it: an input
  without a value, a concept without a value form, a relationship without a
  valid definition, or relationships that depend on each other in the same
  instant. Each has a _Show_ link. See
  [Incomplete design](../troubleshooting/incomplete-design.md).
- **A tick stops with a message on the controls' line** (for example a division
  by zero). The message names the relationship. Change the formula or the inputs
  and step again.
- **The trace is empty after you edited the design.** Expected — the run belongs
  to one version of the design. Step again.

## Next

[Your first deployment check](first-deployment.md) — does this lamp fit an
Arduino Nano?
