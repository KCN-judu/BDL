---
kind: architecture
area: studio
status: current
---

# Studio UI design

Screenshots of the built UI live in the user guide
(`docs/user-guide/SCREENSHOT_PLAN.md`, captured by `just docs-shots`); the
monospace sketches on this page are the design intent the UI was built from and
are kept as such — each is followed by a link to what was built.

Three references, each used for one thing:

| Reference               | What we take from it                                                                                                                                                                                                                                 | What we do not take                                                   |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| **DaVinci Resolve**     | one window, one workflow, _pages_ in workflow order on a bottom page bar that is always available; a status line above it; each page owns a fixed panel arrangement (Fusion: node editor centre, effects library left, inspector right, viewers top) | its dark-only look, its density, timeline metaphors                   |
| **Blender node editor** | node anatomy (header, collapse, coloured sockets left-in / right-out), links as curves dragged socket-to-socket, drop-on-empty-space discards, cut links by drawing across them, box select, active-vs-selected, data flows left→right               | its modal keymap as a _requirement_ (G/R/S), its dozens of node types |
| **macOS (Apple HIG)**   | system font, 13 pt body / 11 pt secondary, flat controls, sidebar–content–inspector, toolbar in the title area, translucent panels, accent colour only for selection and the primary action, no ripples, native menus and ⌘ shortcuts                | —                                                                     |

Sources: DaVinci Resolve 18 Reference Manual ch. 1 ("Switching Among Pages",
"The Fusion Page": Node Editor, Inspector, Effects Library, Status Bar); Blender
Manual, _Interface › Controls › Nodes › Parts / Editing / Selecting_; Apple HIG,
_Typography_ (macOS text styles) and _Designing for macOS_.

## 0. Two screens (DaVinci)

Resolve opens on the **Project Manager** and only shows the page workspace once
a project is open. Studio does the same: with no project, the window is the
launcher below; opening or creating a project replaces it with the workspace of
§1; ⊞ in the page bar closes the project and returns.

_As built:
[project manager](../user-guide/assets/getting-started/project-manager.png) (the
user guide's screenshot; the sketch below is the design intent it was built
from)._

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│    ┌──────────────────────────┐    Recent                                │
│    │ ╲  ╱   ╲    ╱  ╲   BDL   │    ▸ lamp     ~/Projects/lamp   2 h ago  │
│    │  ╲╱  ╱  ╲  ╱    ╲        │    ▸ rover    ~/Projects/rover  yesterday│
│    │ Behavior                 │    ▸ cup      /Volumes/old/cup  not found│
│    │ Designer                 │                                          │
│    │ Product behavior as a    │                                          │
│    │ design material.  0.1.0  │                                          │
│    └──────────────────────────┘                                          │
│    Start                                                                 │
│    ⊕ New Project…  ⌘N                                                   │
│    ▭ Open Project… ⌘O                                                   │
├──────────────────────────────────────────────────────────────────────────┤
│ no project                                        bdld 0.1.0 · protocol  │
└──────────────────────────────────────────────────────────────────────────┘
```

Laid out like VS Code's welcome page (Start on the left, Recent on the right)
rather than Resolve's thumbnail grid — BDL projects have no thumbnail yet. There
is one _New Project…_ (ADR-0023): a project is sources under `src/` and a
behaviour system, whatever it is later shown as; nothing asks for a kind.
**Hero**: the wordmark _Behavior Designer_ set in Chakra Petch (a square sans
with 45° chamfered corners, SIL OFL, bundled) over a field of 45°-routed traces
drawn deterministically — PCB routing is the one visual idiom that belongs to
both halves of the product. Recent projects are an app preference (`recent.json`
in the per-user application support directory), never project data; missing
directories are shown greyed with _not found_ and can be removed on hover.

## 1. Window structure (DaVinci)

_As built: [workspace](../user-guide/assets/studio/workspace.png)._

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ ● ● ●   lamp •                      [⌘Z ⌘⇧Z]   [Save]      [Build] [Run] │  toolbar (title area)
├───────────────┬──────────────────────────────────────┬───────────────────┤
│ Library       │                                      │ Inspector         │
│  Concepts     │            page content              │  (selected object)│
│  Mappings     │                                      │                   │
│  Timing dom.  │                                      │                   │
│  Outputs      │                                      │                   │
├───────────────┴──────────────────────────────────────┴───────────────────┤
│ r12 · 2 concepts · 1 mapping · dimByTilt: declared            bdld 0.1.0 │  status line
├──────────────────────────────────────────────────────────────────────────┤
│ [⊞]        Design      Simulate      Deploy      Monitor            [⚙] │  page bar
└──────────────────────────────────────────────────────────────────────────┘
```

Pages, in workflow order (the design's own sequence: intent → refine → time →
contexts → outputs → domains → board → observe):

| Page         | Centre                                                                                                                                                                                          | Left                                                                                                                                                           | Right                                                       | Answers                    |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- | -------------------------- |
| **Design**   | the project as **Design** (node canvas), **Code** (its source files) or **Split** (both) — §12                                                                                                  | sidebar: _Project_ tab (concepts, mappings, timing domains, outputs; _Contexts_ and _Components_ are empty headings) and _Library_ tab (concept templates, §2) | inspector of the selection                                  | what the product does      |
| **Simulate** | readiness blockers with _Show_ links, Step / Step ×10 / Reset, the trace as a table (tick, active domains, one column per relationship without inputs and per driven output); value plots later | inputs as controls by value form; a period per timing domain                                                                                                   | probe of the selection: value now and over the run, Explain | what it does over time     |
| **Deploy**   | one verdict _for this board_, the placement device → requirement → pin, the dead end in the solver's terms; a board picture later                                                               | boards from bdld; devices, edited in place                                                                                                                     | —                                                           | whether it fits            |
| **Monitor**  | _spec_: the same canvas with live values; a placeholder page (`placeholder_page.dart`) until telemetry exists (roadmap priority 4)                                                              | telemetry sessions                                                                                                                                             | probe inspector                                             | what it is doing right now |

Resolve's page bar can hide labels (icons only) and hide pages; we keep labels
by default and allow ⌘1–⌘4. The page bar's left button is the project manager
(open/new/recent), the right is project settings — as in Resolve. Nothing on the
page bar is a compiler badge.

