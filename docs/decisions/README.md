# Architecture decision records

One record per consequential choice: why it was made and what else was
considered. A record is append-only once accepted — status, supersession links,
a dated amendment and factual corrections are the only edits. A replaced
decision is marked `superseded` and stays here; the current specification and
architecture pages cite only its replacement. IDs are never reused. The
lifecycle and the frontmatter fields are in
[governance.md](../project/governance.md); start a new record from
[TEMPLATE.md](TEMPLATE.md) and run `just docs-check`.

The two records marked _was ADR-00nn_ shared a number with an earlier record
until 2026-09-17 and were renumbered once
([migration report](../project/migration-report.md)); older commit messages and
documents may still use the old number.

| ID                                                                                 | Decision                                                                                                                            | Status                 | Date       | Area             | Superseded by |
| ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ---------- | ---------------- | ------------- |
| [ADR-0001](0001-flutter-does-not-own-semantic-truth.md)                            | Flutter does not own semantic truth                                                                                                 | accepted               | 2026-09-15 | studio           | —             |
| [ADR-0002](0002-bdld-is-a-process-boundary.md)                                     | bdld is a separate process                                                                                                          | accepted               | 2026-09-15 | daemon           | —             |
| [ADR-0003](0003-layout-is-separate-from-semantics.md)                              | Project layout is stored apart from semantics                                                                                       | accepted               | 2026-09-15 | persistence      | —             |
| [ADR-0004](0004-declarations-are-not-async-tasks.md)                               | BDL declarations are not Embassy tasks                                                                                              | accepted               | 2026-09-15 | runtime          | —             |
| [ADR-0005](0005-supplied-rust-cannot-drive-outputs.md)                             | Supplied Rust cannot access outputs                                                                                                 | accepted               | 2026-09-15 | runtime          | —             |
| [ADR-0006](0006-hardware-allocation-outside-typing.md)                             | Hardware allocation is outside typing                                                                                               | accepted               | 2026-09-15 | deployment       | —             |
| [ADR-0007](0007-protobuf-over-framed-stdio.md)                                     | Protobuf messages over framed child-process stdio                                                                                   | accepted               | 2026-09-15 | protocol         | —             |
| [ADR-0008](0008-sequential-stable-ids.md)                                          | Stable identities are sequential per-project integers                                                                               | accepted               | 2026-09-15 | persistence      | —             |
| [ADR-0009](0009-revisioned-edits-and-stale-results.md)                             | Revisioned edits; stale analysis results are discarded                                                                              | accepted               | 2026-09-15 | daemon           | —             |
| [ADR-0010](0010-lean-is-a-specification.md)                                        | The Lean development is a specification, not a dependency                                                                           | accepted               | 2026-09-15 | formal           | —             |
| [ADR-0011](0011-floats-not-nat.md)                                                 | Production numerics are IEEE floats, recorded as a deviation                                                                        | accepted               | 2026-09-15 | language         | —             |
| [ADR-0012](0012-studio-ui-references.md)                                           | Studio UI follows Resolve's pages, Blender's node editor, and the macOS HIG                                                         | accepted               | 2026-09-15 | studio           | —             |
| [ADR-0013](0013-formula-language-v0.md)                                            | Formula language v0 — names, rep/mk insertion, units, crate split                                                                   | accepted               | 2026-09-15 | language         | —             |
| [ADR-0014](0014-textual-syntax-infrastructure.md)                                  | Textual syntax infrastructure — Logos, hand-written event parser, Rowan                                                             | accepted               | 2026-09-15 | textual          | —             |
| [ADR-0015](0015-deployment-analysis-is-target-relative.md)                         | Deployment analysis is a separate, target-relative function                                                                         | accepted               | 2026-09-15 | deployment       | —             |
| [ADR-0016](0016-generated-rust-is-an-implementation-of-the-reference-evaluator.md) | Generated Rust is an implementation of the reference evaluator                                                                      | accepted               | 2026-09-15 | codegen          | —             |
| [ADR-0017](0017-lsp-is-an-adapter.md)                                              | LSP is an adapter over a semantic-first IDE service                                                                                 | accepted               | 2026-09-15 | ide              | —             |
| [ADR-0018](0018-three-information-levels.md)                                       | Semantics are shown as structure, explained in prose, named formally only on demand                                                 | accepted               | 2026-09-15 | studio           | —             |
| [ADR-0019](0019-behaviour-groups-are-authoring-metadata.md)                        | Behaviour groups are authoring metadata; aggregate sockets are a projection; packaging elaborates into the existing component model | accepted               | 2026-09-17 | behavior-systems | —             |
| [ADR-0020](0020-textual-workspace-and-source-identities.md)                        | A textual project is canonical source plus a source-identity sidecar                                                                | superseded by ADR-0023 | 2026-09-17 | textual          | —             |
| [ADR-0021](0021-behaviour-systems-flatten-into-the-flat-design.md)                 | Behaviour systems are a surface layer that flattens into the flat design (was ADR-0017)                                             | accepted               | 2026-09-16 | behavior-systems | —             |
| [ADR-0022](0022-a-component-interface-is-a-stored-promise.md)                      | A component's public interface is a stored promise, realized by its body (was ADR-0018)                                             | accepted               | 2026-09-16 | behavior-systems | —             |
| [ADR-0023](0023-one-bdl-project-with-code-and-design-views.md)                     | There is one BDL project; Design, Code and Split are views of it                                                                    | accepted               | 2026-09-17 | persistence      | —             |

Areas: language · textual · compiler · runtime · codegen · persistence ·
protocol · daemon · ide · studio · behavior-systems · deployment · formal ·
process. Formal backing, where any, is in each record's `fv` field with a
strength label and in
[formal-correspondence.md](../project/formal-correspondence.md).
