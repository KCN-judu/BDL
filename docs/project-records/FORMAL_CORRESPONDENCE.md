# Formal correspondence

What the Lean development in `../BDL_FV` (KCN-judu/BDL_FV) proves, what
production follows from it, and how strongly each production claim is
backed. This is the one table to consult before writing "proved"
anywhere. The formal repository is the authority for its own history
(`BDL/DESIGN_DECISIONS.md`, `BDL/MINIMALITY.md`, the phase notes); this
page keeps only the correspondence (ADR-0010: Lean is a specification, not
a dependency).

Strength labels — used here, in ADR `fv` fields and in `STATUS.md`:

| Label | Means |
|---|---|
| **formally proved** | the named theorem proves the stated property of the *formal model*. It never proves the Rust or Dart code; production discharges it by transcription plus tests |
| **informed by FV** | a formal result or counterexample bounded the engineering choice |
| **production-tested** | executable tests (differential, golden, property) support the production claim |
| **engineering choice** | no formal backing is claimed; recorded as a deviation where it departs from the kernel |

## Kernel (Phases 1–7, `BDL/Core`, `BDL/Validation`)

`docs/02-kernel-spec.md` transcribes the kernel; `docs/COMPILER_PIPELINE.md`
implements it pass by pass.

| Production claim | Formal source | Strength | Production evidence |
|---|---|---|---|
| a design is well-formed when every declaration's realization satisfies its interface; refinement preserves it | `BDL/Core/Satisfaction.lean`, `BDL/Core/Env.lean` (Phases 1–2) | formally proved (model) · production-tested | `bdl-check`; `crates/bdl-compiler/tests/surface_expressions.rs` |
| semantic identity: two concepts with equal representations are distinct types | `BDL/Core/Base.lean`, `Experiments/SemanticTypeAlternatives.lean` (Phase 2) | formally proved · production-tested | `crates/bdl-system/tests/contracts.rs` (*equal representations are not equal concepts*) |
| representation binding and dimension checking | `BDL/Core/Typing.lean` (Phase 3) | formally proved (model); dimensions extended from 3 to 7 SI + angle | engineering choice (DI-2); `bdl-check::typing` |
| numerics: the kernel computes over `Nat`; production uses IEEE `f64` with division-by-zero and non-finite results as tick failures | — | **engineering choice**, recorded deviation (ADR-0011, DI-1, DI-15) | `bdl-reactive` runtime errors; differential tests restricted to integer-valued cases |
| dependency graph, `Causal` (conservative for lambda-guarded cycles) | `BDL/Core/Dependency.lean`, `Reactive.lean` (Phases 1, 4) | formally proved · production-tested | `bdl-reactive::causality`; DI-8 records the conservative refusal |
| two-phase reactive evaluation, `delay` state, traces | `BDL/Core/Reactive.lean` (Phase 4) | formally proved (model) · production-tested | `bdl-reactive::simulate`; `docs/RUNTIME_SEMANTICS.md` |
| clock domains, `Clocked`, `sync` reads the source's last activation strictly before now | `BDL/Core/Clock.lean` (Phase 5) | formally proved · production-tested | `bdl-reactive::clocks`; DI-16 (agnostic declarations) is an engineering choice the kernel leaves open |
| physical outputs: `DriveWF`, `SingleDriver`, `CompleteOutputs` | `BDL/Core/Output.lean` (Phase 6) | formally proved · production-tested | `bdl-output`; DI-20 (only nullary drivers) engineering choice |
| hardware feasibility: allocation is sound and complete over capabilities, units and sharing; `diagnose` returns *a* dead end | `BDL/Validation/Hardware.lean`, `Capacity.lean` (Phase 7) | formally proved (model) · production-tested | `bdl-hardware` golden cases; DI-9, DI-21, DI-22 bound the claim ("pins can be allocated", not "the circuit works") |
| generated Rust behaves as the reference evaluator | — | **production-tested only** (ADR-0016): corpus, golden and property differential tests on the host | `crates/bdl-compiler/tests/backend_*.rs` |
| list data and lossless buffered transport | `BDL/Core/ListData.lean`, `BDL/Validation/Buffer.lean` (Phase 9a, `fad79d9`) | formally proved (model) — **not mirrored in production** | ISS-0001 |

## Behaviour systems (Phases 8a–8b, `BDL/Behavior`)

`docs/BEHAVIOR_SYSTEMS.md` carries the object-by-object table with every
theorem name and the test that discharges it; the summary:

| Production claim | Formal source | Strength | Production evidence |
|---|---|---|---|
| fresh instantiation is injective and disjoint from global ids | Theorem A (`inst_*_disjoint`, `Instantiate.lean`) | formally proved · production discharges by the allocator's never-reuse contract | `instances_never_alias_and_shared_identity_is_kept` |
| a binding is an ordinary realization the checker verifies | Theorem C `binding_satisfies` (`System.lean`) | formally proved · production-tested | `bdl-elab::elaborate_reference` |
| the flat judgments are the only judgments (WF, causality, clocks, single driver) | Theorems D–I (`Preservation.lean`) | formally proved · production-tested | `crates/bdl-system/tests/vertical_slice.rs` |
| a modular design and its flattening agree on behaviour | Theorem J `modular_iff_flat` — **restricted** to single-domain wiring designs with direct/constant bindings (D-73) | formally proved for the fragment · production-tested for one example beyond it | `system_and_hand_written_flat_agree_on_every_observable` |
| grouping is semantically transparent; boundary sockets add no dependency | Theorems A–G, H `socket_no_fanout`, I–L (`Group.lean`, `Boundary.lean`, Phase 8b) | formally proved · production-tested | `grouping_is_semantically_transparent`, `boundary_is_a_projection_and_sockets_add_no_dependency` |
| packaging a group preserves behaviour | Theorem R `orig_iff_flat` — **restricted** to single-domain wiring designs without transported bindings | formally proved for the fragment · production-tested beyond it (a `sync`-only clock, a transported binding after packaging) | `extraction_is_a_differential_witness_of_theorem_r` |
| a component interface is a stored promise; substitution on interfaces | `Interface.lean`, `Substitution.lean` (`IfaceRefines`, `Substitutable`) | informed by FV · production-tested | `crates/bdl-system/tests/contracts.rs`; ADR-0022 |

## Not covered by the formal development

Studio interaction, the IDE service, the textual syntax and its identity
sidecar (ADR-0014, ADR-0017, ADR-0020), persistence (ADR-0003, ADR-0008),
the protocol (ADR-0007), code generation (ADR-0016) and the standard
concept library are **engineering choices** with production tests. No
theorem is claimed for them; a Lean result about the kernel does not
transfer to them.
