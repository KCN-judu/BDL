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

The **status line** (Resolve's Fusion status bar) leads with the document
state (*Saved* / *Edited*), then a count summary (concepts, mappings, how
many are not yet defined, how many definitions do not check) and the
compiler connection (*Compiler 0.1.0*; the protocol version only when it
mismatches). The revision counter is not a designer fact and lives in
Explain. Errors from requests appear as a banner above the page content,
never as a modal.

## 2. Node canvas (Blender)

### Anatomy

```
   ●────[ Tilt ]────●                       concept: one row — name, in-socket, out-socket

        ┌─────────────────────────┐
        │ dimByTilt        declared│   header: title · the one state word (only while declared)
   ●────┤ Tilt                     │   input socket per read concept (left), hue = identity
   ◆────┤ Held          Brightness ├────●   output socket (right)
        │ ● clamp(0.2 + 0.8·θ/60°) │   definition region: summary line; red mark = does not check
        └─────────────────────────┘
```

* **Header colour** = category: concept (grey-blue, the whole one-row
  object), mapping (blue strip), context (violet), output (amber),
  transport (teal). Muted, low-saturation, with the title in 12.5 pt
  semibold. Identity hues are the only saturated marks on the canvas.
* **Socket colour = semantic identity.** In Blender a socket's colour is its
  data type; in BDL the type that matters is the nominal concept, so each
  concept gets a stable hue derived from its `SemanticId` (deterministic,
  never from the name; lightness chosen per hue so every identity clears
  3:1 against the canvas in both appearances). A link is only accepted
  between sockets of the same concept — the canvas shows the nominal typing
  rule without a diagnostic: while a link is dragged every compatible
  socket gains a faint halo, the one under the pointer a strong halo, and
  an incompatible socket shows the forbidden cursor.
* **Socket shape = value form.** Quantity ○, on–off ◇, count □; a concept
  whose value form is not chosen yet is a hollow ring. The same glyph, drawn
  by the same code, appears in the library, in chips, toggles and pop-ups.
  No type words are written beside a socket anywhere.
* **Concept node**: a single row — the name, an input socket on the left (a
  mapping producing this concept connects here) and an output socket on
  the right (values of this concept flow out). Nothing else: what it
  measures and what it means are the inspector's.
* **Mapping node**: one input socket per read concept (labelled), one output
  socket, and a definition region below the sockets. The region holds the
  definition's summary line, or nothing while declared. A definition the
  compiler cannot accept gets a **red mark at the definition line** — where
  the problem lives — and no word in the header. A declared mapping is
  drawn with a dashed outline and the word *declared*; dashed survives
  selection (accent changes the colour, never the meaning). Never red.
* **Links** are cubic Béziers from an output socket (right edge) to an
  input socket (left edge), tangents horizontal, colour of the concept,
  2 px, selected links thicker. Data flows left → right.
* **Empty canvas**: one tertiary line naming the first step (add a concept
  from the Library). Painted nodes expose semantics in product language for
  assistive technology.

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

Every operation is an `EditOp` the compiler already accepts. The inspector
is organised by what the designer means, not by the model's fields:

| Object | Section | Fields (designer words) | Op | Kind |
|---|---|---|---|---|
| Concept | **Meaning** | Name, Meaning | Rename, SetDescription | refinement |
| Concept | **Value** | Quantity / On–off / Count / Decide later; Unit (angle, length, … with the symbol in its own column) | SetRepresentation | choosing is a refinement; **changing a chosen value form** is an edit, and the section says which relationships it re-checks |
| Concept | **Relationships** | Produced by, Used by — names as links that select the mapping | — | — |
| Concept | Delete <name> | disabled while used, with the users named *at rest* under the button | DeleteConcept | — |
| Mapping | **Meaning** | Name, Meaning | Rename, SetDescription | refinement |
| Mapping | **Reads** | chips with the socket glyph, removable; a pop-up to add | SetSignature | edit |
| Mapping | **Produces** | pop-up with the socket glyph | SetSignature | edit |
| Mapping | **Relationship** | the definition editor (§4a) — *Add definition* / *Save definition* / *Revert* / *Detach definition*; header word *declared* while empty, *unsaved* while a draft differs; findings about the mapping’s place in the design attached under the editor in product language | AttachDefinition, ReplaceDefinition (chosen by the reducer from the committed state, never by the widget) | add is a refinement; save (replace) and detach are edits |
| Mapping | Delete <name> | — | DeleteMapping | edit |
| both | **Explain** (collapsed) | `SemanticId` / `DeclId`, `Θ` / `Interface`, inferred type, core term, status enum, diagnostic codes and technical detail, revision, the last change's kind and invalidation categories | — | level 3 only |

No static explanatory paragraphs: a sentence appears only when it is
about *this* object *now* ("Changing this re-checks dimByTilt, warmPulse";
"Checked once Temperature's value is decided").

**Last change** (below the inspector) states the consequence as a sentence
about other objects: *Nothing else needs rechecking.* for a refinement;
*This change affects dimByTilt, warmPulse — they will be checked again.*
for an edit, the names being links (from `EditOutcome.origin_decls`). The
words *refinement* / *edit* and the `Invalidation` categories are in Explain.

Words never shown at levels 1–2: `SemanticId`, `DeclId`, `Ty`, `q Dim`,
`Grant`, `Interface`, `realization`, `invalidation`, `Clocked`,
`SingleDriver`, `DriveEnv`, `RequirementId`, `solver`, `protocol`,
`revision`, and any enum name.

### 4a. The definition editor

The formula field is not a text box wired to an `EditOp`; it is an
authoring surface over the compiler's verdict (`lib/ui/definition_editor.dart`,
docs/STUDIO_COMPILER_INTEGRATION.md). Studio owns the draft — its text,
base revision, generation — and the compiler judges it as the designer
types (`AnalyzeDefinitionDraft`, debounced 150 ms, read-only). The project
changes only on *Add definition* (no committed definition) / *Save
definition* (there is one), or *Detach definition*.

