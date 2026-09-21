# The Sem-block model: the canvas draws Sem blocks and mapping blocks; a concept and a rule are templates (protocol 0.30)

- Date: 2026-09-21
- Area: studio, model, layout, protocol
- Affected: designers, project files, protocol clients, developers
- Related: ADR-0044 (supersedes ADR-0034), ADR-0043 (the vocabulary), ADR-0013
  (amendment), ADR-0032, ISS-0020 (resolved), BDL_FV FVD-0163 / Phase 21
  (`10a1a7c`), docs/architecture/studio-ui.md §2,
  docs/user-guide/studio/canvas.md,
  docs/user-guide/getting-started/first-behavior.md

## What changed

- **The canvas is the value graph.** A **Sem block** per unit-domain
  relationship — one value of a concept per tick; a Source when it has no
  definition (ADR-0032's marks unchanged) — a two-row node: the name, then its
  concept at the output socket. Each Sem block with a definition of its own has
  its **mapping block** beside it, a node of its own: a header naming the rules
  the definition applies (the word _Formula_ when none), one read socket per Sem
  block the definition names (the analysis's `references`, typed by the read
  block's concept, labelled with its name), one hollow `?` socket per open
  position (`MappingAnalysis.slots`), an output socket joined to the Sem block,
  and the formula line with its disclosure. **No concept node and no rule
  node**: a concept is the template a Sem block is created from; a rule is the
  template a mapping block applies, kept in the sidebar with its own inspector.
- **Edges**: a read edge from each Sem block a definition names into the mapping
  block's socket for it, in the read block's hue; a produce edge from each
  mapping block into the Sem block it defines — the definition itself, drawn as
  a short joint, never rerouted; the drive edge; bindings; aggregate edges of a
  collapsed group over Sem blocks only. Reference edges into the formula line
  and concept → relationship signature edges are gone. The read edges are the
  kernel's `dependsOn` (`reads_iff_dependsOn`); nothing on the canvas is a
  parse.
- **Every gesture edits text or the drive; none edits a signature.** Dragging a
  Sem block onto a Source makes the block's name its definition; onto a `?`
  socket, onto a mapping block with an open position, or onto a Sem block whose
  mapping block has one, fills the compiler's first slot (`ComposeAction.fill`,
  then one replace edit) — `WireSemBlockRequested`. Dragging a Sem block onto a
  sink accepting its concept is the drive. A read edge's _Disconnect_ (the ×, ⌫,
  the drag-away) is the compiler's `ComposeAction.unreference`: every occurrence
  of the name becomes `?`, committed as one replace edit. The produce edge has
  no _Disconnect_ (the inspector's _Detach definition_ is the one path). The
  Concept → Output authoring gesture is withdrawn with the concept node; the
  sink's inspector keeps its _Connect_ list. `LinkConceptToMappingInput` /
  `LinkMappingOutputToConcept` / `UnlinkMappingInput` are the rule template's
  parameter editor in the inspector, no longer reachable from the canvas.
- **One declaration, two nodes.** Clicking a mapping block selects its Sem block
  (one selection, one inspector, one menu); both wear the selection outline. The
  two nodes have two positions: the mapping block is placed beside the Sem block
  when it appears, and from then on each moves alone. Renaming either renames
  the block.
- **Creation**: the canvas menu's **Add Block ▸ of C** makes a Sem block of an
  existing concept at the point (the Source path, `CreateSource`); **New Concept
  ▸** (the category items) makes the concept and a block of it in one
  transaction; a Library row dropped on the canvas does the same; a library
  item's fragment inserted at a point makes the template, and a block of it
  follows as a second edit, open for naming. The Library's own row (no point)
  makes the template alone. A block created at a point takes its mapping block
  along.
- **Model (ADR-0013 amendment)**: a rule reading one concept twice gets a name
  per repeated input (`tilt1`, `tilt2`; other inputs keep their concept's name;
  given names kept while they cover the signature) on `CreateMapping` and
  `SetMappingSignature`, stored in `MappingBlock::parameters`, so that every
  input is reachable; a Sem block has no inputs and no parameters. Name
  resolution is unchanged and never resolves a value by concept; a formula
  spelling the repeated concept's name is told the names
  (`formula.name.ambiguous`).
- **Inspector**: a concept lists its _Blocks_ and the _Rules_ over it (was
  _Produced by_ / _Used by_); a Sem block _Reads_ / _Applies_ / _Read by_; a
  rule _Depends on_ / _Named in_ and its parameter editor (the _Reads_ chips),
  which a Sem block no longer shows. **Simulate**: the probe on a concept lists
  its Sem blocks, one value per tick each (was _Carried by_).
- **Layout**: the layout service keys Sem blocks, mapping blocks
  (`Layout.definitions`), sinks and instances; a concept and a rule are never
  placed (their stored positions are kept as written and drawn by nothing). A
  mapping block without a position is attached left of its Sem block on open and
  on commit; the whole-graph arrangement ranks mapping blocks before the Sem
  blocks they define. A base relationship a binding realises has no mapping
  block: its realisation socket is filled and the binding's edge is its
  definition (the _= source_ label on the node is gone; the inspector names it).
- **Diagnostics**: none added. No diagnostic counts a concept's blocks (Phase
  20's `ProducerUnique` is an optional judgment of the formal development,
  unbuilt). Kept: unknown name, not-an-input, ambiguous input, two drivers of
  one output, nominal mismatch.
