# Components

A **component** is a reusable behavior with an explicit boundary. It has a
**source** — an ordinary design of its own: concepts, relationships,
timing, groups — and a **promise**: the ports through which the rest of a
system uses it. A component is used by placing **instances** of it; every
instance runs the same source, and editing the source changes every
instance.

A [behavior group](behavior-groups.md) is a way of seeing a design. A
component is a *different thing*: a definition with a public boundary
that the rest of the design can only reach through that boundary. Most
groups never become components; a component is what you make when a
behavior will be used more than once, or should be understood without
looking inside.

Components exist in **system projects** (*New System…*).

## The promise: ports

A port is a relationship of the source that the component exposes. There
are three kinds:

| Port | In designer words | In the source |
|---|---|---|
| **Requires** | what this behavior *needs* from outside — a value it reads but does not produce | a relationship without inputs that is left **declared**; the instance's wiring supplies it |
| **Provides** | what this behavior *offers* — a value others can use | a relationship without inputs that is **defined** inside the source |
| **Parameter** | what is *configured* when the component is used — a constant per instance | a declared relationship without inputs, given a closed value (a number with a unit, `true`, …) on each instance |

Each port carries a **contract**: the concept it produces, and how it
relates to timing — *any domain*, *its own private domain*, or *a timing
parameter of the component* (a domain the instance chooses). The contract
is stored with the port, not read off the source, so the component can
promise something and the tool can tell you when the source **no longer
keeps the promise**: the component's row gets a red mark and its instances
are drawn *unrealized* until the source or the contract is fixed.

**Timing parameters** are the component's domains that each instance maps
to a domain of the system. A component that updates *in its own* domain
brings that domain with it.

## Shared and private

Inside the source a concept is normally **private**: each instance gets
its own *Brightness*, distinct from every other instance's and from the
system's. A component can instead declare a concept **shared** — *this is
the system's Brightness* — so that its ports carry the same concept as
the rest of the design. Sharing is a decision you make; it is never
inferred from names, units or library templates.

## Where components come from

* **Packaging a behavior group** — the usual way. *Package as Reusable
  Component…* on a group turns its members into a component's source, the
  group's boundary into ports, and leaves one instance where the group
  stood. [Workflow: packaging a behavior](../workflows/package-as-component.md).
* **From scratch** — **+** by *Components* in the sidebar, then *Edit
  Source*: an empty design. Select the component in the sidebar: its inspector's
  **Declare a port** section lists the source's relationships and lets you
  expose one as *requires*, *provides* or *parameter*.

## Editing a component

Double-click an instance, or select the component and choose **Edit
Source**. The canvas switches to the component's own design; the bar above
it says *Editing AdaptiveLamp · used by 3 instances* with a *‹ System* link
back. Everything on the Design page works here as in a plain design —
sheets, formula editor, groups, completion, hover in the formula field —
in the component's own names. What you change reaches every instance
when you save the definition.

The component's inspector (select it in the sidebar) shows its **Ports**
with their contracts, **Timing parameters**, **Shared concepts**, and the
instances that use it, and offers **Duplicate as Version**, port
retirement, and deletion (*Delete its instances first.*).

## What a component is not

* Not a group with a different icon: a group's box sockets are a picture;
  a component's ports are a promise other things depend on.
* Not a copy: two instances share one source. To have two versions of a
  behavior, *Duplicate as Version* ([Versioning a component](../workflows/component-versioning.md)).
* Not portable across projects yet: a component lives in the project
  that defines it.
* Not nestable yet: a component's source is a flat design; a behavior
  group inside a source can be made and used, but not packaged into a
  component of its own.

## Going deeper

*For language implementers.* A component is `BehaviorComponent { body:
Design, interface, shared_concepts, external_outputs }`; its interface is a
stored promise per port (`PortContract`), checked against the body by
`Realizes` once per component, not per instance (ADR-0018). Instances are
freshened copies with private identities; bindings are references by
identity, never by name. `docs/BEHAVIOR_SYSTEMS.md` maps every production
object to the formal development; `docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md`
is the implementation design.

## Related

[Behavior groups](behavior-groups.md) · [Behavior systems](behavior-systems.md) ·
[System projects](../studio/system-projects.md) ·
[Workflow: composing components](../workflows/composing-components.md)
