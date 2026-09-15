# The BDL design contract

What the language requires the interface to make perceivable, and the agreed way each
fact is shown. Sources: `docs/01-paper-digest.md`, `docs/02-kernel-spec.md`,
`docs/IR.md`, `docs/COMPILER_PIPELINE.md`, `docs/HARDWARE_MODEL.md`,
`docs/RUNTIME_SEMANTICS.md`, `docs/STUDIO_UI.md`. When the kernel and this file
disagree, the kernel wins and this file is wrong.

Contents: 1 the interaction problems · 2 the semantic ↔ visual matrix · 3 the workspace
state ladder · 4 wording rules · 5 what the UI may never compute.

## 1. The interaction problems BDL Studio must solve

These are not features; they are things a designer has to *perceive and act on*
correctly, or the tool has failed. Each is a design problem before it is a widget.

**Semantic identity.** `Tilt` and `MotorAngle` are different types even when both are
angles. A value of one can never silently become the other; crossing needs an explicit
mapping. Identity is a stable integer id, never the name — renaming changes nothing.
*Problem:* make identity recognisable at a glance, stable across renames, pages and
sessions, and make the typing rule (same identity or no connection) felt rather than
reported.

**Incomplete-but-valid declarations.** A mapping declared with a signature and no
definition is a legal, useful object; others may depend on it; the design is not
"broken". A concept may exist before its representation is chosen. *Problem:* show
incompleteness as an honest state of the object — not an error, not hidden — while
keeping every affordance the object already has (linking, referencing, simulating with
a supplied trace).

**Semantic relationships.** A mapping reads concepts and produces one concept; that is
its interface, frozen by the signature. Dependency is what the canvas shows; execution
order and priority are not. *Problem:* draw a graph whose edges are typed by identity,
whose direction is data flow, and which never implies ordering or arithmetic structure.

**Temporal behaviour.** `previous`, `hold`, `count`, `rise`, `since`, `every` all lower
to `delay init e`. Every held value has an explicit initial value. Cycles through a delay
are legal; instantaneous cycles are not (`Causal`). *Problem:* make "this edge crosses a
tick" visibly different from "this edge is instantaneous", make the initial value
findable where the delay is, and make an illegal instantaneous loop impossible to draw or
locally refused.

**Contexts.** A context is a boolean activation plus entry (rising edge), locally gated
state reset on entry, a default contribution when inactive, and competition for outputs
resolved inside the single driver. Nesting is supported; a context with its own clock
domain is not (the tool must refuse, not silently elaborate). *Problem:* show a context
as a region of behaviour that is on or off, with its entry and default visible, without
turning every context into a separate diagram.

**Clock / domain boundaries.** A domain is a nominal identity, not a rate; rate is
validation data. A declaration belongs to at most one domain; pure mappings are
domain-free. A cross-domain read without `sync` is an error at the reference; `sync`
reads the source's last activation *strictly before* the current tick and needs an
initial value. *Problem:* make domains visible as containment, crossings visible as a
crossing, and the transport (with its initial value) a thing you can point at.

**Physical outputs.** An output is a nominal sink with an accepted type and a clock. The
drive edge is written once; exactly one driver per sink; the driver's type must *equal*
the accepted type (so a `rep` step is explicit) and share the clock. Combination
(priority, blend, max) is ordinary computation upstream of the one edge. *Problem:* show
outputs as the right-hand edge of the world, show ownership at the point of convergence,
and make a second driver impossible to connect rather than diagnosed afterwards.

**Deployment feasibility.** Requirements come from device kinds; the board is data; the
solver is a CSP with `diagnose` giving one dead end, not a minimal core. Feasibility is
recomputed on every change and is the only state depending on something outside the
design. *Problem:* show fit/no-fit against a concrete board with the blockers named,
kept visibly apart from typing and semantics ("counting is not feasibility").

**Simulation.** Inputs are explicit traces per unresolved declaration per tick; every
domain steps deterministically; values are per `DeclId` per activation. *Problem:* let
the designer supply traces, step and scrub time, and read values *on the same objects*
they designed, not in a separate report.

**Live monitoring.** Telemetry samples are tagged by stable ids and activation index.
*Problem:* show the running device on the design canvas with the same encodings as
simulation, plus staleness and connection state, without motion that carries meaning.

**Progressive formalisation.** Every operation is a refinement (established facts
survive) or an edit (dependents reopen), with an explicit invalidation set. Evidence
declared by a supplied-block author must look different from evidence the tool
computed. The workspace state ladder (§3) is the designer's model of "how done is this".
*Problem:* make the ladder legible per object and per project, show consequences of an
edit at the moment of acting, and never confuse "declared by a human" with "checked by
the tool".

