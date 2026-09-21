---
id: ADR-0044
status: accepted
date: 2026-09-21
area: studio
supersedes: [ADR-0034]
superseded-by: []
related:
  [
    ADR-0001,
    ADR-0003,
    ADR-0013,
    ADR-0023,
    ADR-0028,
    ADR-0032,
    ADR-0041,
    ADR-0042,
    ADR-0043,
    ISS-0020,
  ]
fv:
  [
    BDL/Surface/Sem.lean,
    BDL/Experiments/SemExamples.lean,
    FVD-0163,
    docs/notes/the-sem-block-model.md,
  ]
---

# ADR-0044: The canvas draws Sem blocks and mapping blocks; a concept is the template Sem blocks are created from; a rule is the template mapping blocks apply

## Status

Accepted (the Sem-block model, after BDL_FV Phase 21, FVD-0163 at `10a1a7c`; the
vocabulary is ADR-0043's). Supersedes ADR-0034, which was **right on the same
evidence about what an edge is** — dependency read off the analysis, never a
client-side parse — and **wrong about what a node is**: it drew a concept as a
node with an input socket that received every producer, read a relationship's
produce edge as an edge of the value graph, and let a drag edit a signature.
Nothing ADR-0034 said about `MappingAnalysis.references` changes; what it drew
from them does. ADR-0032's Source marks stand. ADR-0013's naming rule stands,
with the amendment recorded there.

Strength: **informed by FV**. FVD-0163 and Phase 21's theorems
(`producedBy_unique`, `producedBy_refine`, `reads_iff_dependsOn`,
`new_sem_transparent`, `sem_value_det`; the executed designs `lamp_picture`,
`rule_template`, `sensors_natural`, `judgment_optional`) are restatements of
Phases 0–1, 5 and 13 about the _formal model_; they bound this projection and
prove nothing about Studio.

## Context

The canvas at `4e87d4f` drew three kinds of node — a concept row, a relationship
box, a sink — and three kinds of edge: a signature edge from a concept's output
socket into each input socket of a relationship reading it, a produce edge from
a relationship's output socket into its concept's input socket, and a reference
edge from a named relationship into the formula line of the relationship naming
it (ADR-0034). Dragging a concept into a relationship edited the relationship's
**signature** (`LinkConceptToMappingInput`, one more parameter), and the canvas
resolved a concept dropped on a sink against "the relationships that produce the
concept".

The kernel has no such object. `Ty.sem C` is a nominal type; the value is the
declaration's (`DesignDecl`, one value per tick, `sem_value_det`); its
realization is write-once and is its one producer (`producedBy_unique`); a
formula names other declarations by `declRef` and nothing is resolved by concept
(`reads_iff_dependsOn`; Phase 19 §19.1, Phase 20 §20.4). Several declarations of
one concept are ordinary (`sensors_natural`), and a rule — an arrow-typed
declaration — is applied as many times as there are values to produce
(`rule_template`). The concept-as-node projection therefore drew an object with
a value the kernel does not have, and drawing it led to the question Phase 20
answered with a global invariant and Phase 21 withdrew: whether a concept has
one producer (ISS-0020).

In production the shipped demo already had the kernel's shape — `lit : Lit` with
`lit() = pressed` names the Source by name — but one drag on the canvas
(`Pressed` onto `lit`) turned it into a rule `lit : Pressed -> Lit` whose
parameter its formula never used. And a rule reading one concept twice
(`f : (Tilt, Tilt) -> C`) authored in Studio had no way to name its second
input: `InputEnv::resolve` took the concept's name for the first input on the
exact spelling and reported the loose spelling as ambiguous.

Bounds: Studio computes no judgment (ADR-0001) — every edge it draws is the
analysis's `references` or the authored signature and drive; every structured
action is a text edit or a compiler action (ADR-0028, ADR-0042); the kernel, the
elaborator's typing and the textual form are unchanged; old projects load
unchanged.

## Decision

1. **Two node kinds, no concept node, no rule node.** The canvas draws the
   **value graph**: a **Sem block** per unit-domain relationship — a declaration
   of a concept holding one value per tick; a Source when it has no definition
   (ADR-0032's marks kept) — a two-row node: the name, then its concept at the
   output socket; and, for each Sem block with a definition of its own, its
   **mapping block** as a node of its own (`NodeKind.definition`, keyed by the
   Sem block's id — one declaration, two nodes, two positions): a header naming
   the rules the definition applies (`NodeShape.applies`; the word _Formula_
   when none), one input socket per Sem block the definition names
   (`SocketRole.read`, typed by the read block's concept, labelled with its
   name), one hollow socket per open position of the definition
   (`SocketRole.slot`, from `MappingAnalysis.slots`), an output socket, and the
   formula line with its disclosure. A **concept** is the template a Sem block
   is created from — its name, value form and hue, in the sidebar and on every
   socket that carries it; the Concept sheet (ADR-0041) creates the template,
   _Add Block ▸ of C_ instantiates it, as often as the product has such values.
   A **rule** — a relationship with inputs — is the template a mapping block
   applies: in the sidebar with its own inspector, where its parameters over
   concepts are edited (ADR-0013), never a node of the value graph.

2. **Edges.** A **read edge** from each Sem block a definition names into the
   mapping block's read socket for it — `MappingAnalysis.references` filtered to
   unit-domain declarations, the kernel's `dependsOn` (`reads_iff_dependsOn`) —
   in the read block's hue. A **produce edge** from a mapping block's output
   socket into its Sem block's produce socket (`SocketRole.produce`): the
   definition itself, functional and write-once (`producedBy_unique`,
   `producedBy_refine`), drawn as a short joint the layout keeps beside the
   block and never rerouted — it starts no link and takes none. The **drive
   edge** from a Sem block into the sink it drives, as authored. Bindings and a
   collapsed group's aggregate edges as before, over Sem blocks only (a realised
   base relationship of a system has no mapping block: the binding's edge into
   its filled realisation socket is its definition). Reference edges into a
   formula line and concept → mapping signature edges are gone.

3. **Every gesture edits text or the drive; none edits a signature.** Dragging a
   Sem block onto a Source gives it the block's name as its definition
   (`AttachDefinition`); onto a hollow slot socket, onto a mapping block with an
   open position, or onto a Sem block whose mapping block has one, fills the
   compiler's first slot (`ComposeAction.fill` on the node id
   `MappingAnalysis.slots` names, then `ReplaceDefinition` with the compiler's
   text) — `WireSemBlockRequested`, one edit, never a draft the designer is
   still typing and never a client-side parse. Dragging a Sem block onto a sink
   accepting its concept is the drive (`SetMappingDrive`). A read edge is a name
   in the formula: its _Disconnect_ (the ×, ⌫, the drag-away) is the compiler's
   own text edit — `ComposeAction.unreference`, every occurrence of the name
   becomes a slot — committed as one `ReplaceDefinition` (`LinkKind.read`). The
   drive edge disconnects by `SetMappingDrive` to none. The produce edge
   disconnects with nothing: the definition goes only through the inspector's
   _Detach definition_. `LinkConceptToMappingInput` /
   `LinkMappingOutputToConcept` / `UnlinkMappingInput` remain the rule
   template's parameter editor in the inspector; the canvas no longer dispatches
   them. The Concept → Output authoring gesture has no socket to start from and
   is withdrawn; the sink's inspector keeps `driveCandidates`. Selecting a
   mapping block selects its Sem block (`asDeclaration`); the two nodes have two
   positions — the mapping block is placed beside the Sem block when it appears,
   and from then on each moves alone.

4. **Parameters are named where a concept repeats** (amends ADR-0013). A rule
   reading one concept twice gets a name per repeated input from the model
   (`derived_parameters` in `bdl-model::edit`: the concept's name with a
   lower-case initial and its ordinal among the repeats, `tilt1`, `tilt2`; every
   other input keeps the empty entry that falls back to its concept's name;
   names the text gave are kept while they still cover the signature) on
   `CreateMapping` and `SetMappingSignature`, so that every input is reachable
   by a name of its own. A Sem block has no inputs and no parameters. Name
   resolution is unchanged: input, then a relationship of the design, then a
   concept that is not an input; never a value by concept — a formula spelling
   the repeated concept's name is told the names (`formula.name.ambiguous`).

5. **No diagnostic for two Sem blocks of one concept.** Phase 20's
   `ProducerUnique` is an optional judgment of the formal development
   (`Validation/Producer.lean`); production has none and adds none. Kept:
   unknown name, not-an-input, ambiguous input, two drivers of one output,
   nominal mismatch.

6. **Protocol 0.30**, additive: `MappingAnalysis.slots` — the `?` positions of
   the committed definition as the projection names them, filled by the daemon
   from `formula_projection` of the committed snapshot; `Layout.definitions` —
   the mapping blocks' positions, keyed by the Sem block's declaration id;
   `ComposeAction.unreference` and `ComposeAction.read` — the compiler's text
   edits behind a read edge's disconnect and a Sem block's insertion into a
   definition's first slot.

7. **Layout and demos.** The layout service (ADR-0023 §7) keys Sem blocks
   (`Node::Mapping`), mapping blocks (`Node::Definition`), sinks and instances;
   a concept and a rule are never placed, their stored positions kept as written
   and drawn by nothing. A mapping block without a position is attached directly
   left of its Sem block, centred on it, on open and on commit
   (`place_missing_with`); the whole-graph arrangement ranks mapping blocks
   before the Sem blocks they define, over read, drive and binding edges. The
   inspector on a concept lists its blocks and the rules over it; on a Sem block
   _Reads_ / _Applies_ / _Read by_; on a rule _Depends on_ / _Named in_ and the
   parameter editor. The Simulate probe on a concept is per Sem block.

## Alternatives

- **Keep the concept node and add a producer marker or a global invariant**
  (Phase 20's reading): needs an invariant the kernel does not have and refuses
  the natural form of redundancy; FVD-0163 withdrew the requirement.
- **A structured application in the surface**
  (`Definition::Apply { rule, args }`) so the canvas could draw wires without
  parsing: unnecessary — the analysis already reports what a definition names
  (`references`) and where its holes are (`slots`), and `ComposeAction.fill` /
  `read` / `unreference` are the compiler's own text edits; a second definition
  form would be a second authored text (ADR-0028). Candidate (a) of the brief,
  chosen.
- **The mapping block as the Sem block's body, one node and one position**: the
  first cut of this change; rejected in favour of the picture the brief and the
  formal model draw — a mapping block with read sockets, a produce edge into the
  Sem block — which the layout service (`Layout.definitions`) already keyed. An
  old layout gains the positions on open; no migration.
- **Keep rules on the canvas as nodes without value edges**: a node the value
  graph never joins, inviting the signature drag back; the sidebar and the
  mapping block's header carry the fact.

## Consequences

- `docs/architecture/studio-ui.md` §2 (anatomy, gestures, menus) and §7 (the
  channel table) describe the value graph; `docs/architecture/overview.md` the
  arrangement; `docs/spec/protocol.md` 0.30; `docs/project/status.md` the Layout
  service and Studio rows; `docs/project/formal-correspondence.md` the FVD-0163
  row; ISS-0020 resolved by this record; the change fragment
  `docs/changes/unreleased/2026-09-sem-blocks.md`; the user guide's canvas,
  inspector, simulate and tutorial pages and their screenshots.
- Harder: a definition with an open position elaborates to nothing, so its read
  edges appear only once every position is filled (`references` is the kernel's
  dependency, never a parse) — recorded in the fragment as a limitation.
- Not done here: transport marks on cross-domain read edges (the formula's
  `sync` is not a socket); disconnecting one occurrence of a name read twice
  (`unreference` takes every occurrence); a confirmation on the produce edge
  (the inspector's _Detach definition_ is the one path).
