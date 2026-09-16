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
| [0016](0016-generated-rust-is-an-implementation-of-the-reference-evaluator.md) | Generated Rust is an implementation of the reference evaluator — explicit executable IR, owned AST, differential tests |
| [0017](0017-behaviour-systems-flatten-into-the-flat-design.md) | Behaviour systems are a surface layer that flattens into the flat design — no kernel construct, one BDL |
| [0018](0018-a-component-interface-is-a-stored-promise.md) | A component's public interface is a stored promise, realized by its body — contracts, Realizes, substitution on interfaces |
| [0019](0019-behaviour-groups-are-authoring-metadata.md) | Behaviour groups are authoring metadata; aggregate sockets are a projection; packaging elaborates into the existing component model — no new kernel or runtime semantics |
| [0017](0017-lsp-is-an-adapter.md) | LSP is an adapter over a semantic-first IDE service — `lsp-server` at the edge, one service for Studio and text editors, GLSP borrowed not adopted |
| [0018](0018-three-information-levels.md) | Semantics as structure; formal vocabulary only in Explain |
