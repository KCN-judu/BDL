---
kind: architecture
area: compiler
status: current
---

# Intermediate representations

```text
Surface Model ──elaboration──▶ Design IR (Θ Δ Κ Ω β over Reactive Core IR) ──reactive lowering──▶ Executable IR ──codegen──▶ Rust AST ──print──▶ crate
(bdl-model)                    (bdl-ir)                                      (bdl-lower → bdl-exec-ir)         (bdl-codegen-rust)
```

Flutter JSON is never translated directly into Rust.

## Textual syntax (`bdl-syntax`)

```text
source ──Logos──▶ tokens ──event parser──▶ Rowan CST ──ast──▶ typed views ──lower──▶ surface tree
```

The CST is lossless (every token, comment and error region; `text() == source`)
and is what editor tooling reads. The _surface tree_ (`bdl-syntax::lower`) is
what the compiler reads: `SurfaceModule` of concept / mapping / enum items, and
`SurfaceExpr` —
`Name | Number{literal, unit?} | Bool | Unary | Binary | If | Call | Match | Block`
— with byte spans and **exact** literals (`NumberLiteral` is the spelling; `f64`
is made only in `bdl-elab`, DI-18). Consumed by `bdl-elab`; never stored, never
sent to Studio. Grammar and precedence: `docs/spec/textual-syntax.md`.

## Surface Model (`bdl-model::surface`)

Designer-level forms; may be incomplete; may contain constructs the kernel does
not have. Today: `Concept { representation? }`,
`MappingBlock { signature, definition?, clock?, drives? }`,
`Definition::Formula`, `ClockDomain`,
`PhysicalOutput { accepts: SemanticId, clock?, required }`,
`DeviceBinding { kind: DeviceKind, output?, fixed_pins }`. Temporal forms and
units are not separate surface objects: `delay`/`sync` are written inside
`Definition::Formula` and units are `Representation::Quantity { dim }` plus
unit-suffixed literals in formulas. Planned: curves, example sets, contexts,
supplied components.

## Design IR (`bdl-ir::design`)

The kernel's environments, faithfully: `(Θ, Δ, Κ, Ω, β)` —

- `ConceptBinding { id, representation: Option<Ty> }` — `Θ`
- `Declaration { id, interface: Interface { expected_type, commitments }, realization: Option<Expr> }`
  — `Δ`
- `ClockEnv = BTreeMap<DeclId, ClockId>` — `Κ`
- `OutputSpec { accepts, clock }` — `Ω`; `DriveEnv = BTreeMap<DeclId, OutputId>`
  — `β`

`DesignIr::ty_view` is the only projection typing may consult.

Elaboration fills `Ω` from every surface output _that has a domain_
(`accepts = sem concept`); an output without one is open and absent from `Ω`.
Every authored drive edge goes into `β` as is — `DriveWF` is what rejects an
edge whose sink is open or does not fit, and the output pass reports it
(`bdl-output`). `clock_names` / `output_names` are display names for diagnostics
only, never consulted by any judgment.

## Reactive Core IR (`bdl-ir::expr`, `bdl-ir::ty`)

```text
Ty   ::= bool | nat | arr | sem s | q d | opt τ | list τ | prod τ σ | ()      -- () is production-only, below
Expr ::= var i | boolLit | natLit | lam | app | declRef d | rep e | mk s e | prim p
       | delay init e | sync src init e | fold f z l
Prim ::= lit d n | add d | sub d | mul d₁ d₂ | div d₁ d₂ | lt d | eq τ | not | and | or
       | ite τ | none τ | some τ | isSome τ | getD τ
       | nil τ | cons τ | length τ | take τ | drop τ | reverse τ | head τ | toList τ
       | pair τ σ | fst τ σ | snd τ σ
```

`fold f z l` (child paths `f` = 0, `z` = 1, `l` = 2) is the list recursor — the
one term former that applies a function value, never general recursion. `eq τ`
is structural equality at any data type; `lt d` compares quantities only (Phase
9c). The checker (`bdl-check`) has the `fold` rule and refuses `eq` at a
function type (`EqualityNotData`).

