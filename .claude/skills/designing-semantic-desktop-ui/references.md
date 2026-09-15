# References

Sources studied for this skill, and where each idea landed. No text is reproduced;
principles are restated in the skill's own words.

## Graphic and layout design
- Josef Müller-Brockmann, *Grid Systems in Graphic Design* — `layout-and-grid.md`.
- Ellen Lupton, *Thinking with Type* — `typography.md`.
- Adam Wathan & Steve Schoger, *Refactoring UI* — `layout-and-grid.md`,
  `typography.md`, `anti-patterns.md` (spacing, borders, hierarchy, colour restraint).
- William Lidwell, Kritina Holden, Jill Butler, *Universal Principles of Design* — used
  as an index (alignment, chunking, constraint, Fitts' law, Hick's law, mapping,
  progressive disclosure, recognition over recall, signal-to-noise, uniform
  connectedness, visibility). Consult when a principle needs a second name.

## Interaction design
- Don Norman, *The Design of Everyday Things* — `interaction-design.md` (affordance,
  signifier, mapping, constraint, feedback, conceptual model, gulfs, slips/mistakes).
- Alan Cooper, Robert Reimann, David Cronin, Christopher Noessel, *About Face* —
  `interaction-design.md` (goal-directed design, posture, perpetual intermediates,
  object–verb, direct manipulation, modeless feedback, excise, considerate software).
- Jenifer Tidwell, Charles Brewer, Aynne Valencia, *Designing Interfaces* —
  `interaction-design.md` (the pattern table), `node-editor.md`.
- Steve Krug, *Don't Make Me Think* — `interaction-design.md`, `critique-checklist.md`.

## Perception and visualisation
- Gestalt psychology (Wertheimer, Köhler, Koffka; Palmer & Rock on uniform
  connectedness and common region) and the Nielsen Norman Group's articles on the
  Gestalt principles and visual hierarchy — `gestalt.md`.
- Colin Ware, *Information Visualization: Perception for Design* — `gestalt.md`
  (preattentive attributes, channel effectiveness, colour for categories, visual
  working memory, visual queries), `semantic-ui.md`.
- Edward Tufte, *The Visual Display of Quantitative Information*; *Envisioning
  Information* — `information-design.md` (data-ink, chartjunk, 1 + 1 = 3, smallest
  effective difference, layering and separation, small multiples, micro/macro).

## Platform
- Apple Human Interface Guidelines, macOS: Designing for macOS, Windows, Sidebars,
  Toolbars, Inspectors (Panels), Sheets, Menus, Pop-up buttons, Segmented controls,
  Typography (macOS text styles), Color, Accessibility, Motion — `desktop-ui.md`,
  `typography.md`, `accessibility.md`.
- WCAG 2.x (1.4.3 contrast, 1.4.11 non-text contrast, 2.1 keyboard, 2.4.7 focus
  visible, 1.4.1 use of colour, 2.3.3 animation from interactions) — `accessibility.md`.

## Professional tools (patterns observed, not copied)
- Blender manual, *Interface › Nodes* (parts, editing, selecting) — `node-editor.md`,
  `desktop-ui.md`.
- DaVinci Resolve reference manual, pages and the Fusion page — `desktop-ui.md`.
- Figma (selection, properties panel, layers, on-demand measurement), Xcode
  (navigator/editor/inspector, inline issues), Logic Pro (track header state, inspector),
  Fusion 360 / Onshape / SolidWorks (constraints as glyphs, feature timeline) —
  `desktop-ui.md`.

## This repository (read before any UI change)
- `docs/STUDIO_UI.md` — the product's own UI spec; authoritative for numbers and gestures.
- `docs/adr/0012-studio-ui-references.md`, `0001`, `0003` — why Resolve/Blender/HIG;
  Rust owns semantics; layout is separate.
- `docs/ARCHITECTURE.md`, `docs/IR.md`, `docs/COMPILER_PIPELINE.md`,
  `docs/PROJECT_FORMAT.md`, `docs/HARDWARE_MODEL.md`, `docs/RUNTIME_SEMANTICS.md` —
  the facts the UI must embody.
- `docs/01-paper-digest.md`, `docs/02-kernel-spec.md` — the semantics themselves.
- `apps/studio/lib/ui/mac/{tokens,theme,widgets,controls,interactive}.dart` — the
  design system as code; `apps/studio/lib/ui/canvas/*.dart` — the canvas.
