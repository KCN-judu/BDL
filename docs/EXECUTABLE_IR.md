# Executable IR

`crates/bdl-exec-ir` — what is left of a Design IR once everything a
backend does not need has been lowered away (`crates/bdl-lower`). It is
the plan a code generator consumes; it never rediscovers anything.

```
DesignIr (Θ Δ Κ Ω β)  ──bdl-lower──▶  ExecIr  ──bdl-codegen-rust──▶  generated crate
                                        │
                                        └──bdl-exec-ir::interp──▶  trace  (same Values as the reference)
```

## What is explicit

| In the plan | From | Order |
|---|---|---|
| `ClockPlan { slot, id, name }` | every `ClockId` in `Κ`, `Ω`, and every `sync` source | `ClockId` |
| `ConceptPlan { id, name, representation }` | every concept a carried type mentions | `SemanticId` |
| `InputPlan { slot, decl }` | unresolved value declarations | `DeclId` |
| `DeclPlan { index, id, ty, activation, kind }` | value declarations | **evaluation order** (below) |
| `CellPlan { slot, cell: StateCellId, owner, ty, writer, operand }` | every `delay`/`sync` site | `StateCellId` (declaration, then path) — also the write order |
| `OutputPlan { slot, id, driver, ty }` | the *validated* drive edges (`OutputAnalysis::valid_bindings`), never raw `β` | `OutputId` |
| `FunctionPlan { id, name, ty }` | declarations inlined away (relationships with inputs) | `DeclId` |
| `has_domains` | `Κ ≠ ∅` | — |

`Activation` is `Domain { clock }` or `Agnostic`. An agnostic declaration
is never given a domain: it runs whenever any domain is active, and at
every tick of a design with no domains (DI-16), exactly as in the
reference evaluator.

## Evaluation order

The reference evaluator is lazy and memoised, so its result does not
depend on order; but *which* failure it reports when several declarations
fail at one tick does. The plan therefore lists declarations in the
reference's traversal: roots in `DeclId` order, instantaneous references
depth-first in expression order (through inlined functions), post-order.
This is a valid topological order of the instantaneous graph (causality
guarantees it exists), it depends on nothing but the design, and it makes
the two engines name the same failing declaration whenever a single one
fails — see DI-25 for the residual case.

## Expressions

```
ExecExpr ::= Bool b | Nat n | Quantity dim v
           | Local l | Let l = e in e
           | ReadDecl i                  -- a declaration evaluated earlier this tick
           | Wrap sem e | Unwrap e       -- mk / rep, the nominal boundary kept
           | Prim op [e…]                -- saturated only
           | ReadCell slot init          -- committed value, else init evaluated now
```

No binders. `Lam`/`App` are gone: a lambda applied to arguments becomes
`Let`s (arguments bound outermost first, in the caller's environment, with
their own expression paths so a `delay` passed as an argument keeps its
`StateCellId`); a reference to a declaration whose realization is a lambda
is inlined at each saturated application (through alias chains). Anything
that would need a closure at runtime — a lambda as a value, a partially
applied primitive, a relationship passed as an argument, a function-typed
input — is refused with `backend.unsupported_higher_order` (DI-24).

`Prim` is strict: every operand is evaluated first, then the primitive is
applied — including `Ite`, `And`, `Or` — because that is what the reference
evaluator does (DI-26).

State: a cell is `ReadCell` wherever its `delay`/`sync` stood, reading the
*previous* state (committed at the end of an earlier tick); its `operand`
is evaluated in the same read mode when its `writer` domain is active and
stored into the *next* state. Nested temporal forms inside an operand or
an initial value are themselves `ReadCell`s.

## Interpreter

`bdl_exec_ir::interp::step(ir, tick, active_slots, state, inputs)` executes
the plan exactly as the generated `step` does — read phase in plan order,
write phase in cell order, commit, outputs projected from the drivers —
over the reference evaluator's `Value`s and `RuntimeError`s. It is the
in-process leg of the differential tests (reference ↔ exec IR, hundreds of
generated designs per run) and a debugging aid. It is not a second
semantics: where it and the reference disagree, one of them is wrong.

## Versioning

`ExecIr.version` = `EXEC_IR_VERSION` (1). The IR is serde data
(JSON-serialisable) so an artefact can carry it; it is not a stable
external format yet.
