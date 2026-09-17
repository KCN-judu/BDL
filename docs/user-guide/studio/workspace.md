# Workspace

Studio has two screens: the **project manager** (no project open) and the
**workspace** (a project open). The workspace is one window with pages in
workflow order along the bottom.

```text
┌──────────────────────────────────────────────────────────────────────┐
│ ● ● ●   lamp  Edited                  ↶ ↷        Save                │  toolbar
├──────────────┬─────────────────────────────────────┬─────────────────┤
│ Sidebar      │                                     │ Inspector       │
│  Project │   │           page content              │  the selected   │
│  Library     │                                     │  object         │
├──────────────┴─────────────────────────────────────┴─────────────────┤
│ Saved   2 concepts  3 mappings  1 not yet defined     Compiler 0.1.0 │  status line
├──────────────────────────────────────────────────────────────────────┤
│ ⊞        Design      Simulate      Deploy      Monitor            ⚙ │  page bar
└──────────────────────────────────────────────────────────────────────┘
```

<!-- figure F2 -->

## Regions

**Toolbar** — the project's name (followed by _Edited_ while there are unsaved
changes), Undo and Redo, and Save (enabled only while edited).

**Sidebar** (left) — on the Design page, two tabs: **Project**, the project's
objects by kind (_Concepts_, _Mappings_, _Timing domains_, _Outputs_; in a
system project also _Components_, _Instances_, _Behaviors_), each with a **+**
to create one; and **Library**, the ready-made concept templates. See
[Library](library.md). The Simulate and Deploy pages put their own controls
here.

**Page content** (centre) — the canvas on Design, the trace on Simulate, the
verdict on Deploy.

**Inspector** (right) — the selected object: what it means, what you can change,
what a change affects. See [Inspector](inspector.md).

**Status line** — the document state (_Saved_ / _Edited_), then counts and what
is still open (_1 not yet defined_, _outputs incomplete_, _not causal_, _reads
across domains_, _N unsaved definitions_), then the compiler connection. Errors
from a refused action appear as a **banner** above the page content with a
_Dismiss_ link — never as a dialog.

**Page bar** — the pages, in the order you use them:

| Page         | Answers                                                                                       | Shortcut |
| ------------ | --------------------------------------------------------------------------------------------- | -------- |
| **Design**   | what does the product do                                                                      | ⌘1       |
| **Simulate** | what does it do over time                                                                     | ⌘2       |
| **Deploy**   | does it fit this board                                                                        | ⌘3       |
| **Monitor**  | what is it doing right now — _not built yet_; the page says live values arrive with telemetry | ⌘4       |

The button at the left of the page bar (⊞) closes the project and returns to the
project manager. The gear at the right is labelled _Project settings_ but has no
settings page yet; while the compiler is disconnected it reconnects.

## Saving, undo, revisions

**⌘S** or **Save** writes the project to its folder. A design change is recorded
as a new _revision_ of the project; **⌘Z** / **⇧⌘Z** step through them. Moving
nodes, resizing panels and collapsing groups are not design changes: they are
saved but make no revision. A typed formula that has not been added is a _draft_
— it is kept when you switch selection, switch pages, or even close the project,
and the status line counts it as _unsaved definition_ until you add or revert
it.

## The three levels of information

Studio shows a design at three levels, and keeps them apart:

1. **Canvas** — structure you can see: shape, colour, links, dashed outlines, a
   red mark. No type names, no identifiers.
2. **Inspector** — designer vocabulary: _Meaning_, _Value_, _Reads_, _Produces_,
   _Updates in_, _Drives_; findings in product language beside the field they
   concern.
3. **Explain** — a collapsed disclosure at the end of every inspector: the
   identity numbers, the formal interface and terms, diagnostic codes, the
   revision. Nothing in the first two levels depends on it.

This guide follows the same order: it explains the canvas and the inspector, and
points to _Explain_ and the technical documentation for the rest.

## Related

[Canvas](canvas.md) · [Library](library.md) · [Inspector](inspector.md) ·
[Keyboard and mouse](../reference/keyboard-and-mouse.md)
