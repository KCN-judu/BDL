# Revision notes

A dated changelog of the document. Entries are historical: each keeps the
vocabulary and the identifiers of its date (the ledger numbers `D-NN` in the
older entries map to `FVD-NNNN` through Appendix E of the monograph), and none
is rewritten when a later revision changes a claim. The first section below is
the note of the original 2026-09 draft.

## Moved to `paper/monograph/` (2026-09-20)

- The monograph moved from `paper/` to `paper/monograph/` so that
  `paper/core_calculus/` — the core-calculus paper derived from Part IV and
  `BDL/Core/` — can sit beside it under `paper/README.md`. Nothing in
  `paper.md` changed except the sentences naming its own directory (the
  preface and Appendix I); `body.typ` and the PDF were regenerated.
- Entries below this one keep the paths of their date.

## The 2026-09 draft (conference manuscript, archived)

- Rewritten in academic English.
- Reorganized into an ACM-style research-paper structure.
- Promoted **signature-first authoring** to a core semantic principle.
- A Mapping Block is now defined by a type signature before its body, e.g. `?f : Tilt -> Brightness`.
- Formula/curve/example definitions are attachments to a Mapping Block, not separate execution nodes.
- Separated semantic nominal types from physical dimensions.
- Kept refinement/range/timing/device feasibility as validation obligations rather than core type equality.
- Restricted the first action-handler model to deterministic request/policy semantics instead of claiming a general algebraic-effect calculus.
- Made clocks explicit in reactive types and prohibited hidden resampling.
- Specified reset-on-entry semantics for local StateHandler temporal state.
- Added deterministic tick semantics, causality conditions, and preservation/determinism theorem statements with explicit proof assumptions.
- Added a designer-centered progressive authoring workflow and evaluation plan.

## Formal-semantics correction pass (2026-09-14)

- Removed `d/H` from the reactive type system. StateHandlers now preserve the enclosing domain and contribute only an activation mask plus reset semantics.
- Added activation stratification so a handler cannot depend instantaneously on values that only exist while that handler is active.
- Added a locally-finite global physical-time schedule for multi-domain execution and rewrote tick semantics over global steps rather than an undefined root clock.
- Made cross-domain synchronization a causal boundary: target ticks observe only source data committed at strictly earlier physical times.
- Corrected the equal-rate case: a cross-domain synchronizer does not degenerate to an identity wire; zero-latency behavior requires an explicit domain merge.
- Replaced generic event `coalesce` with `coalesce(mu)` requiring an explicit deterministic merge function, and made bounded-buffer capacity an engineering obligation.
- Added `Raw[r]` as a representation type and removed the unsound identification of raw device readings with `Q[0]`.
- Replaced the frequency/angular-velocity nominal-typing example with torque/energy; angle is retained as an explicit physical dimension where needed.
- Changed action signatures to request-only operation declarations `Delta(op) = P_op`; runtime requests are dependent pairs and no unused synchronous result type remains.
- Fixed the `Verified(P)` typesetting typo and weakened the workshop anecdote from a causal claim to evidence suggesting a representational problem.
- Removed the unresolved five-axis-printer TODO from the paper's claims; the example now motivates asynchronous outcomes without pretending that unreported timing data are evidence.

## Revision to match the formal development through Phase 7 (2026-09-15)

- Rewritten as a coherent manuscript rather than an amended draft; section
  structure now follows the architecture that survived the Lean 4
  design-space exploration (`../REPORT.md`, `../DESIGN_DECISIONS.md`,
  `../MINIMALITY.md`).
- "Typed hole" is a designer-facing metaphor only; the kernel object is a
  `DesignDecl` with an optional realization, and the key result is client
  stability under monotone refinement (refinement vs edit made precise).
- Signature-first claim narrowed: an unresolved relationship is a legal
  statically meaningful state; no claim that designers think signature-first.
- Semantic identity: nominal `sem SemanticId`, independent of declaration
  identity, display name, dimension and hardware; explicit mappings are
  ordinary declarations, not casts. Representation binding via `ConceptEnv`,
  `rep` free, `mk` licensed by the realized declaration's own signature.
