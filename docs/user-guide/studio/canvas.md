# Canvas

The canvas is the Design page's centre: a picture of the design's structure.
Everything on it means something, and each visual channel means one thing only.

## What the marks mean

| You see                                                               | It means                                                                                                                                                                                                                                            |
| --------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **socket colour**                                                     | _which concept_ — every concept has its own hue, the same everywhere it appears                                                                                                                                                                     |
| **socket shape**                                                      | the concept's _value form_: ○ quantity, ◇ on / off, □ count; a **hollow ring** while the value form is _decide later_                                                                                                                               |
| **a link in a concept's colour, socket to socket**                    | _this formula reads that block_ (block → the formula's input socket for it, in the read block's colour), _this formula defines that block_ (the short joint from a mapping block into its block), or _this block drives that output_ (block → sink) |
| **a hollow socket labelled `?` on a mapping block**                   | an open position in the formula: a block dropped there fills it                                                                                                                                                                                     |
| **dashed outline**                                                    | an output without a driver or without a domain                                                                                                                                                                                                      |
| **a bar down a block's left edge, an entry arrow, the word _Source_** | a **Source**: a block with no formula — the environment provides its value. Nothing is missing                                                                                                                                                      |
| **a red mark at the formula line**                                    | the formula does not check — _wrong now_, never _not yet_                                                                                                                                                                                           |
| **the word _contested_, _ill-formed_, _no domain_**                   | the one state word a node may carry, only while that state holds (an output that must be driven says _required_ meanwhile; a Source says _Source_)                                                                                                  |
| **a small name at a node's right edge**                               | its timing domain                                                                                                                                                                                                                                   |
| **the accent colour**                                                 | selection — and nothing else                                                                                                                                                                                                                        |

Position, size and the direction of links carry **no meaning**. Data is drawn
left to right for readability. A link shows that one block's formula names
another; it does not show the order in which anything runs, and there is no node
per operator — arithmetic lives in the formula field. Concepts and rules are not
on the canvas at all: they are templates, kept in the sidebar.

## The nodes

![The Source block tilt with a bar at its left edge, an entry arrow and the word Source in its header, no input socket and one round output socket labelled Tilt; a link from it into the mapping block dimByTilt, with one input socket on the left labelled tilt — the block its formula reads — an output socket on the right and the formula dimByTilt(tilt) on the line below with a chevron at its end; a short link from that output socket into the block brightness, outlined in the accent colour because it is selected, with an input socket on the left, the domain name interaction beside it and one output socket on the right labelled Brightness; the light sink at the right with the word required in its header, a single input socket and a bar at its right edge; tilt and the output carry the domain name interaction too.](../assets/studio/node-anatomy.png)

_Node anatomy on the tilt lamp: the Source tilt, the mapping block dimByTilt
with its read socket and its formula, the block brightness it defines
(selected), and the light sink._

The canvas draws **blocks** — _Sem blocks_, in the
[terminology](../reference/terminology.md): one per value of the design. A block
is a relationship that reads nothing — one value of a concept, updated once per
tick. It is a small box: a header with the name and, while it applies, the one
state word; a row with one output socket on the right, in the colour and shape
of its concept, labelled with the concept's name; the timing domain's name at
the row's left when the block has one. A block with a formula has that formula
beside it as a node of its own, its **mapping block**: a header naming the rule
the formula applies (_dimByTilt_ above) or, when it applies none, the word
_Formula_; one input socket on the left **per block the formula reads**,
labelled with that block's name; an output socket on the right joined to the
block it defines by a short link — the formula is the block's one definition,
and that joint is never rerouted; and the formula on the line below. The mapping
block sits to the left of its block, and moves with it when you drag the block
(drag the mapping block alone to move only it). It carries a small chevron at
its formula line: click it (or **Show Formula** in the menu) and the mapping
block unfolds to show the formula the way the
[Formula editor](formula-editor.md) draws it — a fraction, a branch, the units —
with its first finding beneath and **Edit formula**, which opens the inspector.
The unfolded formula is for reading: clicking it selects the block and changes
nothing. Which formulas are unfolded is not saved with the project. Clicking a
mapping block selects its block: one declaration, two nodes, one inspector.

