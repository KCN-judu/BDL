# Studio: edges as objects, the contextual affordance, the first-open arrangement and Arrange Automatically (protocol 0.29)

- Date: 2026-09-21
- Area: studio, daemon, layout
- Affected: designers, protocol clients, developers
- Related: ADR-0003, ADR-0023 §7, ADR-0034, ISS-0020,
  docs/architecture/studio-ui.md §2, docs/user-guide/studio/canvas.md

## What changed

- **An edge is an object.** Hovering a signature or drive edge says it is
  interactive — the hand cursor and a soft halo under the same stroke, no icon,
  no thicker line. A click selects it (`LinkSelected`, by its ends; a binding
  keeps `BindingSelected`), drawn in the accent with a ring at each end, and the
  inspector says what it joins and what it means (_brightness drives light_,
  _dimByTilt reads Tilt_, _tiltSensor produces Tilt_). The hit area is eight
  screen pixels from the stroke at any zoom (`linkTolerance`), the nearest edge
  wins where two are in reach, and hover, click and right-click ask the same
  question (`hitTest`).
- **The contextual affordance** (`CanvasAffordance`): a selected object that is
  hovered shows a compact row at a deterministic anchor — an edge: the midpoint;
  a node or group: its top-right corner — with the object's **menu icon** (the
  same contextual menu the right-click opens) and its own quick action: **×
  Disconnect** on an edge the model can take away, **× Delete _name_** on a
  node, **Collapse / Expand** on a group. Nothing shows on hover alone; the row
  stays while the pointer crosses from the object to an icon and goes when the
  pointer leaves both, when the selection changes, when a menu opens or a
  gesture starts. Every icon carries its name as tooltip and accessibility
  label; a destructive icon takes the error tone only while hovered.
- **One disconnect** (`DisconnectLinkRequested`): the link menu's _Disconnect_,
  the ×, the inspector's button, ⌫ / Delete on a selected edge and the drag-away
  all reach the model by the same path — a relationship's read
  (`UnlinkMappingInput`) or a sink's driver (`SetMappingDrive` to none). A
  produce edge and a collapsed group's aggregate edge have no ×, no _Disconnect_
  item and are refused by the reducer: the producer question is under formal
  audit (ISS-0020) and the projection is frozen as it is.
- **Keyboard parity:** the menu key (or ⇧F10) opens the selection's contextual
  menu where the affordance's icon would; ⌫ / Delete disconnects a selected
  edge.
- **The first-open policy** (`bdl-daemon` `place_on_open`, ADR-0023 §7): a
  project with no position anywhere opens **arranged as a whole**
  (`bdl-layout::arrange_with` — ranks by longest path over signature, drive,
  binding and reference edges, orders each column by barycentre, one column per
  rank, no overlap; collapsed groups as boxes, hidden members untouched) and the
  arrangement is saved as its layout; a project with some positions gets only
  its gaps filled (`place_missing`, nothing authored moves); a complete layout
  is preserved. `bdld init` goes through the same policy, so the Welcome page's
  demos and templates open arranged.
- **Arrange Automatically** on the canvas menu asks the daemon for the
  whole-graph arrangement (`ArrangeLayout` → `LayoutResponse`, protocol 0.29,
  answered and never applied by the daemon), applies it as one layout write, and
  frames the whole design once; **Undo Arrange** puts the layout back as it was
  before, until the designer's next move. Layout only: no edit, no revision, the
  viewport is the designer's.
- **The Pico demo fixture** (`docs/fixtures/pico-button-lamp/ui/layout.json`)
  pins the real first-open arrangement of the demo, so the guide's screenshot is
  what a tester's first open shows.

## Compatibility and migration

- Designers: an edge can be clicked; the produce edge's _Disconnect_ item (which
  did nothing) is gone; a project that never had a layout opens arranged instead
  of column-placed. Nothing in a project changes; `ui/layout.json` is written by
  the first open as before.
- Protocol clients: 0.29 adds `ArrangeLayoutRequest` (`arrange_layout = 81`) and
  `LayoutResponse` (`layout = 62`); additive, `compatible` stays true for 0.x
  clients.
- Developers: `LinkId`, `LinkSelected`, `LinkShape.id`, `linkTolerance`,
  `nearestLink`, `hitTest(linkHitTolerance:)`; `DisconnectLinkRequested`,
  `AutoLayoutRequested`, `RestoreLayoutRequested`, `ArrangedLayoutReceived`, the
  `ArrangeLayout` effect; `EditorState.layoutBefore`, `.frameRequest`;
  `NodeCanvas.frameRequest`, `.canUndoArrange`; `CanvasAffordance`,
  `AffordanceAction`, `AffordanceAlignment`; `nodeName`; `LinkInspector`;
  `bdl_layout::{arrange, arrange_with, References, has_positions}`,
  `Session::arranged_layout`. `_unlink` dispatches `DisconnectLinkRequested` for
  every non-binding edge (the aggregate-edge drive bug that dispatched a group
  id is gone with it).

## Deferred

Edge affordances on instances' port sockets (a binding's affordance is on the
binding); alignment of arranged columns to the 8 pt grid beyond the column step;
layout undo beyond one step (a layout history); the items ISS-0020 freezes.

## Evidence

`crates/bdl-layout/tests/arrange.rs`, `crates/bdl-daemon/tests/layout_e2e.rs`
(the real `bdld`: first open arranged and saved, a partial layout kept, the
arrangement answered and deterministic, no revision);
`apps/studio/test/canvas_affordance_test.dart` (hover, selection, the affordance
on edges and nodes, ×, the menu icon, the drag-away, invalidation, the hit test
at every zoom), `canvas_layout_reducer_test.dart` (the one disconnect, refusal,
arrange / undo / frame, the connection inspector), `canvas_menu_test.dart` (C).
