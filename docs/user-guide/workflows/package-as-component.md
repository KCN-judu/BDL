# Packaging a behavior as a component

**Goal.** Turn the *Adaptive lamp* behavior from
[Grouping a behavior](grouping-behavior.md) into a reusable component
with a promise — and keep the design computing exactly what it did.

## Steps

1. **Open the sheet.** Right-click the *Adaptive lamp* region → **Package
   as Reusable Component…** (or the inspector's button of the same name).
   The sheet reads *Working out the boundary…* and then fills in.
2. **Read what was computed.** **Requires: tiltValue** — the members read
   it from outside, so the component will need it. **Provides:
   brightness** — outside reads it, so the component will offer it.
   **Timing parameters: main** — the domain the members use. These are
   the floor: the sheet never offers to remove one.
3. **Make the four decisions.**
   * Component name: `AdaptiveLamp`. The instance name follows
     (`adaptiveLamp`); change it if you like.
   * *Open relationships* — members without a formula, if any: **Treat as
     input** (a required port) or **Keep internal**. None here.
   * *Physical outputs* — `light`, driven by a member: **Stays the
     system's** (the component provides `brightness`, the system keeps
     the light) or **Moves inside** (the light becomes the component's).
     Keep it the system's.
   * Every change re-asks the preview.
4. **Package.** One edit. On the system canvas an instance node
   `adaptiveLamp` stands where the region was, with *tiltValue* on its
   left, *brightness* on its right and *AdaptiveLamp · main* in its body.
   Two links: `tiltValue` → the required port, the provided port → a
   top-level `brightness`, which is now an *open* relationship **bound
   to** the port (select it: *Takes its value from adaptiveLamp.brightness*).
   The light is still driven by that `brightness`.
5. **Check.** ⌘2, Step: the same trace, tick for tick. ⌘3: the same
   placement.
6. **Look inside.** Double-click the instance. The canvas shows the
   component's source: `tiltValue` (*requires*), `dimByTilt`, `brightness`
   (*provides*), laid out as the group was. The bar says *Editing
   AdaptiveLamp · used by 1 instance*. Change `brightness`'s formula to
   `dimByTilt(tiltValue) * 2` and *Save definition*; back at *‹ System*,
   Step again: the light doubled. The source is the one place the
   behavior is defined.

<!-- figure F14 -->

## What BDL means by this

* The **boundary** the tool computed for the group became the
  component's **promise**: crossing-in → *requires*, crossing-out →
  *provides*, the used domains → *timing parameters*. The tool cannot
  infer the four decisions above, so it asks.
* The top-level design **keeps** a relationship for each provided value
  (`brightness`), now unresolved and **bound** to the instance's port.
  A binding is a definition by identity — the top-level `brightness` *is*
  the instance's `brightness` — never a copy and never a lookup by name.
* Packaging **preserves behavior**: the design computes the same values
  before and after. (The formal development proves this for the
  single-domain fragment; Studio's tests check the trace on the wire.)
* A member's **identity** survives: `dimByTilt` inside the component is
  the same relationship, with its place on the canvas.

## If it does not work

* The menu item is missing on a behavior inside a component's source —
  packaging there (nested components) is not built; the inspector says
  so.
* The sheet lists an *Open relationship* you did not expect — a member
  has no formula; decide whether it is an input of the component or
  internal work still to do.
* After packaging the light is *contested* — the light was driven by two
  members; combine them first, or let the sheet move the light inside.

## Next

[Composing components](composing-components.md) — a second lamp.
