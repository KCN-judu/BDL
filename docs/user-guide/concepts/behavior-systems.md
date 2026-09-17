# Behavior systems

A **behavior system** is a design made of component **instances** wired
together, alongside the system's own concepts, relationships, timing
domains and physical outputs. It is what a *system project* holds. A plain
project is the simplest case: a system with no components.

```
concept                 ─┐
relationship             ├─  the system's own design (the "top level")
timing domain            │
physical output         ─┘
component  ──▶ instance ──▶ bindings ──▶ (ports of other instances, top-level values)
```

The whole thing — top level plus every instance's source — is one design
to the checker, the simulator and the deployment check. There is no
second language for systems.

## Source and instance

A **component** is a definition. An **instance** is one use of it, with a
name (`lampA`, `lampB`), a place on the canvas, values for its
parameters, and a domain of the system for each of its timing parameters.
Two instances of one component compute the same thing over different
inputs; edit the component's source and both change.

On the canvas an instance is a node drawn from the component's promise:
one row per port — *requires* and *parameter* on the left, *provides* on
the right — and the component's name in the body. The node never shows
the inside of the source; double-click it to open the source.

## Bindings

A **binding** connects a provided port (or a top-level value) to a
required port (or an open top-level relationship). It is drawn as a link
between the instance nodes. A binding converts nothing: both ends carry
the same concept, by identity, and the destination simply *is* the
source's value.

| Connection | Allowed when |
|---|---|
| provided port → required port | same concept; same domain, or an explicit transport |
| top-level value → required port | same concept; same domain, or transport |
| provided port → open top-level relationship | the relationship reads nothing and has no formula; it becomes *bound to* the port and cannot be given a formula while bound |

**Fan-out** is ordinary: one provided port or one top-level value can feed
any number of required ports. The value is computed once and read by
each; nothing is duplicated.

A required port takes **one** source. Dropping a second link onto a bound
port asks *Replace the connection?* — *Disconnect and Connect* or *Cancel*
— never a silent replacement.

## Open inputs of a system

A required port with no binding is **open**: *Open: nothing supplies it
yet.* An open port is not an error; the instance's value for it is
missing, exactly as a declared top-level relationship's is. On the Simulate
page an open port appears as an input you can type, so a partly wired
system can be simulated. A system is **executable** when every required
port that something depends on is bound and every top-level rule holds.

## Timing across instances

A component's timing parameters are bound per instance to the system's
domains. When a binding crosses domains — a provided port in
*interaction* feeding a required port in *display* — the link asks for a
*Starts at* value and records a **transport**: the destination reads the
source's last activation strictly before its own tick, and the initial
value before then. This is the same rule as `sync` in a formula
([Timing](timing.md)); on the canvas the transport is a gate on the link.

## Physical outputs and instances

Outputs stay the system's. An instance can drive a system output through
a provided port bound to the top-level value that drives it; two instances
driving one output through two values is the same **contested** output
as in a plain design, reported the same way. When you package a group,
the packaging sheet asks whether each driven output *stays the system's*
or *moves inside* the component.

## Simulating and deploying a system

Simulate and Deploy always work on the whole system. An instance's values
appear as `lampA.brightness`; its open ports as inputs. Deployment counts
every device the system's outputs need, whichever instance drives them.

## What is not there yet

Nested systems (a component whose source contains instances), packages
shared between projects, and behavioral checks between component versions
(a substitution is checked for its *wiring*, not for computing the same
values) — see [Versioning a component](../workflows/component-versioning.md).

## Going deeper

*For language implementers.* The authored system is flattened —
instantiate, freshen identities, substitute clocks, realise parameters,
union with the base, realise bindings as `declRef` / `sync` references —
into an ordinary flat design, and every judgment is a judgment of that
flat design (ADR-0021 "behaviour systems flatten into the flat design").
Identities: `ComponentId` (a definition), `ComponentInstanceId` (one use),
`PortId`, `BindingId`, and the flat `SemanticId` / `DeclId` / `ClockId` /
`OutputId` per instance — five different kinds of identity that the tool
never collapses. `docs/BEHAVIOR_SYSTEMS.md` states which formal theorems
each production step follows and which are restricted (the
modular-equals-flat result is proved for a single-domain fragment; the
production claims a differential test, not the theorem).

## Related

[Components](components.md) · [System projects](../studio/system-projects.md) ·
[Workflow: composing components](../workflows/composing-components.md) ·
[Workflow: carrying a value across timing domains](../workflows/cross-domain-transport.md)
