# Behavior groups

A **behavior** (a behavior group) is a named set of relationships that
belong together: *Adaptive lamp* = `dimByTilt` + `brightness`. It is a way
of **organising and seeing** a design — a region on the canvas that can be
collapsed to one box — and it changes **nothing about what the design
does**.

That sentence is the whole point. Grouping is not a semantic operation.
The values, the timing, the outputs, the simulation trace and the
deployment placement are identical before and after you group, ungroup,
move a relationship in or out, or collapse a box. A group has no formula,
no domain, no ports.

Groups exist in **system projects** (*New System…* on the project
manager). A plain project has no groups; the rest of this page assumes a
system project.

## What a group is for

* **Reading** a design with many relationships: the canvas shows *Adaptive
  lamp* as one box with what goes in and what comes out.
* **Managing** related relationships together: move them as one, name
  the idea they implement, describe it.
* **Semantic zoom**: zoomed out, every behavior reads as its summary box;
  zoomed in, the members are visible inside the region.
* **The step before packaging**: a group is what you turn into a
  [component](components.md) — but a group is not a component, and most
  groups never become one.

## Making and changing groups

| To | Do |
|---|---|
| group several relationships | ⇧-drag a box around them on the canvas, then **Group as Behavior** in the inspector or the right-click menu; the group is named *Behavior* and its name opens for editing |
| make an empty group | **+** by *Behaviors* in the sidebar (*New Behavior Group*), then drag relationships in |
| add a relationship | drag its node over the region; or right-click it → **Add to Group ▸** |
| move one to another group | drag it from one region into the other |
| take one out | drag it out of the region onto empty canvas; or right-click → **Remove from …** |
| rename | double-click the region's title band; or *Name* in the inspector |
| collapse / expand | right-click the region or the box → **Collapse** / **Expand**; or *Canvas* in the inspector |
| ungroup | right-click → **Ungroup**, or the inspector's **Ungroup**: *Ungroup keeps every relationship where it is.* |
| delete the group *and* its relationships | the inspector's *Delete group and relationships* — a real edit, unlike everything above |

A relationship belongs to at most one group; adding it to a second is
refused with a banner. Only relationships are members: concepts, timing
domains and outputs are not grouped.

## What the box shows

A collapsed behavior is one box. Its left sockets are what the members
**read from outside** the group; its right sockets are what outside
**reads from** the members. The tool computes these from the design's
dependencies — you cannot edit them, because they are a fact about the
members, not a setting.

These sockets are a *picture* of the boundary, not ports. You cannot draw
a link to the box: dragging from or onto an aggregate socket resolves to
the actual member relationship it stands for (a small chooser appears when
several qualify), and the link is made to that relationship. Nothing
depends on "the group".

The inspector's **Boundary** section lists the same facts by name:
*Inputs*, *Outputs*, *Open* (members still without a formula), *Physical
outputs* driven by members, *Internal* (members nobody outside reads).

## What grouping does not touch

* the project's **revision**: a group edit is not a design change. The
  analysis stays valid, a simulation run is kept, nothing is rechecked.
* **undo**: group edits are in the same history as everything else — ⌘Z
  undoes them — and undoing one changes no revision either.
* the **save mark**: a group edit does dirty the project, because it is
  saved with it.

## Going deeper

*For language implementers.* Groups are authoring metadata on the
authored system (`BehaviorGroup { scope, name, members }`), never read by
flattening or any analysis; the formal development proves every kernel
judgment is that of the ungrouped design (ADR-0019, `docs/evidence/behavior-systems-correspondence.md`
Phase 8b). The boundary (crossing-in, crossing-out, open, driven,
private) is a projection of the dependency graph, and an aggregate socket
adds no dependency edge.

## Related

[Components](components.md) · [System projects](../studio/system-projects.md) ·
[Workflow: grouping a behavior](../workflows/grouping-behavior.md) ·
[Workflow: packaging a behavior](../workflows/package-as-component.md)
