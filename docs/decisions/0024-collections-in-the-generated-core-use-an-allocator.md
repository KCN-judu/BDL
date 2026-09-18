---
id: ADR-0024
status: accepted
date: 2026-09-18
area: codegen
supersedes: []
superseded-by: []
related: ["ADR-0016", "ISS-0011", "ISS-0013"]
fv:
  [
    "informed by FV: BDL_FV Phase 9a (`ListData.lean`, D-83) makes `list τ` an
    unbounded kernel data type; capacity is a validation obligation
    (`Validation/Capacity.lean`, D-86), never a kernel bound",
  ]
---

# ADR-0024: A generated core that carries collections uses an allocator; one that does not stays allocation-free

## Status

Accepted (2026-09-18, the equation-library milestone,
`docs/spec/equation-library.md`).

## Context

The generated core was `no_std`, allocation-free and `Copy` end to end
(ADR-0016). The formal development adds `list τ` as ordinary, unbounded data
(Phase 9a) and derives the lossless cross-domain buffer from it; a bounded
representation would be a semantic change the formal development explicitly
rejects (`bounded_summary_not_lossless`, D-83). Production must represent a list
in the core without either changing its semantics or forcing every design — most
of which carry no list — to link an allocator.

## Decision

- A collection in the generated core is `alloc::vec::Vec<T>`, stored last
  element first (`cons` pushes, the recursor consumes from the front); a grouped
  value is a tuple. The runtime crate exposes the list operators and the
  recursor behind a `collections` feature.
- The generator turns the feature on, adds `extern crate alloc;`, derives
  `Clone` instead of `Copy`, and reads declarations and cells by clone **exactly
  when the plan carries a list** (`ExecIr::uses_lists`); the manifest records
  `requires_allocator`. A program without lists is generated as before:
  allocation-free, `Copy`, no `alloc` mention (a test checks the core mentions
  neither `alloc::` nor `Vec<`).
- No capacity is imposed by the core. Whether a target's memory suffices is a
  validation/deployment obligation (Phase 9a), to be computed by the toolchain
  (ISS-0011); overflow is never a silent policy of the generated code.

## Alternatives

- **A fixed-capacity list type** (`List<T, const N>`) with an overflow error:
  rejected — it would introduce a runtime failure the formal model does not have
  and make every list-carrying declaration's type depend on a deployment fact.
- **Always link `alloc`**: rejected — designs without collections would pay for
  what they do not use, and the "allocation-free" property of ADR-0016 would be
  lost for everyone.
- **A persistent (shared) list in the core**: rejected for now — `Rc`/`Arc` need
  an allocator too and add pointer-chasing on a microcontroller; the reversed
  `Vec` keeps `map`/`filter`/`append` linear (ISS-0013 records the one quadratic
  case that remains).

## Consequences

- A target that runs a list-carrying design must provide a global allocator; the
  first embedded platform adapter (roadmap priority 1) must offer one.
- The host bridge reverses lists at the boundary; the reference evaluator and
  the executable-IR interpreter use a shared cons list, so the three engines
  differ in representation and agree in behaviour (corpus cases `collections`,
  `buffer`).
- Records changed: `docs/architecture/codegen-rust.md`,
  `docs/spec/runtime-semantics.md`, `docs/project/status.md`, the change
  fragment `2026-09-equation-library.md`.
