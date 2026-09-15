# Studio UI design

Three references, each used for one thing:

| Reference | What we take from it | What we do not take |
|---|---|---|
| **DaVinci Resolve** | one window, one workflow, *pages* in workflow order on a bottom page bar that is always available; a status line above it; each page owns a fixed panel arrangement (Fusion: node editor centre, effects library left, inspector right, viewers top) | its dark-only look, its density, timeline metaphors |
| **Blender node editor** | node anatomy (header, collapse, coloured sockets left-in / right-out), links as curves dragged socket-to-socket, drop-on-empty-space discards, cut links by drawing across them, box select, active-vs-selected, data flows left→right | its modal keymap as a *requirement* (G/R/S), its dozens of node types |
| **macOS (Apple HIG)** | system font, 13 pt body / 11 pt secondary, flat controls, sidebar–content–inspector, toolbar in the title area, translucent panels, accent colour only for selection and the primary action, no ripples, native menus and ⌘ shortcuts | — |

Sources: DaVinci Resolve 18 Reference Manual ch. 1 ("Switching Among Pages",
"The Fusion Page": Node Editor, Inspector, Effects Library, Status Bar);
Blender Manual, *Interface › Controls › Nodes › Parts / Editing / Selecting*;
Apple HIG, *Typography* (macOS text styles) and *Designing for macOS*.

## 0. Two screens (DaVinci)

Resolve opens on the **Project Manager** and only shows the page workspace
once a project is open. Studio does the same: with no project, the window
is the launcher below; opening or creating a project replaces it with the
workspace of §1; ⊞ in the page bar closes the project and returns.

```
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

Laid out like VS Code's welcome page (Start on the left, Recent on the
right) rather than Resolve's thumbnail grid — BDL projects have no
thumbnail yet. **Hero**: the wordmark *Behavior Designer* set in Chakra
Petch (a square sans with 45° chamfered corners, SIL OFL, bundled) over a
field of 45°-routed traces drawn deterministically — PCB routing is the
one visual idiom that belongs to both halves of the product. Recent
projects are an app preference (`recent.json` in the per-user application
support directory), never project data; missing directories are shown
greyed with *not found* and can be removed on hover.

## 1. Window structure (DaVinci)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ● ● ●   lamp •                      [⌘Z ⌘⇧Z]   [Save]      [Build] [Run] │  toolbar (title area)
├───────────────┬──────────────────────────────────────┬───────────────────┤
│ Library       │                                      │ Inspector         │
│  Concepts     │            page content              │  (selected object)│
│  Mappings     │                                      │                   │
│  Contexts     │                                      │                   │
│  Outputs      │                                      │                   │
├───────────────┴──────────────────────────────────────┴───────────────────┤
│ r12 · 2 concepts · 1 mapping · dimByTilt: declared            bdld 0.1.0 │  status line
├──────────────────────────────────────────────────────────────────────────┤
│ [⊞]        Design      Simulate      Deploy      Monitor            [⚙] │  page bar
└──────────────────────────────────────────────────────────────────────────┘
```

Pages, in workflow order (the paper's own sequence: intent → refine → time →
contexts → outputs → domains → board → observe):

| Page | Centre | Left | Right | Answers |
|---|---|---|---|---|
| **Design** | node canvas | library (concepts, mappings, contexts, outputs, components) | inspector of the selection | what the product does |
| **Simulate** | trace timeline / value plots | input traces | probe inspector | what it does over time |
| **Deploy** | board picture + allocation table | boards, device kinds | requirement / conflict inspector | whether it fits, build & flash |
| **Monitor** | the same canvas with live values | telemetry sessions | probe inspector | what it is doing right now |

Resolve's page bar can hide labels (icons only) and hide pages; we keep
labels by default and allow ⌘1–⌘4. The page bar's left button is the
project manager (open/new/recent), the right is project settings — as in
Resolve. Nothing on the page bar is a compiler badge.

