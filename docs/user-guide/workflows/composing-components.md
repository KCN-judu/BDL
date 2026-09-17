# Composing components

**Goal.** Place a second instance of _AdaptiveLamp_ from
[Packaging a behavior](package-as-component.md), feed both from the same tilt,
and use each one's brightness — then see what the tool does when two instances
would drive one light.

## Steps

1. **A second instance.** Right-click empty canvas → **Add Instance ▸
   AdaptiveLamp**. Name it `second`. It appears with an open _tiltValue_ socket
   (hollow) on the left and _brightness_ on the right.
2. **Its timing.** Select it. Under **Timing**, the component's parameter _main_
   needs a system domain: choose _main_. (An instance created from a packaging
   already has this set.)
3. **Fan out the tilt.** Drag from `tiltValue`'s output socket onto `second`'s
   _tiltValue_ socket. One top-level value now feeds two instances: the value is
   computed once and read by both.
4. **Use the second brightness.** _Mappings_ **+**: `mirror`, reads nothing,
   produces Brightness, updates in _main_, no formula. Drag from `second`'s
   _brightness_ socket onto `mirror`'s socket: `mirror` is now _bound to
   second.brightness_.
5. **Try to replace a source.** Drag from `adaptiveLamp`'s _brightness_ socket
   onto `mirror` as well. A sheet asks _Replace the connection?_ — **Disconnect
   and Connect** or **Cancel**. Cancel. A required port or a bound relationship
   has one source; the tool never swaps it silently.
6. **Simulate.** ⌘2. The inputs are the system's (`raw`); the trace has columns
   `adaptiveLamp.brightness`, `second.brightness`, `mirror`, `brightness`,
   `indicator`, and the light. Step: both instances compute the same values from
   the same tilt.
7. **Make the conflict.** On Design, connect `mirror` to the _light_ sink as
   well (drag onto the sink). The light is **contested** — two values, from two
   instances, one output — reported exactly as in a plain design. Detach one via
   _Fixes_.

## What BDL means by this

- An **instance** is one use of a component's source with fresh identities of
  its own: `second.brightness` and `adaptiveLamp.brightness` are two values,
  even though one source defines both. Here they are two values of the system's
  _Brightness_, because packaging shares the concepts the group used; a
  component authored from scratch keeps its concepts private — one per instance
  — unless you share them ([Components](../concepts/components.md)).
- A **binding** is a definition by identity: `mirror` _is_ `second.brightness`.
  No conversion, no copying. **Fan-out** is the same value read from several
  places.
- **One source per destination.** Rebinding is explicit: _Disconnect and
  Connect_.
- **Outputs stay the system's.** Two instances cannot both drive the light; the
  rule is the single-driver rule you already know.
- Everything is one design to the checker and simulator: an instance's values
  are simulated alongside the system's, and an instance's open port shows up as
  an input.

## If it does not work

- The link will not land — the concepts differ: the port carries the component's
  _private_ Brightness while your relationship produces the system's, or the
  reverse. Share the concept in the component, or bind to a relationship of the
  right concept. [Connections](../troubleshooting/connection-errors.md).
- The sheet asks _Carry across timing domains_ — the two ends update in
  different domains; see
  [Carrying a value across timing domains](cross-domain-transport.md).
- _Open: nothing supplies it yet._ on a port — expected until you bind it; the
  simulator will ask for it as an input meanwhile.

## Next

[Carrying a value across timing domains](cross-domain-transport.md) ·
[Versioning a component](component-versioning.md)
