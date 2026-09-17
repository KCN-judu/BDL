# Canvas

The canvas is the Design page's centre: a picture of the design's
structure. Everything on it means something, and each visual channel
means one thing only.

## What the marks mean

| You see | It means |
|---|---|
| **socket colour** | *which concept* — every concept has its own hue, the same everywhere it appears |
| **socket shape** | the concept's *value form*: ○ quantity, ◇ on / off, □ count; a **hollow ring** while the value form is *decide later* |
| **a link** | *this relationship reads that concept* (concept → relationship input), or *this value drives that output* (relationship → sink) |
| **dashed outline** | *declared*: a relationship without a formula; an output without a driver or without a domain |
| **a red mark at the formula line** | the formula does not check — *wrong now*, never *not yet* |
| **the word *declared*, *contested*, *ill-formed*, *no domain*** | the one state word a node may carry, only while that state holds |
| **a small name at a node's right edge** | its timing domain |
| **the accent colour** | selection — and nothing else |

Position, size and the direction of links carry **no meaning**. Data is
drawn left to right for readability; edges show dependency, not the order
in which anything runs; there is no node per operator — arithmetic lives
in the formula field.

## The nodes

```
   ●────[ Tilt ]────●                 concept: one row, an input socket (something
                                      produces this) and an output socket (values flow out)

        ┌────────────────────────┐
        │ dimByTilt      declared│    relationship: header with the name and the one
   ●────┤ Tilt                   │    state word; one input socket per concept read;
        │            Brightness ├──●  one output socket; the formula line below
        └────────────────────────┘

        ┃ Light Output          ┃     physical output: a sink at the right edge,
   ●────┃                       ┃     one input socket, a boundary bar
```

In a system project there are also **instance nodes** (one row per port,
the component's name in the body) and **behavior regions** or collapsed
**behavior boxes**; see [System projects](system-projects.md).

<!-- figure F6 -->

## Gestures

| Do | Result |
|---|---|
| drag empty canvas | pan |
| scroll wheel / pinch | zoom about the pointer |
| Home, or ⌘0 | frame the whole design |
| click a node | select it (the inspector follows) |
| ⇧-click | add to / remove from a multi-selection |
| ⇧-drag on empty canvas | box-select |
| drag a node body | move it; released, the position is saved (not a design change) |
| drag from an output socket to an input socket | make a link; while dragging, every socket that can accept it shows a halo, an incompatible socket the forbidden cursor |
| drag from a connected input socket away, release on empty canvas | disconnect |
| drop a dragged link on empty canvas | nothing (no node is created) |
| ⌫ / Delete | delete the selection; a concept in use is refused with a banner naming its users |
| double-click a concept or relationship node | rename in place (an output is renamed in the inspector; an instance opens its source) |
| right-click | the context menu (below) |
| drag a row from the Library tab onto the canvas | insert that concept at the drop point; its name opens for editing |

<!-- figure F7 -->

## The context menu

On empty canvas: **Add Concept ▸** — *Recent*, *Input*, *Output*, the three
most common categories, *More…* (which opens the Library tab); in a
system project also **Add Instance ▸** *component* and **New Behavior
Group**.

On a node: **Rename**, **Delete**; on a relationship in a system project
also **Group as Behavior**, **Add to Group ▸**, **Remove from …**; on an
instance, **Edit Source**; on a behavior, **Collapse** / **Expand**,
**Package as Reusable Component…**, **Ungroup**.

## What the canvas never shows

Draft formulas (a typed but unadded formula changes nothing on the
canvas; the status line counts it), values (those are on the Simulate
page), and identifiers or type names (those are in Explain).

## Related

[Workspace](workspace.md) · [Inspector](inspector.md) ·
[System projects](system-projects.md) ·
[Keyboard and mouse](../reference/keyboard-and-mouse.md)
