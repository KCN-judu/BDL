# A second output

**Goal.** Give the lamp a status indicator that shows the same brightness as the
light — and see what BDL does when two values compete for one output.

## Steps

1. **A second output.** _Outputs_ **+**: `Indicator`, accepts **Brightness**,
   updates in _interaction_, _Required_ off (a missing indicator should not make
   the design incomplete).
2. **A value for it.** _Mappings_ **+**: `indicator`, reads nothing, produces
   Brightness; **Updates in** _interaction_; formula `brightness`. _Add
   definition._

   A value's formula may simply name another value. `indicator` is `brightness`,
   read at the same tick.

3. **Connect.** Drag from `indicator`'s output socket onto the _Indicator_ sink.
4. **Now make a conflict.** Drag from `brightness`'s output socket onto the
   _Indicator_ sink too.

   The sink shows the word **contested**. Select it: the _Driver_ section lists
   both claimants, and under _Fixes_ the tool offers _Detach brightness from
   Indicator_ / _Detach indicator from Indicator_ and _Create upstream
   combination mapping_. The status line says _outputs incomplete_.

5. **Resolve it.** Click _Detach brightness from Indicator_. The indicator is
   driven by `indicator` alone.

## What BDL means by this

- One physical output, **one driver**. Two values driving one light is a
  contradiction in the design; BDL names both claimants and offers to detach one
  or to combine the values upstream. It never picks a winner — there is no
  priority, no "last one wins".
- A **required** output that is undriven makes the design _incomplete_; an
  **optional** one does not.
- Two outputs may show the same value (`brightness` drives the light,
  `indicator = brightness` drives the indicator) — the sharing happens in the
  formula, not at the output.
- A contested output does not make its drivers _invalid_: `brightness` still
  checks. The finding is about the connection and sits under _Drives_ and
  _Driver_.

## If it does not work

- _A rule cannot drive an output — connect the value that applies this rule._ —
  the rule's _Drives_ section offers no output; connect the value it names
  (_Show_ selects it), or write one.
- The connection is recorded but reported under _Driver_ — the value's domain
  differs from the output's, or it produces another concept.
  [Connections](../troubleshooting/connection-errors.md).

## Related

[Physical outputs](../concepts/physical-outputs.md) ·
[Grouping a behavior](grouping-behavior.md)
