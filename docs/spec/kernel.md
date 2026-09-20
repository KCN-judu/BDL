---
kind: specification
area: language
status: current
---

# BDL kernel specification (transcribed from the BDL_FV Lean 4 development)

This is the **exact contract** the engineering implementation aligns with. Every
item corresponds to a definition in `BDL_FV/BDL/Core/*.lean` or
`BDL_FV/BDL/Validation/Hardware.lean`; the implementation may change the data
representation (a hash map instead of a function environment, `Int` or floating
point instead of `Nat`), but **every judgement must agree with the Lean
definition**.

Lean source files:

| Section of this page                         | Lean file                                 |
| -------------------------------------------- | ----------------------------------------- |
| §1 Identities and types                      | `Core/Base.lean`                          |
| §2 Declarations and environments             | `Core/Interface.lean`, `Core/Decl.lean`   |
| §3 Typing                                    | `Core/Typing.lean`                        |
| §4 Satisfaction, well-formedness, lifecycle  | `Core/Satisfaction.lean`, `Core/Env.lean` |
| §5 Dependency and causality                  | `Core/Dependency.lean`                    |
| §6 Single-domain evaluation                  | `Core/Reactive.lean`                      |
| §7 Clock domains and multi-domain evaluation | `Core/Clock.lean`                         |
| §8 Physical outputs                          | `Core/Output.lean`                        |
| §9 Hardware validation                       | `Validation/Hardware.lean`                |

---

## 1. Identities and types (Base)

Three **mutually independent** nominal identities, plus the declaration
identity:

```text
SemanticId  = { n : Nat }   -- a concept (it is a type)
ClockId     = { n : Nat }   -- a clock domain (not a rate)
OutputId    = { n : Nat }   -- a physical sink (it is a resource)
DeclId      = { n : Nat }   -- a declaration (it is a value)
```

Display names are not in the kernel.

Dimensions: an exponent vector over three base dimensions (not an SI catalogue;
the implementation may extend it):

```text
Dim = { length, time, angle : Int }
Dim.zero, Dim.add, Dim.sub, Dim.Length, Dim.Time, Dim.Angle
```

Types:

```text
Ty ::= bool | nat | arr Ty Ty | sem SemanticId | q Dim | opt Ty
     | list Ty            -- Phase 9a: finite sequence data
     | prod Ty Ty         -- Phase 9b: a value-level product (a pair), never an interface or an output bundle
```

The production-side `bdl_ir::Ty` additionally has `Unit` (the empty product
`()`): a relationship's canonical type is `domain(inputs) -> B`, and with no
inputs the domain is `()`. The kernel has no unit type; the interface type
(`expectedType`) is given in the kernel's encoding — curried, with unit
eliminated (`() -> B` is encoded as `B`); `Unit` never appears in any Core term,
representation or runtime value (ADR-0029; formal follow-up ISS-0014).

Predicates:

- `Ty.SemFree`: contains no `sem` (recursing into arr/opt/list/prod)
- `Ty.Data`: contains no `arr` (recursing into opt/list/prod) — `list τ` is data
  iff `τ` is (`list_data`), `prod a b` iff both are (`prod_data`)

Primitives and their types (**all** of the dimension algebra is here):

```text
Prim ::= lit d n | add d | sub d | mul d₁ d₂ | div d₁ d₂
       | lt d | eq d | not | and | or | ite τ
       | none τ | some τ | isSome τ | getD τ

lit d n   : q d
add d     : q d → q d → q d            sub d : likewise
mul d₁ d₂ : q d₁ → q d₂ → q (d₁+d₂)
div d₁ d₂ : q d₁ → q d₂ → q (d₁−d₂)
lt d, eq d: q d → q d → bool
not : bool → bool ; and, or : bool → bool → bool
ite τ  : bool → τ → τ → τ
none τ : opt τ ; some τ : τ → opt τ
isSome τ : opt τ → bool ; getD τ : opt τ → τ → τ
```

Expressions (de Bruijn):

