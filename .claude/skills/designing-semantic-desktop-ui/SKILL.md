---
name: designing-semantic-desktop-ui
description: Design discipline for BDL Studio (the Flutter desktop app in apps/studio) and any UI whose job is to make language semantics perceivable. Use this before touching anything under apps/studio/lib/ui — a new panel, inspector, sheet, canvas element, status line, page, node, socket, colour, badge, diagnostic, empty state, or any visual/interaction change, however small — and whenever the user asks for a layout, a mockup, a critique, "make it look right", or a decision about what to show where. It forces a workflow (task → semantic facts → canvas/inspector/explanation placement → encoding → grid → platform → feedback → states → accessibility → widgets), keeps one meaning per visual channel, and ends with a strict self-review. Do not skip it because the change "is just styling".
---

# Designing semantic desktop UI

BDL Studio must let a designer *see* what the language means — identity, incompleteness,
dependency, time, domains, physical outputs — through shape, colour, connection, grouping
and constraint, not through compiler vocabulary printed beside a box. Standard desktop
conventions cover everything ordinary; invention is reserved for what BDL semantics
actually require. This skill is the procedure that keeps both true.

Semantic truth is Rust-only (ADR-0001); the UI renders projections. Nothing designed here
may compute a semantic judgment.

## When to activate

- Any change under `apps/studio/lib/ui/` (canvas, inspector, library, sheets, shell,
  status line, tokens, theme, welcome).
- Designing a page that does not exist yet (Simulate, Deploy, Monitor), or a canvas
  element for a semantic fact that is not drawn yet (contexts, clock domains, outputs,
  transports, delays, telemetry).
- Deciding wording for diagnostics, inspector labels, or explanation text.
- The user asks for a critique, mockup, or "does this look right".

## Workflow — do these in order, write the answers down before coding

1. **Task.** One sentence: what is the designer trying to do on this screen? (Not "view
   the mapping" — "decide whether `dimByTilt` is ready to simulate".)
2. **Semantic facts.** List every semantic fact the screen touches, in the language of
   `bdl-contract.md` (identity, representation, resolution state, reads/produces, …).
3. **Rank.** Order those facts by how often the task needs them and how costly a
   mistake is. The top one must be findable in under a second.
4. **Place.** Assign each fact to exactly one primary home:
   - **canvas** — structure and state that must be seen without selecting anything;
   - **inspector** — everything editable about *the selection*, and the detail behind a
     canvas cue;
   - **explanation layer** — prose, diagnostics, "why", kernel vocabulary on demand.
   A fact may echo in a second place only as an overview↔detail pair, never as a
   duplicate badge.
5. **Encode.** For each canvas fact, pick the channel in this order: shape → colour →
   connection → spatial grouping → containment → interaction constraint → object state →
   only then text or a badge. Write the *secondary, non-colour cue* for every colour use.
6. **Check collisions.** Compare against the channel table in `semantic-ui.md` and the
   contract in `bdl-contract.md`. A channel already carrying a meaning may not take a
   second one. If you need a new channel, add it to the table in the same change.
7. **Layout.** Place on the 8 pt grid: fixed label column, one gutter, alignment does the
   grouping, whitespace separates, hairlines only between sections. `layout-and-grid.md`,
   `typography.md`, `gestalt.md`.
8. **Platform.** Is there a macOS convention for this control or arrangement? Use it
   (`desktop-ui.md`). Deviate only with a BDL reason you can write in one sentence.
9. **Interaction and feedback.** Actions live next to the object; impossible actions are
   constrained before they fail; feedback is immediate and local; destructive actions are
   named. `interaction-design.md`, `node-editor.md`.
10. **States.** Design the incomplete state, the error state, the empty state, the stale
    (pending / disconnected) state. Incomplete-but-valid is never red.
11. **Keyboard and accessibility.** Focus order, focus ring, every mouse action reachable
    by keyboard, contrast, meaning survives without colour, no essential motion.
    `accessibility.md`.
12. **Then widgets.** Only now choose Flutter widgets, reusing `MacTokens`, `MacMetrics`,
    `MacStates`, `MacInteractive`, `FormRow`, `MacTable`, `NodePainter`.

If a step's answer is "not applicable", say why in one line — that is allowed; skipping is not.

## Which reference to read

| Problem | Read |
|---|---|
| What BDL needs the UI to solve; the semantic ↔ visual contract; wording | `bdl-contract.md` (always, for any semantic element) |
| Choosing a visual encoding; channel discipline; badge vs structure | `semantic-ui.md` |
| Node, socket, link, region, transport, canvas gestures | `node-editor.md` |
| Panel dimensions, spacing, columns, alignment, whitespace | `layout-and-grid.md` |
| Type sizes, weights, hierarchy, labels vs values, numbers | `typography.md` |
| Grouping, figure/ground, preattentive cues, how many hues | `gestalt.md` |
| Affordance, feedback, selection, command structure, expert paths, disclosure | `interaction-design.md` |
| Density, comparison, layering, removing decoration, tables of facts | `information-design.md` |
| macOS window anatomy, inspectors, sidebars, sheets, menus, and what Blender / Resolve / Figma / Xcode / Logic / CAD teach | `desktop-ui.md` |
| Contrast, keyboard, focus, screen readers, motion | `accessibility.md` |
| Before declaring any UI done | `critique-checklist.md` |
| Something feels "generic dashboard" | `anti-patterns.md` |
| The compressed principle set behind all of the above | `principles.md` |
| Worked examples, good and bad; a dated critique of the real Studio | `examples/` |
| Where these ideas come from; repo docs to reread | `references.md` |

`docs/STUDIO_UI.md` is the product's own spec and is authoritative where it is specific
(numbers, gestures, references). This skill is the reasoning that produced it and should
extend it, not contradict it. If they disagree, say so and propose an edit to the doc.

## Final self-review — run before reporting done

1. Open `critique-checklist.md` and answer every question for the screen you changed,
   in writing. "Yes" without a reason does not count.
2. Scan `anti-patterns.md`; name any you introduced or left in place.
3. Confirm every colour has a non-colour partner and every new encoding is in the
   channel table.
4. Confirm the compiler's words (`declRef`, `Grant`, `SingleDriver`, `Κ`, `revision`,
   `protocol`, `bdld`, `id`) appear only in the explanation layer.
5. Tab through the change with the keyboard. If you cannot, say so.
6. Report: the task, the ranked facts, where each landed, the encodings, and the
   checklist answers — not just the diff.
