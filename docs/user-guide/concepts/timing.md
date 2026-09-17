# Timing

A product's values do not all move together. A touch sensor reports many
times a second; a temperature sensor a few times a minute; a button when
it is pressed. BDL makes those rhythms explicit, so that "when does this
update" is a decision you can see, simulate and be warned about — not
something that falls out of the firmware's loop order.

This page starts from the questions a designer asks and arrives at the
three words the tool uses: **timing domain**, **delay**, **sync**.

## How often does this sensor update?

Every value that updates on its own — a sensor reading, a computed value
that should refresh with it — belongs to a named **timing domain**. In the
lamp, `tilt` and `brightness` both *update in* **interaction**.

A domain is a **name**, not a rate. It says *these values tick together*;
it does not say how often. How often is a property of the simulation
(the period per domain on the Simulate page) or of the device, not of the
design. Two domains with the same period are still two domains.

Create domains in the sidebar (**+** by *Timing domains*), assign them
in a relationship's or output's inspector under **Timing › Updates in**.
The domain's name is shown quietly at the right edge of the node.

## When does this relationship run?

A **rule** — a relationship with inputs, like `dimByTilt` — has no rhythm
of its own. It is applied inside other formulas and runs whenever the
value applying it updates. Its inspector says *Any timing domain*. Leave it
there unless it reads a domain-bound value directly.

A **value** — a relationship that reads nothing — usually gets a domain:
the sensor's, or the rhythm at which the product should refresh it. A value
left on *Any timing domain* is evaluated whenever any domain ticks (every
tick, if the design has no domains at all), which is fine for a small
design and imprecise for a real one. An output only accepts a driver that
updates in the output's own domain, so a value that drives an output needs
that domain.

## What happens inside one tick

Within one tick of a domain, every value in the domain is recomputed from
the current inputs. There is no order to choose: the tool works out which
value needs which, and evaluates in that order. Two values that need each
other *in the same instant* have no answer — that is an **instantaneous
cycle**, reported as *These relationships depend on each other in the same
instant: a, b.* The way out is memory.

## Remembering last tick's value: `delay`

```
acc = delay(0, acc + x)
```

`delay(initial, expression)` is the value the expression had **last
tick** — and `initial` before there was a last tick. This is how a design
holds state: a running total, a previous reading to compare against, a
latch. Because the previous value is explicit in the formula, a cycle
through a `delay` is not instantaneous and is allowed.

`delay` may be written in a value's formula (a relationship without
inputs), at the top level of the formula or inside a `let`, an `if`
branch or a `match` scrutinee — not inside a rule that has inputs, and
not inside a `match` arm or a block's result. The tool says so if you
try.

## Reading a value from another domain: `sync`

If `slowDisplay` (domain *display*) reads `brightness` (domain
*interaction*) directly, the tool reports *reads across domains*. The two
rhythms are unrelated, so "the current brightness" has no meaning in the
display's tick.

```
shown = sync(interaction, 0, brightness)
```

`sync(domain, initial, expression)` reads the value the expression had at
the source domain's **last activation strictly before** this tick, and
`initial` before the source has ever activated. The initial value is
required because the destination may tick first. The rule "strictly
before" means a source value produced at the same global instant is not
yet visible — you see it on the destination's next tick.

The same idea appears between component instances: connecting a
provided port to a required port across domains asks you for a *Starts
at* value and records the transport on the binding
([Carrying a value across timing domains](../workflows/cross-domain-transport.md)).

## Why BDL asks for an initial value

Both `delay` and `sync` are places where a value is read before it has
been produced — last tick at tick 0, the source domain before it has run.
BDL does not invent a zero: you say what the product should show before
the first real value exists, in the destination's own units.

## What the simulator shows

On the Simulate page each domain has a period; the *active* column of the
trace names the domains that ticked. A value whose domain did not activate
keeps its cell empty for that tick. Memory and transports show up as
values: a `delay` reads its initial value at tick 0, a `sync` reads the
source's previous activation. See [Simulate](../studio/simulate.md).

## Going deeper

*For language implementers.* Domains are the kernel's clock environment
`Κ`; the per-declaration judgment is `Clocked`, and a direct cross-domain
read fails it (`clock.cross_domain_reference`). `delay init e` and
`sync src init e` are Core forms with explicit state cells; the tick is a
two-phase read/write. `docs/RUNTIME_SEMANTICS.md`, `docs/COMPILER_PIPELINE.md`
passes 8–10 and 14, DI-17 in `docs/DESIGN_ISSUES.md`. Frequency is
deliberately not part of a domain's identity (`docs/02-kernel-spec.md`).

## Related

[Relationships](relationships.md) · [Simulate](../studio/simulate.md) ·
[Troubleshooting: timing](../troubleshooting/timing-errors.md) ·
[Formula language](../reference/formula-language.md)