```text
Expr ::= var i | boolLit b | natLit n
       | lam dom body | app f a
       | declRef d
       | rep e | mk s e
       | prim p
       | delay init e
       | sync src init e
```

Auxiliaries:

- `refs e`: every `declRef` (including both operands of delay/sync)
- `instRefs e`: the instantaneous references — `delay i _` takes only
  `i.instRefs`; `sync _ i _` takes only `i.instRefs` (a transport is never
  instantaneous)
- `DelayFree e`: contains no delay/sync
- `RefFree e`: `refs e = []`

## 2. Declarations and environments (Interface, Decl)

```text
PropertyId      -- an atomic label (shared by commitments and obligations)
DeclInterface   = { expectedType : Ty, commitments : List PropertyId }
DesignDecl      = { id : DeclId, interface : DeclInterface, realization : Option Expr }
DeclEnv         = DeclId → Option DesignDecl
```

- `InterfaceRefines old new` ⇔
  `old.expectedType = new.expectedType ∧ old.commitments ⊆ new.commitments` (a
  decidable preorder)
- `DeclEnv.update Δ h` = store `h` under `h.id`
- `DeclEnv.tyView Δ d` = `(Δ d).map (·.interface.expectedType)` — **the only way
  typing reads Δ**
- `DeclEnv.realizationOf Δ d` = `(Δ d).bind (·.realization)`

The structural lifecycle order:

- `DeclLeq h₁ h₂` ⇔ same id ∧ `InterfaceRefines h₁.interface h₂.interface` ∧ (∀
  e, h₁.realization = some e → h₂.realization = some e) (write-once)
- `EnvRefines Δ₁ Δ₂` ⇔ ∀ d h₁, Δ₁ d = some h₁ → ∃ h₂, Δ₂ d = some h₂ ∧ DeclLeq
  h₁ h₂ (new declarations allowed)

Concept environment:

```text
ConceptEnv Θ = SemanticId → Option Ty
ConceptEnv.WF Θ  ⇔ ∀ s R, Θ s = some R → R.SemFree ∧ R.Data
ConceptRefines Θ₁ Θ₂ ⇔ ∀ s R, Θ₁ s = some R → Θ₂ s = some R   -- only added, never changed
```

Grant:

```text
Grant = SemanticId → Prop
Ty.grant : Ty → List SemanticId     -- the concepts in result position of a signature
  grant (sem s)   = [s]
  grant (arr _ b) = grant b
  grant _         = []                -- note: opt (sem s) does not grant s
Grant.of τ = fun s => s ∈ τ.grant
Grant.none / Grant.all
```

## 3. Typing

`HasType Θ Δ G Γ e τ`, with `Γ : List Ty` (the de Bruijn context):

```text
var      Γ[i] = τ                                  ⊢ var i : τ
boolLit                                            ⊢ boolLit b : bool
natLit                                             ⊢ natLit n : nat
lam      (dom::Γ) ⊢ body : cod                     ⊢ lam dom body : arr dom cod
app      ⊢ f : arr dom cod ; ⊢ a : dom             ⊢ app f a : cod
declRef  Δ.tyView d = some τ                       ⊢ declRef d : τ
rep      Θ s = some R ; ⊢ e : sem s                ⊢ rep e : R
mk       G s ; Θ s = some R ; ⊢ e : R              ⊢ mk s e : sem s
prim                                               ⊢ prim p : p.ty
delay    τ.Data ; [] ⊢ i : τ ; [] ⊢ e : τ          [] ⊢ delay i e : τ        -- empty context only
sync     τ.Data ; [] ⊢ i : τ ; [] ⊢ e : τ          [] ⊢ sync c i e : τ       -- empty context only; the domain is irrelevant to the type
```

`infer Θ Δ G Γ e : Option Ty` is syntax-directed inference, proved sound,
complete and unique. The implementation can write the rules above directly; the
important details: `app` requires `dom = dom'` exactly (no subtyping);
`delay`/`sync` return none when `Γ ≠ []`.