## 2. Semantic ↔ visual matrix

Status: **now** = implemented in `apps/studio/lib/ui`; **spec** = in `docs/STUDIO_UI.md`,
not yet drawn; **proposed** = this skill's recommendation for planned passes, to be
confirmed when the pass lands. "Secondary cue" is the non-colour partner every colour
encoding needs.

| Semantic fact | Internal representation | Visual representation (canvas) | Secondary cue | Inspector wording | Explanation wording | Status |
|---|---|---|---|---|---|---|
| Semantic identity | `SemanticId` (stable int) | socket and link **hue**, derived from id (golden angle), identical everywhere the concept appears | the concept's name at every socket / row | the name field; never the number | "`Tilt` is its own kind of value; a `MotorAngle` cannot stand in for it" | now |
| Concept representation | `Θ : SemanticId → Option Ty` (`q d` / `bool` / `nat`) | socket **shape**: ○ quantity, ◇ on-off, □ count | the kind word in the inspector | Kind: Quantity / On–off / Count / Decide later; Dimension | "a measured value with dimension …" | spec (all circles today) |
| Representation not chosen | `Θ s = none` | **hollow** socket ring; header word *open* | the word *open* | Kind: Decide later; note that mappings may already use it | "relationships can use this concept before its kind is fixed" | now |
| Unresolved mapping | `realization = none` | **dashed** node outline; body empty | header word *declared* | State ladder at *declared*; Definition section offers attach | "declared, not yet defined — a legal state others may depend on" | now |
| Mapping reads | `Signature.inputs` | one **input socket per input** on the left, in the concept's hue, labelled | socket label text | Reads: chips in concept hue, removable; `+` to add | "reads `Tilt`" | now |
| Mapping produces | `Signature.output` | **one output socket** on the right, in the concept's hue; link to the concept node | socket label | Produces: pop-up with the concept's socket glyph | "produces `Brightness`" | now (glyph missing) |
| Typing rule | `Ty.sem s` nominal | a link **cannot form** between different hues; legal targets gain a halo during a drag | the drop is refused; link retracts | — | "`dimByTilt` reads `Tilt`; `MotorAngle` is a different concept" | partial (no refusal feedback yet) |
| Formula validity | `HasType` under `Grant.of τ`, `Prim.ty` | node outline stays solid; validity is not a canvas mark | — | diagnostic anchored under the formula field, product language; ladder at *type-valid* | "this formula gives a length, but `Brightness` is dimensionless" | proposed |
| Dimensions | `q Dim` (8 exponents) | not on canvas | — | Dimension pop-up: name column, unit symbol column | "angle · time⁻¹" | now |
| Temporal behaviour (delay) | `delay init e` | link carries a **register mark** (two short bars across it) where it crosses a tick; the cycle it closes is drawn, not hidden | mark + the initial value shown at the mark | Initial value field beside the delayed input; "remembers last tick" | "`Held` reads its own previous value; it starts at *false*" | proposed |
| Causality error (instantaneous loop) | `Causal` fails | the closing link **refuses to form**; the would-be loop pulses once in error red | red + the message at the drop point | ladder stuck below *temporally valid* with the loop listed | "`a` needs `b` now and `b` needs `a` now; make one of them read the previous value" | proposed |
| Context | activation decl + entry + gated state + default | a **region** (common-region tint, rounded, labelled) enclosing the mappings that belong to it; **activation socket** on its top-left edge; default contribution at its output | the region label; "when *Warm*" | Active when: concept pop-up; Default when inactive; Entry resets: list | "while `Warm`, these relationships apply; otherwise `heaterTarget` is 0" | proposed |
| Clock domain | `Κ : DeclId → Option ClockId` | a **domain lane**: a large labelled background region (not a hue); domain-free mappings sit outside lanes | lane label; rate as secondary text (`50 Hz`) | Domain pop-up on the mapping; domain list in Library with rates | "updates with the interaction domain" | proposed |
| Cross-domain observation | `sync src init e` | a link crossing a lane boundary passes through a **transport gate** (small vertical bar with the initial value); a crossing without a gate is drawn broken at the boundary with an error red gap | gate glyph and value text | Transport: "last value of `Temperature`, starting at 20 °C" | "`heaterTarget` sees the last committed `Temperature`, never the current one" | proposed |
| Physical output | `OutputId`, `OutputSpec { accepts, clock }` | **output node** anchored in the right-most column, distinct silhouette (flat right edge, plug notch), one input socket | the device kind name | Output: accepts (kind), clock, device kind, driver | "the light accepts a brightness on the interaction clock" | proposed |
| Output ownership conflict | `SingleDriver β` | the second link **cannot connect**; while dragging, the occupied socket shows its current driver's name and a blocked cursor | blocked cursor + text | Driver: `dimByTilt` (only one allowed) | "this output already has a final driver, `dimByTilt`; combine the two brightness values before connecting" | proposed |
| Output completeness | `CompleteOutputs` | undriven required output has a **hollow** input socket | *undriven* word | ladder at *output-complete* lists undriven sinks | "the heater has no driver yet" | proposed |
| Hardware feasibility | `solve` result | Deploy page: **board picture** with resources; satisfied requirements drawn as leads to pins; unsatisfied requirement shown with no lead and its blockers listed | list of blockers | Board pop-up; Requirements table: device, needs, assigned pin | "needs 7 PWM, the Nano has 6: D3 D5 D6 D9 D10 D11 are taken by …" | proposed |
| Allocation | `Assignment` | lead from requirement to pin; pinned (manual) resources with a pin glyph | "pinned" text | Pin pop-up per requirement | "you pinned M1 to D3; the solver works around it" | proposed |
| Simulation values | `Ev` per `DeclId` per tick | **value on the socket** (tabular figures, unit column); booleans as filled/empty ◇; the timeline scrubber selects the tick | numeric text is the cue itself | Probe: value at tick, trace plot | "at tick 12, `Brightness` is 0.62" | proposed |
| Runtime telemetry | samples by stable id + activation index | same as simulation, on the Monitor page; a sample's **age** fades the value toward tertiary; disconnected = values hollow | age text on hover; connection cell in status line | Probe: last sample, rate | "last value 0.4 s ago" | proposed |
| Refinement vs edit | `EditOutcome.kind`, `Invalidation` set | not on canvas | — | after the change, one line: "Edit — reopens typing of 2 dependents" | consequence stated *before* the commit where possible | now (pill; wording ok) |
| Declared vs computed property | supplied-block assertions vs checker facts | not on canvas | — | two labelled groups: "Asserted by the component author" / "Checked" | "the author asserts monotone; the tool has not checked it" | proposed |
| Workspace state ladder | `AcceptanceState` | canvas shows the first two rungs (dashed / solid) and the temporal + clock rungs (marks, gates); the rest are page-level | header state word | ladder control: rungs in order, reached ones solid, the next one explains what is missing | one sentence per missing rung | partial |