![The mapping block dimByTilt with its read socket tilt, its formula line dimByTilt(tilt) and a downward chevron, extended below by a region showing dimByTilt applied to tilt and the link Edit formula; to its right, joined by a short link, the block brightness with its socket labelled Brightness.](../assets/studio/formula-unfolded.png)

_brightness's mapping block unfolded: the chevron at its formula line turned
down, and the saved formula dimByTilt(tilt) drawn inside it, with Edit formula
beneath; the block brightness it defines to its right._

A **concept** is the template a block is made from: its name, its value form,
its colour. It is not a node — it is in the sidebar, and every block of it wears
its colour on the output socket. A product may have as many blocks of one
concept as it has such values (`sensorA`, `sensorB`, `roomTemperature`, all of
_Temperature_); nothing counts them. A **rule** — a relationship that reads a
concept and produces another, `dimByTilt` in the lamp — is a template too: it is
applied inside a block's formula, as many times as there are blocks to produce,
and lives in the sidebar with its own inspector, not on the canvas. The block
that applies it names it in its formula (`dimByTilt(tilt)`) and its inspector
lists it under _Applies_.

Each block a formula **reads** is joined to the formula's mapping block by a
link, from the read block's output socket into the mapping block's input socket
for it, in the read block's colour. These links are the design's dependency,
read off the compiler's analysis of the formula; they appear when the analysis
arrives. To change one you change the formula — drop a block on the mapping
block (below), or **Disconnect** the link: the compiler replaces the name in the
formula by an open position `?`, which the mapping block then shows as a hollow
socket.

A **Source** is a block with no formula — `tilt` above: the environment provides
its value, once per activation. It is drawn with the word _Source_ in the
header, an entry arrow before its name, a bar down its **left** edge (the
environment side; nothing in the design feeds it), one output socket and no
input socket. It is not dashed: nothing is missing. What provides the value on a
real product — a sensor, a button, an analog line — is a deployment matter, not
a mark on the canvas; give the Source a formula and it becomes a value computed
inside the design.

A **physical output** is a sink: a tinted header with its name and a word at the
right — _required_ for an output the design must drive before it is complete, or
the state that overrides it (_no domain_, _contested_, _ill-formed_); one input
socket labelled with the concept it accepts; the timing domain at the right
edge; and a bar down its right side — nothing flows out of it.

The selected node (`brightness` above) is outlined in the accent colour; nothing
else on the canvas uses that colour.

