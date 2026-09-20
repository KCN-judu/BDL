# Canvas

The canvas is the Design page's centre: a picture of the design's structure.
Everything on it means something, and each visual channel means one thing only.

## What the marks mean

| You see                                                              | It means                                                                                                                                                                                                           |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **socket colour**                                                    | _which concept_ — every concept has its own hue, the same everywhere it appears                                                                                                                                    |
| **socket shape**                                                     | the concept's _value form_: ○ quantity, ◇ on / off, □ count; a **hollow ring** while the value form is _decide later_                                                                                              |
| **a link in a concept's colour, socket to socket**                   | the signature: _this relationship reads that concept_ (concept → relationship input), _this relationship produces that concept_ (relationship → concept), or _this value drives that output_ (relationship → sink) |
| **a thin grey link ending at a formula line**                        | a reference: _this formula names that relationship_ — a value applying a rule, or naming a value or a Source. Nothing can be dragged to or from it; the formula is edited in the inspector                         |
| **dashed outline**                                                   | _declared_: a relationship that reads something and has no formula; an output without a driver or without a domain                                                                                                 |
| **a bar down a node's left edge, an entry arrow, the word _Source_** | a **Source**: a value the environment provides — a relationship that reads nothing and has no formula. Nothing is missing                                                                                          |
| **a red mark at the formula line**                                   | the formula does not check — _wrong now_, never _not yet_                                                                                                                                                          |
| **the word _declared_, _contested_, _ill-formed_, _no domain_**      | the one state word a node may carry, only while that state holds (an output that must be driven says _required_ meanwhile; a Source says _Source_)                                                                 |
| **the word _rule_**                                                  | a relationship that reads something — a function; it has no value of its own until a value's formula applies it. Shown when no state word takes the slot                                                           |
| **a small name at a node's right edge**                              | its timing domain                                                                                                                                                                                                  |
| **the accent colour**                                                | selection — and nothing else                                                                                                                                                                                       |

Position, size and the direction of links carry **no meaning**. Data is drawn
left to right for readability. Coloured links show the signature and grey links
show what depends on what; neither shows the order in which anything runs, and
there is no node per operator — arithmetic lives in the formula field.

## The nodes

![Concept rows Tilt and Brightness with a round socket at each end; relationship nodes with a name header, one input socket per concept read on the left, one output socket on the right and the formula in the body; dimByTilt, with the word rule in its header, outlined in the accent colour because it is selected; the Source tilt with a bar at its left edge, an entry arrow and the word Source in its header and no input socket; brightness with no input socket and two thin grey links arriving at its formula line from the output sockets of tilt and dimByTilt; the light sink at the right with the word required in its header, a single input socket and a bar at its right edge; tilt, brightness and the output carry the domain name interaction at their right edge.](../assets/studio/node-anatomy.png)

_Node anatomy on the tilt lamp: concept rows, relationship nodes (dimByTilt
selected), and the light sink._

A **concept** is one row: its name, an input socket on the left (something
produces this concept) and an output socket on the right (relationships read it
from here). Both sockets carry the concept's colour and shape.

A **relationship** is a box: a header with the name and, while it applies, the
one state word (_declared_ on a relationship that reads something and has no
formula yet); one input socket per concept it reads, on the left, each labelled
with the concept; one output socket on the right, labelled with the concept it
produces; and the formula on the line below, with the timing domain's name at
the right edge when the relationship has one.

The box tells the three shapes apart without the formula. A relationship that
reads something — `dimByTilt` above — is a **rule**: it has input sockets and,
when nothing else needs the header's word, the word _rule_. It is a function;
the design computes nothing with it until a value's formula applies it. A
relationship that reads nothing and has a formula — `brightness` — is a
**value**: no input sockets, no word, and its formula names the values and rules
it depends on. Each of those is joined to it by a **reference link**: a thin
grey line from that relationship's output socket to the left end of
`brightness`'s formula line. Reference links are the design's dependency, read
off the compiler's analysis of the formula; they appear when the analysis
arrives and cannot be dragged — change the formula to change them. A rule that
no formula applies has no reference link leaving it into any formula line.

A **Source** is a relationship that reads nothing and has no formula — `tilt`
above: the environment provides its value, once per activation. It is drawn with
the word _Source_ in the header, an entry arrow before its name, a bar down its
**left** edge (the environment side; nothing in the design feeds it), one output
socket and no input socket. It is not dashed and not _declared_: nothing is
missing. What provides the value on a real product — a sensor, a button, an
analog line — is a deployment matter, not a mark on the canvas; give the Source
a formula and it becomes an ordinary relationship computed inside the design.

A **physical output** is a sink: a tinted header with its name and a word at the
right — _required_ for an output the design must drive before it is complete, or
the state that overrides it (_no domain_, _contested_, _ill-formed_); one input
socket labelled with the concept it accepts; the timing domain at the right
edge; and a bar down its right side — nothing flows out of it.

The selected node (`dimByTilt` above) is outlined in the accent colour; nothing
else on the canvas uses that colour.

A project that composes components also shows **instance nodes** (one row per
port, the component's name in the body) and **behavior regions** or collapsed
**behavior boxes**; see [System projects](system-projects.md).

Every node has a place. A node you did not place — one made in the Code view, in
a code editor, or by an older project's first open — is placed for you: in the
column of its kind (concepts left, relationships in the middle, outputs right),
beside what it reads or produces, below anything already there. Nothing you
placed moves; drag it where you like ([Design, Code and Split](code-view.md)).

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
common categories, _More…_ (which opens the Library tab); **Add Source ▸** —
_New Source…_ and the presets (_Temperature Input_, _Tilt Input_, …), each
opening the [Source sheet](library.md#sources), where you choose the concept the
Source provides — an existing one, or a new one made with it; **Add Instance ▸**
_component_ and **New Behavior Group**.

On a node: **Rename**, **Delete**; on a relationship also **Group as Behavior**,
**Add to Group ▸**, **Remove from …**; on an instance, **Edit Source**; on a
behavior, **Collapse** / **Expand**, **Package as Reusable Component…**,
**Ungroup**.

## What the canvas never shows

Draft formulas (a typed but unadded formula changes nothing on the canvas — nor
its reference links; the status line counts it), values (those are on the
Simulate page), a relationship's memory of its own last value (`delay` draws no
link), and identifiers or type names (those are in Explain).

## Related

[Workspace](workspace.md) · [Inspector](inspector.md) ·
[System projects](system-projects.md) ·
[Keyboard and mouse](../reference/keyboard-and-mouse.md)
