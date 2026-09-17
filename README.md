# BDL — Behavior Design Language

Engineering implementation of **BDL**, a behavior design language for
industrial designers: typed semantic relationships (`?dimByTilt : Tilt ->
Brightness`) are first-class design artifacts, an unresolved relationship is
a legal state of the design, and every designer-facing construct elaborates
onto a small formal kernel.

The kernel was derived in the Lean 4 development
[KCN-judu/BDL_FV](https://github.com/KCN-judu/BDL_FV). This repository builds
what that development deliberately did not: **BDL Studio** (Flutter), the
**compiler service `bdld`** (Rust), the simulator, the hardware allocator,
the Rust code generator, the embedded runtime, and deployment tooling. It
follows the formally developed semantics; it is not itself formally verified.

## Status

Milestone 1 (the vertical slice, docs/ROADMAP.md) is implemented through
step O: a design authored in Studio or as text is elaborated onto the
kernel, type- and dimension-checked, checked for causality and timing
domains, checked for physical-output well-formedness, simulated by the
reference evaluator, placed on a board by the hardware allocator, and
compiled to a `no_std` Rust core whose host execution is held trace for
trace to the evaluator. Studio exposes this on three pages — Design
(canvas, inspector, compiler-backed definition editor with completion and
hover, timing domains and outputs as objects), Simulate and Deploy — over
`bdld`; a shared IDE service also serves an LSP; a Standard Concept
Library supplies concept templates; `examples/smart_lamp` walks the slice
end to end. Not built: an embedded platform adapter (Embassy), build/flash
orchestration, telemetry and the Monitor page, contexts, supplied Rust
components, behaviour systems.

## Layout

```
apps/studio/        Flutter BDL Studio (presentation; semantic truth comes from bdld)
crates/             the Rust workspace — model, IR, syntax, elaboration, checking, reactive
                    semantics, outputs, hardware, executable IR, lowering, Rust codegen,
                    compiler driver, concept libraries, IDE service, LSP, protocol, bdld
                    (docs/ARCHITECTURE.md lists each crate and its boundary)
runtime/            bdl-runtime-core (no_std vocabulary of generated cores), bdl-runtime-host
hardware/boards/    board descriptions as data (arduino_nano, big_board)
library/std/        the Standard Concept Library (concepts.toml)
examples/           smart_lamp — the canonical Design → Simulate → Deploy example
docs/               architecture, formats, pipeline, ADRs, paper digest
reference/paper/    the BDL paper
```

## Build

Requirements: Rust 1.89 (pinned in `rust-toolchain.toml`, which also pulls
`rustfmt`, `clippy`, `rust-analyzer` and `rust-src` via rustup), Flutter 3.47
(`brew install --cask flutter`),
`just`; `protoc` + `protoc-gen-dart` only to regenerate the Dart protocol code.

```bash
just check          # fmt, clippy, tests, Flutter analyze/test, proto drift
just studio         # build bdld and run Studio against it (run from Terminal/Finder-launched
                    # shells: macOS refuses file dialogs to children of sandboxed hosts)
just bdld           # run the daemon on stdio for manual experiments
```

## Reading order

Using the tool rather than building it? Start with the
[user guide](docs/user-guide/README.md).

0. [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md) — how to enter the project (toolchain, map, run, reading order by task)

1. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — the four trust layers and why
2. [docs/adr/](docs/adr/README.md) — decisions
3. [docs/STUDIO_UI.md](docs/STUDIO_UI.md) — what Studio looks like and why (Resolve · Blender · macOS); [docs/STUDIO_COMPILER_INTEGRATION.md](docs/STUDIO_COMPILER_INTEGRATION.md) — which semantic capabilities Studio exposes, audited against the code
4. [docs/01-paper-digest.md](docs/01-paper-digest.md) · [docs/02-kernel-spec.md](docs/02-kernel-spec.md) — the language
5. [docs/PROTOCOL.md](docs/PROTOCOL.md) · [docs/PROJECT_FORMAT.md](docs/PROJECT_FORMAT.md) · [docs/COMPILER_PIPELINE.md](docs/COMPILER_PIPELINE.md)
6. [docs/EXECUTION_WALKTHROUGH.md](docs/EXECUTION_WALKTHROUGH.md) · [docs/DEPLOYMENT_WALKTHROUGH.md](docs/DEPLOYMENT_WALKTHROUGH.md) · [docs/DEPLOYMENT_READ_MODEL.md](docs/DEPLOYMENT_READ_MODEL.md) — one value through a tick; one design out to a pin; what a Deploy page is handed
7. [docs/EXECUTABLE_IR.md](docs/EXECUTABLE_IR.md) · [docs/CODEGEN_RUST.md](docs/CODEGEN_RUST.md) — from a checked design to a `no_std` Rust core, and how it is held to the reference evaluator
8. [docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md](docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md) · [docs/BEHAVIOR_SYSTEMS.md](docs/BEHAVIOR_SYSTEMS.md) — reusable behaviour components, instances and bindings, and how they flatten into the one flat design
8. [docs/IDE_SERVICE_ARCHITECTURE.md](docs/IDE_SERVICE_ARCHITECTURE.md) · [docs/TEXTUAL_SYNTAX.md](docs/TEXTUAL_SYNTAX.md) · [docs/STANDARD_CONCEPT_LIBRARY.md](docs/STANDARD_CONCEPT_LIBRARY.md) — the shared language service, the textual surface, the concept templates