The **status line** (Resolve's Fusion status bar) shows the revision, a
count summary, the hovered/selected object's state in one phrase, and the
daemon connection. Errors from requests appear as a banner above the page
content, never as a modal.

## 2. Node canvas (Blender)

### Anatomy

```
        ┌─────────────────────────┐
        │ ▸ dimByTilt      declared│   header: collapse toggle · title · state word
   ●────┤ Tilt                     │   input socket (left), coloured by concept
        │                Brightness├────●   output socket (right)
        │ f(θ) = clamp(…)          │   body: definition summary / properties
        └─────────────────────────┘
```

* **Header colour** = category: concept (grey-blue), mapping (blue), context
  (violet), output (amber), transport (teal). Muted, low-saturation, with
  the title in 13 pt semibold.
* **Socket colour = semantic identity.** In Blender a socket's colour is its
  data type; in BDL the type that matters is the nominal concept, so each
  concept gets a stable hue derived from its `SemanticId` (deterministic,
  never from the name). A link is only accepted between sockets of the
  same concept — the canvas shows the nominal typing rule without a
  diagnostic. Representation kind, when bound, changes the socket *shape*:
  quantity ○, boolean ◇, count □; unbound concept: hollow ring.
* **Concept node**: title = concept name, one output socket "value" on the
  right (values of this concept flow out), one input socket on the left
  (a mapping producing this concept connects here). A concept whose
  representation is not yet chosen shows a hollow socket and the word
  *open* in the header.
* **Mapping node**: one input socket per signature input (labelled), one
  output socket. Body shows the definition summary or *no definition yet*;
  the state word is the paper's workspace state (`declared`, `defined`,
  `type-valid`, …). An unresolved mapping is drawn with a dashed outline —
  distinct, never red.
* **Links** are cubic Béziers from an output socket (right edge) to an
  input socket (left edge), tangents horizontal, colour of the concept,
  2 px, selected links thicker. Data flows left → right.

### Interaction (Blender's, on a Mac)

| Gesture | Result | Blender equivalent |
|---|---|---|
| drag empty canvas | pan | MMB drag |
| scroll / pinch | zoom about the pointer | wheel / ctrl-MMB |
| click node | select (active); ⇧-click adds | LMB / ⇧LMB |
| drag node body | move selected nodes; on release the layout is committed (no revision) | G |
| drag from output socket to input socket | make link (edit: adds the concept to the mapping's inputs, or sets the mapping's output) | LMB drag |
| drag from an input socket away and release on empty space | disconnect | drag link away |
| drop link on empty space | discard (no auto-create) | discard |
| ⌘-drag across links | cut links | Ctrl-RMB cut |
| drag on empty canvas with ⇧ | box select | B |
| ⌫ / Delete | delete selection (concept in use → banner: "*Tilt* is used by *dimByTilt*") | X |
| ⌘A / ⌘D | select all / duplicate (later) | A / ⇧D |
| Home / ⌘0 | frame all | Home |
| ⌘F | find node by name | Ctrl-F |
| double-click node header | rename inline | — |
| H | collapse selected nodes | H |
| right-click | context menu (rename, attach definition, delete, explain) | RMB |

Dragging a node over a link does **not** auto-insert it (Blender's auto
attach); BDL links are typed by concept, and silent insertion would be a
semantic edit. Muting nodes (M) has no BDL meaning and is not offered.

### What the canvas never means

Edges are dependency, not execution order. Drawing order does not set
output priority. Node position is layout only (ADR-0003). No node type
exists per arithmetic operator — formulas live in the inspector.

## 3. Look and feel (macOS)