Surface concepts (`previous`, `hold`, `count`, `rise`, contexts, priority,
blend) do not exist here. This is the alignment point between the formal
semantics, the reference interpreter, and the code generator.

### Elaboration of a formula (implemented)

`dimByTilt : (Tilt, Held) -> Brightness` with
`if Held then Tilt / 90 deg else 0`:

```text
λ(sem#0). λ(sem#3). (mk sem#1
   (ite[q[1]] (rep #0) (div[rad,rad] (rep #1) 1.5708[rad]) 0[1]))
```

Inputs are observed with `rep` (innermost binder is the _last_ input), the
result is constructed with `mk` under the declaration's own grant, units are
scaled literals, and every primitive is dimension-indexed so the checker's
ordinary application rule enforces dimensions.

### Elaboration of an equation (implemented)

`allBelow : Held` with `all(temps, t => t < 30 K)`, `temps : Readings`,
`Readings : List<Temperature>`: the library's `allF` at `list q[K]`, applied to
the observed collection and the rule as a closed lambda —

```text
(mk sem#Held ((λ(list q[K]). λ(q[K] → bool). (fold (λ(q[K]). λ(bool). (and (#2 #1) #0)) true #1))
   (rep decl#temps) (λ(q[K]). (lt[K] #0 30[K]))))
```

The scheme is matched against the arguments' closed types in argument order
(`crates/bdl-equations`); the kernel never sees a type variable
(`docs/spec/equation-library.md`).

## Executable IR (`bdl-exec-ir`) and the Rust AST (`bdl-codegen-rust::ast`)

After analysis, `bdl-lower` turns a checked Design IR into a plan with dense
clock/input/state/output slots, first-order expressions (lambdas inlined, a
function argument carried as a binding and inlined where the receiver applies
it, `fold` as `ExecExpr::Fold` over two locals) and an evaluation order —
`docs/architecture/executable-ir.md`. The Rust backend prints it through a small
owned AST — `docs/architecture/codegen-rust.md`. Neither is a semantic layer:
the reference evaluator over the Design IR remains the definition, and the
differential tests hold the rest to it.

## Deliberate deviations from the Lean development

Recorded so nobody mistakes them for the formal model (see
`docs/archive/design-issues-ledger.md`):

|                   | Lean                                                           | Here                                                                                                                                                                                                                            | Why                                                                 |
| ----------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| numeric literals  | `Nat` (truncating `-`, floor `/`)                              | `Scalar(f64)` with bit-pattern equality                                                                                                                                                                                         | products measure reals; DI-1                                        |
| base dimensions   | length, time, angle                                            | 7 SI + angle (`i8` exponents)                                                                                                                                                                                                   | the paper says three were "enough to test the abstraction"          |
| commitments       | abstract `PropertyId`                                          | closed enum `{Monotone, Deterministic, Total, BoundedRange}`                                                                                                                                                                    | needs a vocabulary; grows with the validation layer                 |
| environments      | functions `DeclId → Option _`                                  | `BTreeMap`                                                                                                                                                                                                                      | determinism, serialization                                          |
| list counts       | `length : list τ → q 0` over `Nat`; `take`/`drop` read a `Nat` | a dimensionless `f64`, read as a whole number towards zero, never below zero                                                                                                                                                    | the numeric deviation above (ADR-0011); `bdl_reactive::eval::count` |
| `eq` off data     | unwritable: `eq τ (h : τ.Data)` carries the proof              | `Prim::Eq { ty }` is writable; `bdl-check` refuses it (`EqualityNotData`)                                                                                                                                                       | no proof fields in Rust data; the checker is the authority          |
| list values       | `Value.list (vs : List Value)`                                 | a shared, immutable cons list (`bdl_reactive::value::List`) with `O(1)` `cons`/clone                                                                                                                                            | the library's `map`/`filter`/`append` stay linear in the evaluator  |
| the empty product | no unit type; a declaration without inputs is typed at `B`     | `Ty::Unit` is the domain of a relationship without inputs in its canonical type `() -> B` (ADR-0029); the kernel interface is that type with the unit eliminated (`B`), so `Unit` reaches no Core term, representation or value | ISS-0014 asks the formal development for the equivalence            |
