# BDL — Behavior Design Language

Engineering implementation of **BDL**, a behavior design language for industrial
designers: typed semantic relationships (`?dimByTilt : Tilt -> Brightness`) are
first-class design artifacts, an unresolved relationship is a legal state of the
design, and every designer-facing construct elaborates onto a small formal
kernel.

The kernel was derived in the Lean 4 development
[KCN-judu/BDL_FV](https://github.com/KCN-judu/BDL_FV). This repository builds
what that development deliberately did not: **BDL Studio** (Flutter), the
**compiler service `bdld`** (Rust), the simulator, the hardware allocator, the
Rust code generator, the embedded runtime, and deployment tooling. It follows
the formally developed semantics; it is not itself formally verified.

## Status

A design authored in Studio or as `.bdl` text is elaborated onto the kernel,
type- and dimension-checked, checked for causality and timing domains and for
physical-output well-formedness, simulated by the reference evaluator, placed on
a board by the hardware allocator, and compiled to a `no_std` Rust core held
trace for trace to the evaluator; reusable behaviours compose as components and
flatten into that one design. Not built: an embedded platform adapter,
build/flash orchestration, telemetry and the Monitor page, contexts, supplied
Rust components. The [status matrix](docs/project/status.md) is the authority
for what exists, the [roadmap](docs/project/roadmap.md) for what is next, and
[docs/changes/](docs/changes/README.md) for what changed.

## Layout

```text
apps/studio/        Flutter BDL Studio (presentation; semantic truth comes from bdld)
crates/             the Rust workspace — model, IR, syntax, elaboration, checking, reactive
                    semantics, outputs, hardware, executable IR, lowering, Rust codegen,
                    compiler driver, the Standard Library, IDE service, LSP, protocol, bdld
                    (docs/architecture/overview.md lists each crate and its boundary)
runtime/            bdl-runtime-core (no_std vocabulary of generated cores), bdl-runtime-host
hardware/boards/    board descriptions as data (arduino_nano, big_board)
library/std/        the Standard Library (concepts.toml)
examples/           smart_lamp — the canonical Design → Simulate → Deploy example
docs/               architecture, formats, pipeline, ADRs, paper digest, the user guide
locale/             the terminology glossary and the user guide's zh-Hans / ja catalogs and pages
reference/paper/    the BDL paper
```

## Build

Requirements: Rust 1.89 (pinned in `rust-toolchain.toml`, which also pulls
`rustfmt`, `clippy`, `rust-analyzer` and `rust-src` via rustup), Flutter 3.47
(`brew install --cask flutter`), `just`; `protoc` + `protoc-gen-dart` only to
regenerate the Dart protocol code.

```bash
just check          # everything the Linux CI jobs prove (python scripts/preflight.py full)
python scripts/preflight.py fast   # before a commit; `platform` for the Windows-sensitive checks
just studio         # build bdld and run Studio against it (run from Terminal/Finder-launched
                    # shells: macOS refuses file dialogs to children of sandboxed hosts)
just bdld           # run the daemon on stdio for manual experiments
```

## Reading order

Using the tool rather than building it? Start with the user guide —
[English](docs/user-guide/README.md) ·
[简体中文](locale/user-guide/zh_Hans/README.md) ·
[日本語](locale/user-guide/ja/README.md). Studio itself speaks the same three
languages (_Preferences…_); the rules are in
[localization-style.md](docs/project/localization-style.md).

Building it? Start at the
[engineering documentation front door](docs/README.md), which routes every
question to one page. The folders under `docs/` are one per kind of record:
`spec/` (what BDL means), `architecture/` (how it is built), `decisions/` (why —
ADRs), `proposals/` and `issues/` (what is undecided), `project/` (status,
roadmap, governance), `changes/` (what changed), `evidence/`, `guides/`
([getting started](docs/guides/getting-started.md) is the first read),
`background/`, `archive/`.
