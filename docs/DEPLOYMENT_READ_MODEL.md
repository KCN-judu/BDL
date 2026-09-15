# Deployment read model

What a Deploy surface asks, and where each answer comes from. The
frontend renders; it reconstructs nothing — not a requirement, not a unit
relation, not a dead end. Everything below is on the wire in
`DeploymentAnalysis` fields 10–15 and `TargetView` (docs/PROTOCOL.md 0.5),
composed at the daemon by `bdl_compiler::deployment_report` from two
analyses that never see each other.

## Four questions, four answers

| Question | Truth | Field |
|---|---|---|
| Is the design semantically valid? | `ProjectAnalysis`: every relationship checks, `Causal`, `Clocked` | `design_ready` (with output completeness), `missing[]` items of kinds `RELATIONSHIP_NOT_CHECKING`, `NOT_CAUSAL`, `NOT_CLOCK_CONSISTENT` |
| Are its outputs complete? | `ProjectAnalysis.output_complete` (`DriveWF ∧ SingleDriver ∧ CompleteOutputs`, no open output) | `design_ready`; `missing[]` kinds `OUTPUT_NO_DOMAIN`, `OUTPUT_NO_DRIVER`, `OUTPUT_CONNECTION_INVALID` |
| Is the deployment configuration complete? | `DeploymentAnalysis`: every output with a domain has a device, every device an output | `status == INCOMPLETE`; `missing[]` kinds `OUTPUT_NO_DEVICE`, `DEVICE_NO_OUTPUT` |
| Do the devices fit *this* target? | `DeploymentAnalysis`: `solve` / `diagnose` | `status` (`FEASIBLE` / `INFEASIBLE` / `INCOMPLETE`), `rows[]`, `blocker` |

`deployable = design_ready && status == FEASIBLE`. It is the only field
that means "you can deploy this"; `status` alone never does — a design
whose devices fit perfectly is still not deployable while a relationship
does not check, and the message then names the relationship, not the
board.

The distinctions are load-bearing:

* **Semantic invalidity is never infeasibility.** A broken definition
  leaves `status` exactly as the solver found it (usually `FEASIBLE`) and
  adds a `RELATIONSHIP_NOT_CHECKING` item; `blocker` stays empty.
  Target choice cannot change typing, causality, clocks or
  `MappingStatus` — `RunAnalysis` returns the same `ProjectAnalysis`
  before and after any `AnalyzeDeployment`, on any target (tested).
* **Incomplete is not an error.** A design may stop with an output
  undriven or a device unbound; each is a `missing` item with product
  wording and the entity it is about, by id and by name.
* **Infeasible is about the target.** The same devices may be
  `INFEASIBLE` on the Nano and `FEASIBLE` (or `INCOMPLETE`) on the big
  board; the `blocker` says which requirement of which device could not be
  placed and why, in the target's own pin names.

## The pieces

### Target chooser — `ListTargets → TargetView[]`

`id` (opaque; never parse it), `display_name`, `description`, `family`,
`resource_count`, `capabilities[] { capability, label, resource_count,
shareable }`. Enough for a pop-up with a one-line description and a
"6 × PWM, 2 × interrupt, I²C" summary.

### Status — `status`, `design_ready`, `deployable`

Three-way target-relative status, plus the two booleans above.

### What is missing — `missing[] MissingItem`

`kind`, the entity (`output_id`/`output_name`, `device_id`/`device_name`,
`mapping_id`/`mapping_name` — whichever applies), `message` (product
language: *motor has no device on Arduino Nano.*), `explanation`
(consequence and remedy). Semantic items come first, then deployment
items, each group in id order. Empty exactly when `deployable`.

### Assignment table — `rows[] AssignmentRow`

One row per requirement of every device, in (output, device, index)
order, unbound devices last:

| column | field |
|---|---|
| output | `output_name` (`output_id`; empty for an unbound device) |
| device | `device_name`, `device_kind_label` (*H-bridge channel*) |
| what it needs | `requirement_label` (*PWM*, *direction*, *SDA*), `capability_label` |
| chosen by hand | `fixed` (a pin id, or absent) |
| placed on | `resource` (pin id), `resource_label` (*D3: digital in, digital out, PWM (timer 2), interrupt*) — absent when no placement exists (infeasible) |

`requirement_index` is the stable key a pin editor uses with
`SetDevicePin`; `device_id`/`output_id` are the stable identities.

### Blocker — `blocker`

Present exactly when `status == INFEASIBLE`: the requirement that could
not be placed (`device_name`, `requirement_label`, `capability_label`),
its `kind` — `no_capable_resource`, `fixed_unavailable(pin)`, or
`blocked { candidates[] { resource, resource_label, held_by_device_name,
held_by_requirement_label } }` — and `message`/`explanation` in product
language (*No free pin on arduino_nano can carry L7 PWM.* / *Every pin that
could serve L7 is already needed by something else: D3 (L1 PWM), …*).

It is the solver's **first dead end under its placement order** — one
conflict, honestly reported. It is not a minimal unsatisfiable core and
must not be presented as "the" cause (DI-21).

### Revision — `revision`

Every result carries the project revision it describes. A client keeps a
result only while it equals the revision it holds; sending
`AnalyzeDeploymentRequest.revision` makes the daemon refuse a stale
question with `deploy.stale_revision` instead of answering about a
project the client no longer has.

## What stays below the read model

`requirements[]`, `assignment[]`, `dead_end` (fields 4–6) are the analysis
as computed, by stable ids — for tools, tests and the expert view. A Deploy
page needs none of them; nothing in `details_json` or any string needs
parsing.

## Not in this model

Hardware feasibility is not a `MappingStatus` (ADR-0015) and not a
readiness condition of code generation (the core is target-independent;
ADR-0016). Electrical/numeric constraints are out of scope (DI-22). A
`DeviceKind` is always set — "device kind not selected" cannot occur.