- Dimensions as `q Dim` with the algebra in primitive operator types; units
  are surface (linear scaling only).
- Reactive section replaced: one primitive `delay init e`, tick semantics,
  causality on instantaneous dependency; no `Signal`/`Event` types; every
  temporal operator derived; StateHandler reduction stated for tested cases.
- Clock domains: nominal `ClockId`, schedule, `Clocked` judgment, `sync src
  init e` with strictly-before reads; `delay` is `sync` at the own domain;
  event transport via the window model.
- Effect rows, action requests, policies and arbitration removed; physical
  outputs are nominal sinks with one explicit driver; negative results
  stated narrowly.
- New section on target-specific hardware validation (Phase 7): resources,
  capabilities, units, requirements, a sound and complete solver, the
  Arduino Nano case study, and target-sensitive evidence.
- Elaboration and interaction model rewritten around visible acceptance
  levels; evaluation plan retained without results; figures replaced by
  Typst-native diagrams (`authoring_layers.png` no longer used).
- References added (Lean 4, Lustre, Esterel, Hazel live holes, Dechter);
  none fabricated.

## Interaction chapter and prose pass (2026-09-15, second pass)

- New section "Designer Interaction Model" between the surface vocabulary and
  the formal architecture: a running tilt-lamp scenario (Tilt, Brightness,
  Temperature, Held; heater with warning/critical thresholds; Arduino Nano
  chosen last) carried through intent, local refinement, temporal phrases,
  contexts, the single-driver diagnostic and its repair, cross-domain
  transport, board selection with SAT/UNSAT and manual pins, workspace
  states, an explanation view, and the workflow as a whole. Diagnostics are
  phrased in product terms; all user-side statements remain hypotheses.
- The former "Interaction Model" layer section is absorbed; the acceptance
  level figure now lists workspace states and lives in the new section.
- Prose pass over the whole manuscript to remove report-style
  meta-language ("the development proves/shows…", "tested formulation…"),
  vary paragraph openings, and restore ordinary prose where lists had been
  used for conceptual material. No technical claims changed; formulas,
  tables and figures kept verbatim.

## The conceptual restructuring and the production snapshot `de8154f` (2026-09-20)

- The monograph is restructured by concept: Purpose and design position;
  the designer-facing language; the core declaration model; semantic identity
  and physical quantities; the data and equation language; units, coordinates
  and charts; reactive semantics; behavior systems; the environment and the
  physical boundaries (the canonical type `() -> B`, the source role, physical
  sinks, `A -> ()`, and Phase 13's provision, integrated as the completion of
  the input boundary); validation and deployment; production compiler,
  runtime and daemon; Studio and the IDE; formal ↔ production correspondence;
  minimality and rejected alternatives (with the two kinds of minimality made
  explicit); the open agenda with the empirical questions kept apart. Phase
  numbers remain as provenance and in Appendix G; no chapter is a phase.
- Production is described at `de8154f5153495de2ad8a09f3ca3166c3678dc93`
  (2026-09-20): the derived relationship role (Source, Rule, Value) with one
  home in the model; a Source may drive an output; `applied_by` as the direct
  inverse of `references`; `reactive.rule_unapplied` and `rule.apply`; the
  generalized Standard Library (items, fragments); the unified project with
  invalid text saveable; the Formula Composer's structured forms (binders,
  ranges, `??`, boolean logic, `if`) and its opaque forms; semantic
  highlighting from one classifier on the LSP token vocabulary; protocol
  0.20/0.21. Commitments are stated as empty in production. Earlier snapshot
  hashes (`f1ce82c`, `3c6c8be`, `876005c`) survive only in this changelog and
  in Appendix H.
- The formal decisions are cited as `FVD-NNNN` and the open items as
  `FVI-NNNN`, each with its production record; Appendix E is the migration map
  from `D-01 … D-130`. Theorem names are unchanged.
