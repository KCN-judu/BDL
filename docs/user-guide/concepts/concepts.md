# Concepts

A **concept** is something the product senses, decides or shows, given a name:
_Tilt_, _Brightness_, _Held_, _Battery level_. A concept is the unit of meaning
in a BDL design. Relationships read concepts and produce concepts; outputs
accept a concept; everything else is built from them.

## A concept is a meaning, not a number

_Brightness_ and _Opacity_ may both be plain numbers between 0 and 1. _Battery
level_ may be a plain number between 0 and 1 too. They are three concepts, and
BDL keeps them apart: a relationship that produces _Opacity_ cannot be connected
where _Brightness_ is read, however the numbers happen to line up. This is what
makes a design checkable — the tool can tell you that you have wired a level
into a brightness, which no spreadsheet ever will.

On the canvas this shows as colour. Every concept has its own hue, and a link
can only join sockets of the same hue. While you drag a link, the sockets that
can accept it light up; the others do not.

## Value form

A concept has a **value form** — what kind of value it carries:

| Value form                                    | Socket      | Examples                                                                                 |
| --------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------- |
| **Quantity** — a measured amount, with a unit | ○ round     | an angle in `rad`, a length in `m`, a temperature in `K`; or _no unit_ for a plain level |
| **On / off** — a truth value                  | ◇ diamond   | _Held_, _Button pressed_, _Door open_                                                    |
| **Count** — a whole number                    | □ square    | _Steps_, _Items_, _Presses_                                                              |
| **Decide later** — not chosen yet             | hollow ring | any concept you have named but not pinned down                                           |

For a quantity the unit is chosen from a list of quantity kinds (angle, length,
time, mass, temperature, current, and derived kinds such as speed, frequency,
voltage, illuminance). The symbol is shown beside the kind. The choice fixes the
concept's _dimension_, which is what formulas are checked against: a _Tilt_ in
radians plus a _Time_ in seconds is refused; a _Tilt_ divided by `90 deg` is a
plain number.

A quantity with _no unit_ is still a quantity. Brightness as a level from 0 to 1
is the usual example.

## Decide later

You can name a concept before you know how it is measured. Its sockets are drawn
as hollow rings, relationships can already read and produce it, and any formula
over it waits: the inspector says _Checked once Temperature's value is decided._
Nothing is wrong. See [Incomplete designs](incomplete-designs.md).

Choosing a value form the first time is a refinement — nothing that depends on
the concept needs rechecking. _Changing_ a value form that was already chosen is
an edit, and the inspector names the relationships that will be checked again.

## Where concepts come from

- **The _New concept_ sheet** (sidebar → _Project_ → **+** by _Concepts_): name,
  value form, unit, meaning. The sheet previews the row as you type.
- **The Library tab**: ready-made concepts — _Temperature_, _Tilt_, _Button
  Pressed_, _Motor Speed_, … — grouped by role. Drag one onto the canvas or use
  the canvas's right-click **Add Concept** menu. The result is an ordinary
  concept with a suggested name and value form; two insertions of the same item
  are two concepts. See [Library](../studio/library.md).

## What the inspector shows

- **Meaning** — the name and a free-text description of what the concept means
  to the product.
- **Value** — the value form and unit.
- **Relationships** — _Blocks_ (the blocks of this concept) and _Rules_ (the
  rules over it), as links.
- **Delete** — disabled while a relationship uses the concept, with the users
  named.

## Going deeper

_For language implementers._ A concept is a nominal type — `sem C`, read "a Sem
of _C_" — and so a template: each Source or value of that concept in a design is
a _Sem block_, an instance holding one value per tick, and its formula is its
_mapping block_ — the node drawn beside it; a rule is a template in the same
way. Its value form is its _representation_, chosen once (a write-once binding).
A formula reads a concept through `rep` and produces one through `mk`, and the
dimension check is the representation's dimension algebra. `docs/spec/kernel.md`
and `docs/architecture/compiler-pipeline.md` (passes 3–7) are the reference;
`docs/spec/concept-library.md` explains why a library item is not an identity.

## Related

[Relationships](relationships.md) · [Incomplete designs](incomplete-designs.md)
· [Library](../studio/library.md) ·
[Types, units and concepts](../troubleshooting/type-and-concept-errors.md)
