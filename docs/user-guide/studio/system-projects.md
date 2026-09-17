# System projects

A **system project** (*New System…* on the project manager) is a project
that can hold behavior groups, components and instances. Everything a
plain project can do, a system project can do; this page covers what it
adds on the Design page. The ideas are in [Behavior groups](../concepts/behavior-groups.md),
[Components](../concepts/components.md) and
[Behavior systems](../concepts/behavior-systems.md).

A plain project cannot be turned into a system project later; choose at
creation.

## Two canvases, one design

The canvas shows **one design at a time**:

| Canvas | Shows | Bar above the canvas |
|---|---|---|
| **System** | the top level: the system's own concepts, relationships, domains, outputs; component **instance nodes**; **binding links**; behavior **regions** and **boxes** | *System · N instances · M components* |
| **Component source** | one component's own design, in its own names; its port-backed relationships carry *requires* / *provides* / *parameter*; its own behavior regions | *Editing AdaptiveLamp · used by 3 instances — its promise is what instances see; edits here reach every instance.* with **‹ System** |

Open a component's source by double-clicking an instance, or with **Edit
Source** on an instance or component. Return with *‹ System* or the
inspector's **Back to System**. Simulate and Deploy always work on the
whole system, whichever canvas is open.

## Instance nodes

```
        ┌──────────────────────────────┐
        │ lampA                        │   the instance's name
   ○────┤ tiltValue                    │   requires / parameter: left; hollow while open
        │                  brightness ├────●   provides: right
        │ AdaptiveLamp        ↻ main   │   the component; its timing parameters
        └──────────────────────────────┘
```

Socket colours are the concepts the ports carry *in the system*: a
shared concept's own hue, or the instance's private hue for a private
concept — so two instances of one component have two colours for their
private *Brightness*. A red mark on the component's name means the source
no longer keeps its promise.

<!-- figure F12 -->

## Gestures added on the system canvas

| Do | Result |
|---|---|
| right-click empty canvas → **Add Instance ▸** *component* | an instance at the pointer, its name open for editing |
| drag from a provided socket (or a top-level value's output socket) to a required socket or parameter | a binding, when the destination is free and the domains agree |
| … to an open top-level relationship's socket | the relationship is *bound to* the port |
| … onto a destination that already has a source | a sheet: *Replace the connection?* — **Disconnect and Connect** or **Cancel** |
| … across timing domains | a sheet: *Carry across timing domains* — a **Starts at** value; never an implicit transport |
| drag a bound input away, release on empty canvas | disconnect |
| click a binding link, then ⌫ | disconnect |
| ⇧-drag a box around relationships → **Group as Behavior** (inspector or right-click) | a behavior named *Behavior*, then renamed inline |
| drag a relationship over an expanded region | it joins; from another region, it moves |
| drag a relationship out of a region onto empty canvas | it leaves the behavior |
| drag a region's title band | moves every member |
| double-click a region's title | rename |
| right-click a behavior → **Collapse** / **Expand** / **Package as Reusable Component…** / **Ungroup** | |
| drag a collapsed box | the hidden members travel with it |
| drag from / onto a collapsed box's socket | resolves to the concrete member (a small chooser when several qualify) — never a link to the behavior |
| zoom below half size | every behavior reads as its box; the authored collapse state is untouched |

Pan and zoom are remembered per canvas.

## Inspectors added

**Behavior** — *Name*, *Meaning*; **Relationships** with *+ Add
relationship*; **Boundary** (*Inputs*, *Outputs*, *Open*, *Physical
outputs*, *Internal* — read off the dependency graph; *Computed once the
analysis arrives.*); **Canvas** (Collapse / Expand); **Package as Reusable
Component…**; **Ungroup**; *Delete group and relationships*. In a
component's source the inspector says packaging a behavior inside a
component comes with nested components.

**Component** — the source's name and meaning; **Ports** with each port's
kind, concept and timing contract (*Updates in main (a parameter of the
component)*, *Updates in its own …*, *Any timing domain*), and **Retire
port**; **Declare a port** (expose a relationship of the source as
*requires* / *provides* / *parameter*); **Timing parameters**; **Shared
concepts**; **Findings** (whether the source keeps its promise); *Used by
N instances*; **Edit Source**; **Duplicate as Version**; **Place
Instance**; *Delete …* (*Delete its instances first.*).

**Instance** — *Name*; **Implemented by** the component with *Go to
Source* and **Replace with** (another component with a compatible
promise); **Timing** — one system domain per timing parameter; **Ports**
— each port's state: *Bound to …* with *Show Binding*, *Open: nothing
supplies it yet. Draw a link from a provided port or a top-level …*, or a
parameter's **Value** (a closed constant); **Findings**; *Delete …*
(*Disconnect its ports first; the component stays.*).

**Binding** — *From* / to, and the transport: *Direct: the destination
reads the value as it is.* or *Carried across timing domains: the
destination sees the last value …* with the *Starts at* value; *A binding
converts nothing: both ends carry the same concept, by identity.*;
**Disconnect**.

**Multi-selection** — *N selected* and **Group N relationships as
Behavior** for the free relationships, with a quiet note when they read
each other or share a domain.

## The packaging sheet

*Package as Reusable Component…* on a behavior opens a sheet with what
the compiler computed — **Requires** and **Provides** are the boundary,
which the sheet never lets you shrink — and the four choices only you can
make:

1. the **component's name** and the **instance's name**;
2. for each **open relationship** among the members: *Treat as input* (it
   becomes a required port) or *Keep internal*;
3. for each **physical output** driven by a member: *Stays the system's*
   or *Moves inside*;
4. the **timing parameters** it will take (listed; every domain the
   members use).

Every change re-asks the preview. **Package** is one edit: the members
become the component's source laid out as they were, one instance stands
where the behavior stood, the bindings are made, and the design computes
the same values as before. [Workflow: packaging a behavior](../workflows/package-as-component.md).

## Not built

Entity hover cards and *Fixes* inside a component's source; packaging a
behavior that is inside a component (nested components); a minimap;
converting a plain project into a system project.

## Related

[Behavior groups](../concepts/behavior-groups.md) · [Components](../concepts/components.md) ·
[Behavior systems](../concepts/behavior-systems.md) ·
[Workflow: composing components](../workflows/composing-components.md)