- New apparatus: the notation appendix, the theorem index by concept (every
  name checked against the Lean sources), the decision index, the
  evidence-strength ledger, the production snapshot appendix, the development
  chronology, the revision log. The internal "possible paper slices" section
  is removed; the front matter states what the document is and is not.
- `paper/README.md` and `main.typ` describe the monograph pipeline; the
  ACM-era description is gone. `archive/` keeps the conference manuscript as
  submitted.

## Phase 14 as one physical boundary and the production snapshot `6be778b` (2026-09-20, later revision)

- **Phase 14 integrated and hardened.** Output realization by device encoders
  is written into the physical-boundary chapter as the output half of one
  story rather than an addendum: the logical Output stays behavior semantics;
  realization is deployment structure below it — a pure encoder
  `Rep(C) -> Raw` typed in the empty design under no grant, `EFits`, and a
  machine sink added by a lowering that leaves the behavior environment and
  every trace literally unchanged; multiple realizations of one output;
  quantizing encoders admitted; the `RawCommand` relation as the machine
  boundary. The hardening result is the current truth: admissibility is the
  encoder's typing ∧ fit ∧ a solvable board (FVD-0139 supersedes FVD-0137), and
  "fits + allocates" is recorded as the rejected criterion, never repeated as
  current.
- **The physical boundary as one whole.** A new section draws the chain from
  the environment through provision, the behavior, the logical Output,
  realization, the raw command, the platform adapter, to the physical world,
  with a table stating per arrow what is formally modelled, formally proved,
  production-implemented, production-tested and still open. The two halves are
  presented as symmetric in kind and asymmetric in construction and in status:
  the input side proved and not built, the output side proved and built
  through the raw command, the last arrow built and not proved (FVI-0022,
  ISS-0017).
- **Production snapshot moved** from `de8154f` to
  `6be778b07f07bebaba26f580f2b4af74a13ce9df` (protocol 0.24), pinned in
  `docs/project/production-correspondence.md` first and cited from the front
  matter, Appendix F and the title page: output realization implemented
  (ADR-0036: profiles, three-judgment admissibility, plan-level sinks,
  `Tick.commands`); the first embedded platform adapter on the Raspberry Pi
  Pico over Embassy (ADR-0037) described at production-test strength only —
  generated glue and firmware, the solver's pad per sink, the explicit
  reject-and-hold `duty8` policy, the compiled schedule, halt on fault, the
  cross-build in CI; the Source sheet (0.23); the Code view as an IDE surface
  (0.22); `drive light by brightness` as the current spelling with the `=`
  form as legacy syntax.
- **Design position.** BDL is stated as a behavior-design medium for
  interactive physical products — sensing, timing, computation and physical
  response crossing in one artifact — and the industrial-design question is
  framed as whether such behavior can become a design material before
  implementation dominates the representation; a general software-interaction
  language is explicitly out of scope. Nothing about designer cognition is
  claimed.
- **Studio/IDE and toolchain snapshot refreshed**: the Code view's completion,
  hover, navigation, references and format; the Source sheet with presets; the
  Deploy page's realization chooser; the adapter's crates and the target plan;
  the pipeline diagram with admissibility, sink lowering and adapter
  generation.
- **Correspondence, indexes, agenda.** New rows for output realization, the
  generated commands and the platform adapter, with their deviations (the
  plan-level lowering; the third numeric domain at the boundary; tested-not-
  proved); the decision index marks FVD-0137 superseded and links the Phase 14
  records to ADR-0036/0037 and ISS-0017; the theorem index re-verified against
  the Lean sources; the open agenda split into formal questions, production
  engineering work and empirical questions, the last extended with the
  reverse-readability question and the industrial-design study questions;
  the counts (phases, modules, theorems, decisions, items) derived from the
  repository.
- **Mirror policy.** `paper/README.md` states that this directory is the
  canonical monograph source and that `KCN-judu/BDL/reference/paper/` is a
  reference mirror that does not evolve independently.

## The narrative restructure and the production snapshot `081296d` (2026-09-20, latest revision)

