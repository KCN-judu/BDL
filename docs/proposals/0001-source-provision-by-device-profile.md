---
id: PRP-0001
status: draft
date: 2026-09-20
area: formal
related-issues: [ISS-0016]
superseded-by: []
---

# PRP-0001: A Source is provisioned at deployment by a device profile's transducer

> **Revision of 2026-09-20.** The first draft's claims were tested in the formal
> development (BDL_FV Phase 13, `af25567`, `BDL/Surface/Provision.lean`,
> `PROVISION_NOTE.md`). The construction exists and is a construction over
> designs; four of the seven claims were too strong or wrongly worded and are
> restated below as proved. Nothing in production implements this proposal.
> Status stays _draft_ for human review; what remains is a design decision, not
> a formal question.

## Problem

A Source is an unresolved `() -> C` (ADR-0032; FV Phase 12
`Source Δ d := realizationOf d = none`). Its value at a tick is the kernel's
input `I d t` — already _at the concept's type_ `C`: a `Temperature`, a
`ButtonHeld`. Nothing in the kernel, the design or the deployment says how a
value of `C` comes to exist. On a product it does not: an ADC channel yields a
count or a voltage, a GPIO yields a level, an I2C sensor yields a register
image. The step from that raw reading to the concept — `RawReading -> C` — is
today the platform adapter's, in host code, unnamed by the design, unchecked by
the compiler and outside every theorem.

So a Source is really a composition the model does not write down:

```text
s : () -> C   =   ( () -> R ) ∘ ( R -> rep(C) )   then  mk C
                   provision        transducer         the Source's own grant
                   the peripheral   the device's transfer function
```

The first factor is what deployment allocates (ISS-0016: no device binding for a
Source exists). The second has no home at all: it is neither a design
relationship (ADR-0032 §1: the design graph is unchanged by deployment; the
designer must not write `adcToTemp`) nor a device binding (ADR-0015: a device is
a kind and pins, never a formula). The raw type `R` is fixed the moment a device
kind is chosen, which is exactly when the transducer becomes known.

## Goals and non-goals

Goals:

- One formal notion, _provision_, that turns a Source `s : () -> C` into a raw
  declaration `r : () -> R` plus a realization `s := tr(r)` taken from a device
  profile, and a theorem that the design cannot tell the difference. **Done in
  the formal development; see the claims below.**
- The raw type `R` and the transducer `tr` come from the device, are checked by
  the kernel's own rules (typing, dimension, grant), and would be compiled by
  the generated core — never hand-written in an adapter.
- The design file is untouched. The designer keeps
  `mapping TempSensor : () -> RoomTemp`; provision is a deployment artefact
  (ADR-0032 §1, ADR-0015).

Non-goals:

- A `source` kind, an effect type, or any change to `Ty`, `HasType`, `Grant`,
  causality or clocks (D-118, ADR-0006). Provision is a construction _over_
  designs, like instantiation (Phase 8), not a new typing rule. **Confirmed:**
  nothing entered `Core`; `provision` is a function on environments built from
  `DesignDecl`, `declRef`, `app`, `mk` and the clock environment (D-121).
- The output side is stated for symmetry but not designed here (see the last
  section). The formal audit found no shared abstraction that makes it free.
- Sampling rates, bus scheduling, interrupt latency: the raw declaration has the
  Source's clock; a device with its own rate is a later `sync` question.
- Transducers with memory (debouncing, filtering): outside this version; see
  Open questions.

## Proposed design

### Vocabulary (as formalized)

For a design `Δ` with concept environment `Θ` and clocks `Κ`:

- A **channel** over a raw type `raw` is `⟨rep, tr, transfer, computes⟩`: a
  representation type `rep` (sem-free data), a term `tr`, its **transfer
  function** `transfer` on values (what the data sheet says), and the coherence
  `computes` — `tr` computes `transfer` on every raw-typed value. The term is
  **pure** (`Expr.Pure`: no `declRef`, `delay`, `sync`). Typing is
  `Channel.WF Θ ch := HasType Θ ∅ Grant.none [] tr (arr raw rep)` — in the
  _empty_ design, under _no_ grant. The channel is generic in the concept: it
  produces a representation value (`q Temperature`), not a `Temperature`.
- A **device profile** is `⟨raw, channels⟩` with `raw` sem-free data. A profile
  mentions no concept, no declaration and no clock: it is a catalog entry.
  Examples: an ADC channel over counts; a thermistor `λn. n·2 + 250` at
  `q Temperature` from counts (`exB`); a GPIO `λb. b` (`exA`); an IMU register
  image `q Angle × q Angle` with the channels `fst` and `snd` (`exF`).
