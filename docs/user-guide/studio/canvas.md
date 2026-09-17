# Canvas

The canvas is the Design page's centre: a picture of the design's structure.
Everything on it means something, and each visual channel means one thing only.

## What the marks mean

| You see                                                         | It means                                                                                                                        |
| --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| **socket colour**                                               | _which concept_ — every concept has its own hue, the same everywhere it appears                                                 |
| **socket shape**                                                | the concept's _value form_: ○ quantity, ◇ on / off, □ count; a **hollow ring** while the value form is _decide later_           |
| **a link**                                                      | _this relationship reads that concept_ (concept → relationship input), or _this value drives that output_ (relationship → sink) |
| **dashed outline**                                              | _declared_: a relationship without a formula; an output without a driver or without a domain                                    |
| **a red mark at the formula line**                              | the formula does not check — _wrong now_, never _not yet_                                                                       |
| **the word _declared_, _contested_, _ill-formed_, _no domain_** | the one state word a node may carry, only while that state holds (an output that must be driven says _required_ meanwhile)      |
| **a small name at a node's right edge**                         | its timing domain                                                                                                               |
| **the accent colour**                                           | selection — and nothing else                                                                                                    |

Position, size and the direction of links carry **no meaning**. Data is drawn
left to right for readability; edges show dependency, not the order in which
anything runs; there is no node per operator — arithmetic lives in the formula
field.

## The nodes

![Concept rows Tilt and Brightness with a round socket at each end; relationship nodes with a name header, one input socket per concept read on the left, one output socket on the right and the formula in the body; dimByTilt outlined in the accent colour because it is selected; tilt drawn dashed with the word declared; the Light Output sink at the right with the word required in its header, a single input socket and a bar at its right edge; tilt, brightness and the output carry the domain name interaction at their right edge.](../assets/studio/node-anatomy.png)

_Node anatomy on the tilt lamp: concept rows, relationship nodes (dimByTilt
selected), and the Light Output sink._

A **concept** is one row: its name, an input socket on the left (something
produces this concept) and an output socket on the right (relationships read it
from here). Both sockets carry the concept's colour and shape.

A **relationship** is a box: a header with the name and, while it applies, the
one state word (_declared_ on `tilt` above, which has no formula); one input
socket per concept it reads, on the left, each labelled with the concept; one
output socket on the right, labelled with the concept it produces; and the
formula on the line below, with the timing domain's name at the right edge when
the relationship has one. A relationship that reads nothing — `tilt`,
`brightness` — has no input sockets; it is a value, and its formula, if any,
names the values and rules it depends on.

A **physical output** is a sink: a tinted header with its name and a word at the
right — _required_ for an output the design must drive before it is complete, or
the state that overrides it (_no domain_, _contested_, _ill-formed_); one input
socket labelled with the concept it accepts; the timing domain at the right
edge; and a bar down its right side — nothing flows out of it.

The selected node (`dimByTilt` above) is outlined in the accent colour; nothing
else on the canvas uses that colour.

In a system project there are also **instance nodes** (one row per port, the
component's name in the body) and **behavior regions** or collapsed **behavior
boxes**; see [System projects](system-projects.md).

## Gestures

| Do                                                               | Result                                                                                                                 |
| ---------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| drag empty canvas                                                | pan                                                                                                                    |
| scroll wheel / pinch                                             | zoom about the pointer                                                                                                 |
| Home, or ⌘0                                                      | frame the whole design                                                                                                 |
| click a node                                                     | select it (the inspector follows)                                                                                      |
| ⇧-click                                                          | add to / remove from a multi-selection                                                                                 |
| ⇧-drag on empty canvas                                           | box-select                                                                                                             |
| drag a node body                                                 | move it; released, the position is saved (not a design change)                                                         |
| drag from an output socket to an input socket                    | make a link; while dragging, every socket that can accept it shows a halo, an incompatible socket the forbidden cursor |
| drag from a connected input socket away, release on empty canvas | disconnect                                                                                                             |
| drop a dragged link on empty canvas                              | nothing (no node is created)                                                                                           |
| ⌫ / Delete                                                       | delete the selection; a concept in use is refused with a banner naming its users                                       |
| double-click a concept or relationship node                      | rename in place (an output is renamed in the inspector; an instance opens its source)                                  |
| right-click                                                      | the context menu (below)                                                                                               |
| drag a row from the Library tab onto the canvas                  | insert that concept at the drop point; its name opens for editing                                                      |

<!-- figure F7: a link in mid-drag with the halo — pending, see SCREENSHOT_PLAN.md -->

## The context menu

On empty canvas: **Add Concept ▸** — _Recent_, _Input_, _Output_, the three most
common categories, _More…_ (which opens the Library tab); in a system project
also **Add Instance ▸** _component_ and **New Behavior Group**.

On a node: **Rename**, **Delete**; on a relationship in a system project also
**Group as Behavior**, **Add to Group ▸**, **Remove from …**; on an instance,
**Edit Source**; on a behavior, **Collapse** / **Expand**, **Package as Reusable
Component…**, **Ungroup**.

## What the canvas never shows

Draft formulas (a typed but unadded formula changes nothing on the canvas; the
status line counts it), values (those are on the Simulate page), and identifiers
or type names (those are in Explain).

## Related

[Workspace](workspace.md) · [Inspector](inspector.md) ·
[System projects](system-projects.md) ·
[Keyboard and mouse](../reference/keyboard-and-mouse.md)