* Font: the system font (`.AppleSystemUIFont` → SF Pro on macOS). Sizes
  from the HIG macOS table: body 13 regular, headline 13 bold, subheadline
  11, caption 10. Titles in panels: 11 pt semibold uppercase-free, secondary
  colour (like Finder's sidebar section headers).
* Controls: flat, 20–22 pt tall, 5 pt radius, 1 px hairline borders at
  ~12 % opacity; push buttons filled with the accent only for the default
  action. No ripple, no elevation shadows on panels, no FAB, no snackbar.
* Colour: neutral greys for chrome (light: window `#ECECEC`, content
  `#FFFFFF`, sidebar `#F5F5F5`; dark: window `#1E1E1E`, content `#262626`,
  sidebar `#2B2B2B`), text at 85 % / 50 % / 25 % opacity, the accent
  (`#0A84FF`-class blue) for selection and focus only. Semantic status uses
  the system palette: green for settled, orange for open, red only for
  errors.
* Layout: sidebar 220 pt, inspector 280 pt, both resizable and hideable
  (⌥⌘S / ⌥⌘I); 8 pt grid; list rows 22–24 pt; inspector as a form of
  right-aligned labels and left-aligned fields.
* Dialogs: open/new project go through the **OS's own pickers**
  (`file_selector`: folder picker to open, save dialog to name a new project
  directory — the native "create a document" idiom on both macOS and
  Windows). Object-creation sheets are attached to the window with
  Cancel/primary on the right; destructive actions confirm with the
  destructive button named ("Delete *Tilt*"), never "OK".
* Keyboard: full ⌘ shortcut set in a native menu bar (File, Edit, Design,
  View, Window, Help) — planned; Flutter's `PlatformMenuBar` on macOS.
* Motion: 150 ms ease for selection and panel toggles; none for data.

### Windows and macOS

One design, platform *details* adapt (`lib/platform/desktop.dart`):

| | macOS | Windows |
|---|---|---|
| font | SF Pro via the system font | Segoe UI (fallback chain) |
| primary modifier | ⌘ | Ctrl |
| toolbar | gap for traffic lights | none |
| pickers | NSOpenPanel / NSSavePanel | IFileDialog |
| daemon | `bdld` beside the app or `$BDLD_PATH` | `bdld.exe` |
| menus | native menu bar (planned, `PlatformMenuBar`) | in-window menu (planned) |

Colours, spacing, controls and the canvas are identical; nothing Material
(ripples, FAB, snackbars) appears on either.

## 3a. Spacing, alignment and separation — the foundations

Text is not glued together with punctuation. Structure is shown by
**proximity** (Gestalt: things that belong together sit closer than things
that do not), **alignment** (a shared edge makes a column; Müller-Brockmann's
grid), and **contrast** (weight and colour rank information). A middle dot,
dash or slash between two facts is a typewriter habit that makes the reader
parse instead of see.

Rules, with the numbers:

| Rule | Value |
|---|---|
| Base unit | 8 pt grid; 4 pt for optical adjustments only |
| Gap inside one item (label ↔ value, icon ↔ text) | 4–8 pt (`MacMetrics.gapTight`, `gap`) |
| Gap between items of one group (the facts in a status line) | 16 pt (`gapGroup`) |
| Gap between groups / sections | 24 pt (`gapSection`) |
| Separators | whitespace, never `·` `—` `/` or `|`; a hairline only between *sections* |
| Secondary facts | secondary/tertiary colour, same size; never parentheses to demote |
| Columns | anything with ≥ 2 items sharing the same fields is a grid: fixed column widths, one gutter (16 pt), labels right-aligned, text left-aligned, numbers right-aligned in tabular figures |
| Form labels | right-aligned column, 78 pt, baseline-aligned with the field |
| Units and symbols | in their own column, secondary colour, never appended with a separator |
| Status lines | facts as separate cells with `gapGroup`; the leading fact is the one a glance needs (revision, connection) |

`Row(spacing: MacMetrics.gapGroup)` and `MacTable` (fixed-width columns,
one gutter) are the two ways to lay out facts; string concatenation is not.

## 4. Editing model (what the inspector must expose)

Every operation is an `EditOp` the compiler already accepts:

| Object | Inspector fields | Op | Kind |
|---|---|---|---|
| Concept | name (inline), description, representation (none / quantity + unit / boolean / count) | Rename, SetDescription, SetRepresentation | refinement; **rebinding** a representation is an edit and the inspector says so |
| Concept | Delete | DeleteConcept | refused while used; banner names the users |
| Mapping | name, description, inputs (add/remove concept), output | Rename, SetDescription, SetSignature | signature change is an edit; inspector shows "will reopen validation of dependents" |
| Mapping | definition: attach formula / replace / detach | AttachDefinition, ReplaceDefinition | attach is a refinement; replace/detach is an edit |
| Mapping | Delete | DeleteMapping | edit |

The inspector shows the refinement/edit classification the compiler
returns (`EditOutcome.kind`) as a one-line note after each change, so the
paper's distinction is visible where the designer acts.

## 5. Sheets teach by showing, not by example text

A creation sheet never carries a sample value as a hint ("Tilt") — that
repeats the paper and tells the designer nothing about what the field
*means*. Instead the sheet renders, live and at canvas fidelity, the node
the entries will become (`NodePreview`, painted by the same `NodePainter`
as the canvas):

* **New concept**: Name · Kind (Quantity / On–off / Count / Decide later) ·
  Unit (when quantity) · Meaning. The preview's socket fills when a
  kind is chosen and stays hollow with *open* when it is not; a one-line
  caption under the preview states what that socket means. The socket is
  grey because its colour is the identity the compiler will allocate.
* **New mapping**: Name · Reads (concept toggles drawn as their sockets, in
  their colours) · Produces. The preview is the mapping node with those
  input sockets, dashed, *declared* — the state it will be in.

Text fields (`MacTextField`): hairline, 5 pt radius, 24 pt; focus = 1 px
accent border + 3 pt soft glow, nothing moves. Push buttons (`MacButton`):
22 pt, ≥ 72 pt wide, primary filled with the accent, secondary hairline,
destructive red text; Cancel left of the default action; Return submits.

## 6. Interaction states — the one standard

Every control in Studio answers hover, press, keyboard focus and disabled
the same way. Implemented once: `MacStates` in `lib/ui/mac/theme.dart`
feeds every Material control theme; `MacInteractive` / `MacLink` in
`lib/ui/mac/interactive.dart` cover rows, links and chips; the canvas
painter applies the same rules to nodes and sockets.

| State | Treatment | Numbers |
|---|---|---|
| **hover** | a flat overlay of the ink colour on the control's own surface; on accent-filled controls the overlay is white so it lightens | 6 % |
| **pressed** | the same overlay, stronger | 12 % |
| **focused** (keyboard only) | 2 pt accent ring; pointer clicks never show it | `accent`, 2 pt |
| **selected** | accent at 20–25 % as the row/segment background; the text stays primary | `selection` token |
| **disabled** | 40 % opacity, no hover, arrow cursor | 0.4 |
| **motion** | one ease-out, everywhere; nothing animates data | 120 ms |

Per control family:

| Family | Rest | Hover | Pressed | Cursor |
|---|---|---|---|---|
| text link (`MacLink`) | accent text, no underline | hover pill behind the text | stronger pill | hand |
| list row (`MacInteractive`) | transparent | hover pill | stronger pill | arrow |
| push / outlined / icon button | hairline or accent fill | overlay | overlay | arrow (macOS convention) |
| segmented control | selected segment raised on `control` | unselected segment gets the hover overlay | — | arrow |
| pop-up (dropdown) | `control` + hairline | `controlHover` | menu opens | arrow |
| text field | `control` + hairline | — | — | I-beam; focus = 2 pt accent border |
| checkbox / switch / slider | Material shapes recoloured to tokens | overlay halo | halo | arrow |
| canvas node | hairline outline | outline in secondary text colour | — | grab |
| canvas socket | filled/hollow by binding | 4 pt halo in the concept colour | drop target: stronger halo | crosshair |
| destructive button | outlined, red text | overlay | overlay | arrow |

Rules that keep it one system: no ripples, no elevation change on hover, no
colour change of text on hover except links (which are already accent),
hover never conveys information that is not also visible at rest, and the
focus ring is the only place the accent appears on a control that is not
selected or primary.

## 8. When the OS dialog cannot be shown

`file_selector` dialogs are hosted by macOS's view-bridge, which refuses
children of sandboxed hosts (Studio launched from an embedded terminal, for
example). A `null` returned faster than a person could cancel is treated
as *refused*: Studio shows a banner explaining it and the Start list gains
*Open by path…* / *New at path…* as a typed fallback.

## 9. Non-goals for this iteration

Contexts, outputs, transports and clock boundaries on the canvas;
simulate/deploy/monitor content; native menu bar; drag-and-drop from the
library; inline formula editor. Each lands with its compiler pass.
