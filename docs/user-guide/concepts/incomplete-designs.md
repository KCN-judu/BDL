# Incomplete designs

In most tools an unfinished thing is an error. In BDL it is a state. You can say
_the lamp needs a brightness behavior_ before you know the formula, _there will
be a temperature_ before you know how it is measured, _this output exists_
before anything drives it — and save, simulate what is ready, and check the rest
tomorrow.

This page explains the states a design and its parts can be in, what each looks
like, and which ones actually stop you.

## The states of a relationship

| State           | You see                                                                                    | It means                                                                                                                                      | Stops you?                                                                               |
| --------------- | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| **declared**    | dashed node, the word _declared_ in the header                                             | the relationship reads something and has no formula yet                                                                                       | only where its value is needed: an output it should drive stays undriven                 |
| **Source**      | a bar at the node's left edge, an entry arrow, the word _Source_ in the header             | the relationship reads nothing and has no formula: the environment provides its value                                                         | no — nothing is missing; the simulator asks you for the value                            |
| **open**        | solid node, but a hollow socket; inspector: _Checked once Temperature's value is decided._ | it has a formula, but a concept it reads has no value form yet, so the formula cannot be checked                                              | no — it is waiting, not wrong                                                            |
| **invalid**     | a red mark at the formula line; the finding under the field                                | it has a formula and the formula does not check                                                                                               | simulation, and anything that depends on it                                              |
| **valid**       | solid node, no marks                                                                       | it checks, has a value at every tick it is asked for, and its timing is consistent                                                            | no                                                                                       |
| **not applied** | a rule with a hollow output socket and, once defined, the word _not applied_ in the header | no value calls the rule, so nothing in the design has its result: the simulator shows no column for it and no output can be driven through it | no — the note on the Simulate page and in the inspector offers the value that applies it |

_Declared_, _open_ and _not applied_ are the intentional states. They are drawn
in orange, with dashes or hollow — never red. Red is reserved for _wrong now_.

## The states of a design

A whole design is **executable** when every relationship an output depends on is
valid and every required output has exactly one driver. Below that, the status
line at the bottom of the window tells you what is still open, as counts and
short phrases: _2 not yet defined_, _outputs incomplete_, _not causal_, _reads
across domains_.

The Simulate page turns the same facts into sentences with links: _tilt needs a
value before simulation can step._ — _dimByTilt has no definition._ — _level has
no valid definition._ Each _Show_ link selects the object. Nothing here is a
failure; it is the list of what remains. Below those, with a hollow dot, the
notes that stop nothing but explain the trace: _dimByTilt is a rule nothing
applies yet._, with the fix that writes the value applying it.

## Why BDL is built this way

Design is discovered in an order that does not match the order of dependency.
You know the lamp will have a light before you know the formula; you know a
relationship reads _Temperature_ before you know whether temperature is a
quantity in kelvin or a coarse _cold / warm / hot_. A tool that refused to hold
the design until every piece was decided would push you back to sketches. BDL
holds the design, checks the parts that can be checked, and reports the rest as
_open_ — with the same precision it uses for errors, so that "what is left" is
always a concrete list.

The practical consequences:

- A relationship without inputs and without a formula is a **Source**, not a
  declared one: it is exactly what the simulator asks you for. A sensor reading
  is a Source with a domain. "Comes from the environment" is a complete state,
  on purpose; give the Source a formula and it becomes a computed value instead.
- An **open** relationship does not turn red when you change the concept it
  waits on; it is re-checked and becomes valid or invalid.
- An **undriven** required output is _incomplete_, an **undriven optional**
  output is fine, and a **contested** output (two drivers) is an error — because
  the first two are missing information and the third is a contradiction.

## Finishing a design

The Simulate page's list, the status line, and the **Fixes** section of an
inspector all point at the same work. Fixes are actions the tool can offer for a
finding — _Choose what Temperature is represented by_, _Connect a driver to
light_, _Add a value that applies dimByTilt_ — each applied as an ordinary edit
you can undo. A fix is ready when the tool can do it alone, a pop-up when you
must choose, and a sentence saying why when it cannot be done yet; it never
guesses.

## Going deeper

_For language implementers._ The kernel treats a declaration without a
realization as a legal, unresolved declaration; the ladder a mapping climbs is
_Declared → Open → Invalid → TypeValid → TemporallyValid → ClockConsistent_, and
output completeness and hardware feasibility are properties of the design and of
a (design, target) pair, not rungs. `docs/architecture/compiler-pipeline.md`
("Driver and result") and ADR-0015.

## Related

[Relationships](relationships.md) ·
[Status meanings](../reference/status-meanings.md) ·
[Troubleshooting: incomplete design](../troubleshooting/incomplete-design.md)
