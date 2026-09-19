# Source is a derived role; standard Source templates (ADR-0032)

- Date: 2026-09-19
- Area: studio, ide, library, protocol, formal
- Affected: designers, protocol clients, developers
- Related: ADR-0032, ADR-0029, ADR-0031, ISS-0014, ISS-0016

## What changed

- **A Source is what the canvas calls a relationship that reads nothing and has
  no definition** — `mapping TempSensor : () -> RoomTemp` — when it stands at
  the environment boundary (backing no port, realised by no binding). It is no
  longer drawn _dashed, declared_: nothing is missing. The node has no input
  socket, one output socket, the word _Source_, an entry arrow and a solid bar
  on its left edge (the environment side; the sink's bar is on the right), and a
  green header strip. The role is derived from the shape and the state in Studio
  and in the IDE service alike; nothing stores it, renaming keeps it, adding a
  definition ends it, and a resolved `() -> A` is an ordinary relationship (FV
  Phase 12 `resolved_not_source`).
- **Inspector**: _Role: Source_ with one sentence; the output section is
  _Provides_; the Relationship section opens with _Realization: Provided by the
  environment; no device is bound yet_ and keeps the definition editor.
  **Hover** carries `role: Source` and _none — provided by the environment,
  observed once per activation_; **Explain** adds `role`,
  `provision: environment` and the reading that a Source is observed once per
  activation and is not an effectful zero-argument call (`refForms_agree`); a
  resolved `() -> A` cites `resolved_not_source`.
- **Library**: a _Sources_ category with seven templates — Temperature Sensor,
  Tilt Sensor, Distance Sensor, Ambient Light Sensor, Button / Switch State,
  Encoder Position, Analog Input — each creating a concept **and** its
  `() -> concept` Source in one commit (one Undo removes both). Template names
  and descriptions are localized (`i18n` in `concepts.toml`), identifiers never.
  The canvas menu gains _Add Source ▸_ (the templates, then _New source…_ over
  an existing concept); the Library rows and the sidebar wear the Source
  silhouette.
- **Simulate**: the section is _Sources_ — the simulation's inputs are exactly
  the Sources (`SimulationInput = Source ∧ UnitDomain`). The **Formula
  Composer** inserts a Source by reference (`TempSensor`, never `TempSensor()`),
  as before.
- **Formal**: FV Phase 12 (`BDL/Surface/UnitDomain.lean`) proves the Unit-domain
  normalization ISS-0014 asked for and the Source boundary; the correspondence
  rows are _formally proved (model)_; ISS-0014 is resolved.
- A device binding for a Source is not modelled: ISS-0016.

## Compatibility and migration

- Designers: nothing to do. Every project means what it did; a relationship that
  was _declared_ with no reads is now shown as a Source. The word _declared_
  still means a relationship with reads and no definition.
- Project files: nothing — no file, sidecar or text carries the role.
- Protocol clients: protocol **0.16**, additive — `ConceptTemplateView` gains
  `source_default_name`, `display_names`, `descriptions`;
  `InstantiateConceptTemplateRequest` gains `source_name`; a Source template's
  `SystemEditApplied` outcome carries `created_concept` and `created_mapping`. A
  client that ignores the new fields still instantiates the concept and gets the
  Source with it.
- Developers: `bdl_ide::{relationship_role, RelationshipRole, provision}`;
  `IdeHost::set_port_backed`, `AnalysisSnapshot::port_of`;
  `bdl_library::{instantiate_with, Instantiation, SourceSpec, TemplateText}`;
  `Session::apply_system_then`; Studio `relationshipRole`, `AppState.isSource`,
  `TemplateText.displayNameIn`, `AppLocalizations.libraryLocale`,
  `showNewSourceSheet`, `NodeShape.source`, `MappingGlyph(source:)`.

## Evidence

`crates/bdl-ide/tests/source_role.rs`, `crates/bdl-library/src/lib.rs`
(`source_templates_create_a_concept_and_an_explicit_unit_domain_relationship`),
`crates/bdl-daemon/tests/text_e2e.rs`
(`a_source_template_is_two_ordinary_edits_in_one_commit_and_writes_the_unit_domain`),
`apps/studio/test/source_role_test.dart`,
`apps/studio/test/concept_library_e2e_test.dart`.
