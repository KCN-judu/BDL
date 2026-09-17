# Library

The sidebar on the Design page has two tabs.

## Project tab

The project's objects, by kind, each section with a **+** that opens the
creation sheet for that kind:

| Section | + opens | Row |
|---|---|---|
| **Concepts** | *New concept* — name, value form (Quantity / On / off / Count / Decide later), unit, meaning; a live preview of the row | the concept's glyph and name; click to select |
| **Mappings** | *New mapping* — name, *Reads* (toggle concepts), *Produces* (choose one; Create stays disabled until you do); a live preview of the node | the relationship; click to select |
| **Timing domains** | *New timing domain* — a name | the domain; rename in place; delete while unused |
| **Outputs** | *New output* — name, *Accepts*, *Updates in*, *Required* | the sink with its state glyph |
| **Contexts** | — (a heading; contexts are not part of the tool yet) | |

In a **system project** the tab also lists **Components** (*New
component* — a name; the source is empty until you edit it), **Instances**
(when the system canvas is showing), and **Behaviors** (*New Behavior
Group* — an empty group to drag relationships into). A component's row carries a
red mark when its source no longer keeps its promise, and is filled while
its source is the one open on the canvas.

Selecting a row selects the object on the canvas and in the inspector.

## Library tab

Ready-made **concept templates**: a search field, categories
(*Environment*, *Human interaction*, *Geometry & motion*, *Mechanical*,
*Electrical & system*, *Visual & display*, *Actuation*, *Audio*) and rows with a grey socket glyph — filled when the template
comes with a value form, hollow when it leaves the value to you — the unit
or kind in the right column, and the description on hover.

To use one: **drag the row onto the canvas**, or double-click it. The concept is
created with the template's default name and value form, lands where you
dropped it, and its name opens for editing — type *MotorTemperature* over
*Temperature* and keep going. The same templates are on the canvas's
right-click **Add Concept ▸** menu, with your recent choices first.

A template is a way of *making* a concept, not an identity: inserting
*Temperature* twice gives two independent concepts, and editing the
project later never reaches back into the library. Once inserted, a
concept is yours — rename it, change its unit, delete it.

The templates and the list of quantity kinds come from the compiler
service, so the Library, the unit picker and the textual syntax agree on
names.

## Related

[Concepts](../concepts/concepts.md) · [Canvas](canvas.md) ·
[Inspector](inspector.md)
