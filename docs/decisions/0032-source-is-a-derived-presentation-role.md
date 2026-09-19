---
id: ADR-0032
status: accepted
date: 2026-09-19
area: studio
supersedes: []
superseded-by: []
related: [ADR-0001, ADR-0029, ADR-0031, ISS-0014]
fv: [BDL/Surface/UnitDomain.lean]
---

# ADR-0032: A Source is a derived presentation role, never a kernel type

## Status

Accepted (Source-role milestone, after the unit-domain normalization of ADR-0029
and FV Phase 12).

## Context

A designer reaches for "a sensor" and finds nothing called one: BDL has
concepts, relationships and outputs, and the value a product observes from its
environment is `mapping tilt : () -> Tilt` with no definition — a relationship
that reads nothing and is not resolved. Studio drew it _dashed, declared_, the
word for _something is missing_, and the simulation called its row an _input_.
Nothing was wrong in the model, but the picture said the opposite of the
meaning: an environment-provided value is complete, not unfinished.

The formal kernel bounds what may be done about it. FV Phase 12
(`BDL/Surface/UnitDomain.lean`, `UNIT_DOMAIN_NOTE.md`) defines
`Source Δ d := realizationOf d = none` and `UnitDomain Δ d`, proves
`source_value` (the value at a tick is the environment's input), `source_reads`
(a Source reads nothing), `resolved_not_source` (a `() -> A` with a definition
is not a Source — its value is the definition's, whatever the environment says),
`refForms_agree` (`f`, `f()` and `f(())` are one reference) and
`consumers_indistinguishable` (a consumer cannot tell a Source from a resolved
relationship). `SimulationInput := Source ∧ UnitDomain`. There is no Source
type, no effect, no `A -> ()` sink: the unit is eliminated above the kernel, and
the kernel gains nothing.

## Decision

1. **Source is a role Studio and the IDE service derive; nothing stores it.** A
   relationship is a Source exactly when it has no definition and the unit
   domain (`inputs` empty), and it stands at the environment boundary — in a
   component's source it backs no port, and on the system canvas no binding
   realises it (a realised base relationship carries a reference definition in
   the projection and is therefore a relationship). The rule is one function in
   each layer: `bdl_ide::relationship_role` (with the host's port-backed
   knowledge) and Studio's `relationshipRole` / `AppState.isSource`
   (`app/state.dart`). No project file, sidecar, protocol field or library
   template says "source"; the text is `mapping S : () -> C`, and reopening
   re-derives the role.
2. **The kernel, the text and the protocol are untouched.** No `MappingKind`, no
   effect or IO type, no sink of type `A -> ()`, no name-based rule: a
   relationship called `TempSensor` with inputs is a relationship, and renaming
   a Source keeps it one. A resolved `() -> A` is a relationship that computes
   internally (`resolved_not_source`), and adding a definition to a Source is
   the ordinary way to make it one.
3. **The canvas distinguishes the role without colour alone.** A Source node has
   zero input sockets and one output socket, never a `()` port; its header
   carries the word _Source_, an entry glyph (an arrow crossing a boundary tick)
   and a solid bar on the node's **left** edge — the environment side, the
   mirror of the sink's bar on the right — with a green header strip as the
   redundant colour cue. It is not dashed: nothing is missing. The system
   canvas's _realise_ socket (where a provided port or another base relationship
   may bind to an open declaration) is drawn only while something could bind —
   an instance exists, or another relationship produces the concept — so a
   Source in a design with neither has nothing on its left. Assistive technology
   hears _name, Source: a value entering the behavior model from the
   environment, provides C_.
4. **The inspector says what it is, not what it lacks.** A Source's Meaning
   section has a _Role: Source_ row with one sentence; its output section is
   _Provides_ (a relationship's is _Produces_); the Relationship section opens
   with _Realization: Provided by the environment; no device is bound yet._ and
   keeps the definition editor. Never _calls_, _reads hardware_ or _sensor_:
   what realises a Source is deployment's.
5. **Hover and Explain speak the formal vocabulary.** Hover: `role: Source`,
   `type: () -> C`, definition _none — provided by the environment, observed
   once per activation_. Explain: the canonical type, the kernel interface,
   `role`, `provision: environment`, and a reading that the declaration is
   observed once per activation and is not an effectful zero-argument call
   (`refForms_agree`); a resolved `() -> A` cites `resolved_not_source`.
6. **Standard Source templates are ordinary library templates.** A template with
   a `source` field creates two ordinary items in one commit — the concept and
   `mapping <source.default_name> : () -> <concept>` with no definition —
   through the same `InstantiateConceptTemplate` request (protocol 0.16,
   additive: `source_default_name`, `display_names`, `descriptions`,
   `source_name`). The library category is _Sources_, not _Sensors_: a button
   state and an analog level are Sources and not sensors. Template names and
   descriptions are localized in the library (`i18n`), identifiers never.
7. **Simulation inputs are exactly the Sources** (`SimulationInput`); the
   Simulate page's section is _Sources_. The Formula Composer inserts a Source
   by reference (`TempSensor`, never `TempSensor()`), like any unit-domain
   relationship.

## Alternatives

- _A `Source` kind on `Mapping` (or a `MappingKind::Source`)_: rejected — it
  would persist a fact the shape already states, could disagree with it after a
  definition is added, and the kernel has no such distinction to check it
  against.
- _A sink type `A -> ()` and an effect/IO type to pair with Sources_: rejected —
  `consumers_indistinguishable` and `unit_codomain_collapse` say a unit codomain
  carries nothing; outputs are a separate declaration with drive rules
  (ADR-0015).
- _Name-based detection_ (`*Sensor`, `*Input`): rejected — presentation by
  spelling is magic, breaks on rename and lies in every other language.
- _A "Sensors" category with `sensor` metadata in the library_: rejected — not
  every Source is a sensor, and the template must not carry semantics the text
  cannot (ADR-0001).
- _A protocol `role` field on `MappingView`_: rejected — Studio derives the role
  from what the projection and the system view already carry; the IDE service
  derives it for hover and Explain. The only protocol change is the library's
  localized text and the template's relationship name, which the client cannot
  invent.

## Consequences

- Five distinct things stay distinct: the concept (what is measured), its
  representation (value form), the relationship (`() -> C`), its role (Source,
  presentation), and the device binding (deployment). A device binding for a
  Source is not modelled in this milestone (ISS-0016).
- `docs/architecture/studio-ui.md` §2, §4, §10 and
  `docs/spec/concept-library.md` describe the treatment;
  `docs/spec/textual-syntax.md` notes that the text has no Source keyword;
  `docs/project/formal-correspondence.md` carries the Phase 12 row; the change
  fragment is `docs/changes/unreleased/2026-09-source-role.md`.
- The glossary term is _Source_ (zh-Hans 来源, ja 入力元), never _source code_,
  _file_, _sensor_ or _signal_.