A project that composes components also shows **instance nodes** (one row per
port, the component's name in the body) and **behavior regions** or collapsed
**behavior boxes**; see [System projects](system-projects.md).

Every node has a place. A project that has never been laid out — a hand-written
one, a template, a demo — opens **arranged**: a block comes after the blocks it
reads, left to right, in columns, nothing overlapping; the arrangement is saved
as the project's layout. A block you did not place in a project that has a
layout — one made in the Code view, in a code editor — is placed for you beside
what it reads, below anything already there. Nothing you placed moves; drag it
where you like ([Design, Code and Split](code-view.md)). To lay the whole design
out again, choose **Arrange Automatically** from the canvas menu; **Undo
Arrange** puts it back the way it was, until your next move. Arranging changes
the layout only — never the design.

## Selecting

The canvas selects the way desktop CAD tools do.

| Do                                          | Result                                                                                                                                                       |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| click a node                                | select it alone (the inspector follows); clicking a node that is already part of a selection keeps the selection and makes it the active one                 |
| click a link                                | select it: the inspector names its two ends and what the link means (_brightness drives light_, _brightness reads tilt_); a binding is selected the same way |
| click empty canvas                          | clear the selection                                                                                                                                          |
| ⌘-click (Ctrl-click on Windows and Linux)   | add a node to the selection, or remove one that is in it                                                                                                     |
| ⇧-click                                     | select the chain of connections from the active node to this one — when there is exactly one; otherwise the node is added on its own                         |
| drag on empty canvas from **left to right** | a **window**: every node wholly inside the rectangle is selected (solid outline)                                                                             |
| drag on empty canvas from **right to left** | a **crossing**: every node inside _or touched_ by the rectangle is selected (dashed outline); up or down makes no difference                                 |
| ⌘-drag / ⇧-drag a rectangle                 | add the rectangle's nodes to the selection / remove them from it (a `+` or `−` beside the pointer); the nodes show what will happen before you let go        |
| ⌘A                                          | select every node in view                                                                                                                                    |
| Esc                                         | cancel what is in progress — a rectangle, a move, a link; with nothing in progress, clear the selection                                                      |

The selected nodes are outlined in the accent colour; when several are selected
the _active_ one — the one you clicked last, the one the inspector shows first —
wears a second ring around it. A link under the pointer shows a soft glow and
the hand cursor; a selected link is drawn in the accent colour with a ring at
each end. The Project sidebar selects the same way: a click for one row, ⌘-click
to add or remove, ⇧-click for every row between the active one and it.

**The quick actions.** Point at the selected object — a link, a block, a
behavior — and a small row of icons appears beside it: **⋯** opens the same menu
a right-click does, and beside it the object's own action — **× Disconnect** on
a link that can be disconnected, **× Delete** on a block, **Collapse** /
**Expand** on a behavior. Hover over an icon for its name. The row stays while
you move from the object to the icons and goes when you leave them, change the
selection or open a menu. Nothing needs the mouse: ⌫ / Delete disconnects a
selected link, and the menu key (or ⇧F10) opens the selected object's menu.

## Moving and looking around

| Do                                                          | Result                                                                                                           |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| drag a selected node                                        | the whole selection moves together, keeping its spacing; released, the positions are saved (not a design change) |
| drag an unselected node                                     | it becomes the selection and moves                                                                               |
| drag a behavior's title band                                | its relationships move                                                                                           |
| ← → ↑ ↓                                                     | nudge the selection one grid step; with ⇧, one point                                                             |
| middle-button drag, Space + drag, two fingers on a trackpad | pan                                                                                                              |
| scroll wheel, pinch, ⌘ + two fingers                        | zoom about the pointer                                                                                           |
| Home, or ⌘0                                                 | frame the whole design                                                                                           |

## Connecting

| Do                                                                                      | Result                                                                                                                                                     |
| --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| drag a block's output socket onto a block with no formula                               | the block's formula becomes that name (`= tilt`): a text edit of the formula, which the compiler checks; a mapping block appears beside it                 |
| drag a block's output socket onto a hollow `?` socket, or onto a mapping block with one | fill that open position of the formula with the block's name (the compiler places it); every open position shows a halo while you drag                     |
| drag a block's output socket onto an **output** that accepts its concept                | make it the output's driver; while dragging, the sockets that can accept it show a halo, an incompatible socket the forbidden cursor                       |
| drag from a driven output's socket away, release on empty canvas                        | disconnect the driver — the same as **×**, **Disconnect** in the link's menu, the inspector's button, or ⌫ on the selected link                            |
| drag from a mapping block's input socket away, release on empty canvas                  | the same for a read link: the name becomes an open position `?` in the formula (the compiler edits it)                                                     |
| drop a dragged link on empty canvas                                                     | nothing (no node is created)                                                                                                                               |
| ⌫ / Delete                                                                              | delete the selection — several objects at once; if one of them is still used by something outside the selection, nothing is deleted and a banner says what |
| double-click a block or its mapping block                                               | rename the block in place (an output is renamed in the inspector; an instance opens its source)                                                            |
| right-click, or Control-click                                                           | the context menu (below)                                                                                                                                   |
| drag a row from the Library tab onto the canvas                                         | open the concept sheet for that category; the concept you name is created with a block of it at the drop point                                             |

**Driving an output.** An output is driven by a block that produces exactly what
the output accepts, in the output's timing domain — never by a rule, which has
no value of its own. Drag the block's output socket onto the output, or select
the output and choose the block in its inspector's _Connect_ pop-up, which names
the blocks that qualify. The edge on the canvas runs from the driving block, and
the output's context menu names it (_Show Driver: brightness_).

<!-- figure F7: a link in mid-drag with the halo — pending, see SCREENSHOT_PLAN.md -->

## The context menu

The menu is about what you right-clicked, and only that. While it is open the
canvas waits: nothing behind the menu moves, scrolls or lights up; the first
click outside closes it and does nothing else; a right-click somewhere else
moves it there. Use ↑ ↓ ⏎ and Esc as in any menu.

On **empty canvas**: **Add Block ▸** — _of Tilt_, _of Brightness_, … one entry
per concept of the design, each making a block of that concept where you clicked
(a Source until you give it a formula); then **New Concept ▸** — _Recent_, the
four kinds of value (_On / off_, _Count_, _Level_, _Decide later_), _Quantities
▸_ (_Angle_, _Length_, …), _More…_ (which opens the Library tab), each opening
the [concept sheet](library.md#creating-a-concept) where you name the concept,
which is created together with a block of it at the point; **Add Source ▸** —
_New source…_, opening the [Source sheet](library.md#sources), where you choose
the concept the Source provides — an existing one, or a new one made with it;
**Add Instance ▸** _component_ and **New Behavior Group**; then **Select All**
and **Frame All**; then **Arrange Automatically** and **Undo Arrange** (see
[The nodes](#the-nodes)).

On a **block** or its **mapping block** (one menu: they are one declaration):
**Edit Definition** (not for a Source — the environment provides its value),
**Show Formula** / **Hide Formula** (when it has one), **Rename**, **Reveal in
Code** (the Split view opens at its declaration); **Fix ▸** — the fixes the
compiler offers for it, as in the inspector's Fixes section: a ready fix runs,
one that needs a choice lists the choices, one the language cannot express yet
is shown greyed with the reason; then **Group as Behavior**, **Add to Group ▸**
or **Remove from …**; and **Delete _name_**.

On an **output**: **Show Driver: _name_**, **Rename**, **Reveal in Code**, **Fix
▸** (connect a value, disconnect a driver), **Delete _name_**. On a **link**:
**Show _one end_**, **Show _the other_**, **Disconnect** (and **Show Binding**
for a binding between components). A right-click selects the link first, as it
selects a node. **Disconnect** is offered where the design can lose that one
link: an output's driver, a binding, a read link (the name in the formula
becomes `?`). The short joint from a mapping block into its block is the formula
itself and has none — _Detach definition_ in the inspector takes the whole
formula away. On an **instance**: **Edit Source**, **Rename**, **Reveal in
Code**, **Delete _name_**. On a **behavior**: **Rename**, **Collapse** /
**Expand**, **Package as Reusable Component…**, **Ungroup**. A concept has no
node: its menu is in the sidebar.

On one of **several selected nodes**: the menu is about all of them — **Group as
Behavior (_n_ relationships)** and **Delete _n_ objects**.

## What the canvas never shows

Draft formulas (a typed but unadded formula changes nothing on the canvas — nor
its links; the status line counts it), values (those are on the Simulate page),
a relationship's memory of its own last value (`delay` draws no link), and
identifiers or type names (those are in Explain).

## Related

[Workspace](workspace.md) · [Inspector](inspector.md) ·
[System projects](system-projects.md) ·
[Keyboard and mouse](../reference/keyboard-and-mouse.md)
