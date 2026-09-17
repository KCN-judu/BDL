# Incomplete designs

In most tools an unfinished thing is an error. In BDL it is a state. You
can say *the lamp needs a brightness behavior* before you know the formula,
*there will be a temperature* before you know how it is measured, *this
output exists* before anything drives it — and save, simulate what is
ready, and check the rest tomorrow.

This page explains the states a design and its parts can be in, what each
looks like, and which ones actually stop you.

## The states of a relationship

| State | You see | It means | Stops you? |
|---|---|---|---|
| **declared** | dashed node, the word *declared* in the header | the relationship has a signature and no formula yet | only where its value is needed: the simulator will ask for it as an input, and an output it should drive stays undriven |
| **open** | solid node, but a hollow socket; inspector: *Checked once Temperature's value is decided.* | it has a formula, but a concept it reads has no value form yet, so the formula cannot be checked | no — it is waiting, not wrong |
| **invalid** | a red mark at the formula line; the finding under the field | it has a formula and the formula does not check | simulation, and anything that depends on it |
| **valid** | solid node, no marks | it checks, has a value at every tick it is asked for, and its timing is consistent | no |

*Declared* and *open* are the two intentional states. They are drawn in
orange or with dashes — never red. Red is reserved for *wrong now*.

## The states of a design

A whole design is **executable** when every relationship an output depends
on is valid and every required output has exactly one driver. Below that,
the status line at the bottom of the window tells you what is still open,
as counts and short phrases: *2 not yet defined*, *outputs incomplete*,
*not causal*, *reads across domains*.

The Simulate page turns the same facts into sentences with links: *tilt
needs a value before simulation can step.* — *dimByTilt has no
definition.* — *level has no valid definition.* Each *Show* link selects
the object. Nothing here is a failure; it is the list of what remains.

## Why BDL is built this way

Design is discovered in an order that does not match the order of
dependency. You know the lamp will have a light before you know the
formula; you know a relationship reads *Temperature* before you know
whether temperature is a quantity in kelvin or a coarse *cold / warm /
hot*. A tool that refused to hold the design until every piece was
decided would push you back to sketches. BDL holds the design, checks the
parts that can be checked, and reports the rest as *open* — with the
same precision it uses for errors, so that "what is left" is always a
concrete list.

The practical consequences:

* A **declared** relationship without inputs is exactly what the simulator
  treats as an **input** — you supply its value. A sensor is a declared
  value with a domain. So "unfinished" and "comes from outside" are the
  same state, on purpose.
* An **open** relationship does not turn red when you change the concept it
  waits on; it is re-checked and becomes valid or invalid.
* An **undriven** required output is *incomplete*, an **undriven optional**
  output is fine, and a **contested** output (two drivers) is an error —
  because the first two are missing information and the third is a
  contradiction.

## Finishing a design

The Simulate page's list, the status line, and the **Fixes** section of an
inspector all point at the same work. Fixes are actions the tool can offer
for a finding — *Choose what Temperature is represented by*, *Connect a
driver to Light Output* — each applied as an ordinary edit you can undo.

## Going deeper

*For language implementers.* The kernel treats a declaration without a
realization as a legal, unresolved declaration; the ladder a mapping climbs
is *Declared → Open → Invalid → TypeValid → TemporallyValid →
ClockConsistent*, and output completeness and hardware feasibility are
properties of the design and of a (design, target) pair, not rungs.
`docs/architecture/compiler-pipeline.md` ("Driver and result") and ADR-0015.

## Related

[Relationships](relationships.md) · [Status meanings](../reference/status-meanings.md) ·
[Troubleshooting: incomplete design](../troubleshooting/incomplete-design.md)
