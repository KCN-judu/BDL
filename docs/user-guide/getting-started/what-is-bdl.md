# What is BDL?

BDL — the Behavior Design Language — is a way of describing what a product
does in terms of **what its values mean** and **how they relate**, so that a
design can be checked, simulated and placed on hardware before it is
programmed. Behavior Designer is the desktop tool that lets you draw,
type, check and run BDL designs.

This page gives you the mental model in five ideas. Each has its own page
in [Concepts](../concepts/concepts.md); here they are only as big as you
need them to be before the [first tutorial](first-behavior.md).

## 1. A concept is a value with a meaning

A lamp has a *Tilt*, a *Brightness*, maybe an *Ambient light*. Each is a
**concept**: a named thing the product senses, decides or shows. A concept
has a *value form* — a quantity with a unit (an angle, a length, a
temperature, or a plain number), an on/off state, or a count.

Two concepts with the same value form are still different concepts.
*Brightness* and *Opacity* may both be numbers between 0 and 1; BDL will
not let you use one where the other is expected, because they mean
different things. This is deliberate, and it is the first thing that makes
a BDL design more than a spreadsheet.

## 2. A relationship says how one value follows from others

`dimByTilt` reads *Tilt* and produces *Brightness*. That is a
**relationship**: a named rule from some concepts to one concept. Its
formula — `Tilt / 90 deg` — is checked for units and dimensions: dividing
an angle by an angle gives a plain number, which is what a brightness is.
Adding an angle to a time would be refused, with the reason.

A relationship that reads nothing is a **value**: either something that
arrives from outside (a sensor reading, a switch) or something computed
from other values in the design.

## 3. A design can be unfinished on purpose

You can create `dimByTilt` before you know its formula. It is then
*declared*: it exists, it has a signature, and everything that depends on it
can be designed around it. The tool marks it as open work, not as an error.
The same goes for a concept whose value form you have not chosen yet.
[Incomplete designs](../concepts/incomplete-designs.md) explains the
states a design can be in and why none of them stops you working.

## 4. Timing is explicit

Values in a product update at different rhythms: a touch sensor at one
rate, a temperature sensor at another. In BDL every value that updates on
its own rhythm belongs to a named **timing domain**; a relationship that
reads a value from another domain must say so and give the value to use
before the first reading arrives. Memory — using last tick's value — is
written into the formula, not hidden. [Timing](../concepts/timing.md)
starts from the questions a designer asks and works towards the rules.

## 5. Outputs are where behavior leaves the design

A **physical output** — the light, the motor, the display — accepts values
of one concept and is driven by exactly one relationship. A second
relationship trying to drive the same output is a conflict the tool names,
not a race it resolves. Which board the design will run on is a separate
question: **deployment** checks whether the outputs' devices can be placed
on a chosen board's pins, and never changes what the design means.

## Growing from a lamp to a system

Once a design has several relationships that belong together, you can
**group** them into a *behavior* — a way of organising and looking at the
design that changes nothing about what it does. A behavior can be
**packaged** as a reusable *component* with an explicit boundary (what it
needs, what it offers). Components are then placed as *instances* and
wired together into a *behavior system*. The [concept pages](../concepts/behavior-groups.md)
take these one at a time; the [workflows](../workflows/grouping-behavior.md)
walk through them on the same lamp you build in the tutorial.

## Next

[Install and launch](install-and-launch.md), then
[Your first behavior](first-behavior.md).
