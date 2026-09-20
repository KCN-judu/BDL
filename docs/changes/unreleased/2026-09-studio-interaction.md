# Studio: one interaction state machine, CAD selection, Concept → Output, contextual menus

- Date: 2026-09-20
- Area: studio
- Affected: designers, developers
- Related: ADR-0003, ADR-0015, ADR-0034, docs/architecture/studio-ui.md §2

## What changed

- **One owner per pointer sequence.** The canvas is an explicit state machine
  (`node_canvas.dart`: idle, press candidate, marquee, pan, node drag, group
  drag, link drag, consumed-by-menu) over a raw pointer listener — no more three
  Flutter recognizers racing for one press. A press becomes a gesture only past
  a 4 pt threshold; a click is a click; double-clicks are timed, not delayed
  (single clicks no longer wait 300 ms for a second one).
- **Desktop CAD selection.** A plain drag on empty canvas is a rectangle
  selection — left → right a _window_ (wholly inside, solid outline), right →
  left a _crossing_ (inside or touched, dashed outline); ⌘ (Ctrl on Windows /
  Linux) adds the rectangle's nodes, ⇧ removes them, with the outcome previewed
  on the nodes before release. ⌘-click toggles a node; ⇧-click selects the one
  shortest chain of signature edges from the active node when exactly one
  exists. Pan is the middle button, Space + drag or a trackpad's two fingers;
  wheel, pinch and ⌘ + two fingers zoom. Dragging a selected node moves the
  selected set together, committed as one layout write (`NodesMoved`); ⌘A
  selects every visible node; arrows nudge; Esc cancels a gesture before it
  clears the selection. A selection has an _active_ object
  (`MultiSelected .active`), drawn with a second ring, named first by the
  inspector; a set keeps its surviving members when one is deleted. The Project
  list selects the same way (⌘ toggle, ⇧ range from the active row). Shortcuts
  fire only while the canvas has the keyboard — Backspace inside the inline
  rename no longer deletes the node.
- **Multi-delete.** Deleting a selected set is a checked plan of the model's own
  deletes (relationships, sinks, concepts, instances); a concept or sink still
  used from outside the set stops the plan before it starts, with the users
  named.
- **Concept → Output.** A concept's socket dropped on a sink accepting it
  connects the relationship that can drive the sink — read off the projection by
  the output pass's rule (a value or Source producing exactly the concept, in
  the sink's domain, not driving another sink): one connects at once, several
  are offered by name, none is explained at the pointer, a driven sink is
  offered a replacement as one plan (`ReplaceDriverRequested`: the old driver
  lets go, then the new connects). The sink's inspector offers the same list.
  `canLink`, DriveWF, SingleDriver, the kernel: unchanged.
- **Contextual menus** are a modal input state: the canvas takes no pointer,
  hover or wheel behind an open menu; the outside press dismisses it and does
  nothing else; a right-click elsewhere retargets; the chooser and the menu are
  never both open; a deleted target closes its menu; Esc closes a menu before it
  clears the selection; Control-click opens the menu on macOS; the first row has
  focus. The context is explicit (`MenuContext`: canvas, node, group, link,
  selection) and each context's commands are its own — a link no longer gets the
  blank-canvas menu, a selected set gets _Delete n objects_ and _Group as
  Behavior (n relationships)_, a Source has no _Edit Definition_, a sink names
  its driver. New commands: _Edit Definition_, _Reveal in Code_ (the Split view
  at the declaration), _Fix ▸_ (the IDE service's actions for the object —
  ready, a choice, blocked with its reason; a stale list is not offered),
  _Select All_, _Frame All_, _Show Driver_, _Show_ either end of a link,
  _Disconnect_ on a link.
- **One menu primitive** (`ui/mac/menus.dart`, the theme's `menuTheme` and
  `menuButtonTheme`): the content colour, a hairline, 24 pt rows in the body
  size, a shortcut column, a destructive row in the error colour, disabled at 40
  % with the reason — for the contextual menu, the chooser and every menu since.
- **Typography.** The type scale is `MacType` (body 13, secondary 11, caption
  10, node title 12.5, code 13); every literal size under `lib/ui` moved onto it
  (12 pt, never in the scale, became body or secondary), a test refuses new
  literals, the Library's section headers are sentence-case panel titles, its
  unit column is readable secondary, the Code pane's file name is a panel title,
  the status line is four clusters (document · design · deployment · compiler) a
  section gap apart.

## Compatibility and migration

- Designers: the gestures changed — a plain drag on empty canvas now selects,
  not pans (pan with the middle button, Space, or two fingers); ⇧-click no
  longer toggles (⌘-click does); ⇧-drag now subtracts. The guide's canvas page
  and the keyboard reference say so. Nothing in a project changes.
- Project files, protocol: nothing.
- Developers: `MultiSelected(nodes, {active})`, `selectedNodes`, `activeNode`,
  `selectionOfNodes`, `singleSelection`, `surviving`; `NodesMoved`,
  `RevealInCodeRequested`, `EditDefinitionRequested`, `ReplaceDriverRequested`;
  `driveCandidates`, `currentDriver`; `MarqueeMode`, `marqueeNodes`,
  `isAuthoringTarget`, `uniqueSignatureChain`; `MacType`,
  `primaryModifierIsControl`; `MacMenuAnchor`, `MacMenuItem`, `MacSubmenu`,
  `MacMenuDivider`. `_CanvasPainter` takes the selected set and the active node,
  not one selected node. The test harness's `rightClick` pumps a frame before
  the menu shows.

## Deferred

Align and distribute; a snap policy (none today — nodes move freely on an 8 pt
nudge); auto-pan at the viewport's edge during a long drag; zoom to selection;
duplicate and copy/paste; a lasso; keyboard navigation between nodes; a
whole-graph relayout; selection filters; _Find References_ from the canvas
(needs a name site the source anchors do not give); a compiler _Explain_ command
(no protocol request yet); a multi-delete transaction (today one revision per
step); reveal on the canvas of a row selected in the sidebar when the node is
off screen.

## Evidence

`apps/studio/test/canvas_selection_test.dart`, `canvas_menu_test.dart`,
`output_authoring_test.dart`, `multi_move_e2e_test.dart` (the real `bdld`),
`canvas_gestures_test.dart`, `typography_test.dart`, `outputs_test.dart`.
