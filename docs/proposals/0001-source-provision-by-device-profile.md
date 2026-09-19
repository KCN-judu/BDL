---
id: PRP-0001
status: draft
date: 2026-09-20
area: formal
related-issues: [ISS-0016, ISS-0004]
superseded-by: []
---

# PRP-0001: A Source is provisioned at deployment by a device profile's transducer

## Problem

A Source is an unresolved `() -> C` (ADR-0032; FV Phase 12
`Source Δ d := realizationOf d = none`). Its value at a tick is the kernel's
input `I d t` — already _at the concept's type_ `C`: a `Temperature`, a
`ButtonHeld`. Nothing in the kernel, the design or the deployment says how a
value of `C` comes to exist. On a product it does not: an ADC channel yields a
count or a voltage, a GPIO yields a level, an I2C sensor yields a register
image. The step from that raw reading to the concept — `RawReading -> C` — is
today the platform adapter's, in host code, unnamed by the design, unchecked by
the compiler and outside every theorem (`REPORT.md` Phase 12: "Sensor reads,
buses, GPIO/PWM and device I/O are the platform adapter's; nothing in the kernel
names them").

So a Source is really a composition the model does not write down:

```text
s : () -> C   =   ( () -> R ) ∘ ( R -> C )
                   provision        transducer
                   the peripheral   the sensor's transfer function
```

The first factor is what deployment allocates (ISS-0016: no device binding for a
Source exists). The second has no home at all: it is neither a design
relationship (ADR-0032 §1: the design graph is unchanged by deployment; the
designer must not write `adcToTemp`) nor a device binding (ADR-0015: a device is
a kind and pins, never a formula). The raw type `R` is not chosen anywhere — it
is fixed the moment a device kind is chosen, which is exactly when the
transducer becomes known. The FV has the pieces (D-110: ADC calibration and unit
conversion are one affine chart, `exH`; Phase 3: `mk c` under the grant of a
declaration whose result is `c`; Phase 7: the capability vocabulary) and no
statement joining them.

## Goals and non-goals

Goals:

- One formal notion, _provision_, that turns a Source `s : () -> C` into a raw
  input `r : () -> R` plus a realization `s := f(r)` taken from a device
  profile, and a theorem that the design cannot tell the difference.
- The raw type `R` and the transducer `f` come from the device, are checked by
  the kernel's own rules (typing, dimension, grant), and are compiled by the
  generated core — never hand-written in an adapter.
- The design file is untouched. The designer keeps
  `mapping TempSensor : () -> RoomTemp`; provision is a deployment artefact
  (ADR-0032 §1, ADR-0015).

Non-goals:

- A `source` kind, an effect type, or any change to `Ty`, `HasType`, `Grant`,
  causality or clocks (D-118, ADR-0006). Provision is a construction _over_
  designs, like instantiation (Phase 8), not a new typing rule.
- The output side is stated for symmetry but not designed here: a sink's
  `accepts` and a device's `C -> R_out` are the same shape (`REPORT.md` §6.5
  already asks for "an explicit `rep`-typed declaration in between").
- Sampling rates, bus scheduling, interrupt latency: the raw input has the
  Source's clock; anything faster or slower is a later `sync` question.

## Proposed design

### Vocabulary

For a design `Δ` with concept environment `Θ` and clocks `Κ`:

- A **device profile** is `P = ⟨raw : Ty, tr : Expr, rep : Ty⟩` with `raw`
  sem-free data (`Ty.SemFree ∧ Ty.Data`: `bool`, `nat`, `q d`, products and
  lists of those — never a `sem`), and `tr` a closed term such that
  `HasType Θ [] tr (arr raw rep)` under `Grant.none`. The profile is polymorphic
  in the concept: it produces a _representation_ value (`q Temperature`), not a
  `Temperature`. Examples: an ADC channel `⟨q Voltage, chart, q Voltage⟩`; a
  thermistor `⟨nat, λn. chart₂(chart₁ n), q Temperature⟩` (D-110's `exH`
  composed); a GPIO `⟨bool, λb. b, bool⟩`; an I2C sensor whose driver already
  converts, `⟨q Temperature, λx. x, q Temperature⟩`.
