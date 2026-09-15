# Design issues

Places where implementation pressure exposed an ambiguity or a gap in the
paper / formal development. Each is recorded rather than silently resolved;
the resolution, when one is taken, is an engineering decision that does not
claim formal backing.

| Id | Issue | Where it bites | Status |
|---|---|---|---|
| DI-1 | The kernel computes over `Nat` (truncating subtraction, floor division); products need reals. | literals, simulation, codegen | **Decided**: `Scalar(f64)` in IR, floats in generated code; symbolic rewrites must record numeric obligations. Differential tests with Lean fixtures restricted to integer-valued cases. |
| DI-2 | Three base dimensions in Lean; the paper says this is not an SI catalogue. | `Dim` | **Decided**: 7 SI + angle, `i8` exponents. |
| DI-3 | The kernel has no list type, so cross-domain event buffering (window model) is a semantic-level result only. | occurrences across domains | Open. Needs a kernel extension when multi-domain events land. |
| DI-4 | "Several candidate definitions, one active" is a surface convenience the paper leaves open against the write-once realization. | `Definition` | Open. Proposed: surface stores candidates, kernel sees the active one; switching is an edit. |
| DI-5 | Interface-level references (commitments mentioning other declarations) are not modelled; the dependency graph is over realizations only. | invalidation, evidence | Open. |
| DI-6 | Contexts with their own clock domain are untested in the formal development. | elaboration of StateHandler | **Decided**: refuse with a diagnostic rather than elaborate silently. |
| DI-7 | Affine units (°C vs K) are unmodelled; only linear scaling. | units | Open; v0.1 supports linear units only. |
| DI-8 | `Causal` is conservative for lambda-guarded cycles. | causality diagnostics | Accept; diagnostic will say the cycle is rejected conservatively. |
| DI-9 | `diagnose` reports a first dead end, not a minimal unsat core. | hardware explanations | Accept; UI wording says "one conflict", not "the conflict". |
| DI-10 | Refinement vs edit is formalized for the kernel; surface operations (rename, description, layout) have no formal status. | `EditOutcome.kind` | **Decided**: rename/description/layout are refinements with empty invalidation (display names are not identity, D-14). |
| DI-11 | Duplicate display names: the kernel does not care; the tool must decide. | `apply_edit` | **Decided**: refuse duplicates within concepts and within mappings (usability policy, not semantics). |
| DI-12 | The kernel's `Prim` set has arithmetic and comparison only over `q d`; `nat` (counts) has literals but no operators, and `bool` has no equality. | formulas over Count concepts; `a == b` on booleans | **Decided** for v0: boolean `==` is encoded as `(a∧b)∨(¬a∧¬b)`; counts can only be passed through (`type.operand_kind` otherwise). Extending `Prim` with `nat` operators is a kernel change to be mirrored in Lean first. |
| DI-13 | Formula input names: the paper keeps display names out of the kernel but gives no surface naming rule. | `bdl-elab::names` | **Decided** (ADR-0013): display name at analysis time, re-resolved every analysis; no parameter layer yet. |
| DI-14 | Dimension errors can be reported twice — by the elaborator (with spans) and by the checker (authority). | diagnostics | **Decided**: when elaboration fails the checker is not run for that mapping; when elaboration succeeds and the checker still fails, its error is mapped to a span through the elaborator's path table. |
| DI-15 | Numeric runtime policy: the kernel computes over `Nat`; production uses IEEE `f64`. | evaluator, traces, later codegen | **Decided**: division by zero and any non-finite result (NaN, ±∞) are runtime errors that fail the tick (`simulation.division_by_zero`, `simulation.non_finite`); equality is exact bitwise `f64`; serialization is the JSON number. Generated code must implement the same checks or be recorded as a deviation. |
| DI-16 | Which declarations evaluate at a tick when they have no domain: the kernel treats agnostic declarations as pure mappings usable from any domain, and unresolved inputs as `I d t` at every tick; it never says when an agnostic *value* declaration is sampled. | simulation of designs without domains | **Decided**: agnostic declarations are evaluated whenever any domain is active, and at every tick in a design with no domains. |
| DI-17 | Surface formulas cannot reference other mappings (`brightness := dimByTilt(tilt)`), so a full lamp run (input → mapping → sample) is only expressible at Core level today. | Studio-driven simulation | Open: the next surface step is declaration references and application in formulas; the Core and evaluator already support them. |