The **status line** (Resolve's Fusion status bar) leads with the document state
(_Saved_ / _Edited_), then a count summary (concepts, mappings, how many are not
yet defined, how many definitions do not check) and the compiler connection
(_Compiler 0.1.0_; the protocol version only when it mismatches). The revision
counter is not a designer fact and lives in Explain. Errors from requests appear
as a banner above the page content, never as a modal.

## 2. Node canvas (Blender)

### Anatomy

_As built: [node anatomy](../user-guide/assets/studio/node-anatomy.png); the
header word precedence is `Source` › `declared` › port word › sink state ›
`required` › `rule`._

```text
   ●────[ Tilt ]────●                       concept: one row — name, in-socket, out-socket

        ┌─────────────────────────┐
        │ dimByTilt        declared│   header: title · the one state word (only while declared)
   ●────┤ Tilt                     │   input socket per read concept (left), hue = identity
   ◆────┤ Held          Brightness ├────●   output socket (right)
        │ ● clamp(0.2 + 0.8·θ/60°) │   definition region: summary line; red mark = does not check
        └─────────────────────────┘
```

- **A relationship without inputs has no input socket.** Its canonical type is
  `() -> B` (ADR-0029); the unit domain `()` is a type-theoretic normalization,
  not a design input, so the node shows one output socket and nothing on the
  left — the Explain disclosure says `type: () -> Brightness` and that the
  domain is the empty product. Wherever "read as a value" matters (simulation
  inputs and columns, output drivers, the value/rule word) Studio reads the
  **role the daemon states** on every `MappingView` (`role`, protocol 0.20;
  `relationshipRole` in `app/state.dart` only maps the enum), never the
  signature's shape and never a formula: the canonical matrix is
  `docs/architecture/relationship-roles.md`.
- **A Source is a role, not a node kind** (ADR-0032). A relationship the daemon
  states as a Source — the unit domain and no realization — that backs no port
  of the open component is drawn as a Source: the word _Source_ in the header,
  an entry glyph (an arrow crossing a boundary tick) before the title, a solid 3
  px bar on the node's **left** edge — the environment side, the mirror of the
  sink's bar on the right — and a green header strip as the redundant colour
  cue. It is never dashed: nothing is missing. A base relationship a binding
  realises arrives from the system view already as a Value (its flattened copy
  carries the binding); a port-backed Source of an open component wears the
  port's word (`SystemSceneInput.portWords`) — its provider is the port, a fact
  beside the role. The painter reads `NodeShape.source`. The _realise_ socket of
  an open base relationship (§11) is added only while something could bind to it
  — the system has an instance, or another relationship produces the same
  concept — so a Source in a design with neither keeps its left edge clear.
  Assistive technology hears _name, Source: a value entering the behavior model
  from the environment, provides C_.

```text
        ┌─────────────────────────┐
       ▐│ ⇥ tilt           Source │   Source: boundary bar (left), entry glyph, the role word
       ▐│                    Tilt ├────●   one output socket; no input socket, never a `()` port
        └─────────────────────────┘
```

- **Header colour** = category: concept (grey-blue, the whole one-row object),
  mapping (blue strip), context (violet), output (amber), transport (teal).
  Muted, low-saturation, with the title in 12.5 pt semibold. Identity hues are
  the only saturated marks on the canvas.
- **Socket colour = semantic identity.** In Blender a socket's colour is its
  data type; in BDL the type that matters is the nominal concept, so each
  concept gets a stable hue derived from its `SemanticId` (deterministic, never
  from the name; lightness chosen per hue so every identity clears 3:1 against
  the canvas in both appearances). A link is only accepted between sockets of
  the same concept — the canvas shows the nominal typing rule without a
  diagnostic: while a link is dragged every compatible socket gains a faint
  halo, the one under the pointer a strong halo, and an incompatible socket
  shows the forbidden cursor.
- **Socket shape = value form.** Quantity ○, on–off ◇, count □; a concept whose
  value form is not chosen yet is a hollow ring. The same glyph, drawn by the
  same code, appears in the library, in chips, toggles and pop-ups. No type
  words are written beside a socket anywhere.
- **Concept node**: a single row — the name, an input socket on the left (a
  mapping producing this concept connects here) and an output socket on the
  right (values of this concept flow out). Nothing else: what it measures and
  what it means are the inspector's.
- **Mapping node**: one input socket per read concept (labelled), one output
  socket, and a definition region below the sockets. The region holds the
  definition's summary line, or nothing while declared. A definition the
  compiler cannot accept gets a **red mark at the definition line** — where the
  problem lives — and no word in the header. A defined mapping's region ends in
  a **disclosure** (a chevron at the definition line): open, the node extends
  downward with the formula **unfolded** — the same rendering the Formula view
  draws (`ui/expanded_formula.dart` over `FormulaRender`, dense), read-only,
  from the daemon's projection of the committed definition
  (`GetFormulaProjection` per revision, `EditorState.formulaPreviews`), with the
  first finding beneath it and _Edit Formula_ to open the inspector; while the
  projection is on its way, or when the compiler cannot draw it, the region says
  so in one line. Which nodes are open and how tall each is (measured, capped at
  220 px) is `EditorState.expandedFormulas` — view state like a collapsed
  group's box, never a revision and never saved; the scene
  (`buildScene(expanded:)`, `NodeShape.formulaRegion`) grows the node and moves
  nothing else. A tap on the unfolded formula selects the node and nothing more;
  no click on the canvas rewrites a formula (ADR-0042). A declared mapping — one
  with reads and no definition — is drawn with a dashed outline and the word
  _declared_; dashed survives selection (accent changes the colour, never the
  meaning). Never red. A relationship with no reads and no definition is a
  Source (above), not declared.
- **Links** are cubic Béziers from an output socket (right edge) to an input
  socket (left edge), tangents horizontal, colour of the concept, 2 px, selected
  links thicker. Data flows left → right. These are the **signature edges**:
  concept → a relationship that reads it, relationship → the concept it
  produces, value → the sink it drives — the interface, and what dragging edits.
- **Reference edges** (ADR-0034) are the kernel's `dependsOn`: from the output
  socket of every relationship a definition names into the **formula line** of
  the relationship naming it — the left end of its definition region, not a
  socket — in the secondary text colour, 1.5 px, under the signature edges, with
  no hit area, halo or selection. They come from `MappingAnalysis.references`
  (`buildScene(refs:)`, `LinkShape.reference`), never from the formula text;
  without an analysis of the revision there are none, and an edge whose end the
  projection does not show (a deleted relationship, an instance's private one)
  is dropped. A self-reference draws nothing (the register mark, §7, is not
  built). A hidden member of a collapsed group adds none of its own; an edge
  from a hidden member leaves from the group's aggregate output socket.
- **Rule, value, Source** are the daemon's three roles, told apart without the
  formula: a relationship that reads something is a **rule** — its input sockets
  are the shape and the header says _rule_ when no state word takes the slot
  (`NodeShape.rule`); one that reads nothing and has a realization is a
  **value** with no word; one that reads nothing and has none is a Source
  (above). _Declared_ (dashed) is a state of a rule only — the one hole a
  designer fills; _not applied_ (a hollow output socket) is the compiler's
  `reactive.rule_unapplied`, a state of a rule; _does not check_ is a state of a
  rule or a value. Assistive technology hears _name, rule, reads …, produces …,
  depends on …, applied by nothing_ / _name, value, produces …_ / for a
  port-backed relationship of an open component the port's word.
- **Empty canvas**: one tertiary line naming the first step (add a concept from
  the Library). Painted nodes expose semantics in product language for assistive
  technology.

### Interaction — desktop CAD selection over a node editor

The canvas is one interaction state machine (`ui/canvas/node_canvas.dart`):
every pointer sequence has exactly one owner, decided once — a press is a
_candidate_ until it moves 4 pt (`kDragThreshold`), then it becomes a marquee, a
node move, a group move, a link drag or a pan, and nothing else can claim it; a
press that never moves is a click. The selection algebra is the one desktop CAD
and EDA tools share (AutoCAD and Altium for the directional rectangle; KiCad and
Blender's node editor for the rest); the node anatomy above and the drop rules
stay Blender's. Blender's own selection keymap (⇧-click adds, drag on empty
space pans, B for box select) was the first version and is replaced: it made a
plain drag ambiguous between pan and select, needed a modifier to start a
rectangle, and used ⇧ where every other desktop tool uses ⌘.

| Gesture                                                         | Result                                                                                                                                                                                                  |
| --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| click node                                                      | select it alone; on a member of the selected set: keep the set, make it the active object                                                                                                               |
| click empty canvas                                              | clear the selection                                                                                                                                                                                     |
| ⌘-click (Ctrl on Windows / Linux)                               | toggle the node in the set; the node toggled in becomes active                                                                                                                                          |
| ⇧-click node                                                    | the one shortest chain of signature edges from the active object to it, when exactly one exists; otherwise the node alone is added (never a guessed branch; reference edges never count)                |
| drag on empty canvas, left → right                              | **window** marquee: nodes wholly inside are selected — solid outline, restrained fill                                                                                                                   |
| drag on empty canvas, right → left                              | **crossing** marquee: nodes inside or touched — dashed outline, fainter fill; the vertical direction means nothing                                                                                      |
| ⌘-marquee / ⇧-marquee                                           | union into / subtraction from the selected set (a `+` / `−` at the moving corner); what the release will do is previewed on the nodes before the button comes up                                        |
| drag a selected node                                            | the selected movable set moves together, offsets kept, committed on release as one layout write (`NodesMoved`, no revision); a lone relationship dropped into or out of a region changes its membership |
| drag an unselected node                                         | it becomes the selection (⌘: joins it) and moves                                                                                                                                                        |
| drag a group's title band                                       | its members move                                                                                                                                                                                        |
| middle-button drag · Space + drag · trackpad two-finger scroll  | pan                                                                                                                                                                                                     |
| mouse wheel · pinch · ⌘ + trackpad scroll                       | zoom about the pointer (0.25–3×)                                                                                                                                                                        |
| drag from output socket to input socket                         | make link (adds the concept to the mapping's reads, sets the mapping's output, drives the sink); every socket the link could land on wears a halo, an incompatible one the forbidden cursor             |
| drag a **concept**'s value socket onto a **sink** accepting it  | Concept → Output (below): the relationship that can drive the sink connects                                                                                                                             |
| drag from a connected input socket away, release on empty space | disconnect                                                                                                                                                                                              |
| drop a new link on empty space                                  | discard (no auto-create)                                                                                                                                                                                |
| ⌫ / Delete                                                      | delete the selection — a set as one checked plan (below); a concept in use → banner: "_Tilt_ is used by _dimByTilt_"                                                                                    |
| ⌘A                                                              | select every visible node of the canvas on screen (a collapsed group is its box; hidden members, links and reference edges are not nodes)                                                               |
| Esc                                                             | the innermost thing first: a submenu, a menu, a gesture in progress (nothing is committed), then the selection                                                                                          |
| ← → ↑ ↓                                                         | nudge the selected set one grid step (8 pt); ⇧: one point                                                                                                                                               |
| Home / ⌘0                                                       | frame all                                                                                                                                                                                               |
| double-click node header                                        | rename inline (an instance: open its source)                                                                                                                                                            |
| right-click · Control-click (macOS)                             | the contextual menu for what is under the pointer (below)                                                                                                                                               |
| drop a Library row                                              | a Concept item: insert the concept at the drop point; a Source preset: open the Source sheet                                                                                                            |

Cursors: precise over a socket and during a marquee; forbidden over a socket the
link cannot reach; move while nodes are dragged; grab while Space is held,
grabbing while panning; the arrow otherwise. Shortcuts fire only while the
canvas itself has the keyboard: a text field inside it (the inline rename) or
beside it (the Code view, the formula editor, a sheet) keeps every key.

**Selected and active.** A selection is a _set_ of canvas nodes and, among them,
the _active_ object (`MultiSelected.nodes`, `.active`; a single selection is its
own active object): the one the last plain or ⌘-click named, the anchor of a ⇧
range or chain, what the inspector shows first. It is named, never the first of
the set. The set wears the accent outline on every member; the active one also a
second, outer ring (the platform's focus-ring shape — a shape, not a hue). A
selection is editor state: it never dirties the project, never enters Undo, and
survives a new projection, an analysis, hover and viewport changes; a member
that ceases to exist leaves the set, the rest stays (`surviving`).

**The Project list** follows the same algebra: a plain click selects one row; ⌘
/ Ctrl toggles it; ⇧ selects the contiguous range from the active object (the
anchor) to the clicked row, in the list's own order; ⌘⇧ adds the range. The
active row is semibold; every selected row wears the selection tint.

**Multi-delete.** Deleting a selected set is one plan of the model's own
deletes, in the order it accepts — relationships, sinks, concepts, instances —
checked before its first edit is sent: a concept still used by a relationship
outside the set, or a sink still driven from outside it or realised by a device,
would be refused half-way and leave a partial design, so nothing is sent and the
banner names the objects and their users (_Nothing was deleted: Tilt is used by
dimByTilt_). The plan is then one queued edit per step, each a revision (Undo
steps back one at a time — the model has no multi-delete transaction; recorded
as a follow-up); a refusal ends the plan, as every queued plan does. A lone
group deletes as an ungroup (its relationships stay); a group among other
objects is left alone.

**Concept → Output.** A sink is driven by a relationship (`SetMappingDrive`,
DriveWF, SingleDriver), never by a concept — and the designer reaches for the
concept, not for whichever relationship happens to produce it. So a concept's
value socket dropped on a sink that accepts the concept is an _authoring
gesture_ over the model's own edit: the candidates are read off the projection
by the pass's rule (`driveCandidates`: a value or a Source — never a rule, whose
type is an arrow — producing exactly the accepted concept, updating in the
sink's domain when it has one, not the final target of another sink), the same
list the sink's inspector offers under _Connect_. One candidate connects at
once. Several are offered by name in the chooser (_Drive with brightness_ /
_Drive with dimmer_); nothing is chosen for the designer. None: the chooser says
so where the pointer is (_Servo accepts ServoPosition, but no current
relationship can drive it_) and offers the sink in the inspector; nothing is
synthesized. A driven sink is offered a _replacement_ (_Replace lifted with
rest_), one plan in which the current driver lets go before the next connects
(`ReplaceDriverRequested`) — the sink never has two drivers in between, and the
direct Mapping → Output drag keeps today's semantics (a second drive is a
conflict the output pass reports). `canLink` is unchanged: the concept never
links the sink; `isAuthoringTarget` names the gesture, and the sink's socket
wears the halo while the concept is dragged. The projection is unchanged too:
the drive edge is drawn from the driver's own output socket (the true
SingleDriver identity), the producer edge from the driver to its concept — both
true; routing the drive through the concept would say the concept drives, which
is false. Its inspector, its contextual menu (_Show Driver: brightness_) and the
socket's halo make the driver legible from the concept's side.

### Contextual menus

A contextual menu is a **modal input state** of the canvas: while one is open,
the canvas takes no pointer, no hover, no wheel — nodes under the menu do not
react, the canvas neither pans nor zooms — and the first outside press dismisses
the menu and does nothing else (the same press never selects, pans, moves or
draws). A right-click elsewhere while a menu is open retargets it: the old menu
closes, the new context is set, the items are rebuilt from it in the next frame
and the menu opens there — never the last target's items at a new place. The
chooser (aggregate sockets, Concept → Output) is the same kind of overlay and
the two are never both open. A menu whose target is deleted, or whose canvas is
left, closes. Esc closes a submenu, then the menu, before it touches anything
else. Menus are keyboard menus: the first row has focus when the menu opens, ↑ ↓
move, → opens a submenu, ⏎ runs the row.

The context is one explicit thing (`MenuContext`): the empty canvas at a point,
a node, an expanded group's band, a link, or the selected set (a right-click on
one of several selected nodes keeps the set and asks about all of them; on an
unselected object, that object is selected first; on blank canvas the selection
stays as it is and the menu is about the canvas alone). Each context's items are
stable groups in a fixed order — the object's primary command, IDE navigation,
the service's fixes, structure, the destructive command — and each item is
present only when its command exists for that object today:

| Context                          | Items                                                                                                                                                                                                                                                     |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| empty canvas                     | Add Concept ▸ (Recent, the four value forms, Quantities ▸, a third-party library's groups, More…) · Add Source ▸ (New Source…, a third-party library's presets) · Add Instance ▸ (system canvas) · New Behavior Group — then Select All ⌘A · Frame All ⌘0 |
| relationship — a rule or a value | Edit Definition · Show Formula / Hide Formula (when defined) · Rename · Reveal in Code — Fix ▸ — Group as Behavior · Add to Group ▸ / Remove from … — Delete _name_                                                                                       |
| relationship — a Source          | Rename · Reveal in Code — Fix ▸ — the group commands — Delete _name_ (no definition to edit: the environment provides it; a port-backed relationship of an open component has no Delete — it goes with its port)                                          |
| concept                          | Rename · Reveal in Code — Fix ▸ — Delete _name_                                                                                                                                                                                                           |
| sink                             | Show Driver: _name_ · Rename · Reveal in Code — Fix ▸ (the output pass's actions: connect a value, disconnect a claimant, the blocked sync) — Delete _name_                                                                                               |
| instance                         | Edit Source · Rename · Reveal in Code — Delete _name_                                                                                                                                                                                                     |
| group (band or collapsed box)    | Rename · Collapse / Expand · Package as Reusable Component… — Ungroup (a group's destructive delete is the named action in its inspector)                                                                                                                 |
| link (signature edge or binding) | Show Binding (a binding) · Show _from_ · Show _to_ — Disconnect                                                                                                                                                                                           |
| the selected set                 | Group as Behavior (_n_ relationships) — Delete _n_ objects (no single-object command on a set)                                                                                                                                                            |

**Fix ▸** is the IDE service's own list for the object — the one the reducer
asks for on every selection (`ListSemanticActions`, `EditorState.actions`) and
the inspector's _Fixes_ section shows: a _ready_ action is a command, one that
_needs a choice_ is a submenu of the service's options, a _blocked_ one is
disabled with its reason beneath it; the titles are the service's. The list is
offered only when it is about this object at this revision; a stale list is not
a menu, and `SemanticActionApplied` refuses a stale one anyway. **Reveal in
Code** selects the object, shows the Split view when the canvas alone was on
show, and opens the file declaring it at its declaration through the Code view's
own reveal (`RevealInCodeRequested` → the same `SourceReveal` definition
navigation uses; the anchors of the sources). **Edit Definition** selects the
relationship and gives the inspector's definition editor the keyboard. Not
offered, because no service Studio has today backs them: _Find References_ (the
daemon's `references_at` needs a name site, and a source anchor gives the item's
range, not its name — a name-based search is not acceptable), _Explain_ as a
compiler command (`bdl_ide::explain` has no protocol request; the inspector's
disclosure is Studio's own), alignment and distribution.

The menu itself is `ui/mac/menus.dart` — `MacMenuAnchor`, `MacMenuItem` (label,
shortcut column, an optional detail line, destructive in the error colour,
disabled at 40 %), `MacSubmenu`, `MacMenuDivider` — over Flutter's `MenuAnchor`
family, which owns the overlay, focus traversal and roles; the look is the
theme's (`menuTheme`, `menuButtonTheme`: the content colour, a hairline, 24 pt
rows in the body size, the accent tint on the focused row). Nothing styles a
menu anywhere else.

Dragging a node over a link does **not** auto-insert it (Blender's auto attach);
BDL links are typed by concept, and silent insertion would be a semantic edit.
Muting nodes (M) has no BDL meaning and is not offered. Deferred, recorded in
`docs/changes/unreleased/2026-09-studio-interaction.md`: align and distribute, a
snap policy, auto-pan at the viewport's edge during a long drag, zoom to
selection, duplicate and copy/paste, a lasso, keyboard navigation between nodes,
a whole-graph relayout, selection filters, Find References, a multi-delete
transaction.

### What the canvas never means

Signature edges are the interface and reference edges are dependency; neither is
execution order. Drawing order does not set output priority. Node position is
layout only (ADR-0003): the canvas draws every node where the layout puts it and
arranges nothing itself — an entity without a position is placed by the daemon's
layout service on open and on commit (ADR-0023 §7,
`docs/architecture/overview.md`), and the projection carries the result. No node
type exists per arithmetic operator — formulas live in the inspector.

### The concept sheet (library items)

A Standard Library item is a **value category** — a value form or a named
physical quantity — and never a product concept (ADR-0041,
`docs/spec/concept-library.md`): choosing one anywhere — the right-click menu, a
drag from the Library tab, a row's double-click or Return, the Project tab's `+`
— dispatches `NewConceptRequested(presetId?, position?)` and opens the **concept
sheet** (`ui/concept_sheet.dart`, a `SheetScrim` sheet like the Source sheet's)
with the category preset. Its one required answer is the **name** the concept
has in this product — _Angle_ → `LidAngle` — an identifier, free in the design,
refused on the sheet before the daemon would refuse it; then the category (one
pop-up over the same items), _Measured in_, the meaning, a live preview of the
node and the declaration the Code view will write. _Create_ dispatches
`CreateConceptRequested` — one `CreateConcept` edit; when the projection arrives
the concept lands where the pointer was (layout, never a revision), selected and
named as typed: nothing opens for renaming, because the name came first. Cancel
dispatches `ConceptSheetDismissed`. Create-then- rename
(`InstantiateLibraryItem` with the item's default name, then an inline rename)
is no longer a concept flow; the code path stays for a third-party item a client
inserts without a name.

**_Measured in_ is a fact, not a choice.** A concept stores its dimension and no
unit, so the row lists the compiler's unit candidates for the category
(`AppState.valueCategories`, `ListValueCategories` at the handshake;
`UnitExprView.display`, preferred first — `rad`, `deg`, `turn`; `rad/s`,
`deg/s`) as what a formula over the concept may write, and sends none of them.
Studio concatenates no unit string and infers no dimension: a composite is the
compiler's rendering. A per-concept display unit is ISS-0019.

The sidebar has two tabs: **Project** (the project's objects, "+" per section —
the concepts' opens the sheet with no category, _choose a category_) and
**Library** (`ui/library_panel.dart`: search, then _Recent_, **Values** (the
four forms), **Quantities** (the vocabulary's order), a **Sources** row and one
section per third-party library served; a category row carries a grey socket
glyph, filled when the category is a value form, hollow for _decide later_; the
vocabulary's preferred unit or the form's word in the right column — a fact; the
description on hover; the _Sources_ row the Source silhouette
(`MappingGlyph(source:)`) and what it does — _a value the environment provides,
over a concept you choose_). Recent items in the menu are a Studio preference,
never project state.

Item names, descriptions and search tags are Studio strings like any other:
generated by id from `locale/library/std.json` into the ARB catalogs
(`scripts/gen_library_l10n.py`, `lib/l10n/library_strings.dart`), looked up by
`itemName` / `itemDescription`; the daemon's canonical English is the fallback
for an item the catalog does not know. `searchItems` matches the localized name
and tags, the English name and synonyms (the product words the presets used to
be: _tilt_, _servo_, _heater_, _battery_ …), the category's type name, the group
and section words and the display and source spellings of the category's units
(`deg`, `lux`, `rad/s`). Nothing localized reaches a request or a project.

**The Source sheet** (`ui/source_sheet.dart`, `docs/spec/concept-library.md` §
Creating a Source): a Source is never created without a concrete concept, so
every Source entry point — the canvas menu's **Add Source ▸** (_New Source…_,
then any preset a third-party library ships), the Library's _Sources_ row's
double-click, Return or drag — dispatches
`NewSourceRequested(presetId?, position?)` and nothing else; the reducer asks
the daemon for the ranked candidates (`ListSourceCandidates`) and the sheet
opens over the design (`SheetScrim`, in-tree and prop-driven like the packaging
sheet) once they arrive. Its one decision is the **concept**: _Existing concept_
(a pop-up of the design's concepts, the preset's value form first, each with its
value form beside it — two concepts of one form are two rows, never merged) or
_New concept_ (the concept sheet's form: name, category, the units as a fact,
meaning — prefilled by a preset when one was named); then the **Source name**
(suggested as `<concept>Input`, made free, and never replacing what the designer
typed) and its meaning; a live node preview; and the exact declarations that
will be committed, as the Code view will write them. _Create Source_ dispatches
`CreateSourceRequested` (one `CreateSource` request: one edit over an existing
concept, one transaction for a new concept and its Source); Cancel dispatches
`SourceSheetDismissed` and the project is untouched. The `() -> ?` of an unmade
choice is drawn on the sheet only. When the answer arrives, a new concept lands
where the designer pointed and opens for renaming with its Source a node width
(240 px) to the left (the environment side); a Source over an existing concept
lands where the designer pointed and is selected. The preset never decides the
identity: Studio sends the chosen `SemanticId`, or the new concept's
description, and infers nothing from names, value forms or units — the daemon
ranks (`bdl_library::rank_concepts`) and hides nothing.

## 3. Look and feel (macOS)

- Font: the system font (`.AppleSystemUIFont` → SF Pro on macOS). Sizes from the
  HIG macOS table: body 13 regular, headline 13 bold, subheadline 11,
  caption 10. Titles in panels: 11 pt semibold uppercase-free, secondary colour
  (like Finder's sidebar section headers).
- Controls: flat, 20–22 pt tall, 5 pt radius, 1 px hairline borders at ~12 %
  opacity; push buttons filled with the accent only for the default action. No
  ripple, no elevation shadows on panels, no FAB, no snackbar.
- Colour: neutral greys for chrome (light: window `#ECECEC`, content `#FFFFFF`,
  sidebar `#F5F5F5`; dark: window `#1E1E1E`, content `#262626`, sidebar
  `#2B2B2B`), text at 85 % / 50 % / 25 % opacity, the accent (`#0A84FF`-class
  blue) for selection and focus only. Semantic status uses the system palette:
  green for settled, orange for open, red only for errors.
- Layout: sidebar 220 pt, inspector 280 pt, both resizable and hideable (⌥⌘S /
  ⌥⌘I); 8 pt grid; list rows 22–24 pt; inspector as a form of right-aligned
  labels and left-aligned fields.
- Dialogs: open/new project go through the **OS's own pickers**
  (`file_selector`: folder picker to open, save dialog to name a new project
  directory — the native "create a document" idiom on both macOS and Windows).
  Object-creation sheets are attached to the window with Cancel/primary on the
  right; destructive actions confirm with the destructive button named ("Delete
  _Tilt_"), never "OK".
- Keyboard: full ⌘ shortcut set in a native menu bar (File, Edit, Design, View,
  Window, Help) — planned; Flutter's `PlatformMenuBar` on macOS.
- Motion: 150 ms ease for selection and panel toggles; none for data.

### Windows and macOS

One design, platform _details_ adapt (`lib/platform/desktop.dart`):

|                  | macOS                                        | Windows                   |
| ---------------- | -------------------------------------------- | ------------------------- |
| font             | SF Pro via the system font                   | Segoe UI (fallback chain) |
| primary modifier | ⌘                                            | Ctrl                      |
| toolbar          | gap for traffic lights                       | none                      |
| pickers          | NSOpenPanel / NSSavePanel                    | IFileDialog               |
| daemon           | `bdld` beside the app or `$BDLD_PATH`        | `bdld.exe`                |
| menus            | native menu bar (planned, `PlatformMenuBar`) | in-window menu (planned)  |

Colours, spacing, controls and the canvas are identical; nothing Material
(ripples, FAB, snackbars) appears on either.

## 3a. Spacing, alignment and separation — the foundations

Text is not glued together with punctuation. Structure is shown by **proximity**
(Gestalt: things that belong together sit closer than things that do not),
**alignment** (a shared edge makes a column; Müller-Brockmann's grid), and
**contrast** (weight and colour rank information). A middle dot, dash or slash
between two facts is a typewriter habit that makes the reader parse instead of
see.

Rules, with the numbers:

| Rule                                                        | Value                                                                                                                                                                                 |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Base unit                                                   | 8 pt grid; 4 pt for optical adjustments only                                                                                                                                          |
| Gap inside one item (label ↔ value, icon ↔ text)            | 4–8 pt (`MacMetrics.gapTight`, `gap`)                                                                                                                                                 |
| Gap between items of one group (the facts in a status line) | 16 pt (`gapGroup`)                                                                                                                                                                    |
| Gap between groups / sections                               | 24 pt (`gapSection`)                                                                                                                                                                  |
| Separators                                                  | whitespace, never `·` `—` `/` or `\|`; a hairline only between _sections_                                                                                                             |
| Secondary facts                                             | secondary/tertiary colour, same size; never parentheses to demote                                                                                                                     |
| Columns                                                     | anything with ≥ 2 items sharing the same fields is a grid: fixed column widths, one gutter (16 pt), labels right-aligned, text left-aligned, numbers right-aligned in tabular figures |
| Form labels                                                 | right-aligned column, 78 pt, baseline-aligned with the field                                                                                                                          |
| Units and symbols                                           | in their own column, secondary colour, never appended with a separator                                                                                                                |
| Status lines                                                | facts as separate cells with `gapGroup`; the leading fact is the one a glance needs (revision, connection)                                                                            |

`Row(spacing: MacMetrics.gapGroup)` and `MacTable` (fixed-width columns, one
gutter) are the two ways to lay out facts; string concatenation is not.

## 4. Editing model (what the inspector must expose)

Every operation is an `EditOp` the compiler already accepts. The inspector is
organised by what the designer means, not by the model's fields:

| Object                      | Section                        | Fields (designer words)                                                                                                                                                                                                                                                                                                                     | Op                                                                                                                                               | Kind                                                                                                                         |
| --------------------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| Concept                     | **Meaning**                    | Name, Meaning                                                                                                                                                                                                                                                                                                                               | Rename, SetDescription                                                                                                                           | refinement                                                                                                                   |
| Concept                     | **Value**                      | a pop-up — Quantity / On–off / Count / Collection of… / Grouped value / Optional… / Decide later — then what the form needs: Unit (angle, length, … with the symbol in its own column), _Each_ (a collection's element form), _When present_, _First_ / _Second_ (nested editors of the same kind); _Order_ (checkbox, quantity forms only) | SetRepresentation                                                                                                                                | choosing is a refinement; **changing a chosen value form** is an edit, and the section says which relationships it re-checks |
| Concept                     | **Relationships**              | Produced by, Used by — names as links that select the mapping                                                                                                                                                                                                                                                                               | —                                                                                                                                                | —                                                                                                                            |
| Concept                     | Delete `<name>`                | disabled while used, with the users named _at rest_ under the button                                                                                                                                                                                                                                                                        | DeleteConcept                                                                                                                                    | —                                                                                                                            |
| Mapping                     | **Meaning**                    | Name, Meaning                                                                                                                                                                                                                                                                                                                               | Rename, SetDescription                                                                                                                           | refinement                                                                                                                   |
| Mapping                     | **Reads**                      | chips with the socket glyph, removable; a pop-up to add                                                                                                                                                                                                                                                                                     | SetSignature                                                                                                                                     | edit                                                                                                                         |
| Mapping                     | **Produces**                   | pop-up with the socket glyph; titled **Provides** for a Source                                                                                                                                                                                                                                                                              | SetSignature                                                                                                                                     | edit                                                                                                                         |
| Mapping                     | **Meaning › Role**             | _Source_ / _Rule_ / _Value_ (the daemon's `MappingView.role`) / the port word for a port-backed relationship of an open component — derived, never edited; a Source adds one sentence (_A value that enters the behavior model from the environment, observed once per activation. Nothing is missing…_)                                    | —                                                                                                                                                | —                                                                                                                            |
| Source                      | **Relationship › Realization** | _Provided by the environment; no device is bound yet._ above the definition editor; the header word is _Source_, never _declared_                                                                                                                                                                                                           | —                                                                                                                                                | — (a device binding for a Source is ISS-0016)                                                                                |
| Mapping                     | **Relationship**               | the definition editor (§4a) — _Add definition_ / _Save definition_ / _Revert_ / _Detach definition_; header word _declared_ while empty, _unsaved_ while a draft differs; findings about the mapping’s place in the design attached under the editor in product language                                                                    | AttachDefinition, ReplaceDefinition (chosen by the reducer from the committed state, never by the widget)                                        | add is a refinement; save (replace) and detach are edits                                                                     |
| Mapping                     | Delete `<name>`                | —                                                                                                                                                                                                                                                                                                                                           | DeleteMapping                                                                                                                                    | edit                                                                                                                         |
| Mapping                     | **Timing**                     | _Updates in_: a domain, or _any_ (pure); `clock.*` findings under it                                                                                                                                                                                                                                                                        | SetMappingClock                                                                                                                                  | edit (Clock)                                                                                                                 |
| Mapping                     | **Drives**                     | the output this relationship is the final target of, or none; drive findings under it                                                                                                                                                                                                                                                       | SetMappingDrive                                                                                                                                  | edit (Output)                                                                                                                |
| Timing domain (library row) | —                              | Name (inline), create from the section's "+", delete while unused                                                                                                                                                                                                                                                                           | CreateClockDomain, RenameClockDomain, DeleteClockDomain                                                                                          | —                                                                                                                            |
| Output                      | **Meaning** · **Output**       | Name, Meaning; Accepts (concept, with glyph), _Updates in_, Required                                                                                                                                                                                                                                                                        | RenameOutput, SetOutputAccepts, SetOutputClock, SetOutputRequired                                                                                | edit (Output)                                                                                                                |
| Output                      | **Driver**                     | the driver or _none_; claimants while contested; Connect (pop-up of eligible relationships) / disconnect                                                                                                                                                                                                                                    | SetMappingDrive                                                                                                                                  | edit (Output)                                                                                                                |
| Output                      | Delete `<name>`                | —                                                                                                                                                                                                                                                                                                                                           | DeleteOutput                                                                                                                                     | —                                                                                                                            |
| Device (Deploy page, left)  | row edited in place            | Name, kind (pop-up), _for_ (one pop-up: an output, or a Source marked _— Source_, or _not connected_), the Realization row for an output or the Provider row for a Source, one pin field per requirement, Remove                                                                                                                            | CreateDevice, RenameDevice, SetDeviceKind, SetDeviceOutput, SetDeviceSource, SetDeviceRealization, SetDeviceProvider, SetDevicePin, DeleteDevice | Deployment only                                                                                                              |
| Mapping, Output             | **Fixes**                      | the service's actions for the selection: ready → button, needs a choice → pop-up, blocked → the reason                                                                                                                                                                                                                                      | `ListSemanticActions`; edits applied one revision at a time                                                                                      | —                                                                                                                            |
| both                        | **Explain** (collapsed)        | `SemanticId` / `DeclId`, `Θ` / `Interface`, inferred type, core term, status enum, diagnostic codes and technical detail, revision, the last change's kind and invalidation categories                                                                                                                                                      | —                                                                                                                                                | level 3 only                                                                                                                 |

No static explanatory paragraphs: a sentence appears only when it is about
_this_ object _now_ ("Changing this re-checks dimByTilt, warmPulse"; "Checked
once Temperature's value is decided").

**Last change** (below the inspector) states the consequence as a sentence about
other objects: _Nothing else needs rechecking._ for a refinement; _This change
affects dimByTilt, warmPulse — they will be checked again._ for an edit, the
names being links (from `EditOutcome.origin_decls`). The words _refinement_ /
_edit_ and the `Invalidation` categories are in Explain.

Words never shown at levels 1–2: `SemanticId`, `DeclId`, `Ty`, `q Dim`, `Grant`,
`Interface`, `realization`, `invalidation`, `Clocked`, `SingleDriver`,
`DriveEnv`, `RequirementId`, `solver`, `protocol`, `revision`, and any enum
name. A Source is never said to _call_ or _read hardware_ and is not a _sensor_:
it is a value the environment provides (the glossary's _Source_).

### 4a. The definition editor

The formula field is not a text box wired to an `EditOp`; it is an authoring
surface over the compiler's verdict (`lib/ui/definition_editor.dart`,
docs/architecture/studio-compiler-integration.md). Studio owns the draft — its
text, base revision, generation — and the compiler judges it as the designer
types (`AnalyzeDefinitionDraft`, debounced 150 ms, read-only). The project
changes only on _Add definition_ (no committed definition) / _Save definition_
(there is one), or _Detach definition_.

What is shown, in rank order: one status line under the field (dot for tone,
words for meaning: _Checking…_, _Valid definition_, _Tilt has no representation
yet._, the first error's message, _Not saved: …_); the offending spans
underlined in the field, each repeated as a diagnostic row with excerpt,
explanation and fixes; the names in scope as the field's hint (_expression over
Tilt, Held_); _unsaved_ in the section header and _Revert_ beside the primary
button while draft and committed differ; and, when the committed definition
changed under a dirty draft, a notice with _Reload_ / _Keep mine_ — never a
silent overwrite. Open is orange and worded as what is still to decide; only
_Invalid_ is red.

The text itself is coloured by what each word is — the IDE service's semantic
tokens through the syntax theme, exactly as the Code view (§12, _Colour_);
Studio classifies nothing.

Keys in the field: ⌘↩ saves the definition while it is dirty; ⌘S keeps its
meaning, _Save project_, and never commits a draft; Esc reverts; Return inserts
a line. The canvas draws committed state only; the status line counts _N unsaved
definitions_. Closing a project stashes dirty drafts by path and reopening
restores them — no modal.

### 4b. The Formula view

The definition editor has two projections of the one draft, chosen with a
**Formula | Text** segmented control at its top (`lib/ui/formula_composer.dart`,
the rendering `lib/ui/formula_render.dart`, the caret `lib/app/caret.dart`;
ADR-0028, ADR-0042): the Text view is the field of §4a; the Formula view draws
the compiler's `FormulaProjection` as the mathematics it states — `clamp` of a
fraction `[Tilt]` over `[90][deg ▾]`, `[0]`, `[1]` — never as a graph inside the
node, and is edited two ways at once over that one draft: a **structural caret**
the keyboard types at and a **selected part** the palette acts on. Studio owns
the caret, the selection, the mode and the open pop-up; the compiler owns the
tree, every expected type, every candidate and the text every action makes
(`ComposeFormula` → `DefinitionDraftChanged` → the ordinary verdict). Nothing in
Studio parses, types, converts a unit or decides what fits.

**Encodings**, by channel (the table in §3 gains these rows): a **slot** — an
expression not yet written, `?` in the text — is a dashed hollow chip (dashed =
not decided, as the canvas's _declared_ node); a **reference** is a chip with
its concept's socket glyph (shape = kind of value, as on the canvas); a
**literal** is its coordinate and its unit — a composite drawn as the compiler
renders it (`unit_display`, `m/s²`) — with a unit pop-up when selected, listing
the compiler's candidates for the literal's own dimension (`UnitCandidate`,
`display`) — the pop-up switches the unit and keeps the quantity (`180 deg` →
`3.141592653589793 rad`). Only a literal has a unit pop-up: a reference's kind
is its declaration's, and the Composer never rewrites it. A **quotient** (`/`)
is a **fraction**: the numerator over a rule over the denominator, the rule the
operator's own part; the other **operators** are their glyphs (× − ≤ ≥ ≠) and
the logical ones (`&&`, `||`, `!`) their words, _and_, _or_, _not_, in the
keyword weight — the operator's reading, never a localized label;
**parentheses** grow with their content; a **call** is its name and parentheses;
a **choice** (`if c then a else b`) is a branch diagram — `if` and the condition
on a spine, `then` and `else` with their outcomes on rows under it, the words
selecting the choice and every part an ordinary component (the condition expects
true or false, both outcomes what the choice gives); a **match** is the subject
on the spine and one row per arm — the pattern (the arm's own text, in the
local's italics; it is not a part) before `⇒` and the body; a **`let` block** is
one row per binding (`let`, the pattern, `=`, the value) over a rule over the
result; a **rule** (`x => …`) its parameters in italics, `⇒`, the body; a
**collection** or a **group** its items between `[ ]` or `( )`; **`delay`** and
**`sync`** a region with a bar on its left — the temporal boundary — the word
and its arguments. Only `()` stays opaque, as the compiler says
(`kind: opaque`); nothing else is monospace text. A **binder**
(`all reading in readings: …`) is a head row — the word in bold, the local as an
italic chip with a lighter frame, `in`, the collection, the colon — over its
body indented beneath it; the local's uses in the body are the same italic chip,
so what the formula itself binds is told apart from what the design provides (a
design reference stays upright with its socket glyph). A **range** is its two
ends around a `..` glyph. Selection is the selection tint; the caret a thin
accent bar; keyboard focus the accent ring; a finding is a red underline on the
component _and_ its row under the field — one diagnostic, two projections. Every
part has a **reading** for assistive technology (`describeNode`), assembled from
the tree in product words — _Tilt over 90 deg_, _a choice: if Held, then 1, else
0_, _a match on Tilt with 2 cases_, _a block with 1 local bindings_, _a delay
boundary of 0, half_ — never from the text.

**Typed structure** (ADR-0042). The **caret** is a place in the tree: a byte
offset with the _stop_ it stands at (`CaretState`), the stops read off the
projection by `caretStops` — _before_ and _after_ each part at its byte range,
_in_ an empty slot, _open_ / _close_ just inside a parenthesised part, and the
character positions inside a leaf (a name, a number, a unit) — in reading order,
each with the part it belongs to and the part enclosing it (the positions
`NavigateFormula` moves between, walked locally so no key waits). It is drawn
where the rendered part is (`FormulaGeometry`: a key per part, its inner row,
its text; `_stopRect` measures the text up to the caret's character with the
leaf's own style), never over character columns. Keys: ← → the previous / next
stop (the last stop in a denominator is followed by the stop after the fraction
— how a nested part is left; a parenthesis is passed, not skipped); ↑ ↓ the
nearest stop on the row above / below (a numerator from its denominator, a
branch from the next; measured, not counted); Home / End the first / last stop
of the enclosing part, again of the whole formula; Tab / ⇧Tab the next /
previous empty slot, wrapping; `)` leaves the enclosing parentheses; `,` moves
to the next argument. What a key **does** at a stop is one `KeyPlan`
(`characterAt`, `operatorAt`, `openParenAt`, `backspaceAt`, `deleteAt`): a
**text edit** of the draft at the stop's byte offset — letters and digits into a
slot or extending the name or number the caret touches, a space after a number
starting its unit, a character of a leaf deleted — sent as
`DefinitionDraftChanged` and read by the compiler like any typing; a
**structured action** the compiler answers with text — `+ − * / < > = & | !` on
the part the caret touches with a slot for the other side (`operator`), `(`
after a name applying it (`clamp` → `clamp(?, ?, ?)`, `apply`, 0.28), ⌫ / ⌦ on a
whole part (`remove`); a **move**; or a **refusal** with a sentence under the
field (_type an operator first_ — a letter after a complete part). Nothing here
parses: the compiler reads the result. While the text differs from what the
compiler last read, the part being typed into is shown as text in place
(`PendingText`, the region `changedRegion` names) and the rest keeps its
structure; the picture follows the compiler's next reading, and the caret is put
back at the byte it was typed at (`FormulaCaretMoved`). An empty draft is typed
into directly. **Completion** is asked on every typed character at the caret's
byte offset (`CompletionRequested` → `CompleteDefinitionDraft`, ranked by the
position's expected type) and on ⌃Space, shown beside the caret
(`CompletionPopup`); ↑ ↓ choose, Return or Tab accepts (Tab moves to the next
slot when the candidate is already written), Esc closes it, then clears the
caret and the selection. **Pointer** and keyboard share the draft: a click
places the caret at the nearest stop of the part under it and selects the part
(`onTapNode`; a click in the field's empty space places the caret at the nearest
stop), the caret follows every compiler answer to the part it selects, and the
palette below acts on the selection as before — there is no mode to enter or
leave.

**The palette**, beneath the field, for the selected component: the compiler's
sentence — _Expected: an angle, because an angle ÷ an angle = a dimensionless
quantity._ — with the kernel's notation behind **Explain**; for a slot, a number
entry whose unit pop-up holds the units of the expected dimension (none for a
dimensionless slot; none, with a sentence, when the position is not determined)
— or, where the compiler answers with the truth values (`booleans`: a position
that is true or false, or a concept represented by one), **true** and **false**
buttons in its place — then _References_ by type and _Equations_ folded, then
the forms a slot opens: **Choose** (`if ? then ? else ?`) and, for a truth value
or an undetermined position, **not** (`!?`); for a component, **+ − × ÷**,
**and** / **or** (a slot after it) and **not** (in place, no slot) unless the
component is known not to be a truth value, **Compare**, **Function** (the
equations whose result fits, wrapping the component as the first argument),
**Each element** (all / any / map / filter — offered when the component is a
collection or its kind is unknown; the compiler picks the local's name),
**Range** (`… in ? .. ?`, not offered on a truth value), **Choose** (the
component becomes the `then` outcome, `if ? then … else ?`, the condition
selected next) and **Remove** (an empty operand of a two-sided operator, `&&`
and `||` included, removes the operator; an empty negation is its slot). Whether
a component _may_ be a truth value is read off the projection (`actual.kind`, a
concept's representation) — presentation only, never a judgment. The palette is
contextual — what the slot expects, what fits the selected part — and never a
whole keyboard: the keyboard is the way to write.

The Formula and Text views express the same set of forms and switching loses
nothing. The stale-projection policy (`app/composer.dart` `composerInSync`): a
projection is current only when it is of exactly the text on screen and that
text parsed; otherwise the field is dimmed with a notice — _Waiting for the
compiler to read the formula…_ while the verdict for this text is on its way,
_The text cannot be read as a formula._ when it never will be (then no tree is
shown: none is invented) — no component answers a click, no slot panel opens, no
structural key acts, and the reducer refuses a structured action (or a second
one in flight) and discards an answer for text that has moved on; **Edit as
text** is the way out. Save, revert, conflict and detach are §4a's, unchanged: a
formula with a slot may be saved and is _invalid_ until filled.

What was borrowed from the mature math editors, and what was not: from GeoGebra,
the fraction built by `/` with the caret staying in the denominator until an
arrow leaves it, the right bracket exiting a group, completion while typing a
name; from MathLive, Tab across placeholders, Home / End to a group's ends, a
reading per part; from Desmos, the sentence under the field. Not borrowed: any
styling, LaTeX or a local parser (the compiler reads every character), a virtual
keyboard, a local unit algebra, and the free-text field with rendered
decorations.

Screenshot: `docs/user-guide/assets/studio/formula-composer.png`
(`docs/user-guide/screenshots/manifest.json`, `formula-composer`).

## 5. Sheets teach by showing, not by example text

A creation sheet never carries a sample value as a hint ("Tilt") — that repeats
the label and tells the designer nothing about what the field _means_. Instead
the sheet renders, live and at canvas fidelity, the node the entries will become
(`NodePreview`, painted by the same `NodePainter` as the canvas):

- **New concept**: Name · Value (Quantity / On–off / Count / Decide later) ·
  Unit (when quantity; the same table as the inspector) · Meaning. The preview's
  socket takes the chosen shape (○ ◇ □) and stays a hollow ring while the value
  is undecided; a one-line caption under the preview names that mark and what it
  means. The socket is grey because its colour is the identity the compiler will
  allocate.
- **New mapping**: Name · Reads (concept toggles drawn with their socket glyphs,
  in their colours) · Produces. **Produces has no default** — the pop-up reads
  _choose_, the preview's output socket is a hollow neutral ring, and Create
  stays disabled until a concept is chosen; a mapping that "produces" the first
  concept in the list would otherwise be created without anyone deciding so. The
  preview is the mapping node with those input sockets, dashed, _declared_ — the
  state it will be in.

Text fields (`MacTextField`): hairline, 5 pt radius, 24 pt; focus = 1 px accent
border + 3 pt soft glow, nothing moves. Push buttons (`MacButton`): 22 pt, ≥ 72
pt wide, primary filled with the accent, secondary hairline, destructive red
text; Cancel left of the default action; Return submits.

## 6. Interaction states — the one standard

Every control in Studio answers hover, press, keyboard focus and disabled the
same way. Implemented once: `MacStates` in `lib/ui/mac/theme.dart` feeds every
Material control theme; `MacInteractive` / `MacLink` in
`lib/ui/mac/interactive.dart` cover rows, links and chips; the canvas painter
applies the same rules to nodes and sockets.

| State                       | Treatment                                                                                                                    | Numbers           |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | ----------------- |
| **hover**                   | a flat overlay of the ink colour on the control's own surface; on accent-filled controls the overlay is white so it lightens | 6 %               |
| **pressed**                 | the same overlay, stronger                                                                                                   | 12 %              |
| **focused** (keyboard only) | 2 pt accent ring; pointer clicks never show it                                                                               | `accent`, 2 pt    |
| **selected**                | accent at 20–25 % as the row/segment background; the text stays primary                                                      | `selection` token |
| **disabled**                | 40 % opacity, no hover, arrow cursor                                                                                         | 0.4               |
| **motion**                  | one ease-out, everywhere; nothing animates data                                                                              | 120 ms            |

Per control family:

| Family                        | Rest                                 | Hover                                     | Pressed                    | Cursor                             |
| ----------------------------- | ------------------------------------ | ----------------------------------------- | -------------------------- | ---------------------------------- |
| text link (`MacLink`)         | accent text, no underline            | hover pill behind the text                | stronger pill              | hand                               |
| list row (`MacInteractive`)   | transparent                          | hover pill                                | stronger pill              | arrow                              |
| push / outlined / icon button | hairline or accent fill              | overlay                                   | overlay                    | arrow (macOS convention)           |
| segmented control             | selected segment raised on `control` | unselected segment gets the hover overlay | —                          | arrow                              |
| pop-up (dropdown)             | `control` + hairline                 | `controlHover`                            | menu opens                 | arrow                              |
| text field                    | `control` + hairline                 | —                                         | —                          | I-beam; focus = 2 pt accent border |
| checkbox / switch / slider    | Material shapes recoloured to tokens | overlay halo                              | halo                       | arrow                              |
| canvas node                   | hairline outline                     | outline in secondary text colour          | —                          | grab                               |
| canvas socket                 | filled/hollow by binding             | 4 pt halo in the concept colour           | drop target: stronger halo | crosshair                          |
| destructive button            | outlined, red text                   | overlay                                   | overlay                    | arrow                              |

Rules that keep it one system: no ripples, no elevation change on hover, no
colour change of text on hover except links (which are already accent), hover
never conveys information that is not also visible at rest, and the focus ring
is the only place the accent appears on a control that is not selected or
primary.

## 7. Three information levels and the semantic UI matrix

The semantics of a design are shown as _structure_ first, explained in _prose_
second, and named in _formal vocabulary_ only on demand. Every semantic fact is
designed for exactly one primary level; a fact may echo at the next level only
as detail behind the first, never as a duplicate badge.

| Level           | Answers                                                                                                  | May use                                                                                                                                                                                      | May not use                                                                                                                                                            |
| --------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **1 Canvas**    | what does the product do; what depends on what; what is still open; where does behaviour become physical | object silhouette, socket shape, socket hue, links, grouping, containment, line style, the object's own state; one state word where a word is unavoidable                                    | type labels, ids, compiler words, badges, counts                                                                                                                       |
| **2 Inspector** | what does the selected object mean; what can I change; what will the change affect                       | designer vocabulary: _Meaning, Value, Unit, Reads, Produces, Relationship, Used by, Produced by, affects, checked again_; diagnostics in product language attached to the field they concern | `SemanticId`, `DeclId`, `Ty`, `Grant`, `Interface`, `realization`, `invalidation`, `Clocked`, `SingleDriver`, `DriveEnv`, `solver`, `protocol`, `revision`, enum names |
| **3 Explain**   | why was this accepted or refused; what did the surface form elaborate into; which rule applies           | all of the above, kernel notation, Core IR, diagnostic codes, technical details, revision                                                                                                    | —                                                                                                                                                                      |

Level 3 is one collapsed disclosure, **Explain**, at the end of the inspector,
and the technical part of a diagnostic. It is never open by default and nothing
in levels 1–2 depends on it.

### The matrix

_now_ = implemented; _spec_ = agreed here, drawn when its compiler pass lands.
"Canvas" is level 1, "Inspector" level 2, "Explain" level 3.

| Semantic fact                               | Internal representation                                                                   | Canvas                                                                                                                                                                                                                                                                                                                                                                                 | Inspector wording                                                                                                                                                                                                                    | Explain wording                                                                         |                                                                                                                                                                                      |
| ------------------------------------------- | ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| semantic identity                           | `SemanticId`                                                                              | socket and link **hue** from the id, identical on every page; the name at every socket                                                                                                                                                                                                                                                                                                 | the name; never a number                                                                                                                                                                                                             | `SemanticId 3`                                                                          | now                                                                                                                                                                                  |
| representation                              | `Θ s = q d / bool / nat / list R / R₁ × R₂ / opt R`                                       | socket **shape**: ○ quantity, ◇ on–off, □ count, ⧉ collection (a stack), ▯ grouped value (a split square), ◎ optional value (a ring with a hole) — what a collection holds is the inspector's word, not a second shape                                                                                                                                                                 | Value: Quantity / On–off / Count · Unit: angle, length, … with the symbol in its own column                                                                                                                                          | `Θ(3) = q[rad]`                                                                         | now                                                                                                                                                                                  |
| representation not chosen                   | `Θ s = none`                                                                              | **hollow** socket ring                                                                                                                                                                                                                                                                                                                                                                 | Value: Decide later; "relationships can already use it"                                                                                                                                                                              | `Θ(3) = none`                                                                           | now                                                                                                                                                                                  |
| order by declaration                        | `Concept::ordered` (`OrdDecl`, Phase 9c)                                                  | — (nothing on the canvas: order is not structure)                                                                                                                                                                                                                                                                                                                                      | _Order_ checkbox under a quantity value form: _values are magnitudes: <, smallest, largest, clamp, in range_ / _values are compared for equality only_                                                                               | `ordered concept …`; `semantic.no_order` names the fix                                  |                                                                                                                                                                                      |
| unresolved declaration                      | `realization = none`                                                                      | **dashed** outline, empty definition region, header word _declared_                                                                                                                                                                                                                                                                                                                    | Relationship: empty field + Attach                                                                                                                                                                                                   | `Δ(d).realization = none`                                                               | now                                                                                                                                                                                  |
| mapping relationship                        | `Signature { inputs, output }` → `Interface`                                              | one input socket per read concept on the left, one output socket on the right, links in the concepts' hues                                                                                                                                                                                                                                                                             | Reads · Produces (chips and pop-up carry the socket glyph)                                                                                                                                                                           | `Interface: sem#0 → sem#3 → sem#1`                                                      | now                                                                                                                                                                                  |
| dependency (`dependsOn`)                    | `MappingAnalysis.references` (`DependencyGraph::all`)                                     | a **reference edge**: secondary-colour 1.5 px link from the referenced relationship's output socket into the referencing relationship's formula line; no socket, no hit area (ADR-0034)                                                                                                                                                                                                | _Depends on_ / _Named in_ rows of name links; the _Role_ row _Rule_ / _Value_ / _Source_ with one sentence                                                                                                                           | `references` in kernel notation                                                         | now                                                                                                                                                                                  |
| rule vs value                               | `Signature.inputs` non-empty vs empty with a definition                                   | input sockets (the shape) and the header word _rule_ when no state word takes the slot; a value has neither                                                                                                                                                                                                                                                                            | _Role: Rule_ / _Role: Value_; the creation sheet's sentence as the reads change                                                                                                                                                      | `type: A -> B` vs `() -> B`                                                             | now                                                                                                                                                                                  |
| semantic construction                       | `mk s` under `Grant.of τ`                                                                 | a link forms only between sockets of one hue; the output socket is the produced concept                                                                                                                                                                                                                                                                                                | Produces                                                                                                                                                                                                                             | `Grant permits mk sem#1 in this realization`                                            | now (grant is invisible by design)                                                                                                                                                   |
| dimension mismatch                          | `Prim.ty` fails                                                                           | a **red mark at the formula line** on the node, nothing in the header                                                                                                                                                                                                                                                                                                                  | under the formula: "This adds an angle and a time." + fixes                                                                                                                                                                          | `+ : q[rad] → q[rad] → …, found q[s]`, code                                             | now                                                                                                                                                                                  |
| rule nothing applies                        | `reactive.rule_unapplied` (no reverse edge in the dependency graph, no drive edge)        | **hollow** output socket (no value comes out of it), header word _not applied_ once defined; the accessibility label says "applied by nothing"                                                                                                                                                                                                                                         | the finding in _Relationship_ with its fix beside it (_Add a value that applies dimByTilt_); on Simulate a readiness note and the probe's _Applied by_                                                                               | `reverse_all = ∅`, `β = none`; the note's technical text                                | now                                                                                                                                                                                  |
| waiting on an open value                    | `MappingStatus.OPEN`                                                                      | solid node whose read socket is hollow                                                                                                                                                                                                                                                                                                                                                 | under the formula: "Checked once _Temperature_'s value is decided."                                                                                                                                                                  | status enum                                                                             | now                                                                                                                                                                                  |
| temporal state                              | `delay init e`                                                                            | **register mark** on the link that crosses a tick, initial value beside it                                                                                                                                                                                                                                                                                                             | "Remembers _Held_, starting at _no_"                                                                                                                                                                                                 | `delay false (declRef d)`                                                               | spec for the canvas; today `delay(init, e)` is written in the formula and its value shows in the Simulate trace                                                                      |
| clock / domain                              | `Κ d = some c`                                                                            | **lane**: labelled background region; domain-free mappings outside                                                                                                                                                                                                                                                                                                                     | _Updates in_ pop-up (pure = any domain); the domain is a name, never a rate                                                                                                                                                          | `Κ(d) = c₀`, `Clocked`                                                                  | now as a quiet word at the node's right edge, the library's _Timing domains_ section and the inspector pop-up; the lane is spec                                                      |
| cross-domain observation                    | `sync src init e`                                                                         | **gate** on the link at the lane edge with the initial value; a crossing without a gate is broken at the boundary                                                                                                                                                                                                                                                                      | "Observes the latest _Temperature_, starting at 20 °C"                                                                                                                                                                               | `sync c₁ 293.15 (declRef d)`                                                            | spec for the canvas; today `sync(domain, init, e)` is written in the formula, an ungated crossing is a `clock.*` finding under _Timing_ and the status line's _reads across domains_ |
| physical output                             | `OutputId`, `OutputSpec { accepts, clock }`                                               | **terminal node** (sink) at the right: boundary bar, one input socket, dashed while open or undriven; a drive link is dragged onto it and dragged away to disconnect                                                                                                                                                                                                                   | Output: accepts, _Updates in_, required, driver, claimants, connect / disconnect; the mapping's _Drives_                                                                                                                             | `Ω(o) = ⟨q[1], c₀⟩`                                                                     | now (the device name on the node: not yet)                                                                                                                                           |
| output conflict                             | `SingleDriver β` fails                                                                    | the sink carries the red word _contested_ and every claimant's link; the drop is not refused (the model records the second driver, the analysis reports it)                                                                                                                                                                                                                            | "_Light_ already has a final target, _dimByTilt_. Combine the values before the output." + the fixes _Detach `d` from `Light`_ and _Create upstream combination mapping_                                                             | `β d₁ = β d₂ = o`                                                                       | now (the refused drop and red gap of the original spec were not built)                                                                                                               |
| hardware feasibility                        | `solve` result                                                                            | Deploy: board picture; each requirement a lead to a pin; an unsatisfied one has no lead                                                                                                                                                                                                                                                                                                | the verdict _for this board_, the dead end as a sentence about a device and a pin, the pins that block it                                                                                                                            | `DeadEnd { requirement, reason, placed }`                                               | now as a table on the Deploy page; the board picture is spec                                                                                                                         |
| deployment allocation                       | `Assignment`                                                                              | lead from requirement to pin; pinned ones marked                                                                                                                                                                                                                                                                                                                                       | device → requirement → pin rows; fixed pins edited in the device row                                                                                                                                                                 | resource ids                                                                            | now as a table; leads on a picture are spec                                                                                                                                          |
| runtime value                               | telemetry by `DeclId` + activation                                                        | **number at the socket**, unit column; ◇ filled/empty; stale fades                                                                                                                                                                                                                                                                                                                     | Tilt 31.4° · Held yes                                                                                                                                                                                                                | `DeclId 4 @ activation 1203`                                                            | spec for the canvas and for telemetry; simulated values show in the Simulate trace and probe (§10)                                                                                   |
| change consequence                          | `EditOutcome { kind, invalidates }`                                                       | —                                                                                                                                                                                                                                                                                                                                                                                      | "Nothing else needs rechecking." or "This change affects _dimByTilt_, _warmPulse_; they will be checked again."                                                                                                                      | `refinement` / `edit`, `Invalidation::{…}`                                              | now                                                                                                                                                                                  |
| component instance                          | `ComponentInstance`, ports' `PortContract`s                                               | **instance node**: teal-grey header (the instance's name), the component's name in the body row, one row per port — required and parameters on the left, provided on the right — sockets typed by the concept each port carries _in the system_ (a shared concept's hue; a private concept's flat identity, so two instances differ); the domains it is placed in as the quiet word    | Instance: name, _Of_ (+ Edit Source), timing parameters as pop-ups over the system's domains, ports with their status word, findings, replace with a version                                                                         | `ComponentInstanceId`, `PortRef`, `ResolvedConcept::Private{instance, local}`           | now (§11)                                                                                                                                                                            |
| open port / open base relationship          | `PortStatus::Open`, an unresolved base relationship in a system                           | **hollow socket in its hue** (the value form is known, the value is not supplied) at the port row, and at an open base relationship's _realisation socket_ on its definition row                                                                                                                                                                                                       | Port: _open_, "nothing supplies it yet"; a design with open ports still simulates                                                                                                                                                    | `PortStatus::Open`, FV Theorem H                                                        | now                                                                                                                                                                                  |
| binding                                     | `Binding { source, destination: BindingEnd }` → `Definition::Reference`                   | a **link** between two binding ends (port ↔ port, base relationship → port, port → base realisation socket) in the concept's hue; selected: accent, 3 px; dragged away from its destination: disconnected                                                                                                                                                                              | Binding: from, to, timing (direct / carried across), Disconnect; a bound base relationship reads _takes its value from lampA.brightness_ with _Show Binding_ and no formula field                                                    | `declRef target` / `sync src init (declRef target)`, `bind#n`                           | now                                                                                                                                                                                  |
| transported binding                         | `BindingTransport { init }`                                                               | a **gate** on the link (a short bar with its initial value)                                                                                                                                                                                                                                                                                                                            | "carried across timing domains, starting at 0"                                                                                                                                                                                       | `sync c₀ 0 (declRef d)`                                                                 | now                                                                                                                                                                                  |
| second source for a taken port              | `system_edit.destination_bound`                                                           | the drop is not refused: a sheet asks _Replace the connection?_ — Disconnect and Connect, or Cancel; never a silent replace                                                                                                                                                                                                                                                            | the current source named in the question                                                                                                                                                                                             | —                                                                                       | now                                                                                                                                                                                  |
| binding across timing domains               | `Incompatibility::NeedsTransport`                                                         | the drop opens _Carry across timing domains_ with a _Starts at_ field; the value is required before Connect                                                                                                                                                                                                                                                                            | the two domains named                                                                                                                                                                                                                | —                                                                                       | now (Studio compares the resolved domains only to _ask_; the compiler judges the binding)                                                                                            |
| a body that no longer keeps its promise     | `contract::realizes` fails                                                                | a **red mark in the instance node's body row** (every instance of the component)                                                                                                                                                                                                                                                                                                       | Component: _promise broken_ with the `component.*` findings; Instance: "…'s source no longer keeps its promise; open it to see why"                                                                                                  | `Realizes`, `component.port_clock_mismatch`, …                                          | now                                                                                                                                                                                  |
| behaviour group (expanded)                  | `BehaviorGroup { id, scope, name, description, members }` — authoring metadata (ADR-0019) | a **tinted region** around the members with a title band (violet, 12 %; brighter while a dragged relationship would join it); drag the band to move them together; drag a relationship in or out to change membership; nothing else changes                                                                                                                                            | Behavior: name, meaning, relationships (+ Add relationship), inputs / outputs / open / physical outputs / internal, package (system's own design only), collapse, ungroup / delete with relationships                                | `group#n`, `GroupScope`, Theorems A–G (`eraseGroups` is the identity on every judgment) | now                                                                                                                                                                                  |
| behaviour group (collapsed)                 | the group's **boundary** (`GroupBoundary`, computed in Rust)                              | a **group box** (region tint as a header) with **aggregate sockets**: crossing-in and open members on the left, crossing-out and driven members on the right, each in the declaration's hue and labelled by it; links re-route to them; a link started or dropped on one **resolves to the concrete declaration** (one directly, several through a chooser) and never binds to the box | Boundary: inputs / outputs / open / physical outputs / internal — "a picture of the cut, not a connection of its own"                                                                                                                | `crossIn`, `crossOut`, `crossing_edges`, Theorem H (`socket_no_fanout`)                 | now                                                                                                                                                                                  |
| packaging                                   | `preview_extraction`, `ExtractGroupAsComponent`                                           | after packaging the instance node stands where the group's box stood (or at the members' top-left); the component's own canvas starts from the members' positions                                                                                                                                                                                                                      | the sheet: Requires / Provides (the floor, not negotiable), open relationships _Treat as input · Keep internal_, physical outputs _Stays the system's · Moves inside_, timing parameters, internal, the compiler's warnings; Package | `Extract`, Theorem R (restricted)                                                       | now                                                                                                                                                                                  |
| port-backed relationship (component source) | `Port.decl`                                                                               | header word _requires_ / _provides_ / _parameter_ on the relationship node                                                                                                                                                                                                                                                                                                             | Place: "Backs the port …: what instances see of it is the promise"                                                                                                                                                                   | `Port { decl, contract }`                                                               | now                                                                                                                                                                                  |
| multi-selection                             | none — `MultiSelected` (Studio state)                                                     | accent outline on every selected node; a translucent accent marquee while box-selecting                                                                                                                                                                                                                                                                                                | _N selected_: the relationships as links; _Group as Behavior_ with a quiet suggestion when they read each other or share a domain                                                                                                    | —                                                                                       | now                                                                                                                                                                                  |

Rules the matrix implies: hue is identity and nothing else; shape is
representation and nothing else; dashed is _declared_ and nothing else; a red
mark means _wrong now_ and never _not yet_; a hollow socket at a _port_ means
_unbound_ (its value form is known — only a concept's hollow ring means
_undecided_); region tint means _grouped_ and never a domain or a category; the
accent is selection, focus and the default button. Adding an encoding adds a row
here.

### What Studio reads, and what it still needs from the read model

Studio computes nothing semantic (ADR-0001). Everything above is read off
`ProjectProjection`, `ProjectAnalysis`, `EditOutcome`, the draft, hover,
completion and semantic-action responses, `SimulationResponse` and
`DeploymentAnalysis` (docs/spec/protocol.md). Facts it shows today that are
derived _structurally_ from the projection, not judged: which mappings read or
produce a concept (from signatures), which read concept of an `OPEN` mapping
still has no value form (from `Θ`), and which declarations are simulation inputs
(no inputs, no definition). Facts the UI wants and the protocol does not yet
carry — to be added on the compiler side, never invented in Dart:

| Needed for                                                           | Field                                                                                                                       | Today                                                                                        |
| -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| "This change affects _A_, _B_" for signature edits and detaches      | dependents (by `DeclId`) in `EditOutcome`, not only the origin                                                              | `origin_decls` names the origin; a mapping edit lists only itself                            |
| concept-level findings under the Value section                       | `Diagnostic.entity = concept_id` populated by the checker                                                                   | shape exists, unused                                                                         |
| Explain's _why_ (dependencies, grant, clock, output relation)        | a request serving `bdl_ide::explain`                                                                                        | the disclosure shows the projection's technical fields; `explain` is LSP-only                |
| contexts on the canvas                                               | a `ContextView`                                                                                                             | contexts do not exist in the model                                                           |
| entity hover and fixes inside a component's source                   | a body-scoped `EntityRef`                                                                                                   | the flat-entity services are not offered in the source view (DI-40)                          |
| domains as regions, `sync` gates and `delay` registers on the canvas | the `sync`/`delay` sites of a definition with their initial values (the elaborated Core has them; no projection lists them) | a word per node; the phrases in the formula text                                             |
| values at sockets (Simulate on the canvas, Monitor)                  | none missing for simulation — `TickSample` is per `DeclId` and activation; telemetry has no protocol yet                    | trace table and probe on the Simulate page; nothing on the canvas                            |
| the Deploy page from the read model                                  | none missing — `rows[]`, `missing[]`, `blocker` are on the wire (0.5)                                                       | the page still renders from fields 4–9 (docs/architecture/studio-compiler-integration.md §3) |

## 8. When the OS dialog cannot be shown

`file_selector` dialogs are hosted by macOS's view-bridge, which refuses
children of sandboxed hosts (Studio launched from an embedded terminal, for
example). A `null` returned faster than a person could cancel is treated as
_refused_: Studio shows a banner explaining it and the Start list gains _Open by
path…_ / _New at path…_ as a typed fallback.

## 9. Not built

Contexts, transports and clock boundaries as canvas regions (a domain is a word
on the node today); cycles emphasised on the canvas; value plots and a board
picture (both pages are tables); Monitor content; a native menu bar; draft
indication on the canvas; the device name on a sink node; editing a formula on
the canvas (the unfolded formula is read-only; the inspector edits); a
per-concept display unit (ISS-0019); the daemon's `NavigateFormula`,
`CompleteFormulaCaret` and `ComposeAction.insert` are served but Studio walks
the tree it holds and types as text (ADR-0042).
docs/architecture/studio-compiler-integration.md §3 places each.

### Collections, grouped values, equations

The equation language (`docs/spec/equation-library.md`) reaches Studio in three
places and nowhere else: the socket shapes above; the concept inspector's
value-form pop-up with nested forms and the _Order_ checkbox; and the Simulate
page, where an input whose value form is a collection, a grouped value or an
optional value is one text field written the way the design shows it
(`[20, 25.5]`, `(21, true)`, `none` / `some(3)`), read by shape only — what it
means stays with the compiler. Equations are typed in the definition editor like
any call; completion offers them in the designer's words (`clamp(x, low, high)`
— _the value held between a low and a high bound; needs values of an ordered
kind_), and their diagnostics name concepts, collections and rules. _Scheme_,
_fold_, _product_, _type variable_ never appear at levels 1–2.

## 10. Simulate

The page answers _what does the product do over time_, one tick at a time, with
bdld's reference evaluator (docs/spec/runtime-semantics.md) doing every
evaluation. Studio holds the values the designer fed, the schedule they chose,
and the samples that came back — tied to one project revision
(`app/simulation.dart`) — and computes none of them.

_As built: [Simulate page](../user-guide/assets/studio/simulate-page.png) and
[readiness](../user-guide/assets/studio/simulate-readiness.png)._

```text
┌───────────────┬────────────────────────────────────────┬───────────────────┐
│ Sources       │ [Step] [Step ×10] [Reset]   tick 3     │ Probe             │
│ ● tilt  Tilt  │ ● held needs a value before simulation │ brightness   ●    │
│   [0.5  ] rad │   can step.  Show                      │   Now   0.333     │
│ ◆ held  Held  │ tick active       tilt  brightness     │ Over the run      │
│   [off]       │   0  interaction  0.52  Brightness(…)  │   0  0.333        │
│ Timing domains│   1  interaction  0.52  Brightness(…)  │ ▸ Explain         │
│ ↻ interaction │                                        │                   │
│   every [1]   │                                        │                   │
└───────────────┴────────────────────────────────────────┴───────────────────┘
```

- **Sources** are the unresolved value declarations — mappings that read nothing
  and have no definition (`I d t`, DI-16; FV Phase 12
  `SimulationInput = Source ∧ UnitDomain`): the same relationships the canvas
  draws as Sources. Each gets a control from the concept's _value form_, never
  from its name: a number with the unit beside it (quantity), an off | on
  segmented control (on–off), a whole number (count). A Source without a value
  is a third state, not _off_: the number fields carry the hint _no value yet_,
  the on–off control is drawn empty with a dashed outline
  (`MacSegmented undecided`, the same "not decided" mark as a declared node and
  a formula slot) and the words _no value yet_ beside it; one click on a segment
  gives exactly that value and nothing is defaulted — a missing input is a
  runtime error by design (docs/spec/runtime-semantics.md). The row carries the
  concept's glyph and hue from the canvas and selects the same object as Design.
- **Timing domains**: an activation period per domain, the `Schedule` the
  evaluator activates by; a period, never a rate. Changing one starts over.
- **Readiness** is read off the compiler's analysis and the projection and never
  judged here. Each blocker is a sentence about a named object with a _Show_
  link that selects it: _tilt needs a value before simulation can step._ · _Tilt
  needs a value form (Quantity, On / off or Count) before tilt can be given a
  value._ · _level has no valid definition._ · _dimByTilt has no definition._ ·
  _These relationships depend on each other in the same instant: a, b._ While
  any is listed Step is disabled and a Step sends nothing; while the analysis
  for this revision is pending the list says _Checking the design…_ and nothing
  is wrong. Entering the page never starts a run. Below the blockers, with a
  hollow dot (does not stop Step), the **notes**: the compiler's
  `reactive.rule_unapplied` on every defined rule no value applies — _X is a
  rule nothing applies yet._ with its explanation naming the call — carrying the
  service's fix `rule.apply` (_Add a value that applies X_) and a _Show_ link.
  The fix is chosen where the finding is (`SemanticActionChosen`): Studio
  selects the rule, asks `ListSemanticActions`, and applies the action when it
  arrives READY; NEEDS_CHOICE renders the pop-up of calls, BLOCKED the reason
  (`OfferedFix`, `FixItem` — one drawing of a fix everywhere). Studio matches
  the action by its kind, never by spelling the entity itself
  (`SemanticActionsState.ofKind`).
- **Step** evaluates the next tick(s) with the inputs on screen. The daemon
  fixes an input trace at `StartSimulation`, so each step is a deterministic
  **replay**: `Start` with every tick's inputs so far and the schedule, then
  `Step(n)` — the same design, schedule and inputs give the same trace.
  **Reset** returns to tick 0; inputs and periods stay.
- **The trace** is the semantic diagram over time: rows are ticks, columns the
  value declarations (a relationship with inputs is a function and has no
  column) and the driven outputs; cells are bdld's own rendering; `·` where a
  declaration was not evaluated because its domain did not activate (the
  _active_ column names the domains that did). An input is not echoed by the
  evaluator, so its cell is the value Studio fed for that tick, shown only where
  its domain activated according to the sample's `active_clock_ids`
  (`sampleOf`); a read-model echo of inputs would remove that lookup. Column
  order is identity, not time.
- **The probe** (right) is the selection's value now and over the run, with the
  object's glyph; an output shows its final target. A rule has no value per
  tick: the probe says so (_A rule: it has no value of its own. A value whose
  formula applies it is what the simulator samples._) and links, under _Applied
  in_, the values whose definition references it — `MappingAnalysis.applied_by`,
  the compiler's direct reverse dependency edges (0.20), never inverted or
  searched in Studio — or says _No value applies it yet._ and offers the same
  fix. Explain holds `DeclId`, the run's revision, the rendered value and an
  error's code and technical text.
- **Errors** that stop a tick are the controls' line, about the object (_bad
  divided by zero._), never the global banner.
- **Stale results**: a new revision drops the samples, keeps the fed values for
  inputs that still exist, and ignores an answer in flight; the daemon's
  `SimulationResponse.revision` is checked against the revision on screen.

Memory and transports show up as values: `acc = delay(0, acc + x)` reads its
initial value at tick 0 and the previous sum after; `y = sync(fast, -1, x)` in a
slower domain reads the source's last activation strictly before its own, so a
source value produced at the same global tick is not yet visible (DI-17). The
e2e cases in `test/simulation_test.dart` hold the same traces as
`crates/bdl-compiler/tests/surface_to_backend.rs`.

## 11. System projects: three zoom levels, one canvas

Every project is a behaviour system (docs/architecture/behavior-systems.md,
ADR-0019, ADR-0023) with two truths on the wire — the authored system and the
flat design derived from it — and Studio shows one design at a time
(`EditorState.context`, `app/system.dart`):

| Context                                                                         | The canvas shows                                                                                                                                                                                                                                                                                                            | Edits go to                                                                                                                                            |
| ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **System**                                                                      | the top level: shared concepts, domains, sinks, top-level relationships (with a _realisation socket_ when open), component **instance nodes** drawn from their ports' contracts, **binding links** (a gate where a value is carried across domains), **behaviour regions** or collapsed **behaviour boxes**                 | `ApplySystemEdit { Base }` for the design's own objects; instance / binding / port / component ops; `ApplyGroupEdit` for behaviours (never a revision) |
| **Component source** ("Editing AdaptiveLamp · used by 3 instances", _‹ System_) | the body as an ordinary design in the component's own names and identities; a port-backed relationship carries the word _requires_ / _provides_ / _parameter_; drafts, completion and hover run in the body's scope; the component's own **behaviour regions and boxes** (scoped to it, with their own canvas and viewport) | `ApplySystemEdit { EditComponentBody }`; the component inspector's contract edits; `ApplyGroupEdit { component }` for its behaviours                   |

A design with no components is the system context of the degenerate system
(`SystemView.is_flat`); nothing about it is a separate kind.

Simulate and Deploy read the derived flat design in every context
(`AppState.flat`): an instance's relationships appear as `lampA.brightness`, its
open ports as inputs.

### Instance node anatomy

_As built: [instance nodes](../user-guide/assets/studio/instance-nodes.png) and
a [component's source](../user-guide/assets/studio/component-source.png)._

```text
        ┌──────────────────────────────┐
        │ lampA                        │   header: instance name (teal-grey strip)
   ○────┤ tiltValue                    │   required port / parameter: left, hollow while open
        │                  brightness ├────●   provided port: right
        │ AdaptiveLamp        ↻ main   │   body row: the component's name (red mark: promise broken), its domains
        └──────────────────────────────┘
```

208 pt wide; one row per port; the hue is the concept the port carries _in the
system_ (a shared concept's own; a private one's flat identity — two instances
of one component have two hues for their private Brightness). Double-click or
_Edit Source_ opens the component; the component is never rendered from its body
on the system canvas.

### Gestures added

| Gesture                                                                                                                                                              | Result                                                                                                                                                                                                                                  |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| click a node / region band / link; ⇧-click                                                                                                                           | select; ⇧ toggles the node into a multi-selection                                                                                                                                                                                       |
| ⇧-drag on empty canvas                                                                                                                                               | box select (nodes whose centre is inside); the inspector shows _N selected_ and offers _Group as Behavior_ for the free relationships, with a quiet suggestion when they read each other or share a domain — never grouped on their own |
| right-click a multi-selection → Group as Behavior (n relationships)                                                                                                  | one group named _Behavior_, then renamed inline; no creation modal                                                                                                                                                                      |
| drag a relationship over an expanded region                                                                                                                          | the region brightens (the insertion affordance); on release it joins (or moves from its old behaviour) — membership only                                                                                                                |
| drag from / drop on an aggregate socket of a collapsed behaviour                                                                                                     | resolves to the concrete declaration it stands for (a member's socket, or the member); several → a small chooser naming _member · reads X_ / _member · produces Y_; never a link to the behaviour                                       |
| collapse (menu, inspector)                                                                                                                                           | the box starts where the region was                                                                                                                                                                                                     |
| drag a collapsed box                                                                                                                                                 | the hidden members travel with it                                                                                                                                                                                                       |
| expand                                                                                                                                                               | the members appear where the box now is (the stored layout translated by the box's delta)                                                                                                                                               |
| zoom below 0.5×                                                                                                                                                      | every behaviour reads as its summary box; the authored collapse state is untouched                                                                                                                                                      |
| pan / zoom                                                                                                                                                           | remembered per canvas (`Layout.viewport`)                                                                                                                                                                                               |
| drag provided socket → required socket / parameter (or base relationship output → required socket, provided socket → an open base relationship's realisation socket) | `BindPorts`, when the destination is free and the domains agree                                                                                                                                                                         |
| … onto a taken destination                                                                                                                                           | a sheet: _Disconnect and Connect_ or _Cancel_ — never a silent replace                                                                                                                                                                  |
| … across timing domains                                                                                                                                              | a sheet asking _Starts at_ — never an implicit transport                                                                                                                                                                                |
| drag a bound input away, release on empty                                                                                                                            | `UnbindPorts`                                                                                                                                                                                                                           |
| click a binding link                                                                                                                                                 | select it (accent, 3 px); ⌫ disconnects                                                                                                                                                                                                 |
| right-click empty canvas → Add Instance ▸ component                                                                                                                  | `CreateInstance` at the pointer, name open for editing                                                                                                                                                                                  |
| right-click a relationship → Group / Add to Group ▸ / Remove from …                                                                                                  | group edits (membership only)                                                                                                                                                                                                           |
| drag a relationship into / out of a region                                                                                                                           | `AddMember` / `MoveMember` / `RemoveMember`                                                                                                                                                                                             |
| drag a region's title band                                                                                                                                           | moves every member (layout)                                                                                                                                                                                                             |
| double-click a region's title                                                                                                                                        | rename inline                                                                                                                                                                                                                           |
| right-click a group → Collapse / Expand, Package as Reusable Component…, Ungroup                                                                                     | layout; the packaging sheet; `DeleteGroup`                                                                                                                                                                                              |
| double-click an instance                                                                                                                                             | open its component's source                                                                                                                                                                                                             |

### The packaging sheet

Requires and Provides are the boundary as the compiler computed it — the floor;
the sheet never offers removing a crossing-in or crossing-out declaration. The
four decisions it asks are the ones the formal work cannot infer: the
component's name, the instance's name, whether each open relationship becomes an
input or stays open inside, whether each driven sink stays the system's or moves
inside. Every change re-asks the preview (`PreviewComponentExtraction`,
generation-tagged); _Package_ is one atomic edit; afterwards the instance stands
where the group stood and the component's canvas opens laid out as the group
was. The design computes the same values (FV Theorem R for the single-domain
fragment; the e2e test compares the simulated trace before and after).

### Behaviours in both scopes

A behaviour (group) is a permanent way of seeing an authored design, not only a
step toward a component: it exists wherever an authored design exists — the
system's own design and every component's source — and is invisible to BDL
semantics in both (ADR-0019 amendment). In a component's source the inspector
shows the same sections (Name, Meaning, Relationships with _+ Add relationship_,
Inputs / Outputs / Open / Physical outputs / Internal, Canvas, Remove) and says
that packaging a behaviour inside a component comes with nested components. Undo
(⌘Z) covers behaviour edits — they are authored actions in one history with the
semantic edits, and undoing one moves no revision. A behaviour edit dirties the
project (the save mark) although the revision is unchanged.

### Not built

Entity hover and fixes inside a component's source (DI-40); packaging a
behaviour inside a component (nested components, DI-44); a minimap.

## 12. Design, Code and Split: views of one project

The Design page's context bar ends in a segmented control — _Design_ · _Code_ ·
_Split_ — the platform's shape for a mode (Xcode's editor modes). Switching is a
view change; no conversion, no project kind (ADR-0023 §3). `EditorState.view`,
`app/sources.dart`, `ui/pages/code_pane.dart`.

**Task.** Change the design as text and see the graph follow — or see, at once,
that what was typed does not build yet and where.

**Facts and where they land.** (1) Is the graph in step with the text — the
costliest mistake is to read the graph as the text's meaning when the text did
not build. A _document-level condition_, so a banner (never a colour on the
text): on the Code pane, _This file does not build yet: the design shows the
last version that did_; on the canvas, _Showing the last version that built; the
text has changes that do not build yet_. Both are the daemon's `draft` flag;
Studio judges nothing. (2) Where the fault is — the explanation layer: a list
under the editor, one row per reason in product language, the line number in
tabular figures, a mark that survives without colour (× for an error, the
canvas's hollow ring for something still open — incompleteness is never red);
activating a row puts the caret on the range. (3) The text itself — the primary
object, a monospace editor over the daemon's sources, one file at a time (a
pop-up names the others; a draft file is labelled _— not built_). (4) The
selection — shared in Split: selecting a node scrolls the editor to its item;
the caret in an item selects its node. Both directions use the daemon's source
anchors (`GetSources`); Studio does not parse.

**Flow.** The Code pane shows `GetSources`: the files with every graph edit
written back, so a rename on the canvas appears in the text with its comments
intact. Typing edits the editor's own buffer; after a 400 ms pause (or on
leaving the field) the whole file is one `ApplySourceEdit` against the revision
it was read at. Accepted: a new revision, the graph follows, the new node is
already placed. Refused: the graph stays at the last revision that built, the
banner appears, the draft stays exactly as typed with its reasons. A stale edit
(the project moved under it) is resent once the sources catch up; typing during
a send is the next send; nothing typed is discarded and no error banner is
raised for a race the designer did not cause. `pendingRequests` counts the send,
as it counts every edit.

**Colour.** The text is coloured by what each word _is_ — the IDE service's
semantic tokens (`docs/architecture/syntax-highlighting.md`, ADR-0035) through
`ui/code/syntax_theme.dart`, the one place a colour is chosen. A name's ink is
the _category_ of what it names, in the hue family of that category's node
header tint (concept, relationship, Source, output/device, instance), so the
text and the graph agree without a legend; keywords, operators and units are the
secondary ink, comments the tertiary; a declaration is heavier (the title of its
item — the weight channel), a name bound in the formula itself (a parameter, a
binder's local) is italic, the `?` slot is the _open_ colour (the text form of
the Composer's dashed hollow chip). Non-colour partners: the declaring keyword
(`concept`, `mapping`, `output`, …), the shape of the use, the hover card's
word, the weight or slant. _Unresolved_ is not a colour here: the canvas draws
it dashed and the fault list names it. The channel row is _ink colour in code
text_ (`semantic-ui.md`). Text that does not build keeps its lexical colours;
while the designer types the spans on show are shifted, never recomputed in
Dart, and the service is asked again 150 ms after typing pauses — no flicker, no
stale colour over new text. The definition editor's field is the same controller
and the same theme.

**The editor knows the design.** The same IDE service that colours the text
answers it (protocol 0.22, `app/code_tooling.dart`): ⌃Space opens the completion
pop-up at the caret — the service's candidates for this spot, a rule as a call,
a Source or a value as a name, a unit after a number, best first, ↑/↓, Return or
Tab, Esc — the pop-up above the caret when the editor has no room below; a 250
ms dwell over a name shows its card at the name (title, declaration, what it
produces, its state, _role_, the description), gone on typing, not asked again
inside the card's own span; ⌘-click or F12 goes to where the name is declared,
in this file or another, and selects it; ⇧F12 lists every place that names it
under the editor — file and line in tabular figures, the line's text — a row
goes there, Esc or × closes; _Format_ in the file bar or ⌥⇧F asks for the
canonical layout, which replaces the text as one edit with the caret kept at its
line and column, or changes nothing when the file does not parse cleanly. The
pop-up and the card are the formula field's (`ui/code/`); the Code view and the
field differ in document scope and geometry only. Studio classifies, resolves
and composes nothing: an unknown word has no card, a candidate's insertion text
is the service's, the definition's range is the daemon's.

**States.** No project: the page's empty state. Sources not yet here: _Reading
the sources…_. Incomplete-but-valid (open faults): listed with the hollow ring,
the graph in step; completion and hover still answer what the tree and the last
build know, and _Format_ changes nothing. Not building: the banners and the
list. Disconnected: the editor is read-only; the last colours stay; the pop-up,
the card and _Format_ are not offered.

**Saved as typed.** A save keeps the text of the editor whether or not it
builds, and every formula draft (ADR-0030); reopening returns to it, the banners
included. Nothing the designer can see and edit is "unsavable" because it is
incomplete.

## 13. Leaving a project: one guard

Every path that unloads a project — the toolbar's _Close_, the project manager
button, _Open Project…_ / _New Project…_ while one is open, ⌘W, ⌘Q, the menu's
_Quit_, the window's close button — dispatches through one guard
(`app/lifecycle.dart`). Studio keeps no dirty flag of its own: it sends whatever
typing has not reached the project (the Code view's unsent text, a debounced
formula draft), asks the project for its projection and reads `dirty` off it.
Clean: the project unloads at once and the intent (open the other project, quit)
runs after the close. Dirty: a sheet — the platform's shape for a document about
to be lost —

> Save changes to “lamp”? [Don't Save] [Cancel] [Save]

_Don't Save_ apart on the left; _Cancel_ beside the default; _Save_ is the
default (Return), _Cancel_ answers Esc. _Save_ unloads only after the save
succeeded; a refused save (a source changed on disk) keeps the project open with
the existing conflict banner. _Don't Save_ unloads; the next open returns to
what was saved. _Cancel_ leaves everything as it was. The window's close button
and the application's exit request are declined (`didRequestAppExit` → cancel;
the macOS runner turns the close button into a terminate request) until the
guard has run, so nothing is torn down while the question is open.

Where the designer was — the view, the page, the open file, the component whose
source was open — is a per-user note in the recent list (`recent.json`),
restored on the next open of that project; never project data.

**Not built.** Inline squiggles on the fault's range (the list and the caret
jump stand in); rename from the editor (rename on the canvas, and the text
follows); the unit-domain hint (`text.legacy_unit_domain`) in the pane's list
(the language server raises it; `GetSources` carries load faults only); creating
a second source file from Studio (a file made by an external editor appears on
reload).

## 14. Deploy: the boundary view, and the Source half

The Deploy page is the one place the design meets a board, and it stays a
**boundary view**: Target, verdict, devices, placement — never a pin, a bus or a
board on the Design canvas (ADR-0015, ADR-0038). With Source provision the page
gained one fact and one row, and the shape was chosen against the tools a
designer already knows.

**The task.** _Decide whether this design can run on this board, and connect the
devices it needs._ The facts, ranked: (1) the verdict for the board; (2) what is
still unconnected — an output without a device, a Source without a device, a
device for nothing; (3) per device, what it is for and whether its profile is
admissible; (4) the placement.

**Where each fact lives.**

| Fact                                           | Home                                                                            | Encoding                                                                                                                                   |
| ---------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| a Source no device provides                    | the verdict (_so far_, orange) and the `missing` line _tilt has no device on …_ | the same state colour and the same sentence shape as an output without a device; never red — an old project is not wrong                   |
| what a device is _for_                         | the device card's one pop-up                                                    | outputs by name; Sources as _tilt — Source_ (the role word the canvas and the inspector already use); _not connected_                      |
| the provision's admissibility                  | the device card's Provider row                                                  | four ✓/✗ judgments — _transducer_, _fits_, _placed_, _readable_ — with the analysis's sentence; icon + word, never colour alone            |
| _readable_ failing                             | the same row, in the secondary ink                                              | a fact about this board's firmware, not about the design or the placement: the verdict stays, the sentence names the board                 |
| where a profile comes from                     | _from package …_ beside the profile, only when not builtin                      | display data; the judgments never read it (ADR-0038)                                                                                       |
| the Source's realization, from the design side | the inspector's _Realization_ row                                               | a projection of the Deploy page's provision for the chosen board (_Provided by sensor as … on …_); the environment when no board is chosen |

**Why one pop-up and not a Sources list.** KiCad's footprint assignment and
Altium's component-to-footprint mapping put the design object on the left and
the physical choice on the right, one row per object; LabVIEW and Simulink bind
a signal to hardware from the signal's own block. BDL's device is the physical
object, and a device is _for_ exactly one thing — so the row is the device and
its one pop-up says what it is for, outputs and Sources together, the role word
telling them apart. A separate Sources list would repeat the device rows in a
second order and give the same fact two homes; the missing-item line already
lists every unprovided Source in one place (the overview), and the card is the
detail.

**Why the same row shape for a provider and a realization.** Simulink's block
parameters and JetBrains' run configurations show the same mask for every
target: what changes is the content, not the anatomy. The Provider row is the
Realization row mirrored — label, pop-up, raw type, judgments, sentence — so the
designer who has chosen a realization reads a provider without learning a second
layout, and the compiler's symmetry (encoder ↔ transducer) is visible.

**Why _readable_ is a fourth check and not a fifth colour.** VS Code's problem
markers keep _error_, _warning_ and _information_ and never invent a fourth
severity for "fine here, not there"; Blender greys out an operator the current
context cannot run and says why in the tooltip. A profile this board's firmware
cannot read is that situation: the placement stands, the design is untouched,
the firmware is refused. It is a warning-coloured sentence and one unchecked
judgment, not a new verdict.

**Not built.** A board picture with leads (§7); the collections report on this
page; choosing a board's tick and periods here (`bdld compile` only); a
Sources-first view for a design with many Sources and few devices — if a real
project asks, it is a filter on the same cards, not a second list.

## 15. Deploy: build, flash, observe

The Deploy page ends where the product begins: the board running the design.
This section records what was studied before the Firmware section was drawn,
what it borrows, what it refuses, and what the first study will test.

**The task.** _Put this design on this board, and try it._ The facts, ranked:
(1) the next thing to do — one action, or the one blocker in its way; (2)
whether the last image is still this design; (3) what a build or flash is doing
now; (4) which board a flash would reach; (5) the developer's facts (crate,
command, compiler output).

**Where each fact lives.**

| Fact                      | Home                     | Encoding                                                                                                                                                                   |
| ------------------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| where the designer stands | the section header       | _Deployment · Build · Flash · Observe_: done steps ✓ in the settled colour, the current step in the primary ink and semibold, the rest tertiary — position, then colour    |
| the next thing to do      | the card, right          | exactly one primary (accent-filled) button — _Build for …_, _Build again_, _Flash_ — or none, when a sentence stands in its way; secondary actions are links               |
| the one blocker           | the card, left           | an orange dot (the _open_ state, never red: an incomplete deployment is not wrong), the daemon's sentence and its remedy; a design blocker offers the Design page          |
| the image's freshness     | the card's first line    | green dot + _Firmware built at hh:mm_ when current; orange dot + _from an earlier design or deployment_ when stale, with Flash withdrawn — the sentence, not a badge       |
| progress                  | the card while running   | a spinner, the stage as a word, the daemon's message, the count of crates, an indeterminate accent bar (cargo gives no total), _Stop_                                      |
| a failure                 | the card, red            | the stage's sentence and the remedy in product words; the compiler's words only behind _Details_, which opens itself on a failure                                          |
| the reachable board       | under the image          | none: orange dot, _No board is reachable._ and the BOOTSEL line; one: green dot and its name; several: the sentence and a pop-up — the choice is shown as a choice         |
| the outcome               | under the board          | _Flashed to … at hh:mm_, the daemon's restart sentence, and _Try it: act on pressed; lamp should follow the design_ — the design's own names, never a claim of correctness |
| the developer's facts     | _Details_ (a disclosure) | crate, command, target, image, compiler output in the code face; closed unless a build failed                                                                              |

**Design validity and deployment feasibility, kept apart.** The verdict line is
about the board and says so (§14); the Firmware section reads the daemon's
ordered blockers, whose first entries are the design's own problems — so a
semantically invalid design with a feasible placement reads _Feasible on …_
above and _Not ready to build — lit is not fully defined_ below, with a link to
the Design page. Neither line ever says "ready" for the other's reason.

**What was studied.**

| Tool        | What was looked at                                                                                                                          | Borrowed                                                                                                                                                     | Rejected, and why                                                                                                                                                                   |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| KiCad       | the schematic apart from the board; footprints assigned per symbol; ERC as markers with severities and exclusions; annotation before layout | the logical/physical split as two pages with one object on each side; a readiness gate before the physical step (annotation → our blockers); one-object rows | a marker list the designer sifts; the "?" designator as the readiness cue — BDL's blockers are sentences with a remedy, one at a time, ordered by the daemon                        |
| Simulink    | the hardware board in the model's configuration; _Build, Deploy & Start_; the Diagnostic Viewer's stages and Fix links                      | stages as the unit of progress and of failure; a fix beside the message; the build reported per stage, not per line                                          | the configuration dialog as the home of the board (BDL's board is a page-level choice, a session preference); a viewer separate from the page — the stage lives where the action is |
| LabVIEW     | the target as a node in the project tree; deploy on Run; _Deployment Progress_; build specifications for a standalone image                 | the deploy as one gesture from the same window; an explicit "deployed and running" outcome                                                                   | the target as a container of the design's files (BDL's design is target-free); build specifications as a second artifact the user manages                                           |
| Arduino IDE | one board+port selector; Verify and Upload as two buttons; the console below; auto-detection of the board with a fallback dialog            | a detected device named in the flow; the port question asked only when it matters; Upload as the primary verb after a successful build                       | Verify as a separate top-level button (the build is the gate to the flash, not a peer); a raw console as the primary feedback — the console is _Details_                            |
| PlatformIO  | a status-bar toolbar (Build, Upload, Monitor, env selector); auto-detected upload port; errors to the Problems panel via a matcher          | build and upload as a pair; auto-detection with an override                                                                                                  | a toolbar of peers (five equal buttons is what §25 of the brief forbids); the env selector's config file (BDL's target is a pop-up on the page)                                     |
| Blender     | a modal job's progress bar with a cancel in the status bar; the report line coloured by kind; header text for a modal operator              | a running job as a bar with a cancel; one line of state in the state's colour; the modal state shown where the eye is                                        | the status bar as the home of the job — the build belongs to the page whose task it completes; the status line keeps its four clusters                                              |

**What BDL keeps that none of them has.** A device is _for_ one design object
and never on the design; a stale image is the daemon's judgment by content, not
a timestamp; the blocker is ordered by the compiler's own layers (design →
deployment → firmware) so the page never explains a firmware refusal before a
design error; and no button is ever disabled without the sentence that says why
— there is either the action or the reason.

**Tested before the study** (`apps/studio/test/firmware_test.dart`,
`firmware_e2e_test.dart`; the daemon's `firmware_e2e.rs`): every state above,
the choice between two boards never made for the tester, the stale image never
offered Flash, the trial sentence in the design's names.

**What the first study asks** (`docs/project/ux-study-pico.md`): whether the
path Design → Deploy is discoverable; whether the tester sees why the button's
pin is absent from Design; whether _active low_ is read correctly; whether the
Provider and Realization rows are read as the same shape without conflating a
Source and an output; whether the verdict, the blocker, Build's readiness and
Flash's BOOTSEL line are actionable; whether the status line and the verdict are
read as two answers or one.

**Not built.** A progress fraction (cargo has no total); a board picture
lighting the pin the flash would use; the Nano's `avrdude`; a probe-first path;
the runtime shipped with an installed Studio (a checkout or `BDL_RUNTIME_DIR`
today).