## 4. Satisfaction, global well-formedness, lifecycle (Satisfaction, Env)

```text
Evidence = DeclEnv → Expr → PropertyId → Prop        -- supplied by the validation layer
Evidence.Monotone ev ⇔ ∀ Δ₁ Δ₂ e p, EnvRefines Δ₁ Δ₂ → ev Δ₁ e p → ev Δ₂ e p

Satisfies ev Θ Δ Γ e S ⇔
    HasType Θ Δ (Grant.of S.expectedType) Γ e S.expectedType
  ∧ ∀ p ∈ S.commitments, ev Δ e p

WellFormedDecl ev Θ Δ Γ h ⇔ ∀ e, h.realization = some e → Satisfies ev Θ Δ Γ e h.interface

GlobalWF ev Θ Δ ⇔ ∀ d h, Δ d = some h → h.id = d ∧ WellFormedDecl ev Θ Δ [] h
```

Lifecycle steps (`DeclRefines ev Θ Δ Γ h h'`; the side conditions are checked in
the **current** Δ):

1. `refine`: h is unresolved, `InterfaceRefines S S'`
2. `realize`: h is unresolved, `Satisfies ev Θ Δ Γ e S`
3. `strengthen`: h is realized by e, `InterfaceRefines S S'` and
   `Satisfies ev Θ Δ Γ e S'` (re-verified)

Theorems (guarantees the implementation may rely on):

- `DeclLeq B B'` ⇒ every typing judgement in Δ still holds in `Δ[B']` (only the
  unchanged expectedType is used)
- `GlobalWF` ∧ one or more `DeclRefines` steps ∧ `ev` monotone ⇒
  `GlobalWF (Δ[B'])`
- binding an unbound concept preserves GlobalWF

## 5. Dependency and causality

```text
dependsOn Δ a b     ⇔ b ∈ (realizationOf Δ a).refs
instDependsOn Δ a b ⇔ b ∈ (realizationOf Δ a).instRefs
Acyclic Δ           -- structurally acyclic (meaningful only on the delay-free fragment)
Causal Δ ⇔ ∃ rank R, (∀ d, rank d < R) ∧ ∀ a b, InstDependsOn Δ a b → rank b < rank a
```

- on the delay-free fragment, `Causal ⇔ Acyclic`
- `A := delay 0 B; B := A` is causal; a cycle only partly through a delay is not
- a known conservative point: `A := λx. A x` is rejected by Causal although
  `declRef A` evaluates to a closure
- on a finite design, implement rank by a topological sort

`Unfolds Δ e e'`: expands every declRef into a closed term (delay-free fragment
only; undefined on a cycle).

## 6. Single-domain evaluation (Reactive)

```text
Value ::= bool b | nat n | sem s v | none | some v | clo ρ body | prim p args
Input  = DeclId → Nat → Value          -- the input of each unresolved declaration at each tick
```

`Prim.arity`, `Prim.compute` (saturated evaluation; returns `nat 0` on a shape
mismatch, which typing guarantees never happens),
`applyPrim p args = if args.length = p.arity then p.compute args else prim p args`
(partial application).

`Ev Δ I t ρ e v` (tick t, local environment ρ):

```text
var        ρ[i] = v
boolLit/natLit/lam(→ clo ρ body)
appClo     f ⇓ clo ρ' body ; a ⇓ va ; (va::ρ') body ⇓ v
appPrim    f ⇓ prim p args ; a ⇓ va ; ⇓ applyPrim p (args ++ [va])
refRealized  realizationOf d = some b ; [] b ⇓ v   at tick t      -- note: the body is evaluated in the empty environment
refInput     realizationOf d = none ; ⇓ I d t
rep        e ⇓ sem s w ; ⇓ w
mk         e ⇓ w ; ⇓ sem s w
prim       ⇓ applyPrim p []
delayZero  t=0 : i ⇓ v at 0
delaySucc  t+1 : e ⇓ v at t
syncZero/syncSucc  in a single domain, sync behaves as delay
```

