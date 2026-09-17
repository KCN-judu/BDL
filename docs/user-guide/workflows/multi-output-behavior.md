# A second output

**Goal.** Give the lamp a status indicator that shows the same brightness
as the light — and see what BDL does when two values compete for one
output.

## Steps

1. **A second output.** *Outputs* **+**: `Indicator`, accepts
   **Brightness**, updates in *interaction*, *Required* off (a missing
   indicator should not make the design incomplete).
2. **A value for it.** *Mappings* **+**: `indicator`, reads nothing,
   produces Brightness; **Updates in** *interaction*; formula
   `brightness`. *Add definition.*

   A value's formula may simply name another value. `indicator` is
   `brightness`, read at the same tick.
3. **Connect.** Drag from `indicator`'s output socket onto the *Indicator*
   sink.
4. **Now make a conflict.** Drag from `brightness`'s output socket onto
   the *Indicator* sink too.

   The sink shows the word **contested**. Select it: the *Driver* section
   lists both claimants, and under *Fixes* the tool offers *Detach
   brightness from Indicator* / *Detach indicator from Indicator* and
   *Create upstream combination mapping*. The status line says *outputs
   incomplete*.
5. **Resolve it.** Click *Detach brightness from Indicator*. The
   indicator is driven by `indicator` alone.

## What BDL means by this

* One physical output, **one driver**. Two values driving one light is a
  contradiction in the design; BDL names both claimants and offers to
  detach one or to combine the values upstream. It never picks a winner
  — there is no priority, no "last one wins".
* A **required** output that is undriven makes the design *incomplete*;
  an **optional** one does not.
* Two outputs may show the same value (`brightness` drives the light,
  `indicator = brightness` drives the indicator) — the sharing happens
  in the formula, not at the output.
* A contested output does not make its drivers *invalid*: `brightness`
  still checks. The finding is about the connection and sits under
  *Drives* and *Driver*.

## If it does not work

* *Only a relationship without inputs can drive an output* — you connected
  a rule; connect the value that applies it.
* The connection is recorded but reported under *Driver* — the value's
  domain differs from the output's, or it produces another concept.
  [Connections](../troubleshooting/connection-errors.md).

## Related

[Physical outputs](../concepts/physical-outputs.md) ·
[Grouping a behavior](grouping-behavior.md)
