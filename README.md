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

Milestone 1 (vertical slice), steps A–F: Studio connects to `bdld`, shows
compiler/protocol versions, and edits concepts and (possibly unresolved)
mappings on a node canvas with a Resolve-style page workflow and a macOS
look; projects save and reopen crash-safely. Next: the architecture review gate,
then the checker. See [docs/ROADMAP.md](docs/ROADMAP.md).

## Layout

```
apps/studio/        Flutter BDL Studio (presentation; semantic truth comes from bdld)
crates/
  bdl-model/        stable ids · surface model · revisioned edits · persistence
  bdl-ir/           Design IR · Reactive Core IR (the kernel's types and terms)
  bdl-protocol/     protobuf schema · framing · conversions
  bdl-daemon/       bdld — the compiler service
runtime/            generated-core runtimes (planned)
hardware/           board & device descriptions (planned)
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

1. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — the four trust layers and why
2. [docs/adr/](docs/adr/README.md) — decisions
3. [docs/STUDIO_UI.md](docs/STUDIO_UI.md) — what Studio looks like and why (Resolve · Blender · macOS)
4. [docs/01-paper-digest.md](docs/01-paper-digest.md) · [docs/02-kernel-spec.md](docs/02-kernel-spec.md) — the language
5. [docs/PROTOCOL.md](docs/PROTOCOL.md) · [docs/PROJECT_FORMAT.md](docs/PROJECT_FORMAT.md) · [docs/COMPILER_PIPELINE.md](docs/COMPILER_PIPELINE.md)