Theorems: determinism (unconditional); Causal ∧ GlobalWF ∧ well-typed inputs ⇒
every declaration has a value at every tick and it is related to its type (a
logical relation). The executable interpreter
`evalF Δ I fuel t ρ e : Option Value` is proved sound.

> Implementation note: Lean's `nat` subtraction truncates at 0 and division
> rounds down; the value of `q d` is also a `nat`. An engineering implementation
> that uses signed or floating-point numbers must record the deviation.

## 7. Clock domains and multi-domain evaluation (Clock)

```text
Sched = ClockId → Nat → Bool                     -- whether each domain is active at a global tick; outside the design
prevAct S c t = the largest t' < t with S c t', or none
Sched.periodic period c t = (t % period c = 0)
Sched.always
ClockEnv Κ = DeclId → Option ClockId             -- none = a domain-independent pure mapping
```

The domain judgement `clockedB Κ (c : Option ClockId) e`:

```text
lam/app/rep/mk    recursive
declRef d         Κ d = none ∨ Κ d = c
delay i e         when c = some c': i and e both under some c'; c = none → false
sync c' i e       when c = some c₀: i under some c₀, e under some c'; c = none → false
otherwise         true
WellClocked Κ Δ ⇔ ∀ d b, realizationOf Δ d = some b → Clocked Κ (Κ d) b
```

Multi-domain evaluation `MEv S Δ I c t ρ e v` (c = the current domain): as `Ev`,
except

```text
syncNone   prevAct S c' t = none      ; i ⇓ v (in c, at t)
syncSome   prevAct S c' t = some t'   ; e ⇓ v (in c', at t')
delay      in domain c ≡ sync c
```

Theorems: under `Sched.always`, MEv agrees with Ev; determinism, and no order is
observable between domains active at the same time; a cross-domain cycle is
never instantaneous (Causal is unchanged); Causal ∧ GlobalWF ∧ WellClocked ⇒
every domain has a value at every tick. The interpreter is
`mevalF S Δ I fuel c t ρ e`.

## 8. Physical outputs (Output)

```text
OutputSpec = { accepts : Ty, clock : ClockId }
OutputEnv Ω = OutputId → Option OutputSpec       -- declared by deployment
DriveEnv  β = DeclId → Option OutputId            -- the drive edge, write-once

DriveWF Ω Κ Δ β ⇔ ∀ d o, β d = some o →
    ∃ spec, Ω o = some spec ∧ Δ.tyView d = some spec.accepts ∧ Κ d = some spec.clock
SingleDriver β    ⇔ ∀ d₁ d₂ o, β d₁ = some o → β d₂ = some o → d₁ = d₂
Driven β o        ⇔ ∃ d, β d = some o
CompleteOutputs β req ⇔ ∀ o ∈ req, Driven β o

PartialOutputWF   = DriveWF ∧ SingleDriver
ExecutableOutputs = PartialOutputWF ∧ CompleteOutputs

PhysicalOutput S Δ I Ω β o t v ⇔ ∃ d spec, β d = some o ∧ Ω o = some spec ∧ MEv S Δ I spec.clock t [] (declRef d) v
```

- the edge converts and synchronises nothing: a slow driver reading a fast value
  must `sync` upstream; a sink that accepts a representation type needs an
  explicit `rep`-typed declaration in front of it
- `DriveRefines β₁ β₂`, `DriveEnv.bind β d o` (binding to an undriven sink is a
  refinement)
- composition (priority / mixing / maximum / clamping) = ordinary computation
  inside the single driver

## 9. Hardware validation (Hardware, the validation layer)