**Colours that are not identity**: header tint by category (concept grey-blue, mapping
blue, context violet, output amber, transport teal) is low-saturation so it never competes
with socket hue; `settled` green and `open` orange are used only as text or a 12 % tint,
never as a fill; `error` red is reserved for a fact that is actually wrong; the accent is
selection, focus and the default button, nothing else.

## 3. The workspace state ladder

| Rung | Established | Shown where |
|---|---|---|
| declared | named, typed by signature; others may depend on it | canvas (dashed) |
| defined | a formula / curve / example set / component is attached | canvas (solid), inspector |
| type-valid | the definition produces the promised type; dimensions agree; grant respected | inspector |
| temporally valid | every held value has an initial value; no instantaneous loop | canvas (marks), inspector |
| clock-consistent | every cross-domain read has a transport and initial value | canvas (gates) |
| output-complete | every required output has exactly one final driver | Outputs / canvas right column |
| hardware-feasible | the chosen board carries every requirement (only rung depending on something outside the design; recomputed every change) | Deploy page |

The ladder is monotone per object and per project. A rung not reached is *open*, not
*failed*. Only a definite contradiction (type mismatch, instantaneous loop, second
driver, no pin) is an error.

## 4. Wording rules

- Product language in message, inspector and canvas: *reads, produces, remembers,
  starts at, while, when, driver, fits, pinned*. Kernel language only in the explanation
  layer's technical section: `declRef`, `Grant`, `Κ`, `sync`, `SingleDriver`, `DeclId`.
- Never show a numeric id to a designer. Identity is the colour and the name.
- A missing thing is named by what it is missing ("no definition yet", "no driver yet"),
  not by a code.
- Consequences are stated in terms of *other objects*: "re-checks `dimByTilt` and
  `lampTarget`", not "invalidates Interface".
- Every diagnostic offers the fix in the same sentence when the kernel knows it: add a
  transport, read the previous value, combine before connecting.
- Words for the same fact are identical in sheet, inspector, library and canvas
  (Quantity / On–off / Count / Decide later — one set).

## 5. What the UI may never do

Compute type validity, identity, dimension, causality, clocks, ownership, feasibility,
or simulation; invent an id; reorder anything the projection ordered; store semantics in
layout; treat a hover or drag position as a semantic edit until the compiler accepts the
resulting `EditOp`. It may render an edit optimistically and must reconcile with the
returned projection.
