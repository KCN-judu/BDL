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

Pages, in workflow order (the paper's own sequence: intent → refine → time →
contexts → outputs → domains → board → observe):

| Page         | Centre                                                                                                                                                                                          | Left                                                                                                                                                           | Right                                                       | Answers                    |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- | -------------------------- |
| **Design**   | the project as **Design** (node canvas), **Code** (its source files) or **Split** (both) — §12                                                                                                  | sidebar: _Project_ tab (concepts, mappings, timing domains, outputs; _Contexts_ and _Components_ are empty headings) and _Library_ tab (concept templates, §2) | inspector of the selection                                  | what the product does      |
| **Simulate** | readiness blockers with _Show_ links, Step / Step ×10 / Reset, the trace as a table (tick, active domains, one column per relationship without inputs and per driven output); value plots later | inputs as controls by value form; a period per timing domain                                                                                                   | probe of the selection: value now and over the run, Explain | what it does over time     |
| **Deploy**   | one verdict _for this board_, the placement device → requirement → pin, the dead end in the solver's terms; a board picture later                                                               | boards from bdld; devices, edited in place                                                                                                                     | —                                                           | whether it fits            |
| **Monitor**  | _spec_: the same canvas with live values; today a placeholder page (`placeholder_page.dart`) until telemetry exists (ROADMAP step S)                                                            | telemetry sessions                                                                                                                                             | probe inspector                                             | what it is doing right now |

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
header word precedence is `declared` › port word › sink state › `required`._

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
  domain is the empty product. Studio asks `Signature.isUnitDomain`
  (`app/state.dart`) wherever "read as a value" matters (simulation inputs,
  output drivers, the value/rule word), never a separate kind of node.
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
  problem lives — and no word in the header. A declared mapping is drawn with a
  dashed outline and the word _declared_; dashed survives selection (accent
  changes the colour, never the meaning). Never red.
- **Links** are cubic Béziers from an output socket (right edge) to an input
  socket (left edge), tangents horizontal, colour of the concept, 2 px, selected
  links thicker. Data flows left → right.
- **Empty canvas**: one tertiary line naming the first step (add a concept from
  the Library). Painted nodes expose semantics in product language for assistive
  technology.

### Interaction (Blender's, on a Mac)

| Gesture                                                   | Result                                                                                                                                                                            | Blender equivalent |
| --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------ |
| drag empty canvas                                         | pan                                                                                                                                                                               | MMB drag           |
| scroll / pinch                                            | zoom about the pointer                                                                                                                                                            | wheel / ctrl-MMB   |
| click node                                                | select (active); ⇧-click adds                                                                                                                                                     | LMB / ⇧LMB         |
| drag node body                                            | move selected nodes; on release the layout is committed (no revision)                                                                                                             | G                  |
| drag from output socket to input socket                   | make link (edit: adds the concept to the mapping's inputs, or sets the mapping's output)                                                                                          | LMB drag           |
| drag from an input socket away and release on empty space | disconnect                                                                                                                                                                        | drag link away     |
| drop link on empty space                                  | discard (no auto-create)                                                                                                                                                          | discard            |
| ⌘-drag across links                                       | cut links                                                                                                                                                                         | Ctrl-RMB cut       |
| drag on empty canvas with ⇧                               | box select                                                                                                                                                                        | B                  |
| ⌫ / Delete                                                | delete selection (concept in use → banner: "_Tilt_ is used by _dimByTilt_")                                                                                                       | X                  |
| ⌘A / ⌘D                                                   | select all / duplicate (later)                                                                                                                                                    | A / ⇧D             |
| Home / ⌘0                                                 | frame all                                                                                                                                                                         | Home               |
| ⌘F                                                        | find node by name                                                                                                                                                                 | Ctrl-F             |
| double-click node header                                  | rename inline                                                                                                                                                                     | —                  |
| H                                                         | collapse selected nodes                                                                                                                                                           | H                  |
| right-click                                               | context menu: Rename · Delete over a node; **Add Concept ▸** Recent / Input / Output / Environment / Geometry & motion / Human interaction / More… (docs/spec/concept-library.md) | RMB                |
| drop a Library row                                        | insert that concept template at the drop point                                                                                                                                    | —                  |

