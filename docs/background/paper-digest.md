---
kind: background
area: language
status: current
---

# BDL paper digest (for the engineering implementation)

Sources: the conference-era manuscript _BDL: A Behavior Design Language_ (the
2026-09-15 revision, archived as
`reference/paper/archive/paper-2026-09-conference-manuscript.md`) and the formal
development [KCN-judu/BDL_FV](https://github.com/KCN-judu/BDL_FV) (Lean 4,
Phases 0–7, no `sorry`). That manuscript has been superseded by the **BDL Design
and Formalization Monograph** (`reference/paper/paper.md`, mirrored from
`BDL_FV/paper/`); this digest is kept as the historical record of what the
implementation was first built to.

This page records only what is **binding on the engineering implementation**;
the arguments and the counterexamples are in BDL_FV's `REPORT.md` /
`DESIGN_DECISIONS.md` / `MINIMALITY.md`. The exact data structures and judgement
rules are in [kernel.md](../spec/kernel.md).

---

## 1. In one sentence

BDL is a product-behavior design language for industrial designers: a **typed
semantic relationship** (`?f : Tilt -> Brightness`) is a first-class design
object, a relationship may legitimately exist before it has a realization
(signature-first), and every construct the designer sees (temporal modifiers,
contexts, event policies, output selection) is elaborated onto a very small
formal kernel.

## 2. The three layers (paper, §Architecture)

```text
Surface (what the designer sees)   semantic attributes · Mapping Block · temporal modifiers · contexts (StateHandler) · device kinds · units · display names
        │ elaboration ↓            diagnostics ↑
Kernel (the formal object)          declaration environment Δ + typing + tick evaluation + domain judgement + global well-formedness
        │ commitments/evidence ↓    target board ↓
Validation (outside the kernel)     evidence for commitments · hardware feasibility solver · numeric constraints (not modelled)
```

- Elaboration is **one-way**: the kernel never needs to recover surface
  structure.
- Typing reads only a declaration's **type view** (expectedType) and a concept's
  **representation view** (Θ). It does not read realization, commitment,
  evidence, clock or binding.
- Validation has two kinds of evidence that are **never merged**:
  - evidence that survives refinement (monotone evidence, such as a
    compositional monotonicity proof)
  - evidence recomputed after every change (board feasibility)

## 3. The kernel's constructs (what the implementation must provide)

| Construct                                                  | Layer | Point                                                                                                                                                  |
| ---------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `DesignDecl = ⟨id, interface, realization?⟩`               | K     | stable id; interface = a frozen expectedType + monotonically growing commitments; realization is write-once                                            |
| `DeclEnv : DeclId → Option DesignDecl`                     | K     | the design _is_ the environment; an unresolved declaration is `realization = none`, with no other distinction                                          |
| the `declRef d` typing rule reads only `tyView`            | K     | necessary and sufficient for client stability (refinement never breaks a client)                                                                       |
| `Ty.sem SemanticId`                                        | K     | a nominal semantic type; `Tilt ≠ MotorAngle` even with the same representation; crossing concepts is an ordinary arrow declaration, not a cast         |
| `ConceptEnv Θ : SemanticId → Option Ty`                    | K     | a write-once binding from concept to representation; the representation type must be sem-free and data (no functions)                                  |
| `rep e` / `mk s e` + Grant                                 | K     | `rep` is available everywhere; `mk s` only inside the realization of a declaration whose signature result position declares `sem s` (`Grant.of τ`)     |
| `Ty.q Dim` + the arithmetic primitives' types              | K     | the whole dimension algebra is in `Prim.ty`; no dedicated dimension rule; units are surface (linearly scaled literals)                                 |
| `delay init e`                                             | K     | the only single-domain temporal primitive; top-level only (not under λ), data types only; every delay carries an explicit initial value                |
| `Causal Δ` (the instantaneous dependency graph is acyclic) | K     | replaces structural acyclicity; a cycle through a delay is legal; witnessed by a rank                                                                  |
| `ClockId` + `ClockEnv Κ` + the `Clocked` judgement         | K     | a domain is a nominal identity, not a rate; rates are validation data; domains do not enter types                                                      |
| `sync src init e`                                          | K     | the only cross-domain transport primitive; reads the src domain's last activation **strictly before** the current tick; `delay ≡ sync own`             |
| `OutputId` + `OutputSpec` + `DriveEnv β`                   | K     | a nominal identity for a physical sink; a write-once drive edge; `DriveWF`: the driver's type **equals** the sink's accepted type and the clocks agree |
| `SingleDriver β` / `CompleteOutputs`                       | K     | at most one driver per sink; an executable design drives every required sink                                                                           |
| hardware resources / requirements / solver                 | V     | a finite CSP (unary + binary constraints); the DFS solver is sound and complete; `diagnose` reports the first dead end                                 |

