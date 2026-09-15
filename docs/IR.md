# Intermediate representations

```
Surface Model  ──elaboration──▶  Design IR  ──lowering──▶  Reactive Core IR  ──codegen──▶  Rust backend AST
(bdl-model)                      (bdl-ir)                  (bdl-ir)                       (bdl-codegen-rust)
```

Flutter JSON is never translated directly into Rust.

## Surface expression AST (`bdl-syntax::ast`)

What the designer typed, with byte spans: `Name | Number{unit?} | Bool |
Unary | Binary | If`. Produced by a Pratt parser, consumed only by
`bdl-elab`; never stored, never sent to Studio.

## Surface Model (`bdl-model::surface`)

Designer-level forms; may be incomplete; may contain constructs the kernel
does not have. Today: `Concept { representation? }`, `MappingBlock
{ signature, definition? }`, `Definition::Formula`. Planned: curves, example
sets, temporal modifiers, contexts, supplied components, device kinds, units.

## Design IR (`bdl-ir::design`)

The kernel's environments, faithfully: `(Θ, Δ, Κ, Ω, β)` —

* `ConceptBinding { id, representation: Option<Ty> }` — `Θ`
* `Declaration { id, interface: Interface { expected_type, commitments }, realization: Option<Expr> }` — `Δ`
* `ClockEnv = BTreeMap<DeclId, ClockId>` — `Κ`
* `OutputSpec { accepts, clock }` — `Ω`; `DriveEnv = BTreeMap<DeclId, OutputId>` — `β`

`DesignIr::ty_view` is the only projection typing may consult.

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

## Deliberate deviations from the Lean development

Recorded so nobody mistakes them for the formal model (see `DESIGN_ISSUES.md`):

| | Lean | Here | Why |
|---|---|---|---|
| numeric literals | `Nat` (truncating `-`, floor `/`) | `Scalar(f64)` with bit-pattern equality | products measure reals; DI-1 |
| base dimensions | length, time, angle | 7 SI + angle (`i8` exponents) | the paper says three were "enough to test the abstraction" |
| commitments | abstract `PropertyId` | closed enum `{Monotone, Deterministic, Total, BoundedRange}` | needs a vocabulary; grows with the validation layer |
| environments | functions `DeclId → Option _` | `BTreeMap` | determinism, serialization |