Dragging a node over a link does **not** auto-insert it (Blender's auto attach);
BDL links are typed by concept, and silent insertion would be a semantic edit.
Muting nodes (M) has no BDL meaning and is not offered.

### What the canvas never means

Edges are dependency, not execution order. Drawing order does not set output
priority. Node position is layout only (ADR-0003): the canvas draws every node
where the layout puts it and arranges nothing itself — an entity without a
position is placed by the daemon's layout service on open and on commit
(ADR-0023 §7, `docs/architecture/overview.md`), and the projection carries the
result. No node type exists per arithmetic operator — formulas live in the
inspector.

### Create-then-rename (concept templates)

Inserting a concept template — from the right-click menu, a drag from the
Library tab, or the tab's rows — is one request (`InstantiateConceptTemplate`);
when the projection with the new concept arrives, the node lands where the
pointer was (layout, never a revision), is selected, and its name opens for
editing on the canvas with the default name selected: Return commits a rename
edit, Esc keeps the default, leaving the field commits what was typed (Finder's
new-folder behaviour). Add _Temperature_, type _MotorTemperature_, keep
designing — no sheet for routine insertion.

The sidebar has two tabs: **Project** (the project's objects, "+" per section)
and **Library** (search, categories, rows with a grey socket glyph — filled when
the template chooses a representation, hollow for _decide later_; the unit or
kind word in the right column; the description on hover). Recent templates in
the menu are a Studio preference, never project state.

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

| Object                      | Section                  | Fields (designer words)                                                                                                                                                                                                                                                                                                                     | Op                                                                                                        | Kind                                                                                                                         |
| --------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Concept                     | **Meaning**              | Name, Meaning                                                                                                                                                                                                                                                                                                                               | Rename, SetDescription                                                                                    | refinement                                                                                                                   |
| Concept                     | **Value**                | a pop-up — Quantity / On–off / Count / Collection of… / Grouped value / Optional… / Decide later — then what the form needs: Unit (angle, length, … with the symbol in its own column), _Each_ (a collection's element form), _When present_, _First_ / _Second_ (nested editors of the same kind); _Order_ (checkbox, quantity forms only) | SetRepresentation                                                                                         | choosing is a refinement; **changing a chosen value form** is an edit, and the section says which relationships it re-checks |
| Concept                     | **Relationships**        | Produced by, Used by — names as links that select the mapping                                                                                                                                                                                                                                                                               | —                                                                                                         | —                                                                                                                            |
| Concept                     | Delete `<name>`          | disabled while used, with the users named _at rest_ under the button                                                                                                                                                                                                                                                                        | DeleteConcept                                                                                             | —                                                                                                                            |
| Mapping                     | **Meaning**              | Name, Meaning                                                                                                                                                                                                                                                                                                                               | Rename, SetDescription                                                                                    | refinement                                                                                                                   |
| Mapping                     | **Reads**                | chips with the socket glyph, removable; a pop-up to add                                                                                                                                                                                                                                                                                     | SetSignature                                                                                              | edit                                                                                                                         |
| Mapping                     | **Produces**             | pop-up with the socket glyph                                                                                                                                                                                                                                                                                                                | SetSignature                                                                                              | edit                                                                                                                         |
| Mapping                     | **Relationship**         | the definition editor (§4a) — _Add definition_ / _Save definition_ / _Revert_ / _Detach definition_; header word _declared_ while empty, _unsaved_ while a draft differs; findings about the mapping’s place in the design attached under the editor in product language                                                                    | AttachDefinition, ReplaceDefinition (chosen by the reducer from the committed state, never by the widget) | add is a refinement; save (replace) and detach are edits                                                                     |
| Mapping                     | Delete `<name>`          | —                                                                                                                                                                                                                                                                                                                                           | DeleteMapping                                                                                             | edit                                                                                                                         |
| Mapping                     | **Timing**               | _Updates in_: a domain, or _any_ (pure); `clock.*` findings under it                                                                                                                                                                                                                                                                        | SetMappingClock                                                                                           | edit (Clock)                                                                                                                 |
| Mapping                     | **Drives**               | the output this relationship is the final target of, or none; drive findings under it                                                                                                                                                                                                                                                       | SetMappingDrive                                                                                           | edit (Output)                                                                                                                |
| Timing domain (library row) | —                        | Name (inline), create from the section's "+", delete while unused                                                                                                                                                                                                                                                                           | CreateClockDomain, RenameClockDomain, DeleteClockDomain                                                   | —                                                                                                                            |
| Output                      | **Meaning** · **Output** | Name, Meaning; Accepts (concept, with glyph), _Updates in_, Required                                                                                                                                                                                                                                                                        | RenameOutput, SetOutputAccepts, SetOutputClock, SetOutputRequired                                         | edit (Output)                                                                                                                |
| Output                      | **Driver**               | the driver or _none_; claimants while contested; Connect (pop-up of eligible relationships) / disconnect                                                                                                                                                                                                                                    | SetMappingDrive                                                                                           | edit (Output)                                                                                                                |
| Output                      | Delete `<name>`          | —                                                                                                                                                                                                                                                                                                                                           | DeleteOutput                                                                                              | —                                                                                                                            |
| Device (Deploy page, left)  | row edited in place      | Name, kind (pop-up), output (pop-up), one pin field per requirement, Remove                                                                                                                                                                                                                                                                 | CreateDevice, RenameDevice, SetDeviceKind, SetDeviceOutput, SetDevicePin, DeleteDevice                    | Deployment only                                                                                                              |
| Mapping, Output             | **Fixes**                | the service's actions for the selection: ready → button, needs a choice → pop-up, blocked → the reason                                                                                                                                                                                                                                      | `ListSemanticActions`; edits applied one revision at a time                                               | —                                                                                                                            |
| both                        | **Explain** (collapsed)  | `SemanticId` / `DeclId`, `Θ` / `Interface`, inferred type, core term, status enum, diagnostic codes and technical detail, revision, the last change's kind and invalidation categories                                                                                                                                                      | —                                                                                                         | level 3 only                                                                                                                 |

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
name.

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

Keys in the field: ⌘↩ saves the definition while it is dirty; ⌘S keeps its
meaning, _Save project_, and never commits a draft; Esc reverts; Return inserts
a line. The canvas draws committed state only; the status line counts _N unsaved
definitions_. Closing a project stashes dirty drafts by path and reopening
restores them — no modal.

### 4b. The Formula view

The definition editor has two projections of the one draft, chosen with a
**Formula | Text** segmented control at its top (`lib/ui/formula_composer.dart`;
ADR-0028): the Text view is the field of §4a; the Formula view draws the
compiler's `FormulaProjection` as the expression it is —
`clamp( [Tilt] ÷ [90][deg ▾], [0], [1] )` — never as a graph inside the node.
Studio owns the mode, the selected component and the open pop-up; the compiler
owns the tree, every expected type, every candidate and the text every action
makes (`ComposeFormula` → `DefinitionDraftChanged` → the ordinary verdict).

Encodings, by channel (the table in §3 gains these rows): a **slot** — an
expression not yet written, `?` in the text — is a dashed hollow chip (dashed =
not decided, as the canvas's _declared_ node); a **reference** is a chip with
its concept's socket glyph (shape = kind of value, as on the canvas); a
**literal** is two fields, the coordinate and a unit pop-up listing the
compiler's units for the literal's own dimension — the pop-up switches the unit
and keeps the quantity (`180 deg` → `3.141592653589793 rad`); the coordinate
field is the other action (a new quantity, the same unit). Only a literal has a
unit pop-up: a reference's kind is its declaration's, and the Composer never
rewrites it. **Operators** are their glyphs (× ÷ − ≤ ≥ ≠), a **call** its name
and parentheses, an **unsupported form** (`if`, `match`, a block, a rule, a
collection or grouped literal, `delay`/`sync`) its text in monospace, selectable
and edited as text. A **binder** (`all reading in readings: …`) is a head row —
the word in bold, the local as an italic chip with a lighter frame, `in`, the
collection, the colon — over its body indented beneath it; the local's uses in
the body are the same italic chip, so what the formula itself binds is told
apart from what the design provides (a design reference stays upright with its
socket glyph); selecting the declaration selects the binder and the panel says
_angle is each element: an angle._ A **range** is its two ends around a `..`
glyph; each end is an ordinary component (a literal end has its own unit pop-up,
an empty end a slot expecting the subject's kind). Selection is the selection
tint; keyboard focus the accent ring; a finding is a red underline on the
component _and_ its row under the field — one diagnostic, two projections.

Beneath the field, for the selected component: the compiler's sentence —
_Expected: an angle, because an angle ÷ an angle = a dimensionless quantity._ —
with the kernel's notation behind **Explain**; for a slot, a number entry whose
unit pop-up holds the units of the expected dimension (none for a dimensionless
slot; none, with a sentence, when the position is not determined), then
_References_ by type and _Equations_ folded; for a component, **+ − × ÷**,
**Compare**, **Function** (the equations whose result fits, wrapping the
component as the first argument), **Each element** (all / any / map / filter —
offered when the component is a collection or its kind is unknown; the compiler
picks the local's name), **Range** (`… in ? .. ?`, not offered on a truth value)
and **Remove**. Keys: Tab across components in reading order; on a selected
component `+ − * /` and ⌫; digits and Return in a number entry; Esc clears the
selection. The stale-projection policy (`app/composer.dart` `composerInSync`): a
projection is current only when it is of exactly the text on screen and that
text parsed; otherwise the field is dimmed with a notice — _Waiting for the
compiler to read the formula…_ while the verdict for this text is on its way,
_The text cannot be read as a formula._ when it never will be (then no tree is
shown: none is invented) — no component answers a click, no slot panel opens, no
key acts, and the reducer refuses a structured action (or a second one in
flight) and discards an answer for text that has moved on; **Edit as text** is
the way out. Save, revert, conflict and detach are §4a's, unchanged: a formula
with a slot may be saved and is _invalid_ until filled.

Screenshot: `docs/user-guide/assets/studio/formula-composer.png`
(`docs/user-guide/screenshots/manifest.json`, `formula-composer`).

## 5. Sheets teach by showing, not by example text

A creation sheet never carries a sample value as a hint ("Tilt") — that repeats
the paper and tells the designer nothing about what the field _means_. Instead
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
| semantic construction                       | `mk s` under `Grant.of τ`                                                                 | a link forms only between sockets of one hue; the output socket is the produced concept                                                                                                                                                                                                                                                                                                | Produces                                                                                                                                                                                                                             | `Grant permits mk sem#1 in this realization`                                            | now (grant is invisible by design)                                                                                                                                                   |
| dimension mismatch                          | `Prim.ty` fails                                                                           | a **red mark at the formula line** on the node, nothing in the header                                                                                                                                                                                                                                                                                                                  | under the formula: "This adds an angle and a time." + fixes                                                                                                                                                                          | `+ : q[rad] → q[rad] → …, found q[s]`, code                                             | now                                                                                                                                                                                  |
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
indication on the canvas; the device name on a sink node.
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
│ Inputs        │ [Step] [Step ×10] [Reset]   tick 3     │ Probe             │
│ ● tilt  Tilt  │ ● held needs a value before simulation │ brightness   ●    │
│   [0.5  ] rad │   can step.  Show                      │   Now   0.333     │
│ ◆ held  Held  │ tick active       tilt  brightness     │ Over the run      │
│   [off]       │   0  interaction  0.52  Brightness(…)  │   0  0.333        │
│ Timing domains│   1  interaction  0.52  Brightness(…)  │ ▸ Explain         │
│ ↻ interaction │                                        │                   │
│   every [1]   │                                        │                   │
└───────────────┴────────────────────────────────────────┴───────────────────┘
```

- **Inputs** are the unresolved value declarations — mappings that read nothing
  and have no definition (`I d t`, DI-16). Each gets a control from the
  concept's _value form_, never from its name: a number with the unit beside it
  (quantity), a switch (on–off), a whole number (count). The row carries the
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
  is wrong. Entering the page never starts a run.
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
  object's glyph; an output shows its final target. Explain holds `DeclId`, the
  run's revision, the rendered value and an error's code and technical text.
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
object, a plain monospace editor over the daemon's sources, one file at a time
(a pop-up names the others; a draft file is labelled _— not built_). (4) The
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

**States.** No project: the page's empty state. Sources not yet here: _Reading
the sources…_. Incomplete-but-valid (open faults): listed with the hollow ring,
the graph in step. Not building: the banners and the list. Disconnected: the
editor is read-only.

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
jump stand in); completion and hover in the Code pane (the language server has
them; the pane does not yet ask); a _Format_ command (`bdl-ide::format` exists);
creating a second source file from Studio (a file made by an external editor
appears on reload).
