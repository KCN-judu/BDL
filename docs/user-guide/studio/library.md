# Library

The sidebar on the Design page has two tabs.

## Project tab

The project's objects, by kind, each section with a **+** that opens the
creation sheet for that kind:

| Section            | + opens                                                                                                                                  | Row                                                             |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| **Concepts**       | _New concept_ — the concept sheet (below): a name, a value category, its meaning; a live preview of the node and its declaration         | the concept's glyph and name; click to select                   |
| **Mappings**       | _New mapping_ — name, _Reads_ (toggle concepts), _Produces_ (choose one; Create stays disabled until you do); a live preview of the node | the relationship (a Source with its own glyph); click to select |
| **Timing domains** | _New timing domain_ — a name                                                                                                             | the domain; rename in place; delete while unused                |
| **Outputs**        | _New output_ — name, _Accepts_, _Updates in_, _Required_                                                                                 | the sink with its state glyph                                   |
| **Contexts**       | — (a heading; contexts are not part of the tool yet)                                                                                     |                                                                 |

The tab also lists **Components** (_New component_ — a name; the source is empty
until you edit it), **Instances** (when the system canvas is showing), and
**Behaviors** (_New Behavior Group_ — an empty group to drag relationships
into). A component's row carries a red mark when its source no longer keeps its
promise, and is filled while its source is the one open on the canvas.

Selecting a row selects the object on the canvas and in the inspector.

## Library tab

The **Standard Library**: the kinds of value a concept can be, with a search
field above. It does not know your product — there is no _Motor Angle_ or _Fan
Speed_ here — because to the compiler those are an angle and a level, and the
name is yours to give. The tab has:

- **Recent** — the categories you used last, most recent first.
- **Values** — _On / off_, _Count_, _Level_ (a plain number: a ratio, a
  percentage, a 0–1 setting), _Decide later_ (a concept whose value form you
  choose once you know it).
- **Quantities** — one row per physical quantity: _Angle_, _Length_, _Time_,
  _Mass_, _Current_, _Temperature_, _Amount of substance_, _Luminous intensity_,
  _Speed_, _Acceleration_, _Angular velocity_, _Frequency_, _Force_, _Pressure_,
  _Torque_, _Power_, _Voltage_, _Illuminance_.
- a **Source** row — _A value the environment provides — for a concept you
  choose, existing or new._ (below).

A category row carries a grey socket glyph — filled for a value form, hollow for
_Decide later_ — the unit a value of it is usually written in (`rad`, `K`,
`rad/s`) or the kind (`on–off`, `count`, `no unit`) in the right column, and the
description on hover. Names and descriptions follow the language you chose, and
so does search: type what you are looking for in your words — `servo`, `tilt`
and `encoder` find _Angle_; `heater`, `fan`, `battery` and `brightness` find
_Level_; `lux` finds _Illuminance_; `deg` finds _Angle_ and `rad/s` _Angular
velocity_; `温度` finds _Temperature_ in 简体中文 and 日本語.

### Creating a concept

Double-click a row, press Return on it, or **drag it onto the canvas**, and the
**concept sheet** opens — _A value category from the Library, and the name it
has in this product._ The same categories are on the canvas's right-click **Add
Block ▸ New Concept ▸** menu (_Recent_, the four values, _Quantities ▸_, _More…_
for the tab), and the Project tab's **+** opens the sheet with the category left
for you to choose. A concept is a template and is never itself on the canvas:
created from the canvas or by a drag onto it, the concept _and a block of it_ —
one value of the new concept — arrive in one step, the block where you dropped
or clicked; created from the Project tab, only the concept, and **Add Block ▸ of
_Name_** (or dragging the concept's Project row onto the canvas) puts a block of
it there through the block sheet.

![A sheet titled New concept — A value category from the Library, and the name it has in this product — with an empty Name field, a Value pop-up reading Angle, a Measured in row listing rad, deg and turn, a Meaning field, a preview of the node, the declaration concept … : Angle, the line Creates one concept, as the Code view will write it., and Cancel and Create Concept buttons.](../assets/studio/concept-sheet.png)

_The concept sheet for Angle: the name is yours to give; the units an angle is
measured in are shown, not chosen._

1. **Name** — required, and the one thing the Library cannot know: _LidAngle_,
   _MotorTemperature_, _RoomLight_. A name is letters, digits and `_`, not
   starting with a digit, and must be free in the design; the sheet says
   _LidAngle is already in use._ before anything is created.
2. **Value** — the category, changeable here in one pop-up.
3. **Measured in** — for a physical quantity, the units a formula over this
   concept may use: `rad · deg · turn` for an angle, `rad/s · deg/s` for an
   angular velocity. This is information, not a choice: a concept has a kind of
   value, and any unit of that kind is allowed wherever the concept is used. A
   literal in a formula carries its own unit (`90 deg`).
4. **Meaning** — a sentence for the inspector and the Code view.

The preview shows the node and the line the Code view will write —
`concept LidAngle : Angle`. **Create Concept** makes it: the concept lands where
you dropped or right-clicked, selected and named; nothing opens for renaming,
because you named it first. **Cancel** leaves the project untouched.

A category is a way of _making_ a concept, not an identity: two concepts made
from _Temperature_ — _RoomTemperature_ and _MotorTemperature_ — are two concepts
that share nothing but a kind of value, and editing the project later never
reaches back into the Library. Once created, a concept is yours — rename it,
change its value form, delete it.

### Sources

A Source is a value entering the behavior model from the environment — a
relationship that reads nothing, produces a concept, and has no formula. The
**Source** row (the Source glyph instead of the socket) opens the **Source
sheet** — double-click, Return or a drag onto the canvas; so does _New source…_
on the canvas's **Add Source ▸** menu. Its one decision is the concept the
Source provides:

- **Existing concept** — pick one of the design's concepts. Two concepts of the
  same kind — _RoomTemperature_ and _MotorTemperature_, both temperatures — are
  two rows, and you choose the one you mean. Only the Source is created; the
  concept stays as it is. One Undo removes the Source.
- **New concept** — the concept sheet's fields: a name, a value category, the
  units it is measured in, a meaning. The concept and its Source are created
  together, in one step; one Undo removes both.

Then the **Source name** — suggested from the concept (_roomTemperatureInput_)
and yours to change — and its meaning. The sheet shows the exact objects it will
create, as the Code view will write them; _Create Source_ is enabled once the
choice is complete, and Cancel leaves the project untouched. A new concept lands
where you dropped or right-clicked and is selected, with its Source to the left;
a Source over an existing concept lands there and is selected.

Not every input is a sensor — a button's state, an analog level and a value a
host or a network provides are Sources too — and no Source says which part will
provide it: that belongs to [deployment](deploy.md). For an analog reading or a
host-provided value whose kind you do not know yet, choose _Decide later_ and
set the value form once you do.

The categories, the units and the list of quantity kinds come from the compiler
service, so the Library, the sheet, the formula editor's unit pop-ups and the
textual syntax agree on names.

## Related

[Concepts](../concepts/concepts.md) · [Canvas](canvas.md) ·
[Inspector](inspector.md)
