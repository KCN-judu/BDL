# Library

The sidebar on the Design page has two tabs.

## Project tab

The project's objects, by kind, each section with a **+** that opens the
creation sheet for that kind:

| Section            | + opens                                                                                                                                  | Row                                                             |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| **Concepts**       | _New concept_ — name, value form (Quantity / On / off / Count / Decide later), unit, meaning; a live preview of the row                  | the concept's glyph and name; click to select                   |
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

The **Standard Library**: ready-made items in two sections, **Concepts** and
**Sources**, with a search field above. Each section is grouped (_Environment_,
_Human interaction_, _Geometry & motion_, _Mechanical_, _Electrical & system_,
_Visual & display_, _Actuation_, _Audio_; the Sources add _External_). A Concept
row carries a grey socket glyph — filled when the item comes with a value form,
hollow when it leaves the value to you — the unit or kind in the right column,
and the description on hover. Item names and descriptions follow the language
you chose, and so does search — `温度` finds the Temperature Input in 简体中文,
`センサー` in 日本語; the names of what they create do not change.

To use one: **drag the row onto the canvas**, or double-click it. The concept is
created with the item's default name and value form, lands where you dropped it,
and its name opens for editing — type _MotorTemperature_ over _Temperature_ and
keep going. The same items are on the canvas's right-click **Add Concept ▸**
menu, with your recent choices first.

An item is a way of _making_ objects, not an identity: inserting _Temperature_
twice gives two independent concepts, and editing the project later never
reaches back into the library. Once inserted, a concept is yours — rename it,
change its unit, delete it.

### Sources

A Source is a value entering the behavior model from the environment — a
relationship that reads nothing, produces a concept, and has no formula. The
**Sources** section — _Temperature Input_, _Tilt Input_, _Distance Input_,
_Ambient Light Input_, _Button Input_, _Encoder Input_, _Analog Input_,
_External Input_ — holds **presets** for making one. Its rows wear the Source
glyph instead of the socket; the right column names the value form a preset
suggests, and the hover says what a preset does: _an input for a Temperature
concept you choose — existing, or new_.

Double-click a row, press Return on it, or drag it onto the canvas, and the
**Source sheet** opens. Its one decision is the concept the Source provides:

- **Existing concept** — pick one of the design's concepts. Two concepts of the
  same kind — _RoomTemperature_ and _MotorTemperature_, both temperatures — are
  two rows, and you choose the one you mean; the preset lists the concepts of
  its kind first. Only the Source is created; the concept stays as it is. One
  Undo removes the Source.
- **New concept** — a name, a value form, a unit and a meaning, prefilled by the
  preset (_Temperature_, Quantity, K). The concept and its Source are created
  together, in one step; one Undo removes both.

Then the **Source name** — suggested from the concept (_roomTemperatureInput_)
and yours to change — and its meaning. The sheet shows the exact objects it will
create, as the Code view will write them; _Create Source_ is enabled once the
choice is complete, and Cancel leaves the project untouched. A new concept lands
where you dropped or right-clicked and opens for renaming, with its Source to
the left; a Source over an existing concept lands there and is selected.

Not every input is a sensor — a button's state, an analog level and a value a
host or a network provides are Sources too — and no Source says which part will
provide it: that belongs to [deployment](deploy.md). _Analog Input_ and
_External Input_ leave the value form to you: choose it once you know what the
input measures. _New Source…_ on the canvas's **Add Source ▸** menu is the same
sheet without a preset.

The items and the list of quantity kinds come from the compiler service, so the
Library, the unit picker and the textual syntax agree on names.

## Related

[Concepts](../concepts/concepts.md) · [Canvas](canvas.md) ·
[Inspector](inspector.md)
