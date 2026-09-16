# Intermediate representations

```
Surface Model ──elaboration──▶ Design IR (Θ Δ Κ Ω β over Reactive Core IR) ──reactive lowering──▶ Executable IR ──codegen──▶ Rust AST ──print──▶ crate
(bdl-model)                    (bdl-ir)                                      (bdl-lower → bdl-exec-ir)         (bdl-codegen-rust)
```

Flutter JSON is never translated directly into Rust.

## Textual syntax (`bdl-syntax`)

```
source ──Logos──▶ tokens ──event parser──▶ Rowan CST ──ast──▶ typed views ──lower──▶ surface tree
```

The CST is lossless (every token, comment and error region; `text() ==
source`) and is what editor tooling reads. The *surface tree*
(`bdl-syntax::lower`) is what the compiler reads: `SurfaceModule` of
concept / mapping / enum items, and `SurfaceExpr` — `Name | Number{literal,
unit?} | Bool | Unary | Binary | If | Call | Match | Block` — with byte
spans and **exact** literals (`NumberLiteral` is the spelling; `f64` is
made only in `bdl-elab`, DI-18). Consumed by `bdl-elab`; never stored,
never sent to Studio. Grammar and precedence: `docs/TEXTUAL_SYNTAX.md`.

## Surface Model (`bdl-model::surface`)

Designer-level forms; may be incomplete; may contain constructs the kernel
does not have. Today: `Concept { representation? }`, `MappingBlock
{ signature, definition?, clock?, drives? }`, `Definition::Formula`,
`ClockDomain`, `PhysicalOutput { accepts: SemanticId, clock?, required }`,
`DeviceBinding { kind: DeviceKind, output?, fixed_pins }`. Temporal forms
and units are not separate surface objects: `delay`/`sync` are written
inside `Definition::Formula` and units are `Representation::Quantity { dim }`
plus unit-suffixed literals in formulas. Planned: curves, example sets,
contexts, supplied components.

## Design IR (`bdl-ir::design`)

The kernel's environments, faithfully: `(Θ, Δ, Κ, Ω, β)` —

* `ConceptBinding { id, representation: Option<Ty> }` — `Θ`
* `Declaration { id, interface: Interface { expected_type, commitments }, realization: Option<Expr> }` — `Δ`
* `ClockEnv = BTreeMap<DeclId, ClockId>` — `Κ`
* `OutputSpec { accepts, clock }` — `Ω`; `DriveEnv = BTreeMap<DeclId, OutputId>` — `β`

`DesignIr::ty_view` is the only projection typing may consult.

Elaboration fills `Ω` from every surface output *that has a domain*
(`accepts = sem concept`); an output without one is open and absent from
`Ω`. Every authored drive edge goes into `β` as is — `DriveWF` is what
rejects an edge whose sink is open or does not fit, and the output pass
reports it (`bdl-output`). `clock_names` / `output_names` are display
names for diagnostics only, never consulted by any judgment.

## Reactive Core IR (`bdl-ir::expr`, `bdl-ir::ty`)

```
Ty   ::= bool | nat | arr | sem s | q d | opt
Expr ::= var i | boolLit | natLit | lam | app | declRef d | rep e | mk s e | prim p
       | delay init e | sync src init e
Prim ::= lit d n | add d | sub d | mul d₁ d₂ | div d₁ d₂ | lt d | eq d | not | and | or
       | ite τ | none τ | some τ | isSome τ | getD τ
```

Surface concepts (`previous`, `hold`, `count`, `rise`, contexts, priority,
blend) do not exist here. This is the alignment point between the formal
semantics, the reference interpreter, and the code generator.

### Elaboration of a formula (implemented)

`dimByTilt : (Tilt, Held) -> Brightness` with `if Held then Tilt / 90 deg else 0`:

```
λ(sem#0). λ(sem#3). (mk sem#1
   (ite[q[1]] (rep #0) (div[rad,rad] (rep #1) 1.5708[rad]) 0[1]))
```

Inputs are observed with `rep` (innermost binder is the *last* input), the
result is constructed with `mk` under the declaration's own grant, units are
scaled literals, and every primitive is dimension-indexed so the checker's
ordinary application rule enforces dimensions.

## Executable IR (`bdl-exec-ir`) and the Rust AST (`bdl-codegen-rust::ast`)

After analysis, `bdl-lower` turns a checked Design IR into a plan with
dense clock/input/state/output slots, first-order expressions (lambdas
inlined) and an evaluation order — `docs/EXECUTABLE_IR.md`. The Rust
backend prints it through a small owned AST — `docs/CODEGEN_RUST.md`.
Neither is a semantic layer: the reference evaluator over the Design IR
remains the definition, and the differential tests hold the rest to it.

## Deliberate deviations from the Lean development

Recorded so nobody mistakes them for the formal model (see `DESIGN_ISSUES.md`):

| | Lean | Here | Why |
|---|---|---|---|
| numeric literals | `Nat` (truncating `-`, floor `/`) | `Scalar(f64)` with bit-pattern equality | products measure reals; DI-1 |
| base dimensions | length, time, angle | 7 SI + angle (`i8` exponents) | the paper says three were "enough to test the abstraction" |
| commitments | abstract `PropertyId` | closed enum `{Monotone, Deterministic, Total, BoundedRange}` | needs a vocabulary; grows with the validation layer |
| environments | functions `DeclId → Option _` | `BTreeMap` | determinism, serialization |
