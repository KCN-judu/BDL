# Documentation fixtures

Small, deterministic projects the user guide's screenshots are taken of
(`docs/user-guide/SCREENSHOT_PLAN.md`). They open in Studio like any project
(_Open Project…_), check with `bdld check`, and are read by
`apps/studio/test/docs_screenshots_test.dart`, which fails when one of them no
longer opens or no longer shows what the manifest expects. Change one only
together with the screenshots taken of it (`just docs-shots`); the check in
`just docs-check` reports the images as stale until then.

| Fixture               | Kind                                             | Holds                                                                                                                                                                                                                                                                                                                                                                                                                                              | Used by                                                                                                                  |
| --------------------- | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `tilt-lamp/`          | flat (`design/project.bdl.json`)                 | the tutorial lamp as _Your first behavior_ leaves it: `Tilt`, `Brightness`; `tilt` (input), `dimByTilt` (`Tilt / 90 deg`), `brightness` (`dimByTilt(tilt)`) in `interaction`; _Light Output_ driven by `brightness`; a _PWM light_ device for Deploy                                                                                                                                                                                               | workspace, formula-verdict, complete-lamp, node-anatomy, simulate-page, simulate-readiness, deploy-page, deploy-dead-end |
| `tilt-lamp-declared/` | flat                                             | the tutorial at step 3: the two concepts and a `dimByTilt` without a formula                                                                                                                                                                                                                                                                                                                                                                       | declared-relationship                                                                                                    |
| `behavior-group/`     | text (`src/main.bdl`, `.bdl/*`, `kind = "text"`) | the lamp as a system project, `main` domain, `raw → tiltValue → dimByTilt → brightness → light`, `indicator = brightness`; the group _Adaptive lamp_ over `dimByTilt` and `brightness` in `.bdl/authoring.json`                                                                                                                                                                                                                                    | behavior-region, behavior-collapsed, packaging-sheet                                                                     |
| `pico-button-lamp/`   | text                                             | the Button → Lamp demo as the wired template writes it (`bdld init --template button-lamp-configured`; a test keeps them equal): `Pressed`, `Lit`; the Source `pressed`, `lit = pressed` in `main`; `lamp` driven by `lit`; `button` (digital input, `gpio_level_in_low`, GP2) and `led` (digital output, `gpio_level`, GP25); a `build/` directory a firmware shot leaves is not fixture content (deleted before each shot, left out of the hash) | pico-design, pico-firmware-ready, pico-firmware-built                                                                    |
| `component-system/`   | text                                             | the same after packaging: component `AdaptiveLamp` (`requires tiltValue`, `provides brightness`), instances `adaptiveLamp` and `second`, `brightness` and `mirror` bound to their ports, `indicator`, and `slow` in `aux` left open for the transport sheet                                                                                                                                                                                        | instance-nodes, component-source, transport-sheet                                                                        |

Layout matters here: `ui/layout.json` pins every node's position and the canvas
viewport (pan and zoom) so that a crop by node names lands on the same pixels
every time; the harness refuses a nodes crop on a canvas without a pinned
viewport, and fails when a node lies outside the canvas at the manifest's window
size (1440 × 900). Keep the text fixtures' `.bdl/identities.json` committed: it
is what keeps ids — and with them layout and group membership — stable across
opens.

Two representations on purpose: the tutorial pages create a plain project, so
`tilt-lamp` is one; the system pages are written for text projects, which are
the representation new fixtures should use.
