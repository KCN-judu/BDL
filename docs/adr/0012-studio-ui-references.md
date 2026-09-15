# ADR-0012: Studio UI follows Resolve's pages, Blender's node editor, and the macOS HIG

**Status**: accepted (2026-09-15)

## Context
Studio is a whole workflow (design → simulate → deploy → monitor), a node
editor, and a Mac desktop application. Each of those has a mature reference
whose conventions users already know; inventing our own would cost
learnability for no gain. The first scaffold used stock Material widgets and
read as an Android app.

## Decision
* **Window structure**: DaVinci Resolve — one window, pages in workflow
  order on an always-visible bottom page bar, a status line above it,
  project manager left and settings right; each page owns a fixed
  library / centre / inspector arrangement (Fusion page).
* **Canvas**: Blender's node editor — header, coloured sockets left-in /
  right-out, Bézier links dragged socket-to-socket, drop-on-empty discards,
  data flows left → right. BDL-specific twist: socket and link colour is
  the *semantic identity* (nominal type), and a link is accepted only
  between sockets of the same concept, making the typing rule visible.
  Node position is layout only.
* **Look**: Apple HIG for macOS — system font, 13/11/10 pt, flat hairline
  controls, sidebar–content–inspector, accent only for selection and the
  default action, named destructive buttons, no ripples/FAB/snackbar.
  Implemented as a tuned `ThemeData` + `MacTokens` extension, not a
  third-party macOS widget kit (keeps the desktop build plugin-free).

Full specification: `docs/STUDIO_UI.md`.

## Consequences
* Every visual entity maps to an `EditOp` the compiler already accepts;
  the UI never computes semantics (ADR-0001 unchanged).
* Blender's modal keymap (G/R/S) is not required; ⌘ shortcuts are.
* The canvas geometry is a pure function (`buildScene`) and unit-tested;
  interaction state stays in the widget.
* Contexts, outputs, transports and the other pages land with their
  compiler passes, inside this structure.