- A profile **fits** a Source `s` with `tyView s = sem c` iff `Θ c = some rep`
  (the profile's representation is the concept's, D-29); with `tyView s = q d`
  or `bool` iff `rep` is that type. Fitting is decidable.
- **Provision** of `s` by `P` with a fresh `r ∉ dom Δ`:

  ```text
  provision Δ Κ s P r  :=  Δ [ r ↦ unresolved ⟨raw, []⟩ ]
                             [ s ↦ realized by  mk c (app tr (declRef r)) ]   -- or  app tr (declRef r)  when tyView s is not a sem
  Κ'                   :=  Κ [ r ↦ Κ s ]
  ```

  `mk c` is admissible because `s`'s own grant is `Grant.of (sem c) = {c}`
  (`Decl.lean`): the Source constructs its concept under the authority its
  signature already gives it. Nothing else in the design gains a grant.

`r` is the **monomorphised** Source: the design was polymorphic in how `C`
arises; choosing `P` fixes `raw`. The deployed design `Δ'` has one more
unresolved declaration and one fewer;
`SimulationInput Δ' = SimulationInput Δ ∖ {s} ∪ {r}`.

### Claims to prove

Let `Δ' = provision Δ Κ s P r`, `Κ'` as above, and for an input `I'` of `Δ'`
define the **induced input** `I := I' [ s ↦ λt. ⟦tr⟧(I' r t) ]` (the value `tr`
gives at tick `t`, wrapped by `mk c`).

1. `provision_typed` — if `Δ` is globally well typed (`GlobalWF`), `P` fits `s`,
   and `r` is fresh, then `Δ'` is globally well typed and
   `tyView Δ' s = tyView Δ s`. Every other declaration's interface is unchanged
   (`DeclLeq` holds pointwise: provision is a **refinement** of `s` in the sense
   of `Decl.lean`, never an edit).
2. `provision_causal` — `Causal Δ → Causal Δ'` (`r` has no references; `s`
   references only `r`; every other realization is unchanged).
3. `provision_clocked` — clock consistency (Phase 5) is preserved with
   `Κ' r = Κ s`.
4. `provision_transparent` (the main theorem) — for every schedule `S`, clock
   `c`, tick `t`, environment `ρ`, expression `e` not mentioning `r`, and value
   `v`:

   ```text
   MEv S Δ I c t ρ e v  ↔  MEv S Δ' I' c t ρ e v
   ```

   In words: the deployed design, fed raw readings, behaves exactly as the
   abstract design fed the transduced values. `PhysicalOutput` is unchanged for
   every sink (corollary via `eval_independent_of_drives`).

5. `provision_abstracts` — every trace of `Δ'` is a trace of `Δ` (take the
   induced `I`); hence every property proved of `Δ` for all inputs holds of
   every deployment. When `tr` is surjective onto `rep`'s inhabitants the two
   trace sets are equal; when it is not (an ADC saturates), deployment is a
   strict refinement, and a design property that depends on values `tr` never
   produces is vacuous on that device — a fact worth reporting, not hiding.
6. `provision_comm` — provisioning two Sources commutes; provisioning is
   idempotent per Source (`s` is no longer a Source after provision:
   `resolved_not_source`).
7. `provision_indistinguishable` — a consumer of `s` cannot tell a provisioned
   `s` from a Source (`consumers_indistinguishable` lifted through the induced
   input): the design's clients are unaffected, which is what lets the design
   graph stay unchanged.

### What the kernel gains

Nothing in `Core`. Provision lives in `Surface` (next to `UnitDomain.lean`), as
a definition over `DeclEnv × ClockEnv` and a list of theorems about `MEv`. The
device profile's `tr` is an ordinary term; its charts are Phase 10b
(`Charts.lean`), which is why the affine-unit issue (ISS-0004) is on the same
path: a profile for a °C sensor is an affine chart, and the design's
`Temperature` stays in kelvin.

### And symmetrically, outputs

