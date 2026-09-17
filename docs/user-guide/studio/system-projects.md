# System projects

A **system project** (_New System…_ on the project manager) is a project that
can hold behavior groups, components and instances. Everything a plain project
can do, a system project can do; this page covers what it adds on the Design
page. The ideas are in [Behavior groups](../concepts/behavior-groups.md),
[Components](../concepts/components.md) and
[Behavior systems](../concepts/behavior-systems.md).

A plain project cannot be turned into a system project later; choose at
creation.

## Two canvases, one design

The canvas shows **one design at a time**:

| Canvas               | Shows                                                                                                                                                                                                           | Bar above the canvas                                                                                                                 |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| **System**           | the top level: the system's own concepts, relationships, domains, outputs; component **instance nodes**; **binding links**; behavior **regions** and **boxes**                                                  | _System · N instances · M components_                                                                                                |
| **Component source** | one component's own design, in its own names; its port-backed relationships carry _provides_ / _parameter_ in their header (a _requires_ port has no formula, so it reads _declared_); its own behavior regions | _Editing AdaptiveLamp · used by 3 instances — its promise is what instances see; edits here reach every instance._ with **‹ System** |

Open a component's source by double-clicking an instance, or with **Edit
Source** on an instance or component. Return with _‹ System_ or the inspector's
**Back to System**. Simulate and Deploy always work on the whole system,
whichever canvas is open.

## Instance nodes