**Removed (do not implement as kernel)**: a `Signal τ` type, an `Event τ` type,
clock-in-type, effect rows, action requests, per-context policies / runtime
arbitration, a five-phase tick with a resolve step.

## 4. Refinement vs edit (the organising principle of the interaction model)

Refinement (keeps everything already established; clients need not re-check):

1. adding a commitment to an unresolved declaration's interface
2. writing a realization that satisfies the interface into an unresolved
   declaration
3. strengthening a realized declaration's interface (the body is re-verified)
4. binding an unbound concept to a representation
5. binding an unbound declaration to an undriven sink

Edit (legal, but the verification of every transitive dependent is reopened):

- changing expectedType (keeping the id) → every client's typing breaks
- removing a commitment → typing is silent, but the clients' commitments lose
  their support
- replacing or detaching a realization → evidence that inspected the body is
  void
- replacing an old id with a new one → dangling references
- changing or assigning a clock domain → the clients' domain judgement breaks
- changing an output binding's target → completeness / single-driver may break
- changing a concept's representation, adding or removing a delay, changing an
  initial value, changing a sink's accepted type, renaming a sink, detaching a
  drive edge

## 5. Derived operators (surface → declaration shape; every row is a self-referential, self-delayed loop)

| Surface                                            | Declaration shape                                                                          |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `previous x`                                       | `delay init x`                                                                             |
| `previous x` (no initial value)                    | `delay none (some x)`; absence is pushed to the consumer                                   |
| `hold init e`                                      | `getD e (delay init self)`                                                                 |
| `count e`                                          | `ite (isSome e) (1 + delay 0 self) (delay 0 self)`                                         |
| `since e`                                          | `ite (isSome e) 0 (1 + delay 0 self)`                                                      |
| `once e`                                           | `delay false self ∨ isSome e`                                                              |
| `every n`                                          | a delay-based modulo-n counter                                                             |
| `rise b`                                           | `b ∧ ¬ delay false b`, as an optional Boolean                                              |
| `after e by d` / `p for d` / `while p` / `until e` | combinations of the above + comparison + activation (not executed separately in the paper) |

An event (occurrence) within one domain = an `opt τ` stream. Transporting an
event across domains needs the **window model**: an accumulating log on the
source side + a cursor on the destination side (`sync` one accumulator + `delay`
one cursor); policies such as `latest` / `count` are functions of the window;
the buffer bound is a validation obligation. The kernel of the paper had no list
type, so the buffer reduction was a semantic-level result only — **the
implementation had to supply lists itself** (it did: Phase 9a,
[kernel.md §9b](../spec/kernel.md)).

## 6. Context (StateHandler) elaboration (covers only the tested cases)

A context elaborates to:

- an activation declaration (Boolean)
- an entry declaration = the rising edge `rise` of the activation
- local temporal state = delayed cells gated by the activation and reset on
  entry
- default contributions while inactive
- several contexts competing for one output → an ordinary conditional expression
  inside the **single driver declaration**

Tested: conditional activation, entry, state reset on entry, defaults while
inactive, event-latched activation + exit-wins, context-local output selection,
nested selection + output. **Untested**: a context with its own clock domain,
nesting across independently clocked contexts — the tool should refuse or report
these explicitly, never elaborate them silently.

## 7. The elaborator's passes (paper, §Elaboration; not yet implemented at the time)

1. **Name & signature resolution** — semantic attributes → SemanticId, a Mapping
   signature → DeclInterface, contexts, device kinds, units, imported components
2. **Formula elaboration** — check the definition under `Grant.of(signature)`; a
   scalar formula is wrapped in `mk_Brightness(...)` automatically; units →
   scaled literals; curve / example / component all elaborate to the same
   realization form
3. **Temporal lowering** — temporal phrases → the shapes of §5; contexts → §6; a
   reusable stateful component is instantiated as a fresh declaration at each
   use site
