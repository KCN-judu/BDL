# Screenshot plan

No screenshots are committed yet: the interface is still moving, and a
stale image misleads more than no image. This plan fixes what each
figure must show so that they can be captured — by hand or by the
snapshot test (`apps/studio/test/snapshot_preview_test.dart`, `just
studio-snap`) once it can drive these states — and dropped into the
pages at the marked slots.

Rules: light appearance, the system font, the window at 1280 × 800
unless the crop says otherwise, `examples/smart_lamp` or the tutorial
lamp as the design, no cursor unless the figure is about a gesture.
Every figure has a caption (what it shows) and alt text (what a reader
who cannot see it needs to know).

| Id | Page · slot | Design state | Crop | Caption | Alt text |
|---|---|---|---|---|---|
| F1 | `getting-started/install-and-launch.md` · after "What you see first" | no project open; two recent projects, one *not found* | whole window | The project manager: Start actions on the left, Recent on the right | Studio's start screen with New Project, New System and Open Project buttons and a list of recent projects |
| F2 | `studio/workspace.md` · replaces the ASCII sketch | `smart_lamp` open, `brightness` selected, Design page | whole window, regions annotated: toolbar, sidebar, canvas, inspector, status line, page bar | The workspace with its six regions | The Studio window: sidebar left, canvas centre, inspector right, status line and page bar at the bottom |
| F3 | `getting-started/first-behavior.md` · after step 3 | tutorial lamp with `Tilt`, `Brightness` and a declared `dimByTilt` | canvas only, the three nodes | A declared relationship: dashed outline and the word *declared* | Two concept rows and one dashed relationship node labelled declared |
| F4 | `getting-started/first-behavior.md` · after step 4 | as F3 with `Tilt / 90 s` typed, not added | inspector's Relationship section | The formula field with a red verdict line and the underlined span | The formula field showing a red message that Brightness is a dimensionless quantity but the formula produces a quantity of rad·s⁻¹ |
| F5 | `getting-started/first-behavior.md` · after step 8 | the finished tutorial lamp | canvas | The lamp: inputs, rule, value and the driven sink | Three relationship nodes and a solid sink node at the right, with the domain name at the nodes' edges |
| F6 | `studio/canvas.md` · "The nodes" | `smart_lamp` | one concept row, one relationship node, one sink, side by side | Node anatomy | A concept row with round sockets, a relationship node with input sockets on the left and one output socket on the right, a sink node with a boundary bar |
| F7 | `studio/canvas.md` · "Gestures" | a link being dragged from `brightness` towards the sink | canvas, cursor included | Compatible sockets light up while a link is dragged | A link in mid-drag with a halo on the one socket that can accept it |
| F8 | `getting-started/first-simulation.md` · after step 3 | `smart_lamp`, three ticks stepped with tilt 0.7854 | whole Simulate page | The Simulate page after three steps | Inputs on the left, a three-row trace in the middle showing Brightness values, the probe on the right |
| F9 | `studio/simulate.md` · readiness | tutorial lamp before `tilt` has a value | the readiness list above the trace | A blocker with its Show link | A sentence saying tilt needs a value before simulation can step, with a Show link, and a disabled Step button |
| F10 | `getting-started/first-deployment.md` · after step 3 | `smart_lamp`, Arduino Nano, PWM light on Light Output | whole Deploy page | Feasible on Arduino Nano, with the placement table | Target pop-up and one device row on the left; the verdict and a one-line placement table in the centre |
| F11 | `getting-started/first-deployment.md` · after step 4 | as F10 with pin D4 fixed | verdict area | Not feasible: the pin chosen by hand cannot carry PWM | The verdict line in red and the dead-end explanation naming pin D4 |
| F12 | `studio/system-projects.md` · "Instance nodes" | the packaged lamp system with two instances | the two instance nodes and their bindings | Instance nodes drawn from the component's promise | Two instance nodes with a required socket on the left and a provided socket on the right, joined to top-level relationships by links |
| F13 | `workflows/grouping-behavior.md` · end of Steps | *Adaptive lamp* collapsed | the box and its neighbours | A collapsed behavior with aggregate sockets | A single box labelled Adaptive lamp with one socket on each side and the links that entered and left its members |
| F14 | `workflows/package-as-component.md` · end of Steps | the packaging sheet open for *Adaptive lamp* | the sheet | The packaging sheet: Requires, Provides, and the four decisions | A sheet listing tiltValue under Requires, brightness under Provides, the light under Physical outputs with Stays the system's selected, and name fields |
| F15 | `workflows/cross-domain-transport.md` · after "On a binding" | the *Carry across timing domains* sheet | the sheet | Binding across domains asks for a starting value | A sheet titled Carry across timing domains with the two domain names and a Starts at field |

Slots are marked in the pages as HTML comments (`<!-- figure F3 -->`)
where a figure is intended; a page without a marker needs no figure.
