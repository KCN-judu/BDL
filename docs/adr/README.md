# Architecture decision records

One file per decision, numbered, never edited after acceptance except to
mark superseded. Format: Context · Decision · Consequences.

| # | Decision |
|---|---|
| [0001](0001-flutter-does-not-own-semantic-truth.md) | Flutter does not own semantic truth |
| [0002](0002-bdld-is-a-process-boundary.md) | bdld is a separate process |
| [0003](0003-layout-is-separate-from-semantics.md) | Project layout is stored apart from semantics |
| [0004](0004-declarations-are-not-async-tasks.md) | BDL declarations are not Embassy tasks |
| [0005](0005-supplied-rust-cannot-drive-outputs.md) | Supplied Rust cannot access outputs |
| [0006](0006-hardware-allocation-outside-typing.md) | Hardware allocation is outside typing |
| [0007](0007-protobuf-over-framed-stdio.md) | Protobuf messages over framed child-process stdio |
| [0008](0008-sequential-stable-ids.md) | Stable identities are sequential per-project integers |
| [0009](0009-revisioned-edits-and-stale-results.md) | Revisioned edits; stale analysis results are discarded |
| [0010](0010-lean-is-a-specification.md) | The Lean development is a specification, not a dependency |
| [0011](0011-floats-not-nat.md) | Production numerics are IEEE floats, recorded as a deviation |
| [0012](0012-studio-ui-references.md) | Studio UI follows Resolve's pages, Blender's node editor, and the macOS HIG |
| [0013](0013-formula-language-v0.md) | Formula language v0 — names, rep/mk insertion, units, crate split |
| [0014](0014-textual-syntax-infrastructure.md) | Textual syntax infrastructure — Logos, hand-written event parser, Rowan |
| [0015](0015-deployment-analysis-is-target-relative.md) | Deployment analysis is a separate, target-relative function; outputs are not a mapping status |
