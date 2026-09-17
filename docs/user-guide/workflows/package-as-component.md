# Packaging a behavior as a component

**Goal.** Turn the _Adaptive lamp_ behavior from
[Grouping a behavior](grouping-behavior.md) into a reusable component with a
promise — and keep the design computing exactly what it did.

## Steps

1. **Open the sheet.** Right-click the _Adaptive lamp_ region → **Package as
   Reusable Component…** (or the inspector's button of the same name). The sheet
   reads _Working out the boundary…_ and then fills in.
2. **Read what was computed.** **Requires: tiltValue** — the members read it
   from outside, so the component will need it. **Provides: brightness** —
   outside reads it, so the component will offer it. **Timing parameters: main**
   — the domain the members use. These are the floor: the sheet never offers to
   remove one.
3. **Make the four decisions.**
   - Component name: `AdaptiveLamp`. The instance name follows (`adaptiveLamp`);
     change it if you like.
   - _Open relationships_ — members without a formula, if any: **Treat as
     input** (a required port) or **Keep internal**. None here.
   - _Physical outputs_ — `light`, driven by a member: **Stays the system's**
     (the component provides `brightness`, the system keeps the light) or
     **Moves inside** (the light becomes the component's). Keep it the system's.
   - Every change re-asks the preview.
4. **Package.** One edit. On the system canvas an instance node `adaptiveLamp`
   stands where the region was, with _tiltValue_ on its left, _brightness_ on
   its right and _AdaptiveLamp · main_ in its body. Two links: `tiltValue` → the
   required port, the provided port → a top-level `brightness`, which is now an
   _open_ relationship **bound to** the port (select it: _Takes its value from
   adaptiveLamp.brightness_). The light is still driven by that `brightness`.
5. **Check.** ⌘2, Step: the same trace, tick for tick. ⌘3: the same placement.
6. **Look inside.** Double-click the instance. The canvas shows the component's
   source: `tiltValue` (_requires_), `dimByTilt`, `brightness` (_provides_),
   laid out as the group was. The bar says _Editing AdaptiveLamp · used by 1
   instance_. Change `brightness`'s formula to `dimByTilt(tiltValue) * 2` and
   _Save definition_; back at _‹ System_, Step again: the light doubled. The
   source is the one place the behavior is defined.

<!-- figure F14 -->

## What BDL means by this

- The **boundary** the tool computed for the group became the component's
  **promise**: crossing-in → _requires_, crossing-out → _provides_, the used
  domains → _timing parameters_. The tool cannot infer the four decisions above,
  so it asks.
- The top-level design **keeps** a relationship for each provided value
  (`brightness`), now unresolved and **bound** to the instance's port. A binding
  is a definition by identity — the top-level `brightness` _is_ the instance's
  `brightness` — never a copy and never a lookup by name.
- Packaging **preserves behavior**: the design computes the same values before
  and after. (The formal development proves this for the single-domain fragment;
  Studio's tests check the trace on the wire.)
- A member's **identity** survives: `dimByTilt` inside the component is the same
  relationship, with its place on the canvas.

## If it does not work

- The menu item is missing on a behavior inside a component's source — packaging
  there (nested components) is not built; the inspector says so.
- The sheet lists an _Open relationship_ you did not expect — a member has no
  formula; decide whether it is an input of the component or internal work still
  to do.
- After packaging the light is _contested_ — the light was driven by two
  members; combine them first, or let the sheet move the light inside.

## Next

[Composing components](composing-components.md) — a second lamp.