What is shown, in rank order: one status line under the field (dot for
tone, words for meaning: *Checking…*, *Valid definition*, *Tilt has no
representation yet.*, the first error's message, *Not saved: …*); the
offending spans underlined in the field, each repeated as a diagnostic row
with excerpt, explanation and fixes; the names in scope as the field's
hint (*expression over Tilt, Held*); *unsaved* in the section header and
*Revert* beside the primary button while draft and committed differ; and,
when the committed definition changed under a dirty draft, a notice with
*Reload* / *Keep mine* — never a silent overwrite. Open is orange and
worded as what is still to decide; only *Invalid* is red.

Keys in the field: ⌘↩ saves the definition while it is dirty; ⌘S keeps
its meaning, *Save project*, and never commits a draft; Esc reverts;
Return inserts a line. The
canvas draws committed state only; the status line counts *N unsaved
definitions*. Closing a project stashes dirty drafts by path and reopening
restores them — no modal.

## 5. Sheets teach by showing, not by example text

A creation sheet never carries a sample value as a hint ("Tilt") — that
repeats the paper and tells the designer nothing about what the field
*means*. Instead the sheet renders, live and at canvas fidelity, the node
the entries will become (`NodePreview`, painted by the same `NodePainter`
as the canvas):

* **New concept**: Name · Value (Quantity / On–off / Count / Decide later) ·
  Unit (when quantity; the same table as the inspector) · Meaning. The
  preview's socket takes the chosen shape (○ ◇ □) and stays a hollow ring
  while the value is undecided; a one-line caption under the preview names
  that mark and what it means. The socket is grey because its colour is the
  identity the compiler will allocate.
* **New mapping**: Name · Reads (concept toggles drawn with their socket
  glyphs, in their colours) · Produces. **Produces has no default** — the
  pop-up reads *choose*, the preview's output socket is a hollow neutral
  ring, and Create stays disabled until a concept is chosen; a mapping that
  "produces" the first concept in the list would otherwise be created
  without anyone deciding so. The preview is the mapping node with those
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

## 7. Three information levels and the semantic UI matrix

The semantics of a design are shown as *structure* first, explained in
*prose* second, and named in *formal vocabulary* only on demand. Every
semantic fact is designed for exactly one primary level; a fact may echo at
the next level only as detail behind the first, never as a duplicate badge.

| Level | Answers | May use | May not use |
|---|---|---|---|
| **1 Canvas** | what does the product do; what depends on what; what is still open; where does behaviour become physical | object silhouette, socket shape, socket hue, links, grouping, containment, line style, the object's own state; one state word where a word is unavoidable | type labels, ids, compiler words, badges, counts |
| **2 Inspector** | what does the selected object mean; what can I change; what will the change affect | designer vocabulary: *Meaning, Value, Unit, Reads, Produces, Relationship, Used by, Produced by, affects, checked again*; diagnostics in product language attached to the field they concern | `SemanticId`, `DeclId`, `Ty`, `Grant`, `Interface`, `realization`, `invalidation`, `Clocked`, `SingleDriver`, `DriveEnv`, `solver`, `protocol`, `revision`, enum names |
| **3 Explain** | why was this accepted or refused; what did the surface form elaborate into; which rule applies | all of the above, kernel notation, Core IR, diagnostic codes, technical details, revision | — |

Level 3 is one collapsed disclosure, **Explain**, at the end of the
inspector, and the technical part of a diagnostic. It is never open by
default and nothing in levels 1–2 depends on it.

### The matrix

*now* = implemented; *spec* = agreed here, drawn when its compiler pass
lands. "Canvas" is level 1, "Inspector" level 2, "Explain" level 3.

| Semantic fact | Internal representation | Canvas | Inspector wording | Explain wording | |
|---|---|---|---|---|---|
| semantic identity | `SemanticId` | socket and link **hue** from the id, identical on every page; the name at every socket | the name; never a number | `SemanticId 3` | now |
| representation | `Θ s = q d / bool / nat` | socket **shape**: ○ quantity, ◇ on–off, □ count | Value: Quantity / On–off / Count · Unit: angle, length, … with the symbol in its own column | `Θ(3) = q[rad]` | now |
| representation not chosen | `Θ s = none` | **hollow** socket ring | Value: Decide later; "relationships can already use it" | `Θ(3) = none` | now |
| unresolved declaration | `realization = none` | **dashed** outline, empty definition region, header word *declared* | Relationship: empty field + Attach | `Δ(d).realization = none` | now |
| mapping relationship | `Signature { inputs, output }` → `Interface` | one input socket per read concept on the left, one output socket on the right, links in the concepts' hues | Reads · Produces (chips and pop-up carry the socket glyph) | `Interface: sem#0 → sem#3 → sem#1` | now |
| semantic construction | `mk s` under `Grant.of τ` | a link forms only between sockets of one hue; the output socket is the produced concept | Produces | `Grant permits mk sem#1 in this realization` | now (grant is invisible by design) |
| dimension mismatch | `Prim.ty` fails | a **red mark at the formula line** on the node, nothing in the header | under the formula: "This adds an angle and a time." + fixes | `+ : q[rad] → q[rad] → …, found q[s]`, code | now |
| waiting on an open value | `MappingStatus.OPEN` | solid node whose read socket is hollow | under the formula: "Checked once *Temperature*'s value is decided." | status enum | now |
| temporal state | `delay init e` | **register mark** on the link that crosses a tick, initial value beside it | "Remembers *Held*, starting at *no*" | `delay false (declRef d)` | spec |
| clock / domain | `Κ d = some c` | **lane**: labelled background region; domain-free mappings outside | "Updates with *interaction* (50 Hz)" | `Κ(d) = c₀`, `Clocked` | spec |
| cross-domain observation | `sync src init e` | **gate** on the link at the lane edge with the initial value; a crossing without a gate is broken at the boundary | "Observes the latest *Temperature*, starting at 20 °C" | `sync c₁ 293.15 (declRef d)` | spec |
| physical output | `OutputId`, `OutputSpec { accepts, clock }` | **terminal node** in the right-most column: flat right edge, one input socket, device name | Output: accepts (Value), updates with, device, final target | `Ω(o) = ⟨q[1], c₀⟩` | spec |
| output conflict | `SingleDriver β` fails | the second path **cannot converge**: the socket refuses the drop and names the current target; two existing targets are drawn meeting a red gap before the output | "*Light* already has a final target, *dimByTilt*. Combine the values before the output." | `β d₁ = β d₂ = o` | spec |
| hardware feasibility | `solve` result | Deploy: board picture; each requirement a lead to a pin; an unsatisfied one has no lead | "Needs 7 PWM; this board has 6: D3 M1, D5 M2, …" | `Explanation::Blocked { blockers }` | spec |
| deployment allocation | `Assignment` | lead from requirement to pin; pinned ones marked | Light → PWM → GP15 | resource ids | spec |
| runtime value | telemetry by `DeclId` + activation | **number at the socket**, unit column; ◇ filled/empty; stale fades | Tilt 31.4° · Held yes | `DeclId 4 @ activation 1203` | spec |
| change consequence | `EditOutcome { kind, invalidates }` | — | "Nothing else needs rechecking." or "This change affects *dimByTilt*, *warmPulse*; they will be checked again." | `refinement` / `edit`, `Invalidation::{…}` | now |

Rules the matrix implies: hue is identity and nothing else; shape is
representation and nothing else; dashed is *declared* and nothing else; a red
mark means *wrong now* and never *not yet*; the accent is selection, focus and
the default button. Adding an encoding adds a row here.

### What Studio reads, and what it still needs from the read model

Studio computes nothing semantic (ADR-0001). Everything above is read off
`ProjectProjection`, `ProjectAnalysis` and `EditOutcome`. Two facts it
shows today are derived structurally from the projection, not judged:
which mappings read or produce a concept (from signatures), and which read
concept of an `OPEN` mapping still has no value form (from `Θ`). Facts the
UI wants and the protocol does not yet carry — to be added by the compiler
side, never invented in Dart:

| Needed for | Field | Today |
|---|---|---|
| "This change affects *A*, *B*" for signature edits and detaches | dependents (by `DeclId`) in `EditOutcome`, not only the origin | `origin_decls` names the origin; a mapping edit lists only itself |
| concept-level findings under the Value section | `Diagnostic.entity = concept_id` populated by the checker | shape exists, unused |
| the ladder rungs beyond *type-valid* on the mapping node | `MappingStatus` extended (temporally valid, clock-consistent) or a separate per-rung projection | four statuses |
| contexts, outputs, clock domains, transports on the canvas | their projections (`ContextView`, `OutputView`, `DomainView`, drive edges, `sync` sites with initial values) | none |
| values at sockets (Simulate, Monitor) | per-`DeclId`, per-activation samples with units | none |
| board picture and leads (Deploy) | `Assignment` and `Explanation` projections keyed by requirement and resource | none |

## 8. When the OS dialog cannot be shown

`file_selector` dialogs are hosted by macOS's view-bridge, which refuses
children of sandboxed hosts (Studio launched from an embedded terminal, for
example). A `null` returned faster than a person could cancel is treated
as *refused*: Studio shows a banner explaining it and the Start list gains
*Open by path…* / *New at path…* as a typed fallback.

## 9. Non-goals for this iteration

Contexts, outputs, transports and clock boundaries on the canvas;
simulate/deploy/monitor content; native menu bar; drag-and-drop from the
library; draft indication on the canvas. Each lands with its compiler
pass (docs/STUDIO_COMPILER_INTEGRATION.md §3 places them).