![Two instance nodes, adaptiveLamp (outlined in the accent colour, selected) and second, each with a required socket tiltValue on the left, a provided socket brightness on the right and the component's name AdaptiveLamp with its timing parameter main in the body; links from the top-level tiltValue into both required sockets and from each provided socket to the top-level relationships brightness and mirror, whose bodies read = adaptiveLamp.brightness and = second.brightness.](../assets/studio/instance-nodes.png)

_Two instances of AdaptiveLamp, drawn from the component's promise, fed by one
tiltValue and bound to brightness and mirror._

An instance node has the instance's name in a tinted header; one row per port —
_requires_ ports and _parameters_ on the left, with a hollow socket while
nothing is bound to them, _provides_ ports on the right; and in the body the
component's name with its timing parameters, each shown with the system domain
the instance gave it. A top-level relationship bound to a provided port shows
the binding in its formula line: _= adaptiveLamp.brightness_.

Socket colours are the concepts the ports carry _in the system_: a shared
concept's own hue, or the instance's private hue for a private concept — so two
instances of one component have two colours for their private _Brightness_. A
red mark on the component's name means the source no longer keeps its promise.

![Above the canvas a bar with a System back button and the words Editing AdaptiveLamp, used by 2 instances; on the canvas the component's own design: tiltValue drawn dashed with the word declared (a required port has no formula of its own), the rule dimByTilt, and brightness with the word provides in its header.](../assets/studio/component-source.png)

_A component's source: its own canvas, with the ports marked, and the bar that
leads back to the system._

## Gestures added on the system canvas

| Do                                                                                                    | Result                                                                                                |
| ----------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| right-click empty canvas → **Add Instance ▸** _component_                                             | an instance at the pointer, its name open for editing                                                 |
| drag from a provided socket (or a top-level value's output socket) to a required socket or parameter  | a binding, when the destination is free and the domains agree                                         |
| … to an open top-level relationship's socket                                                          | the relationship is _bound to_ the port                                                               |
| … onto a destination that already has a source                                                        | a sheet: _Replace the connection?_ — **Disconnect and Connect** or **Cancel**                         |
| … across timing domains                                                                               | a sheet: _Carry across timing domains_ — a **Starts at** value; never an implicit transport           |
| drag a bound input away, release on empty canvas                                                      | disconnect                                                                                            |
| click a binding link, then ⌫                                                                          | disconnect                                                                                            |
| ⇧-drag a box around relationships → **Group as Behavior** (inspector or right-click)                  | a behavior named _Behavior_, then renamed inline                                                      |
| drag a relationship over an expanded region                                                           | it joins; from another region, it moves                                                               |
| drag a relationship out of a region onto empty canvas                                                 | it leaves the behavior                                                                                |
| drag a region's title band                                                                            | moves every member                                                                                    |
| double-click a region's title                                                                         | rename                                                                                                |
| right-click a behavior → **Collapse** / **Expand** / **Package as Reusable Component…** / **Ungroup** |                                                                                                       |
| drag a collapsed box                                                                                  | the hidden members travel with it                                                                     |
| drag from / onto a collapsed box's socket                                                             | resolves to the concrete member (a small chooser when several qualify) — never a link to the behavior |
| zoom below half size                                                                                  | every behavior reads as its box; the authored collapse state is untouched                             |

Pan and zoom are remembered per canvas.

## Inspectors added

**Behavior** — _Name_, _Meaning_; **Relationships** with _+ Add relationship_;
**Boundary** (_Inputs_, _Outputs_, _Open_, _Physical outputs_, _Internal_ — read
off the dependency graph; _Computed once the analysis arrives._); **Canvas**
(Collapse / Expand); **Package as Reusable Component…**; **Ungroup**; _Delete
group and relationships_. In a component's source the inspector says packaging a
behavior inside a component comes with nested components.

**Component** — the source's name and meaning; **Ports** with each port's kind,
concept and timing contract (_Updates in main (a parameter of the component)_,
_Updates in its own …_, _Any timing domain_), and **Retire port**; **Declare a
port** (expose a relationship of the source as _requires_ / _provides_ /
_parameter_); **Timing parameters**; **Shared concepts**; **Findings** (whether
the source keeps its promise); _Used by N instances_; **Edit Source**;
**Duplicate as Version**; **Place Instance**; _Delete …_ (_Delete its instances
first._).

**Instance** — _Name_; **Implemented by** the component with _Go to Source_ and
**Replace with** (another component with a compatible promise); **Timing** — one
system domain per timing parameter; **Ports** — each port's state: _Bound to …_
with _Show Binding_, _Open: nothing supplies it yet. Draw a link from a provided
port or a top-level …_, or a parameter's **Value** (a closed constant);
**Findings**; _Delete …_ (_Disconnect its ports first; the component stays._).

**Binding** — _From_ / to, and the transport: _Direct: the destination reads the
value as it is._ or _Carried across timing domains: the destination sees the
last value …_ with the _Starts at_ value; _A binding converts nothing: both ends
carry the same concept, by identity._; **Disconnect**.

**Multi-selection** — _N selected_ and **Group N relationships as Behavior** for
the free relationships, with a quiet note when they read each other or share a
domain.

## The packaging sheet

_Package as Reusable Component…_ on a behavior opens a sheet with what the
compiler computed — **Requires** and **Provides** are the boundary, which the
sheet never lets you shrink — and the four choices only you can make:

1. the **component's name** and the **instance's name**;
2. for each **open relationship** among the members: _Treat as input_ (it
   becomes a required port) or _Keep internal_;
3. for each **physical output** driven by a member: _Stays the system's_ or
   _Moves inside_;
4. the **timing parameters** it will take (listed; every domain the members
   use).

Every change re-asks the preview. **Package** is one edit: the members become
the component's source laid out as they were, one instance stands where the
behavior stood, the bindings are made, and the design computes the same values
as before.
[Workflow: packaging a behavior](../workflows/package-as-component.md).

## Not built

Entity hover cards and _Fixes_ inside a component's source; packaging a behavior
that is inside a component (nested components); a minimap; converting a plain
project into a system project.

## Related

[Behavior groups](../concepts/behavior-groups.md) ·
[Components](../concepts/components.md) ·
[Behavior systems](../concepts/behavior-systems.md) ·
[Workflow: composing components](../workflows/composing-components.md)
