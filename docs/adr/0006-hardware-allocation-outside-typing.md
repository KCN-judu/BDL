# ADR-0006: Hardware allocation is outside typing

**Status**: accepted (2026-09-15)

## Context
The paper proves feasibility is target-relative and not monotone: six
actuators fit, a seventh may not, with nothing semantic changing.

## Decision
Hardware feasibility is a validation-layer analysis over (requirements,
board), never a typing fact. Board changes and manual pins invalidate the
`Deployment` category only. Studio reports "valid design" and "deployable
on this board" in different places with different wording.

## Consequences
* Semantic analysis never depends on a board.
* Two kinds of evidence are kept apart: refinement-surviving and re-solved.
