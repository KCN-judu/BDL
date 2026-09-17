# Concepts

A **concept** is something the product senses, decides or shows, given a
name: *Tilt*, *Brightness*, *Held*, *Battery level*. A concept is the unit
of meaning in a BDL design. Relationships read concepts and produce
concepts; outputs accept a concept; everything else is built from them.

## A concept is a meaning, not a number

*Brightness* and *Opacity* may both be plain numbers between 0 and 1.
*Battery level* may be a plain number between 0 and 1 too. They are three
concepts, and BDL keeps them apart: a relationship that produces
*Opacity* cannot be connected where *Brightness* is read, however the
numbers happen to line up. This is what makes a design checkable — the
tool can tell you that you have wired a level into a brightness, which no
spreadsheet ever will.

On the canvas this shows as colour. Every concept has its own hue, and a
link can only join sockets of the same hue. While you drag a link, the
sockets that can accept it light up; the others do not.

## Value form

A concept has a **value form** — what kind of value it carries:

| Value form | Socket | Examples |
|---|---|---|
| **Quantity** — a measured amount, with a unit | ○ round | an angle in `rad`, a length in `m`, a temperature in `K`; or *no unit* for a plain level |
| **On / off** — a truth value | ◇ diamond | *Held*, *Button pressed*, *Door open* |
| **Count** — a whole number | □ square | *Steps*, *Items*, *Presses* |
| **Decide later** — not chosen yet | hollow ring | any concept you have named but not pinned down |

For a quantity the unit is chosen from a list of quantity kinds (angle,
length, time, mass, temperature, current, and derived kinds such as speed,
frequency, voltage, illuminance). The symbol is shown beside the kind. The
choice fixes the concept's *dimension*, which is what formulas are checked
against: a *Tilt* in radians plus a *Time* in seconds is refused; a *Tilt*
divided by `90 deg` is a plain number.

A quantity with *no unit* is still a quantity. Brightness as a level from
0 to 1 is the usual example.

## Decide later

You can name a concept before you know how it is measured. Its sockets are
drawn as hollow rings, relationships can already read and produce it, and
any formula over it waits: the inspector says *Checked once Temperature's
value is decided.* Nothing is wrong. See
[Incomplete designs](incomplete-designs.md).

Choosing a value form the first time is a refinement — nothing that
depends on the concept needs rechecking. *Changing* a value form that was
already chosen is an edit, and the inspector names the relationships that
will be checked again.

## Where concepts come from

* **The *New concept* sheet** (sidebar → *Project* → **+** by *Concepts*):
  name, value form, unit, meaning. The sheet previews the row as you type.
* **The Library tab**: ready-made concepts — *Temperature*, *Tilt*,
  *Button Pressed*, *Motor Speed*, … — grouped by role. Drag one onto the
  canvas or use the canvas's right-click **Add Concept** menu. The result
  is an ordinary concept with a suggested name and value form; two
  insertions of the same template are two concepts.
  See [Library](../studio/library.md).

## What the inspector shows

* **Meaning** — the name and a free-text description of what the concept
  means to the product.
* **Value** — the value form and unit.
* **Relationships** — *Produced by* and *Used by*, as links.
* **Delete** — disabled while a relationship uses the concept, with the
  users named.

## Going deeper

*For language implementers.* A concept is a nominal semantic type; its
value form is its *representation*, chosen once (a write-once binding). A
formula reads a concept through `rep` and produces one through `mk`, and
the dimension check is the representation's dimension algebra.
`docs/spec/kernel.md` and `docs/architecture/compiler-pipeline.md` (passes 3–7) are
the reference; `docs/spec/concept-library.md` explains why a library
template is not an identity.

## Related

[Relationships](relationships.md) · [Incomplete designs](incomplete-designs.md) ·
[Library](../studio/library.md) · [Types, units and concepts](../troubleshooting/type-and-concept-errors.md)