- A channel **fits** a Source `s` with `tyView s = sem c` iff `Θ c = some rep`;
  with a representation type iff the types coincide. Fitting is decidable
  (`instDecidableFits`); the negatives are executed (`exC`).
- A **provision** is `⟨r, clock, chan : DeclId → Option Channel⟩`: one fresh raw
  declaration, its domain, and the assignment of channels to the target Sources.
  **Several targets may share the one raw reading**; one target is the singleton
  `Provision.one`. Provision of `Δ`:

  ```text
  provision Δ P d =
    if d = r then ⟨r, ⟨raw, []⟩, unresolved⟩
    else match Δ d, chan d with
         | some h, some ch => ⟨h.id, h.interface, realized by  mk c (app tr (declRef r))⟩   -- at sem c
                                                            -- or  app tr (declRef r)   at a representation type
         | some h, none    => h
         | none, _         => none
  provisionΚ Κ P = Κ [ r ↦ clock ]
  ```

  `mk c` is admissible because `s`'s realization is checked under
  `Grant.of (sem c) = {c}` (`realization_checked_under_own_grant`,
  `grant_of_sem` — from `Satisfies` as it stands), and a channel term typed
  under `Grant.none` constructs no concept (`channel_constructs_nothing`). The
  same `λx. mk RoomTemp x` is refused under the empty grant and accepted under
  the Source's own (`exC`).

  The raw declaration's kernel type is `raw`; its canonical type is `() -> raw`
  (Phase 12, `raw_interface`). Nothing in the kernel encoding is a unit.

- **Preconditions** `WF Θ Δ P`: `r` fresh; `raw` sem-free data; every target
  declared, unresolved, fitted, with a well-typed channel. `ClockWF Κ P`: every
  target is read in `clock`. `RawInput P I'`: the raw input is typed at `r` and
  closure-free.

- The **induced input** of a raw input `I'`:
  `induced Δ P I' s t = wrap_s (transfer (I' r t))` at a target, `I' d t`
  elsewhere.

`r` is the **raw declaration**; `s` before provision is the **abstract Source**,
after it the **provisioned Source**. (The first draft said "monomorphised";
BDL's rank-1 polymorphism, Phase 9b, owns that word and nothing here
instantiates a scheme.) The Source role moves from the targets to `r`
(`provision_source_role`).

### Why purity, and why the transfer function

Typing in the empty design already forbids reading a declaration
(`Channel.WF_refFree`), but not memory: `(λk. λn. k) (delay 0 1)` is typed at
`q₀ -> q₀` in the empty design and maps the raw value `7` to `0` at tick 0 and
to `1` at tick 1 (`exD`). Such a term is not a function of the raw reading, and
the theorems below are about functions. So the profile condition is purity —
equivalently, typed in the empty design and delay-free
(`pure_iff_delayFree_of_wf`). The channel carries `transfer` beside `tr` because
the induced input must be a _function_: extracting it from the per-tick
existence of a value is a choice principle, which the formal development does
not use; `computes` is the per-profile obligation that ties the two, discharged
by hand in every example.

### Claims (as proved)

Let `Δ' = provision Δ P`, `Κ' = provisionΚ Κ P`, `I = induced Δ P I'`.

1. `provision_envRefines` — provision is an `EnvRefines` step: every target
   keeps its id and interface and goes from unresolved to realized; `r` is new;
   nothing else moves. `provision_tyView_eq`: every pre-existing type view is
   unchanged.
2. `provision_wf` — `Δ'` is globally well formed **from**: `GlobalWF Δ`,
   monotone evidence, `WF Θ Δ P`, **and evidence for each target's commitments
   on its new realization**. The first draft omitted the last hypothesis: a
   Source's commitments are obligations on the profile.
3. `provision_causal` — `Causal Δ → Causal Δ'`: the instantaneous graph gains
   `s → r` per target and nothing else (purity keeps the channel term
   edge-free); `provision_wellClocked` — the domain judgment is preserved with
   `Κ' r = Κ s`; no device clock.