4. **Domain assignment** — record Κ, check `Clocked`; a cross-domain reference
   without a transport is an error at the reference with two remedies (add a
   sync + initial value / move the reader into the source domain)
5. **Output binding** — record β, check type / clock equality, single-driver and
   (when executable) completeness; when several contexts drive one output,
   generate one selector driver
6. **Hardware validation** — device kinds → requirements → solve → an assignment
   or an explanation
7. **Normalization & erasure** — erase sem / dim / domain (proved sound);
   `rep(mk s e) ⇝ e`; check the flattened term under the universal grant; keep
   the wrappers at the boundaries (supplied blocks, device bindings, the
   generated code's public interface)

## 8. Workspace states (the diagnostic levels the designer sees)

| State             | What is settled                                                                                                                                    | Shown in   |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| declared          | the relationship is named and typed; others may depend on it; no definition                                                                        | canvas     |
| defined           | a formula / curve / example / component is attached                                                                                                | inspector  |
| type-valid        | the definition produces the type the signature promises; the declared properties hold                                                              | inspector  |
| temporally valid  | every held / delayed value has a first-tick value; no instantaneous cycle                                                                          | canvas     |
| clock-consistent  | every cross-domain read has a transport and an initial value                                                                                       | canvas     |
| output-complete   | every required physical output has exactly one final target                                                                                        | outputs    |
| hardware-feasible | the chosen board can carry every derived requirement (**the only state that depends on something outside the design**; recomputed on every change) | deployment |

Diagnostic wording principle: product language, not kernel terminology. For
example:

- not "single-driver violation" but "this output already has a final driver,
  `dimByTilt`; combine the two brightness values before connecting the output"
- across domains: say that the two values update in different domains, and ask
  how the reader should see the source (last value + initial value / move to the
  source domain)
- a hardware conflict: at the seventh PWM channel report "7 PWM channels are
  needed, only 6 exist", listing what occupies each PWM pin

A property that comes from a declaration (the assertion of a supplied block's
author) rather than from analysis must be shown **distinctly** from a property
the tool computed.

## 9. Supplied computation blocks (components written in the host language)

- pure functions or stateful transducers; **cannot drive outputs**
- must declare: determinism, totality on well-typed inputs, output range, the
  properties downstream depends on (monotonicity, for example), worst-case state
  size, the design's update rate — these are validation obligations
- no reusable stateful-component primitive; the elaborator instantiates a fresh
  declaration at each use site (because delay cannot sit under a binder)
- the standard library goes through the same interface

## 10. The running scenario (usable as the first integration test)

**The tilt lamp**: `Tilt`, `Brightness`, `Temperature`, `Held`

- `dimByTilt : Tilt -> Brightness` (declared first, defined later; the curve
  `clamp(0.2 + 0.8·θ/60°, 0, 1)`)
- `hold while not Held`
- a `Warm` condition + an `every 1 s` pulse; `Critical` forces the heater to 0
- two behaviors compete for the light → `lampTarget : Brightness`, the single
  driver
- two domains: interaction (Tilt / Held, 50 Hz) and ambient (Temperature, 1 Hz);
  `heaterTarget` reads across domains → sync + initial value
- board: Arduino Nano; light = PWM, heater = digital output, IMU = I2C,
  temperature = analog input

**Four motors + IMU** (SAT):
`M1 -> D3/D0, M2 -> D5/D1, M3 -> D6/D2, M4 -> D9/D4, IMU -> A4/A5`. **Seven PWM
channels** (UNSAT on the Nano, SAT on the big board); **2 interrupts + 6 PWM**
(enough pins by count, but UNSAT: D3 conflicts).

## 11. Questions the paper leaves open (decided during implementation; see the issue registry)

- how the surface convenience of several candidate definitions with one active
  reduces to a write-once realization
- interface-level references (a commitment mentioning another declaration) are
  not modelled
- whether a realization may delegate its grant to a higher-order parameter
- affine units (°C vs K)
- event buffering needs an object-language list type
- the agreement of unfolding with tick evaluation is proved for first order only
- `Causal` is conservative on a lambda-guarded cycle
- numeric electrical / timing constraints, and sum constraints (non-binary)
- `diagnose` reports the first dead end, not a minimal unsat core
