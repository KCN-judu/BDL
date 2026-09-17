---
id: ADR-0015
status: accepted
date: 2026-09-15
area: deployment
supersedes: []
superseded-by: []
related: []
fv: ["informed by FV: BDL_FV BDL/Validation/Hardware.lean (Phase 7)"]
---

# ADR-0015: Deployment analysis is a separate, target-relative function

## Status

Accepted (outputs/hardware milestone).

## Context

The paper separates the kernel (typing, grants, causality, clocks, output
well-formedness) from target-specific hardware validation: whether a design's
device bindings fit a board is a property of the pair `(design, target)`, and
changing the target flips it without changing the design. A tempting shortcut is
to add `HardwareFeasible` as the top rung of `MappingStatus`, or to let
`analyze` consult "the current board".

## Decision

- `bdl-compiler::analyze(&ProjectSnapshot) -> ProjectAnalysis` takes no target
  and runs passes 1–11 (through the output pass). It ends with
  `output_complete`, a property of the design alone.
- `bdl-compiler::analyze_deployment(&ProjectSnapshot, &Hardware) -> DeploymentAnalysis`
  runs requirement generation and allocation for one target. It reads only the
  device bindings; it never touches `Ty`, `HasType`, `Grant`, causality, clocks
  or the evaluator.
- `MappingStatus` stops at `ClockConsistent`. Output state lives in
  `OutputAnalysis` (per sink), deployment status
  (`Feasible | Infeasible | Incomplete`) in `DeploymentAnalysis`. Neither is a
  rung a single mapping climbs.
- `bdld` answers `AnalyzeDeployment { target_id }` per request against the
  current revision and caches nothing with the project; `RunAnalysis` is
  unchanged.
- `bdl-hardware` depends on `bdl-model` only: the solver sees requirements,
  never declarations, and never learns what an IMU is.

## Consequences

- The same `OutputId` bound to different `DeviceKind`s on different targets has
  identical semantic analysis and different deployment analyses; this is tested.
- A design may stop at `ClockConsistent`, or at output-complete with no device
  bound (`Incomplete`), and be a valid artefact; nothing in the earlier states
  is an error.
- Board selection and manual pins invalidate `Deployment` only
  (`Invalidation::Deployment`).
- The first-dead-end diagnosis is one explanation, not a minimal core (DI-21);
  numeric/electrical constraints are out of scope (DI-22).