4. `provision_transparent` (the main theorem) — for every schedule, domain,
   tick, term `e` with `r ∉ e.refs`, and local environment whose closures avoid
   `r`:

   ```text
   MEv S Δ I c t ρ e v  ↔  MEv S Δ' I' c t ρ e v
   ```

   under `WF`, `RawInput P I'`, and `NoMention Δ r` (no realization of `Δ`
   mentions `r` — true of every globally well-typed design,
   `NoMention.of_globalWF`). The first draft stated none of these. The
   observation boundary has an equivalent typed form: any term typed in the
   abstract design cannot name `r` (`provision_transparent_typed`).
   `PhysicalOutput` is unchanged for every sink (`provision_physicalOutput`);
   every declaration of `Δ` is observed identically
   (`provision_decl_transparent`).

5. `provision_abstracts` — every trace of `Δ'` under a raw input is a trace of
   `Δ` under the induced input: deployment _restricts_ the abstract environment;
   it does not give the abstract design its meaning. `provision_exact` — the
   trace sets coincide exactly for the abstract inputs that have a **joint
   section**: a raw trace every channel transfers to what the abstract input
   gives its target (`JointSection`). For one channel a pointwise right inverse
   on typed values is a joint section (`JointSection.one`). The first draft's
   "surjective ⇒ equal traces" is wrong as stated: pointwise surjectivity is an
   existence per tick, and for a shared raw reading it is insufficient even in
   principle — `id` and `succ` from one reading are each onto, and the abstract
   pair `(5, 9)` has no witness (`no_joint_witness`). The strict case is
   executed: a saturating ADC never yields 451 K; the abstract design observes
   `TempSensor = 451 K`, no provisioned deployment does (`exE`).
6. `provision_not_reapplicable` — after provision no target is a Source and `WF`
   fails (`r` is no longer fresh): provision is a one-way step. The first draft
   said "idempotent per Source"; the totalized function does satisfy
   `P (P Δ) = P Δ` (`provision_idem_total`), but only because a second pass
   overwrites every target with the same body, and a second pass with a
   _different_ term is not a refinement
   (`provision_reprovision_not_refinement`). `provision_comm` — independent
   provisions (distinct `r`, neither `r` a target of the other, disjoint
   targets) commute **exactly**, as environment equality; `provision_perm` — the
   channel assignment is a set.
7. The first draft's "theorem 7" (`consumers_indistinguishable` lifted through
   the induced input) is dropped: that theorem is about `A -> ()` and unrelated;
   the client-side statement is claim 4 itself.

Shared raw reading, several targets (`exF`): one IMU image provisions `pitch`
and `roll`; both keep their identities; each realization uses its own channel;
the consumer `level` reads both; the permuted assignment is the same provision.

### What the kernel gains

Nothing in `Core`. Provision lives in `BDL/Surface/Provision.lean` as a
definition over `DeclEnv × ClockEnv` and theorems about `MEv`. The supporting
lemmas are reusable beyond provision: `MEv.of_ev_pure` (a pure single-domain
derivation is a multi-domain derivation), `Transduces.mev`, the simulation lemma
`simulate`, `input_congr`, `clockedB_congr`, `Value.All`.

Charts (Phase 10b) are the natural language for a channel's `transfer` when it
is a calibration; **nothing here depends on exposing °C/°F to designers
(ISS-0004)**. The thermistor example is a linear chart on counts and the design
keeps kelvin: a profile may calibrate raw data into canonical `Temperature`
while the language presents kelvin only. The first draft's dependency on
ISS-0004 is withdrawn.

### And symmetrically, outputs

A sink `o` with `accepts = C` and a device channel `⟨rep, tr_out, raw_out⟩` with
`tr_out : arr rep raw_out` would provision to a fresh realized declaration
`w := app tr_out (rep (declRef d))` where `d` drives `o`, and a sink `o'` with
`accepts = raw_out` driven by `w`; `DriveWF`, `SingleDriver`, `CompleteOutputs`
and `PhysicalOutput` through `tr_out`. This is the "explicit `rep`-typed
declaration in between" of `REPORT.md` §6.5, made by deployment rather than by
hand. It is listed as the duality note; the input-side proofs did not make it
free (the drive edge's type equality needs the new declaration; only `simulate`
would be reused), and it is not part of this proposal.

## Compatibility and migration

Nothing changes for existing projects, files, protocol clients or the design
graph: provision is computed from a design plus a deployment and stored, if at
all, with the deployment (an extension of ADR-0015's device binding, the
`for <source>` end ISS-0016 asks for). A design with no provisioned Sources is
today's design. The generated core's input vector would change _only for a
provisioned deployment_: the slot for `s` is replaced by a slot for `r` at
`raw`, and `tr` is compiled like any realization.