- **From a topic taxonomy to one argument.** The fifteen concept-Parts of the
  previous revision are reorganized as seven Parts that a reader can follow
  from beginning to end: I *Why Behavior Needs a Design Medium* (the
  asymmetry of media, the central question, what BDL is not, who it is for,
  the empirical discipline); II *The BDL Architecture* (the design path and
  the tooling path in one Typst figure, `fig:architecture`, with the
  formal-layer figure beside it; ten architectural principles; the state of
  the record); III one interactive physical product — the tilt-dimmed lamp
  with a warming base — walked through the whole lifecycle from concepts to a
  board, each step naming the failure the layer avoids and deferring every
  proof; IV the formal model derived in dependency order (declarations and
  refinement → identity and quantities → data and equations → units → time →
  composition → the physical boundary as one chain with per-arrow evidence
  → validation and deployment), with the phase numbers as provenance only; V
  the toolchain by responsibility (owns / consumes / emits / never decides),
  with the reference evaluator as a chapter of its own, the two adapter
  families from one plan, and the IDE service last as the bridge to VI; VI
  Studio and the IDE as a design argument (user, workflow and constraints;
  why not a node editor, a code editor or an Arduino-style IDE; principles;
  what is borrowed and refused from DaVinci Resolve, Blender, CAD/EDA and
  IDE/LSP practice; the semantic projection compiler → IDE service → Code +
  Canvas; canvas, roles, gestures and context menus; diagnostics; the Code
  view; the Formula Composer; the Library and the Source sheet; Simulate and
  Deploy; the UX claims stated as hypotheses); VII the evidence — the
  correspondence, the rejected designs retold as ten arguments
  (pressure / candidate / failure / kept) before the ledger, minimality
  bounded, the open agenda in three classes. Related work follows the
  argument. The old front matter is a preface and a reading guide; the claim
  strength vocabulary and the identifier rules are Appendix I.
- **Content preserved by construction.** The revision was assembled from the
  previous revision's text by section, with headings demoted and Part
  references remapped; every technical claim was classified retained / moved
  / merged / appendix / removed before it moved, and the map is Appendix J.
  Only navigational text and the front-matter duplicate of the revision log
  were removed. New prose: the Part I chapters on what BDL is not and who it
  is for; Part II; the lifecycle spine and the failure-avoided notes of Part
  III; the chapter openings and the anchoring paragraphs of Part IV; the
  ownership table, §V.3 and the two-families paragraph of Part V; §VI.1–VI.3,
  VI.4's diagram, VI.6, VI.7 and VI.12's hypotheses; VII.2's ten arguments.
- **Production snapshot moved** from `6be778b` to
  `081296df606d577eece7e269ed250b255547d497` (2026-09-20): the Arduino Nano
  over `avr-hal` as the second platform-adapter family (ADR-0037 amended; one
  `targets::Entry` per family; a blocking `tick_wait`, PWM on the ATmega
  timers, no arena, a nightly cross-build); the vocabulary crate renamed
  `bdl-runtime-adapter`; the monograph mirrored under `reference/paper/`.
  `docs/project/production-correspondence.md` carries the same hash and the
  Arduino paragraph; `main.typ`'s title page and Appendix F re-pinned.
- **Formal development** described through Phase 15 (the adapter boundary
  and the explicit device clock; the open-item audit), previous commit
  `dce5ac4`; 21 phase reports, 60 modules, about 1 300 theorem and lemma
  declarations, 142 decisions (one superseded), 28 open items
  (8 open, 9 deferred, 11 resolved), derived from the tree.
- **References audited.** No section is cited by its previous Part number;
  ranges such as "Parts III–VIII" became "§IV.1–IV.6"; in-chapter
  cross-references say "this chapter"; the figures sit where first needed
  (`fig:architecture` and `fig:arch` in Part II, `fig:levels` in §III.11).
  Every example uses the current syntax (`drive light by brightness`; the
  `=` form appears only as the legacy spelling it is).
- **Not changed.** `paper/archive/`, the earlier entries of this file, the
  reports and decisions in `docs/`, and production's mirror (refreshed from
  here, never edited there).
