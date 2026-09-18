---
kind: architecture
area: codegen
status: current
---

# Executable IR

`crates/bdl-exec-ir` — what is left of a Design IR once everything a backend
does not need has been lowered away (`crates/bdl-lower`). It is the plan a code
generator consumes; it never rediscovers anything.

```text
DesignIr (Θ Δ Κ Ω β)  ──bdl-lower──▶  ExecIr  ──bdl-codegen-rust──▶  generated crate
                                        │
                                        └──bdl-exec-ir::interp──▶  trace  (same Values as the reference)
```

## What is explicit

| In the plan                                                        | From                                                                          | Order                                                         |
| ------------------------------------------------------------------ | ----------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `ClockPlan { slot, id, name }`                                     | every `ClockId` in `Κ`, `Ω`, and every `sync` source                          | `ClockId`                                                     |
| `ConceptPlan { id, name, representation }`                         | every concept a carried type mentions                                         | `SemanticId`                                                  |
| `InputPlan { slot, decl }`                                         | unresolved value declarations                                                 | `DeclId`                                                      |
| `DeclPlan { index, id, ty, activation, kind }`                     | value declarations                                                            | **evaluation order** (below)                                  |
| `CellPlan { slot, cell: StateCellId, owner, ty, writer, operand }` | every `delay`/`sync` site                                                     | `StateCellId` (declaration, then path) — also the write order |
| `OutputPlan { slot, id, driver, ty }`                              | the _validated_ drive edges (`OutputAnalysis::valid_bindings`), never raw `β` | `OutputId`                                                    |
| `FunctionPlan { id, name, ty }`                                    | declarations inlined away (relationships with inputs)                         | `DeclId`                                                      |
| `has_domains`                                                      | `Κ ≠ ∅`                                                                       | —                                                             |

`Activation` is `Domain { clock }` or `Agnostic`. An agnostic declaration is
never given a domain: it runs whenever any domain is active, and at every tick
of a design with no domains (DI-16), exactly as in the reference evaluator.

## Evaluation order

The reference evaluator is lazy and memoised, so its result does not depend on
order; but _which_ failure it reports when several declarations fail at one tick
does. The plan therefore lists declarations in the reference's traversal: roots
in `DeclId` order, instantaneous references depth-first in expression order
(through inlined functions), post-order. This is a valid topological order of
the instantaneous graph (causality guarantees it exists), it depends on nothing
but the design, and it makes the two engines name the same failing declaration
whenever a single one fails — see DI-25 for the residual case.

## Expressions

```text
ExecExpr ::= Bool b | Nat n | Quantity dim v
           | Local l | Let l = e in e
           | ReadDecl i                  -- a declaration evaluated earlier this tick
           | Wrap sem e | Unwrap e       -- mk / rep, the nominal boundary kept
           | Prim op [e…]                -- saturated only
           | ReadCell slot init          -- committed value, else init evaluated now
           | Fold elem acc step init list  -- the recursor, its step inlined over two locals
```

No closures. `Lam`/`App` are gone: a lambda applied to arguments becomes `Let`s
(arguments bound outermost first, in the caller's environment, with their own
expression paths so a `delay` passed as an argument keeps its `StateCellId`); a
reference to a declaration whose realization is a lambda is inlined at each
saturated application (through alias chains). A **function argument** — the rule
given to `any`, the step of a `fold`, a partially applied primitive such as
`add 5` — is not lowered to a value: the lowerer carries it as a _binding_
(`Fun::Lam` with the environment it closes over, or `Fun::Prim` with the
arguments so far) and inlines it wherever the receiving lambda applies it, so
every combinator of the equation library lowers to first-order code. The
kernel's `fold f z l` becomes `Fold`: `step` is `f` applied to the two locals
`elem` and `acc`, evaluated once per element from the last one (finite
iteration, never a frame per element). Anything that would still need a closure
at runtime — a lambda as a value, a partially applied primitive in value
position, a function-typed input — is refused with
`backend.unsupported_higher_order` (DI-24).

Collections and grouped values **stay structured values** here (`Vec`/tuples in
the generated core, `Value::List`/`Value::Pair` in the interpreter): lowering
them further would be a second interpretation of collection behaviour. The list
operators are `PrimOp`s (`Nil`, `Cons`, `Length`, `Take`, `Drop`, `Reverse`,
`Head`, `ToList`, `Pair`, `Fst`, `Snd`); `Eq` is structural.
`ExecIr::uses_lists()` says whether the program carries a list anywhere — what
decides the runtime's `collections` feature.

`Prim` is strict: every operand is evaluated first, then the primitive is
applied — including `Ite`, `And`, `Or` — because that is what the reference
evaluator does (DI-26).

State: a cell is `ReadCell` wherever its `delay`/`sync` stood, reading the
_previous_ state (committed at the end of an earlier tick); its `operand` is
evaluated in the same read mode when its `writer` domain is active and stored
into the _next_ state. Nested temporal forms inside an operand or an initial
value are themselves `ReadCell`s.

## Interpreter

`bdl_exec_ir::interp::step(ir, tick, active_slots, state, inputs)` executes the
plan exactly as the generated `step` does — read phase in plan order, write
phase in cell order, commit, outputs projected from the drivers — over the
reference evaluator's `Value`s and `RuntimeError`s. It is the in-process leg of
the differential tests (reference ↔ exec IR, hundreds of generated designs per
run) and a debugging aid. It is not a second semantics: where it and the
reference disagree, one of them is wrong.

## Bounds

`bdl_exec_ir::bounds::analyse(ir)` gives every declaration and every state cell
a [`Shape`] — the value's structure with a `Bound` at each list position:
`finite { elements }`, `input` (as large as the host supplies) or `unbounded`
(the design grows it over time). It is an abstract interpretation of the plan:
`cons` adds one, `take k` narrows, a fold is run twice abstractly to tell a
stable accumulator from one growing by a constant per element, a cell is the
fixpoint of its operand with a widening to `unbounded` and one narrowing pass.
Sound (an upper bound of every length the reference produces, tested on every
corpus case), not tight (`take` is the one narrowing operator). It runs on the
plan because the equation library has been inlined there: `map`, `filter`,
`append`, `zip` are folds and need no cases of their own. Consumers: the codegen
manifest (`collections`) and `bdl-compiler::collections`
(docs/spec/deployment-capacity.md).

## Versioning

`ExecIr.version` = `EXEC_IR_VERSION` (2: `Fold`, the list and pair operators).
The IR is serde data (JSON-serialisable) so an artefact can carry it; it is not
a stable external format yet.