## Alternatives

- **Keep the transducer in the platform adapter** (status quo). Rejected as the
  long-term shape: the conversion is unchecked (a thermistor profile that
  returns millivolts where a temperature is due compiles), invisible in Studio
  and Explain, and outside every theorem; ISS-0016 cannot be resolved without
  deciding where the conversion lives.
- **The designer writes the raw Source and the transducer as relationships**
  (`rawAdc : () -> Voltage`, `TempSensor = adcToTemp(rawAdc)`). Expressible
  today, and the provisioned design is _exactly_ this — but written by the
  designer it couples the design to one part number, breaks ADR-0032 §1, and
  gives the library nothing to reuse. Provision is this construction done by
  deployment from a catalog, with the proof that the designer could not have
  told the difference.
- **A `Sensor` kind or an effect type.** Rejected by D-118, D-121 and ADR-0032
  on the same evidence: every `() -> A` unresolved is environment provision
  "whatever its type"; provision needs no new kind, only a construction.
- **Put `raw` on the concept** (`concept RoomTemp : Temperature from Voltage`).
  Rejected: the raw type belongs to the device, not the meaning; two products
  with different sensors would need two concepts (D-124).
- **The singleton as the primitive**, generalized later. Rejected: the IMU case
  and the joint-section finding are invisible in the singleton; the shared-raw
  form is primitive and the singleton its case (D-125).
- **"Closed and well-typed" as the profile condition.** Rejected: `exD` — a
  typed term with memory is not a function of the raw reading (D-122).

## Implementation and evidence

Formal (done, BDL_FV Phase 13, `af25567`): `BDL/Surface/Provision.lean`,
`BDL/Experiments/ProvisionExamples.lean`, `PROVISION_NOTE.md`, D-121..D-130; 92
theorems on `propext`/`Quot.sound`. `docs/project/formal-correspondence.md`
carries the row (**formally proved** (model) · **not implemented**).

Engineering, in order, each its own ADR when this proposal is accepted (not
designed here):

1. `bdl-hardware`: a device catalog entry gains
   `provides { raw, channels: [{ representation, transducer, transfer }] }` as
   BDL formula text; `Device` gains a `for <source>` end (ISS-0016) — for a
   shared reading, one device for several Sources.
2. `bdl-compiler::analyze_deployment` computes the provisioned design and checks
   `Fits`, purity (`Channel.WF ∧ DelayFree`) and the commitments of the
   provisioned Sources, in product words ("this sensor reports a voltage;
   RoomTemp is a temperature"). Whether `computes` is checked (it is a `∀` over
   raw values) or trusted from the catalog with differential tests is an open
   question below.
3. `bdl-lower` / `bdl-codegen-rust`: input slots at `raw`; `tr` inlined; the
   differential tests compare the provisioned core against the reference
   evaluator on the provisioned design (`provision_transparent` is the oracle).
4. Studio: the Deploy page's Sources list (device, raw type, channels); the
   inspector's _Realization_ row names the device; the Simulate page can run "as
   deployed", feeding raw values and showing both columns.

## Open questions

- **Memory in a transducer.** Debouncing and filtering want `delay` inside `tr`.
  Phase 12 permits memory in a zero-input realization (D-116), so `s := tr(r)`
  may hold it — but then the channel is not a function and transparency and
  exactness need restating over streams. This version requires purity; the
  extension is recorded (REPORT open items).
- **Whose clock is the raw declaration's.** `Κ r = Κ s` is the least commitment
  and what is proved; a device with its own rate is a `sync` at deployment,
  which Phase 5/8 already cover but which the Deploy page would have to offer,
  and which adds one transport declaration per target.
- **Commitments.** `provision_wf` needs evidence for each provisioned Source's
  commitments. Which commitments a Source may carry, and how a profile's
  declared range discharges them ("on this device, RoomTemp never exceeds 358 K"
  — the strict-refinement case of claim 5), is a separate proposal.
- **`computes`: checked or trusted.** The examples discharge it by hand.
- **Out-of-type raw readings.** Transparency is stated for `RawInput`; what a
  deployment does with a reading outside `raw` (a stuck realization in the
  kernel) is a validation question.
- **Output provision.** The duality note above; not designed.

## Outcome

Open. Revised from the formal audit (BDL_FV Phase 13); the remaining decision —
whether to adopt provision as the resolution of ISS-0016 and open the device
catalog work — is architectural and for human review.
