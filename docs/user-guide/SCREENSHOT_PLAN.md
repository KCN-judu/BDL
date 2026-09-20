# Screenshots

Every picture of Studio in this guide is a screenshot of the real Studio,
rendered from a checked-in fixture project against the real `bdld`, captured by
a harness that fails when the scene it was asked for cannot be reached. Nothing
is drawn by hand, and nothing is captured by hand.

The rule for the guide: **actual UI → screenshot; abstract semantics →
diagram.** A page that explains what the designer sees on screen shows the
screen. A page that explains something the screen does not show as one view (the
pipeline from design to code, the flattening of a system, a file tree) keeps a
diagram or a table. Drawing Studio in monospace is not an option.

## The pieces

| Piece    | Where                                                               | Role                                                                                                                                                                                                                                                                                                                                                                                                                               |
| -------- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| fixtures | `docs/fixtures/*`                                                   | small, deterministic projects the screenshots are taken of: `tilt-lamp` (the tutorial lamp, flat), `tilt-lamp-declared` (the tutorial at step 3), `behavior-group` (the lamp as a text system project with the _Adaptive lamp_ group), `component-system` (after packaging: `AdaptiveLamp`, two instances, `mirror`, `slow`). Each pins node positions and the canvas viewport in `ui/layout.json`. See `docs/fixtures/README.md`. |
| manifest | `docs/user-guide/screenshots/manifest.json`                         | one entry per screenshot: id, pages, fixture, view, steps, what must be visible, crop, output, caption, alt text, purpose                                                                                                                                                                                                                                                                                                          |
| harness  | `apps/studio/test/docs_screenshots_test.dart`                       | a Flutter test that, per entry, starts `bdld`, renders the whole `StudioShell` with the app's theme and fonts at the manifest's window size, opens the fixture, runs the steps through the store, checks the expectations, cuts the crop and writes the PNG                                                                                                                                                                        |
| ledger   | `docs/user-guide/screenshots/captured.json`                         | per id: the commit, whether the checkout was dirty, the time, the hash of the fixture and of the manifest entry the image came from, the image size                                                                                                                                                                                                                                                                                |
| check    | `scripts/check_screenshots.py` (in `just docs-check`, `just check`) | images exist, pages embed them with the manifest's alt text and caption, no stray images, every entry captured and not stale, every id listed here                                                                                                                                                                                                                                                                                 |

## Regenerating

```bash
just docs-shots
```

builds `bdld`, runs the harness with `DOCS_SHOTS=1`, writes the PNGs under
`docs/user-guide/assets/<area>/` and the ledger, then runs the check. Commit the
images and the ledger together with the change that made them necessary. Without
`DOCS_SHOTS=1` the same test runs in `just studio-test` and writes nothing: it
is the guide's UI regression probe — a fixture that no longer opens, a node that
vanished, a message that changed, a sheet that does not appear, or a node the
fixture's layout pushes off the canvas fails the test.

Run it from a clean checkout when the images are to be committed: the ledger
records `dirty: true` otherwise and `scripts/check_screenshots.py --strict`
refuses it.

### When to regenerate

- the manifest entry's scene changed (fixture, view, steps, expectations, crop,
  output) — the check reports the entry as _stale_;
- a fixture changed — the check reports every entry on it as _stale_;
- Studio's look changed — the check cannot see that; rerun `just docs-shots`
  after a change under `apps/studio/lib/ui` that touches what a screenshot
  shows, and review the diff of the images.

Prose changes (caption, alt, purpose, pages) do not make an image stale; the
check only requires the page to carry the new text.

## Presentation

Fixed by the manifest and the harness, the same for every image:

| Setting     | Value                                                                                                          |
| ----------- | -------------------------------------------------------------------------------------------------------------- |
| window      | 1440 × 900 logical pixels                                                                                      |
| scale       | 2 device pixels per logical pixel (an uncropped window is 2880 × 1800)                                         |
| theme       | light                                                                                                          |
| fonts       | the system UI font and Menlo as macOS resolves them, the Material icon font from the Flutter SDK, Chakra Petch |
| viewport    | pan and zoom from the fixture's `ui/layout.json`; a nodes crop refuses a fixture without one                   |
| recent list | empty on the project manager (paths would be machine-specific)                                                 |
| cursor      | none — the renderer has no pointer, so gestures in progress cannot be shown                                    |

## Writing a new entry

1. Pick or add a fixture. Keep it small; name things as the guide does; put
   nodes where the crop wants them and pin the viewport.
2. Add the manifest entry. Steps name entities
   (`{"select": {"mapping": "brightness"}}`, `{"draft": …}`, `{"input": …}`,
   `{"step": 3}`, `{"target": …}`, `{"pin": …}`, `{"context": …}`,
   `{"view": "code"}`, `{"collapse": …}`, `{"package": …}`, `{"link": …}`,
   `{"sidebar": …}`); expectations name nodes, groups, texts; the crop is a
   `region` (`window`, `page`, `canvas`, `inspector`, `sidebar`, `welcome`), a
   widget `key`, a `widget` type, a `text`, a set of `nodes` / `groups`, or a
   `union` of those, with `pad`, `maxWidth`, `maxHeight`. Everything a step or a
   crop names must exist, or the harness fails and says what is missing.
3. Write the caption (what the reader is looking at, one sentence) and the alt
   text (what a reader who cannot see it needs, in the designer's words — what
   is on screen, not every pixel).
4. `just docs-shots`; look at the image; correct the prose on the page to what
   the image shows, never the other way round.
5. Embed it on each page the entry names as `![alt](../assets/…)` followed by
   the caption in italics, and add a row below.

