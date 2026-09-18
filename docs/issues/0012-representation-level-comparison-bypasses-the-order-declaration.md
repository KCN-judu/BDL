---
id: ISS-0012
state: resolved
area: language
opened: 2026-09-18
resolved-by: ["ADR-0026"]
related: ["ADR-0025", "ADR-0013", "ADR-0026"]
---

# ISS-0012: A concept beside a plain value compares by representation, whatever the concept's order declaration

## Problem

Two concept values compare as concepts (`mode1 < mode2` is refused unless `Mode`
is declared ordered). A concept value beside a plain value of its own
representation — `mode < 3`, `min(mode, 2)` — is observed and compared as that
representation, as arithmetic always has been (ADR-0013), so the order
declaration is not consulted on that path.

## Why it matters

The Phase-9c policy says a numeric encoding is not a magnitude; the mixed
comparison lets one be treated as one without a declaration. Closing the path
would require every physical concept (`Tilt : Angle`) to be declared ordered
before `tilt < 10 deg` is legal, which every existing design writes.

## Current evidence

- Production: `bdl-elab::formula::binary` and `equation` (the observation rule);
  `crates/bdl-elab/tests/equations.rs`
  (`j_min_at_brightness_keeps_brightness_and_refuses_opacity`: `min(b1, 0.5)`
  elaborates while `Brightness` is ordered; the same holds unordered).
- Formal: `rep` is free everywhere (`Stdlib.Comb` admits it);
  `lt d (rep a) (rep b)` is a legal kernel term regardless of `OrdDecl`; the
  note's §11 puts the order policy at the surface.

## Dependencies

- A decision on whether a quantity-represented concept is ordered by default
  (contradicting the "never inferred" rule) or every mixed comparison needs the
  declaration (a migration for existing designs), or the mixed case stays a
  representation comparison by design.

## Resolution

Resolved by ADR-0026 (2026-09-18): the mixed case stays a representation
comparison by design — the order declaration answers exactly one question, may
two values of this concept be put in order — and the full matrix (same concept
ordered/unordered, different concepts, a concept beside its plain
representation, over Brightness, Opacity, Temperature, Mode) is the test
`mixed_comparisons_over_overlapping_representations` in
`crates/bdl-elab/tests/equations.rs`. The audit found no ambiguous case: every
pairing has one answer, and the one asymmetry (`min(o1, o2)` refused,
`min(o1, 0.5)` observed) is the rule itself.