A sink `o` with `accepts = C` and a device profile `⟨rep, tr_out, raw_out⟩` with
`tr_out : arr rep raw_out` provisions to a fresh realized declaration
`w := app tr_out (rep (declRef d))` where `d` drives `o`, and a sink `o'` with
`accepts = raw_out` driven by `w`. `DriveWF`, `SingleDriver` and
`CompleteOutputs` are preserved and `PhysicalOutput` maps through `tr_out`. This
is the "explicit `rep`-typed declaration in between" of `REPORT.md` §6.5, made
by deployment rather than by hand. It is listed so that the input theorem is
stated in a form the output one can reuse; it need not be proved first.

## Compatibility and migration

Nothing changes for existing projects, files, protocol clients or the design
graph: provision is computed from a design plus a deployment and stored, if at
all, with the deployment (an extension of ADR-0015's device binding, the
`for <source>` end ISS-0016 asks for). A design with no provisioned Sources is
today's design. The generated core's input vector changes _only for a
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
- **A `Sensor` kind or an effect type.** Rejected by D-118 and ADR-0032 on the
  same evidence: every `() -> A` unresolved is environment provision "whatever
  its type"; provision needs no new kind, only a construction.
- **Put `raw` on the concept** (`concept RoomTemp : Temperature from Voltage`).
  Rejected: the raw type belongs to the device, not the meaning; two products
  with different sensors would need two concepts.

## Implementation and evidence

Formal first (this proposal's ask):

- `BDL/Surface/Provision.lean`: `DeviceProfile`, `Fits`, `provision`,
  `inducedInput`, theorems 1–7; executed examples for the thermistor (`exH`
  charts composed), a GPIO, an identity profile, and a saturating ADC showing
  strict refinement; a negative example that a profile with `rep ≠ Θ c` does not
  fit and that a profile using `mk` of another concept is rejected by the grant.
- `docs/project/formal-correspondence.md` gains the row; the ADR that accepts
  this proposal cites the theorems in `fv`.

Then engineering, in order (not designed here — each is its own ADR when the
proofs stand):

1. `bdl-hardware`: a device catalog entry gains
   `provides { raw, transducer, representation }` as BDL formula text; `Device`
   gains a `for <source>` end (ISS-0016).
2. `bdl-compiler::analyze_deployment` computes the provisioned design and checks
   `Fits` in product words ("this sensor reports a voltage; RoomTemp is a
   temperature").
3. `bdl-lower` / `bdl-codegen-rust`: input slots at `raw`; `tr` inlined; the
   differential tests compare the provisioned core against the reference
   evaluator on the provisioned design (theorem 4 is the oracle).
4. Studio: the Deploy page's Sources list (device, raw type, transducer
   summary); the inspector's _Realization_ row names the device; the Simulate
   page can run "as deployed", feeding raw values and showing both columns.

## Open questions

- **Memory in a transducer.** Debouncing and filtering want `delay` inside `tr`.
  Phase 12 permits memory in a zero-input realization (D-116), so `s := tr(r)`
  may hold it — but then `tr` is not a function and theorem 5's "surjective ⇒
  equal traces" needs restating over streams. Start with pure `tr`; note the
  extension.
- **One raw reading, several concepts.** An IMU yields one register image for
  three Sources. Provision of `{s₁, s₂, s₃}` by one profile with one shared `r`
  and three projections is the same construction; theorem 6 should be stated for
  a set of Sources from the start.
- **Whose clock is the raw input's.** `Κ r = Κ s` is the least commitment; a
  device with its own rate is a `sync` at deployment, which Phase 5/8 already
  covers but which the Deploy page would have to offer.
- **The grant argument.** Theorem 1 relies on `mk c` being admissible in `s`'s
  own realization. Confirm that `Grant.of (sem c)` is what `HasType` uses for an
  unresolved declaration once realized (it is for realized ones today).
- **Range facts.** Whether the compiler should surface "on this device, RoomTemp
  never exceeds 358 K" from a profile's declared range (theorem 5's
  strict-refinement case) — useful, but a separate proposal.

## Outcome

Open.
