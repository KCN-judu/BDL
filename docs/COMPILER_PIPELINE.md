# Compiler pipeline

Explicit passes with explicit input/output types. No giant visitor. Every
pass runs on an immutable `ProjectSnapshot` and returns
`Result<Output, Diagnostics>`-shaped data; diagnostics accumulate, the
pipeline does not fail fast.

| # | Pass | Input → Output | Kernel notion | Status |
|---|---|---|---|---|
| 1 | load / parse | files → `ProjectSnapshot` | — | done (`bdl-model::persist`) |
| 2 | identity resolution | surface → ids resolved (names never used as refs) | `SemanticId`, `DeclId` | done by construction (model refers by id) |
| 3 | signature / interface resolution | `Signature` over concepts → `Interface { expected_type, commitments }` | `DeclInterface` | **integrated** (`bdl-elab::elaborate_interface`; commitments empty) |
| 4 | surface elaboration | formula → `Expr` (`λ…mk B (… rep xᵢ …)`), units → scaled literals | grant, `rep`/`mk` | **partial**: formulas integrated (`bdl-syntax` + `bdl-elab::formula`); curves, examples, components, temporal phrases, contexts planned |
| 5 | type checking | `Expr` against `ty_view` + `Θ` | `HasType`, `infer` | **core rules complete, integrated** (`bdl-check::infer`, incl. `delay`/`sync`) |
| 6 | semantic-construction checking | `mk s` only under grant | `Grant` | **integrated** (`bdl-check::Grant`, checked inside `infer`) |
| 7 | dimension checking | falls out of 5 via `Prim::ty`; the elaborator reports mismatches with spans first | `q Dim` | **integrated** (no separate pass) |
| 8 | dependency analysis | `refs` / `inst_refs` graph | `DependsOn`, `InstDependsOn` | planned |
| 9 | causality | acyclic instantaneous graph, rank | `Causal` | planned |
| 10 | clock-domain checking | `Clocked` judgment per realization | `Κ`, `Clocked` | planned |
| 11 | physical-output checking | `DriveWF`, `SingleDriver`, completeness | `β`, `Ω` | planned |
| 12 | hardware requirement generation | sinks × device kinds → `Requirements` | validation layer | planned (`bdl-hardware`) |
| 13 | hardware allocation | `solve` / `diagnose` | validation layer | planned |
| 14 | reactive lowering | Design IR → per-domain step schedule, state cells | `Ev` / `MEv` | planned (`bdl-reactive`) |
| 15 | Rust code generation | Core IR → backend AST → Cargo project + `bdl-manifest.json` | erasure | planned (`bdl-codegen-rust`) |

## Driver and result

`bdl-compiler::analyze(&ProjectSnapshot) -> ProjectAnalysis` runs passes
1–7 and returns, tagged with the snapshot's revision: per mapping the
elaborated `Interface`, a status (`Declared | Open | Invalid | TypeValid`),
the inferred type and Core term when it checks, and its diagnostics; plus
all diagnostics in the documented order (entity, span, code, message).
`bdld` answers `RunAnalysis` with it and pushes `AnalysisReady` after every
committed revision; Studio keeps an analysis only while its revision equals
the projection's.

## Diagnostic codes (v0)

| Code | Pass | Severity |
|---|---|---|
| `formula.parse.unexpected_token` | parse | error |
| `formula.name.unknown` · `formula.name.ambiguous` · `formula.name.not_an_input` | elab | error |
| `formula.unit.unknown` | elab | error |
| `type.operand_kind` · `type.branch_mismatch` | elab | error |
| `dimension.mismatch` | elab (type-driven) | error |
| `realization.type_mismatch` | elab / check | error |
| `semantic.unbound_representation` | elab / check | info (the mapping is *open*) |
| `semantic.construction_not_granted` | check | error |
| `type.argument_mismatch` · `type.expected_function` · `type.rep_of_non_semantic` · `type.temporal_*` · `type.unbound_variable` · `type.unknown_declaration` | check | error |

## Incremental invalidation

Each `EditOp` yields an `EditOutcome { kind, invalidates, origin_decls }`.
Analyses declare which categories they depend on:

| Change | Invalidates | Preserves |
|---|---|---|
| attach/replace formula body | Realization, Reactive (that decl) | public type, downstream typing |
| change signature | Interface, Semantic (+Realization if defined) | — |
| rebind concept representation | Semantic, Realization (users of the concept) | — |
| assign/change clock (later) | Clock | typing |
| retarget output (later) | Output | typing, clocks |
| change board / fixed pin (later) | Deployment only | everything semantic |

## Diagnostics

Structured, never a panic, never a bare kernel predicate:

```
Diagnostic { code, severity, entity, relation/span, message, explanation, suggested_fixes, technical_details }
```

`message` speaks product language ("this output already has a final
driver, `dimByTilt`; combine the two brightness values before connecting
the output"); `technical_details` carries the kernel vocabulary for the
explanation view. Stable `code`s (`edit.duplicate_concept_name`,
`edit.stale_revision`, …) are the machine contract.
