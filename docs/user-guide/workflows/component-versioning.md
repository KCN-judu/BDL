# Versioning a component

**Goal.** Make a second version of _AdaptiveLamp_, move one instance to it, and
see what the tool checks when an instance changes component — and what it does
not.

Editing a component's source changes **every** instance at once. When you want
some instances to keep the old behavior, you make a **version**: a separate
component with the same promise.

## Steps

1. **Duplicate.** Select _AdaptiveLamp_ in the sidebar → **Duplicate as
   Version**. Name it `AdaptiveLamp v2`. A second component appears with the
   same ports and the same source; it has no instances yet.
2. **Change v2.** _Edit Source_ on v2; change `brightness` to
   `dimByTilt(tiltValue) / 2`; _Save definition_; _‹ System_.
3. **Move an instance.** Select `second` → under **Implemented by**, **Replace
   with** _AdaptiveLamp v2_. The instance keeps its name, its place, and every
   binding; it now runs v2's source. `adaptiveLamp` still runs the original.
4. **Simulate.** ⌘2, Step: `adaptiveLamp.brightness` and `second.brightness` now
   differ.
5. **Retire a port on v2.** Select v2 → _Ports_ → **Retire port** on
   _tiltValue_. Refused with a banner: `second`'s _tiltValue_ is bound. A port
   in use cannot be retired. Disconnect `second`'s _tiltValue_ first, then
   retire it (the source's `tiltValue` relationship stays; it is just no longer
   promised).
6. **Try to move the first instance to v2.** _Replace with_ v2 on
   `adaptiveLamp`. Refused: `adaptiveLamp` uses a port v2 no longer offers. Move
   it back, or restore the port.

## What BDL means by this

- A version is a **copy with the same promise**, made by _Duplicate as Version_.
  Its ports carry the same identities, which is what lets an instance move
  between the two. A component authored separately, even with identical-looking
  ports, cannot stand in for another.
- **Replace with** is checked for **wiring**: every port the instance uses must
  exist on the new component with a compatible contract (same concept, same
  timing relation). If it does, the bindings survive; if not, the replacement is
  refused and nothing changes.
- It is **not** checked for **behavior**: nothing verifies that v2 computes the
  same values as v1. That is your decision — it is the point of a version — and
  the simulator is how you compare.
- A **contract change** on a port (say, from _a timing parameter_ to _any
  domain_) is an edit like any other: the tool re-checks whether the source
  keeps the new promise and marks the component and its instances when it does
  not. ⌘Z undoes it.

## If it does not work

- _A bound port cannot be retired_ — disconnect the instances' bindings on that
  port first.
- _Replace with_ refused — the instance uses a port the target lacks, or a port
  whose contract differs; compare the two components' _Ports_.
- Deleting a component is refused — _Delete its instances first._ Deleting an
  instance is refused — _Disconnect its ports first; the component stays._

## Related

[Components](../concepts/components.md) ·
[Behavior systems](../concepts/behavior-systems.md)
