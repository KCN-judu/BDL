# Design issues

A design issue is a recognised problem with no chosen answer: the record
for "we know this is unresolved" — not a proposal, not a decision, and
never permission to implement its tentative answer. When a concrete
alternative exists, a [proposal](../proposals/README.md) follows; when a
choice is made, an [ADR](../adr/README.md) closes the issue through
`resolved-by`.

## Active

| ID | Issue | State | Area | Waits on |
|---|---|---|---|---|
| [ISS-0001](0001-cross-domain-occurrence-windows.md) | Cross-domain occurrence windows | open | language | production mirror of BDL_FV Phase 9a; a surface form |
| [ISS-0002](0002-several-candidate-definitions-one-active.md) | Several candidate definitions, one active | open | language | persistence and protocol shape |
| [ISS-0003](0003-interface-level-references-and-the-evidence-model.md) | Interface-level references and the evidence model | open | formal | first concrete `PropertyId`s and evidence sources |
| [ISS-0004](0004-affine-units.md) | Affine units | open | language | a surface conversion rule, or a permanent refusal |
| [ISS-0005](0005-user-defined-enums.md) | User-defined enums | open | language | a kernel sum type (Lean first), or enums outside executable BDL |
| [ISS-0006](0006-numeric-representation-on-the-device-f32.md) | Numeric representation on the device (`f32`) | deferred | runtime | the first embedded platform adapter |
| [ISS-0007](0007-packaging-a-group-inside-a-component-body.md) | Packaging a group inside a component body | deferred | behavior-systems | a nested component model |
| [ISS-0008](0008-a-structural-diagnostic-entity-for-outputs.md) | A structural diagnostic entity for outputs | deferred | protocol | the next protocol change touching `Diagnostic` |
| [ISS-0009](0009-projection-deltas-and-a-persisted-edit-history.md) | Projection deltas and a persisted edit history | open | protocol | a delta encoding; a log format |
| [ISS-0010](0010-temporal-modifiers-and-contexts-in-the-surface.md) | Temporal modifiers and contexts in the surface | open | language | a surface shape per modifier |

## Resolved

| ID | Issue | Resolved by |
|---|---|---|
| — | *none since the registry was created* | |

## Before the registry

`docs/DESIGN_ISSUES.md` is the ledger kept from 2026-09-15 to 2026-09-17
(DI-1 … DI-44, two numbers used twice). Its decided entries stay there as
history — each names the decision or the code that resolved it; its open
entries became the issues above (`related` names the DI). Do not add rows
to the old ledger.

Start a new issue from [TEMPLATE.md](TEMPLATE.md); the lifecycle is in
[GOVERNANCE.md](../project-records/GOVERNANCE.md).