```text
Capability ::= digitalIn | digitalOut | pwm | analogIn | interrupt
             | i2cSDA | i2cSCL | spiMOSI | spiMISO | spiSCK | spiSS | uartTX | uartRX
ResourceId  = { n : Nat }
Resource    = { id, caps : List Capability, units : List (Capability × Nat) }   -- unit = the timer / peripheral number behind it
Hardware    = { resources : List Resource, shareable : List Capability }        -- buses are shareable, everything else exclusive

RequirementId = { n : Nat }
UnitRel ::= same | distinct
Requirement = { id, cap, fixed : Option ResourceId, group : Option (Nat × UnitRel) }
Assignment  = List (Requirement × ResourceId)
```

Validity:

- `ReqOK H req r`: r has the cap, and equals `fixed` when it is given
- `Compatible H (a,ra) (b,rb)`:
  - same resource ⇒ same cap and cap ∈ shareable
  - same group ⇒ `same`: equal units; `distinct`: unequal units
- `PartialValid H A`: every item ReqOK and pairwise Compatible
- `ValidFor H R A`: PartialValid and A covers R exactly
- `HardwareSatisfiable H R ⇔ ∃ A, ValidFor H R A`

Solver: `candidates H req` = the resources supporting the cap (only that one
when fixed); `solveAux` is depth-first, pruned by pairwise compatibility of the
prefix; `solve H R : Option Assignment`, proved sound and complete ⇒ decidable.

`Hardware.Extends H₁ H₂` (adding caps / units / sharing to existing resources)
preserves every valid assignment; removing a resource, adding or strengthening a
requirement, or fixing a pin may not.

Explanation:
`Explanation ::= noCapableResource req | blocked req (List (ResourceId × RequirementId))`;
`diagnose H R` places greedily and reports the first dead end — meaningful only
when `solve` returns none, and not a minimal unsat core.

Pipeline: `OutputId → DeviceKind → Requirements → solve → Assignment`. Example
device kinds:

- H-bridge channel: 1 pwm + 1 digitalOut
- I2C sensor: i2cSDA + i2cSCL, group `same`
- quadrature encoder: 2 interrupt
- UART: uartTX + uartRX, group `same`

### Arduino Nano table (usable as is)

```text
D0  [digitalIn, digitalOut, uartRX]           uartRX→0
D1  [digitalIn, digitalOut, uartTX]           uartTX→0
D2  [digitalIn, digitalOut, interrupt]
D3  [digitalIn, digitalOut, pwm, interrupt]   pwm→2
D4  [digitalIn, digitalOut]
D5  [digitalIn, digitalOut, pwm]              pwm→0
D6  [digitalIn, digitalOut, pwm]              pwm→0
D7, D8 [digitalIn, digitalOut]
D9  [digitalIn, digitalOut, pwm]              pwm→1
D10 [digitalIn, digitalOut, pwm, spiSS]       pwm→1, spiSS→0
D11 [digitalIn, digitalOut, pwm, spiMOSI]     pwm→2, spiMOSI→0
D12 [digitalIn, digitalOut, spiMISO]          spiMISO→0
D13 [digitalIn, digitalOut, spiSCK]           spiSCK→0
A0–A3 [analogIn, digitalIn, digitalOut]
A4  [analogIn, digitalIn, digitalOut, i2cSDA] i2cSDA→0
A5  [analogIn, digitalIn, digitalOut, i2cSCL] i2cSCL→0
A6, A7 [analogIn]
shareable = [i2cSDA, i2cSCL]
```