- **A system project's selection** now survives the edit that refetches its
  system (it was cleared against the empty interim view).
- **Demos and fixtures**: the Button → Lamp template already had the shape
  (`lit() = pressed`). `tilt-lamp-declared` is the tutorial's two blocks before
  `brightness` has a formula. Every fixture opens and simulates as before; the
  pictures change: the concept rows and the rule nodes are gone from every
  screenshot, the mapping blocks appear, and the four fixture layouts were
  re-authored around them (the two system fixtures at zoom 0.75).

## Compatibility and migration

- Designers: a concept is no longer on the canvas — its blocks are; a rule is
  edited from the sidebar; the drag that used to add a parameter to a rule now
  writes the block's formula; a formula is a node of its own beside its block.
  The tutorial and the canvas page say so. Nothing a designer made changes
  meaning.
- Project files: none change form. `ui/layout.json` gains `definitions` — the
  mapping blocks' positions — the first time the daemon opens a project laid out
  before this change (attached left of each Sem block; nothing else moves); it
  keeps concept and rule positions as written, and the canvas ignores them. A
  rule authored in Studio with two inputs of one concept and no parameter names
  gets them on its next signature edit; until then its text writes
  `f(Tilt, Tilt)` as before.
- Protocol clients: 0.30 adds `MappingAnalysis.slots` (repeated string, node
  ids), `Layout.definitions` (`NodePosition`, keyed by the Sem block's id) and
  `ComposeAction.unreference` / `read` (a declaration id); all additive.
- Developers: `NodeKind.definition`, `NodeRef.definition`, `LinkKind`,
  `asDeclaration`, `attachedBlockPosition`; `NodeCanvas.unapplied` → `slots`;
  `LinkShape.reference`, `NodeShape.{declared, rule, unapplied, formulaEntry}`,
  `isAuthoringTarget`, `NodeMetrics.{conceptWidth, conceptHeight}` (now
  `semWidth`, `semHeight`) are gone; `SocketRole.{produce, read, slot}`,
  `NodeShape.{applies, acceptsBlock}`, `blockDropTarget`,
  `WireSemBlockRequested`, `AddBlockRequested`, `EditorState.pendingWire` (a
  `ComposeAction`), `ComposerState.commitOnCompose`, `commitText`,
  `composeAndCommit`, `unreferenceRequested`, `freshBlockName`;
  `bdl_layout::Node::Definition`, `place_missing_with`; `with_slots` in the
  daemon's server.

## Deferred

A transport mark on a cross-domain read edge (the formula's `sync` is not a
socket); disconnecting one occurrence of a name a formula reads twice
(`unreference` takes every occurrence); read edges of a definition with open
positions (it does not elaborate, so `references` is empty until every position
is filled); a confirmation before a definition is detached from the canvas (the
inspector's _Detach definition_ is the one path); an optional
one-block-per-concept check (Phase 20's `ProducerUnique`) should a design want
it.

## Evidence

`crates/bdl-model/src/edit.rs`
(`a_sem_block_has_no_inputs_and_same_concept_inputs_get_parameter_names`),
`crates/bdl-elab/src/design.rs`
(`resolution_is_input_then_mapping_then_not_an_input_and_never_by_concept`),
`crates/bdl-compiler/tests/sem_blocks.rs`, `crates/bdl-ide/tests/sem_blocks.rs`,
`crates/bdl-daemon/tests/sem_blocks_e2e.rs`, `layout_e2e.rs`, `text_e2e.rs` and
`system_e2e.rs`, `crates/bdl-layout/tests/{place,arrange}.rs`;
`apps/studio/test/canvas_geometry_test.dart`, `sem_blocks_canvas_test.dart`,
`wiring_test.dart`, `canvas_selection_test.dart`, `canvas_affordance_test.dart`
(a read edge's × and its produce edge's absence), `sem_blocks_e2e_test.dart`
(the picture of the brief against the real `bdld`: the wire on a Source, the
fill of a slot, a rule applied twice, two Sem blocks of one concept raising
nothing), `docs_screenshots_test.dart` over the re-authored fixtures, and every
canvas suite over the Sem-block fixture.