If a crop needs a widget the harness cannot find by type or text, add a
`ValueKey` to that widget in Studio (as `simulation-controls`,
`simulation-readiness`, `context-bar`, `sheet` and `dead-end` were added) —
never a behavioural change to make a picture easier.

## Status

| Id                    | Pages                                                           | Fixture            | Shows                                                                                                                                                      | Status                                                      |
| --------------------- | --------------------------------------------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| project-manager       | `getting-started/install-and-launch.md`                         | —                  | the project manager (F1)                                                                                                                                   | captured, automated                                         |
| workspace             | `studio/workspace.md`                                           | tilt-lamp          | the whole workspace, `brightness` selected (F2)                                                                                                            | captured, automated                                         |
| declared-relationship | `getting-started/first-behavior.md`                             | tilt-lamp-declared | a dashed _declared_ relationship between its concepts (F3)                                                                                                 | captured, automated                                         |
| formula-verdict       | `getting-started/first-behavior.md`, `studio/formula-editor.md` | tilt-lamp          | the formula field with a red verdict on `Tilt / 90 s` (F4)                                                                                                 | captured, automated                                         |
| formula-composer      | `studio/formula-editor.md`, `getting-started/first-behavior.md` | tilt-lamp          | the Formula view with the denominator slot selected: expected angle, units, references                                                                     | captured, automated                                         |
| complete-lamp         | `getting-started/first-behavior.md`                             | tilt-lamp          | the finished tutorial design (F5)                                                                                                                          | captured, automated                                         |
| node-anatomy          | `studio/canvas.md`                                              | tilt-lamp          | concept rows, relationship nodes (the rule word on `dimByTilt`, the reference links into `brightness`'s formula line), the sink; `dimByTilt` selected (F6) | captured, automated                                         |
| formula-unfolded      | `studio/canvas.md`                                              | tilt-lamp          | `dimByTilt` unfolded: the chevron down, the saved formula drawn as a fraction inside the node, _Edit formula_                                              | captured, automated                                         |
| concept-sheet         | `studio/library.md`                                             | tilt-lamp          | the concept sheet for _Angle_: the empty Name, the Value pop-up, _Measured in_ rad · deg · turn, the preview, the declaration                              | captured, automated                                         |
| —                     | `studio/canvas.md`                                              | tilt-lamp          | a link in mid-drag with the halo and the cursor (F7)                                                                                                       | blocked: no pointer in the renderer; the prose describes it |
| simulate-page         | `getting-started/first-simulation.md`, `studio/simulate.md`     | tilt-lamp          | the Simulate page after three ticks at 45° (F8)                                                                                                            | captured, automated                                         |
| simulate-readiness    | `studio/simulate.md`                                            | tilt-lamp          | the readiness list with its _Show_ link and the disabled Step (F9)                                                                                         | captured, automated                                         |
| deploy-page           | `getting-started/first-deployment.md`, `studio/deploy.md`       | tilt-lamp          | Arduino Nano, one PWM device, _Feasible_, the placement (F10)                                                                                              | captured, automated                                         |
| deploy-dead-end       | `getting-started/first-deployment.md`                           | tilt-lamp          | _Not feasible_ with pin D4 fixed by hand (F11)                                                                                                             | captured, automated                                         |
| pico-design           | `getting-started/pico-demo.md`                                  | pico-button-lamp   | the demo design: the Source `pressed`, `lit`, the `lamp` sink                                                                                              | captured, automated                                         |
| pico-firmware-ready   | `getting-started/pico-demo.md`, `studio/deploy.md`              | pico-button-lamp   | the Deploy page for the Pico: both devices, the placement, the Firmware section at _Ready to build_                                                        | captured, automated                                         |
| pico-firmware-built   | `getting-started/pico-demo.md`                                  | pico-button-lamp   | the Firmware card after a real build: the image, no board reachable, the BOOTSEL line (`firmware: build` runs the cross-build; needs the Rust target)      | captured, automated                                         |
| instance-nodes        | `studio/system-projects.md`, `concepts/behavior-systems.md`     | component-system   | two instance nodes, their bindings, `adaptiveLamp` selected (F12)                                                                                          | captured, automated                                         |
| component-source      | `studio/system-projects.md`                                     | component-system   | the component's own canvas with the bar back to the system                                                                                                 | captured, automated                                         |
| code-view             | `studio/code-view.md`                                           | component-system   | the Code view of the component system, coloured by the IDE service's tokens                                                                                | captured, automated                                         |
| code-completion       | `studio/code-view.md`                                           | component-system   | the completion pop-up at the caret inside the component's body                                                                                             | captured, automated                                         |
| code-hover            | `studio/code-view.md`                                           | component-system   | the hover card over `dimByTilt` in the component's body                                                                                                    | captured, automated                                         |
| behavior-region       | `workflows/grouping-behavior.md`, `concepts/behavior-groups.md` | behavior-group     | the expanded _Adaptive lamp_ region, selected                                                                                                              | captured, automated                                         |
| behavior-collapsed    | `workflows/grouping-behavior.md`                                | behavior-group     | the collapsed box with its aggregate sockets (F13)                                                                                                         | captured, automated                                         |
| packaging-sheet       | `workflows/package-as-component.md`                             | behavior-group     | the packaging sheet with Requires, Provides and the decisions (F14)                                                                                        | captured, automated                                         |
| transport-sheet       | `workflows/cross-domain-transport.md`                           | component-system   | the _Carry across timing domains_ sheet (F15)                                                                                                              | captured, automated                                         |

F-numbers refer to the slots of the earlier plan, kept so that older references
still resolve. Pending: a Library-tab figure for `studio/library.md` and an
inspector figure for `studio/inspector.md` — both reachable with the existing
steps (`{"sidebar": "library"}`, a selection) once those pages need one; the
workspace figure shows the inspector meanwhile.