Verified test cases (the implementation's regression tests):

- 4×H-bridge + IMU: SAT, `M1→D3/D0, M2→D5/D1, M3→D6/D2, M4→D9/D4, IMU→A4/A5`
- 7×pwm: UNSAT (6 PWM pins); the same requirements on the big board (+D40–D45,
  timers 3/4/5) SAT
- 2×interrupt + 6×pwm: UNSAT (D3 is both the only second interrupt pin and a PWM
  pin)
- two pwm fixed to the same pin: rejected
- two I2C sensors both on A4/A5: accepted (the bus is shared)
- 4×pwm group `distinct`: the Nano has only 3 timers → UNSAT
- TX/RX group `same`: stay on the same UART

## 9b. The data core and the equation language (Phases 9a/9b/9c; `ListData`, `Surface/Poly`, `Surface/Stdlib`)

Everything the kernel adds on top of `Base`/`Typing`/`Reactive`:

```text
Prim  += nil τ | cons τ | length τ | take τ | reverse τ | head τ     -- 9a
       | drop τ | toList τ | pair a b | fst a b | snd a b             -- 9b
       ; eq τ (h : τ.Data)   -- structural equality on any data type (9b)
       ; lt d                -- on quantities q d only (9c withdrew 9b's structural order)
Expr  += fold f z l          -- the list recursor: fold f z [x₁,…,xₙ] = f x₁ (… (f xₙ z))
Value += list vs | pair a b
```

- `length : list τ → q 0`; `take`/`drop : q 0 → list τ → list τ`;
  `head : list τ → opt τ`; `toList : opt τ → list τ` (an option is a list of
  length ≤ 1, so `fold` eliminates options too).
- The typing rule `HasType.fold`: `f : τ → σ → σ`, `z : σ`, `l : list τ` ⊢
  `fold f z l : σ`; evaluation `Ev.foldNil`/`Ev.foldCons` is a syntactic
  unfolding through the environment; `fold_total`/`mfold_total` give totality on
  finite lists. `fold` is the only term constructor that applies a function
  value during evaluation; registered operators never apply a closure.
- `Value.beq`: decidable structural equality on data values (closures compare as
  `false`; typing never asks); there is **no** structural order (`Value.blt` has
  been removed).
- `Clocked` has no list rule (`list_clock_conservative`): a cross-domain read
  wrapped in `cons` is still rejected.
- Polymorphism lives in the **surface layer** (`Poly.lean`): pattern types `PTy`
  (type variables, dimension variables `PDim.dvar`), first-order matching
  `matchTy` (`matchTy_sound`/`matchTy_complete`: a unique substitution, no
  unification, no generalisation), a closed capability vocabulary
  `Cap = data | eq | ord` (`Cap.eq_iff_data`; `Ty.ordB O Θ`: a quantity, or a
  concept declared ordered and represented by a quantity — `OrdDecl` is surface
  metadata).
- The equation library (`Stdlib.lean`): every entry is a closed de Bruijn
  combinator (`idF`, `minF o`, `clampF o`, `inRangeF o`, `minByF`, `foldrF`,
  `anyF`, `allF`, `containsF h`, `mapF`, `filterF`, `appendF`, `sumF d`,
  `optElimF`, `mapOptF`, `zipF`, …); `lib_expansion`: inlining a combinator adds
  no privilege whatsoever (type, construction and clock belong to the arguments
  alone).
- The buffer (9a, `Surface/Buffer.lean`): five declarations
  `log`/`logD`/`seen`/`cursor`/`window` write a lossless cross-domain window
  over `delay`/`sync` and list data (theorem M, `buffer_window_correspondence`);
  capacity is a validation obligation (`Validation/Capacity.lean`), the overflow
  policy is explicit, and only refusing deployment preserves the semantics.

The production correspondence is in `docs/spec/equation-library.md` and
`docs/project/formal-correspondence.md`; the production deviations (`f64`
counts, a checker rather than a proof field refusing `eq` on non-data) are
recorded in `docs/architecture/ir.md`.

## 10. Summary of the global well-formedness conditions (an "executable design" satisfies all of them)

1. `GlobalWF ev Θ Δ` (every realization satisfies its own interface)
2. `ConceptEnv.WF Θ`
3. `Causal Δ`
4. `WellClocked Κ Δ`
5. `DriveWF Ω Κ Δ β ∧ SingleDriver β`
6. when executable, additionally `CompleteOutputs β required`
7. hardware: `solve H (requirementsOf Ω deviceKinds) = some A` (not in the
   kernel; recomputed every time)

The corresponding workspace states: 1–2 → type-valid; 3 → temporally valid; 4 →
clock-consistent; 5–6 → output-complete; 7 → hardware-feasible.
