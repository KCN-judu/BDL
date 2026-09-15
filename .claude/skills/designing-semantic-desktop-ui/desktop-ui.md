# Desktop conventions: macOS and the professional tools

Sources: Apple Human Interface Guidelines (macOS: windows, sidebars, toolbars,
inspectors, menus, typography, colour, accessibility); Blender manual (node editor);
DaVinci Resolve reference manual (pages, Fusion); Figma, Xcode, Logic Pro, and
parametric CAD tools (Fusion 360 / Onshape / SolidWorks) observed as products. Patterns,
not pixels. Rule: **when a platform convention already solves an ordinary interaction
problem, use it, unless BDL semantics give a concrete reason not to.**

## macOS window anatomy

- **Title bar + toolbar** (unified): the document name, a dirty marker, and the frequent
  verbs (undo/redo, save, build, run). Not every verb — the menu bar holds the complete
  set with shortcuts.
- **Sidebar** (source list): navigation and the inventory of objects, grouped under
  small semibold secondary headers, selectable rows 24 pt, disclosure triangles for
  groups, the selection in the accent tint. 220 pt, resizable, ⌥⌘S.
- **Content**: the document. For Studio, the canvas.
- **Inspector** (trailing): everything editable about the selection, in titled sections
  that can collapse, as forms with right-aligned labels. Multiple inspectors are tabs
  in a strip at the top (Xcode). 260–320 pt.
- **Status area**: bottom, small text, facts about the document and the connection.
- **Sheets** attach to the window for a task with a clear end (create, confirm); they
  do not float centred over the desktop. Cancel on the left of the default button; the
  default responds to Return, Cancel to Esc; destructive default buttons are named.
- **Popovers** for a small transient control anchored to its origin; **panels** for
  tools that stay open (not needed in Studio yet).
- **Banners** for a document-level condition; **alerts** only when data is at risk.

## Controls (use the standard shape)

Push button, pop-up button, segmented control, checkbox, switch, slider, text field,
search field, stepper, disclosure triangle/button, table/outline, tab view, colour
well. Each has known metrics (22 pt regular controls, 5 pt radius, hairline borders,
accent fill on the default button only). Recreate the *look and behaviour* faithfully
(`MacButton`, `MacTextField`, `MacSegmented`, `MacDropdown`) and give each keyboard
focus and a focus ring, because a look-alike that cannot be tabbed to is not the
control.

## Selection and keyboard

- Click selects; ⇧-click extends; ⌘-click toggles; drag on empty space box-selects;
  Esc clears; ⌘A selects all. Selection is the accent tint on rows and the accent
  outline on canvas objects; the *active* object is distinguishable from the rest of a
  multi-selection.
- Arrow keys move through lists and nudge canvas objects; Return activates or edits;
  Space toggles; Tab moves focus through controls in visual order; full keyboard
  access is a system setting the app must honour.
- Every command in a menu; every frequent command with a ⌘ shortcut shown in the menu
  and in tooltips; the contextual menu repeats what applies to the object.

## Typography, colour, materials

- The system font at 13 (body), 11 (subheadline), 10 (caption); larger styles only for
  titles that are actually titles. Label colours at four levels of opacity for
  hierarchy; never grey hex codes chosen per element.
- The accent is the user's; use it for selection, focus, and the default button. System
  semantic colours (red, orange, green) mean what the system means by them.
- Vibrancy/translucency for sidebars is a system material; imitate it lightly or not at
  all, never with a custom gradient.
- Both appearances (light/dark) always; increased-contrast and reduce-motion honoured.

## Progressive disclosure on macOS

Disclosure triangles in outlines and inspector sections; "Show details" in alerts;
extra rows that appear when a choice needs them (the dimension row under "Quantity");
tooltips for shortcuts and full names; a technical-details disclosure under a
diagnostic. The first view is the common case; the detail is one click away, in place.

## What the professional tools teach

**Blender (node editor).** The canvas shows *topology, type and state*: header tint by
category, sockets by type colour and shape (circle values, diamond fields, square
shaders), links as curves, a dashed/dimmed node for muted or invalid. Properties live in
the side panel (N), shown only for the active node. Creation is a searchable menu
(⇧A / F3) at the pointer, not a palette you drag from. Box select, ⇧ extend, active vs
selected outlines, ⌘-drag cut, drop-on-empty discards, frame nodes for grouping,
reroute dots for tidiness. Lesson: *the node is the object; nothing on it is
decorative; everything else is one keystroke away.* What not to copy: a node per
arithmetic operator, the modal G/R/S keymap as a requirement.

**DaVinci Resolve (pages).** One window; pages in workflow order on a bottom bar that
is always visible; every page owns a fixed panel arrangement tuned to its task; the
same clip is the same object on every page; the inspector shows the selection's
properties in every page; a status bar above the page bar. Lesson: *workflow as pages,
not as modes hidden in menus; identity persists across pages; each page's layout is
fixed so spatial memory survives.* What not to copy: dark-only, density tuned for a
colourist, timeline metaphors.

**Figma (selection + inspector).** The canvas is the document; what is selected drives
the right panel entirely; multi-selection shows shared properties and "mixed" values;
measurements appear on hover with ⌥, not permanently; component/instance are
distinguished by colour *and* icon; layers panel and canvas share one selection;
⌘/ quick actions; everything has a shortcut. Lesson: *one selection, three views
(layers, canvas, properties); on-demand measurement; multi-select edits the common
fields.*

**Xcode.** Navigator / editor / inspector with a tab strip of inspectors by concern;
issues shown inline at the line and collected in the issue navigator; a jump bar for
location; disclosure sections in Interface Builder inspectors; documentation on ⌥-click.
Lesson: *diagnostics are anchored to the object and also collected; inspectors are
split by concern; help is a modifier-click away.*

**Logic Pro.** The track header carries its state controls (mute, solo, record) on the
object; the inspector edits the selected region or track; colour is user-assigned
categorisation; automation lanes reveal on demand; the same region appears in the
arrange view and the editors. Lesson: *state controls on the object; colour that is
user-chosen is categorisation, not semantics — Studio's identity hue is the opposite,
compiler-assigned, so it must never be user-editable.*

**Parametric CAD (Fusion / Onshape / SolidWorks).** Constraints are glyphs on the
geometry; geometry refuses moves the constraints forbid; the feature timeline is a
first-class view of history; the browser tree, the canvas and the dialog share one
selection; handles for direct manipulation with exact-value entry beside them. Lesson:
*rules embodied as behaviour of the geometry; history as a view; a handle plus a
number for every manipulable quantity.*

## Where BDL may deviate, and why

| Deviation | Reason |
|---|---|
| socket colour is identity, not data type | the type that matters in BDL is nominal |
| a link cannot form between different concepts | the typing rule made visible |
| no node per operator | formulas are not structure; the canvas is dependency |
| dashed outline for an undefined node | incomplete-but-valid is a first-class state |
| no auto-insert when dragging a node onto a link | that would be a silent semantic edit |
| no mute/bypass on nodes | no BDL meaning |
| layout stored apart from the design | ADR-0003 |

Anything else custom needs a sentence like these in `docs/STUDIO_UI.md`.
