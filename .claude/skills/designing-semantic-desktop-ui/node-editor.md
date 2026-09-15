# Node editor design for BDL

The canvas is a dependency graph typed by identity, not a dataflow program. Sources:
Blender's node editor conventions, `docs/STUDIO_UI.md` §2, `canvas_geometry.dart` and
`node_canvas.dart` (the current implementation), and the contract in `bdl-contract.md`.

## What the canvas is for

It answers, without selecting anything: what objects exist, what kind each is, which
identity each socket carries, what depends on what, what is still open, which region
(context, domain) each belongs to, and where the physical edge is. It does not answer:
what a formula says (a summary line at most), execution order, priority, or numeric
detail (until simulation/monitoring paint values onto it).

## Anatomy (fixed)

```
        ┌───────────────────────────┐
        │ ▸ title          state word│   header: category tint, 12.5 semibold, state word right
   ●────┤ Tilt                       │   input sockets, left edge, identity hue, labelled
        │                 Brightness ├───●   output socket, right edge
        │ f(θ) = …                   │   body: definition summary or nothing
        └───────────────────────────┘
```

- Node width fixed per kind (168 concept, 200 mapping); height from socket count.
- Header tint = category (low saturation). Never identity.
- Sockets: 5.5 pt radius, 10 pt hit radius; hue = identity; fill = bound, hollow =
  unbound; shape = representation (○ ◇ □; hollow ring when not chosen).
- Links: cubic Bézier, horizontal tangents, 2 px, identity hue; selected 3 px.
- Data flows left → right. Sources (undriven concepts, inputs) sit left; sinks (outputs)
  right; domain-free mappings between.

## Encodings on the canvas (see the channel table in `semantic-ui.md`)

| Fact | Encoding | Not this |
|---|---|---|
| identity | socket + link hue, name label | header colour, node colour |
| representation | socket shape | a word in the body |
| unbound representation | hollow socket + *open* | red, orange fill |
| undefined mapping | dashed outline + *declared* | greyed-out, red, a badge |
| dependency | link, concept → mapping input; mapping output → concept | arrowheads (direction is left→right by construction) |
| selected | accent outline 2 px (dashed stays dashed, in accent) | fill change |
| hover | outline in secondary text colour; socket 4 pt halo | scale, shadow lift |
| legal drop target (during drag) | stronger halo on the target; faint halo on *all* compatible sockets | nothing |
| illegal drop | blocked cursor, no halo, link retracts on release | error dialog |
| delay / memory | register mark on the link with the initial value | dashed link |
| domain | lane region (background tint, label) | hue |
| crossing | transport gate on the link at the lane edge | a badge on the reader |
| context | region enclosing members, activation socket on its edge | a separate diagram |
| output | distinct silhouette in the right column | a concept node with a different colour |
| ownership conflict | second link refused at the socket, with the driver named | diagnostic later |
| value (sim / live) | number at the socket, tabular, unit column | colour intensity |

## Gestures (Blender's, on a Mac)

| Gesture | Result |
|---|---|
| drag empty canvas | pan; ⇧-drag box-selects |
| scroll / pinch | zoom about the pointer (0.25–3×) |
| click node | select (active); ⇧-click extends; ⌘-click toggles |
| drag node body | move selection; commit on release (layout only, no revision) |
| drag output → input socket | link, if same identity; else refused at the pointer |
| drag connected input away, release on empty | disconnect |
| release a new link on empty | discard (never auto-create) |
| ⌘-drag across links | cut |
| ⌫ / Delete | delete selection; a concept in use is refused with the users named |
| ⌘A / Esc | select all / clear |
| Home / ⌘0 | frame all; ⌘F find by name |
| double-click header | rename inline |
| right-click | contextual menu: rename, attach definition, delete, explain |
| arrow keys | nudge selection 8 pt (⇧: 1 pt) |
| Tab / ⇧Tab | move active node in reading order (for keyboard users) |

## Rules

1. **Nothing moves on its own.** Auto-placement only for a node with no stored position,
   and then deterministically by id. Never re-layout, never auto-insert on a link.
2. **Constraint at the pointer.** Identity mismatch, second driver, instantaneous loop,
   crossing without transport: the link does not form, and the reason appears where the
   pointer is. The compiler still has the last word; the canvas merely refuses what it
   knows the compiler will refuse.
3. **The canvas shows structure and state; the inspector shows properties.** The formula
   is a one-line summary on the node and an editor in the inspector. Never a formula
   editor on the canvas; never a socket you can only see in the inspector.
4. **Two zoom levels must read.** At 0.25× the topology, regions and dashed/solid states
   must be visible (hide labels below a threshold, keep hue, shape, dash, regions). At
   1× everything.
5. **Selection never erases a state encoding.** Accent outline replaces the hairline's
   colour, not its dash pattern. A selected undefined node is dashed in accent.
6. **Sockets are labelled.** Identity hue never stands alone; the name is beside every
   mapping socket and is the concept node's title.
7. **Empty canvas is an entry point.** With no nodes: one line in tertiary at the centre
   naming the first action and its shortcut. With nodes but nothing selected: nothing.
8. **Regions are backgrounds, not boxes.** Domain lanes and context regions are tinted
   areas with a label at the top-left; members are inside by position, and moving a
   node across a boundary is a semantic edit that must be confirmed by the compiler
   (the node snaps back until the `EditOp` is accepted).
9. **Marks on links are small and few.** A register mark (memory), a gate (transport), a
   cut point (during ⌘-drag). Each has a datatip; each has its value in the inspector.
10. **Values are numbers.** When simulation or telemetry paint the canvas, a value sits
    at the socket in tabular figures with its unit; booleans fill or empty the ◇; a
    stale value fades to tertiary. Nothing animates a value.

## Anti-patterns specific to node editors

- A node per operator (arithmetic soup) — the canvas stops being readable as dependency.
- Rainbow headers — category hue steals the identity channel.
- Arrowheads on every link — noise; direction is the layout.
- Auto-layout buttons that reshuffle — destroys spatial memory.
- Nodes that grow with content — the module breaks; put content in the inspector.
- Icons in headers instead of words — the state word is the accessible cue.
- Minimap by default — only when the graph outgrows the window, and then optional.
