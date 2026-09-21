---
kind: architecture
area: ide
status: current
---

# IDE service architecture

BDL is one semantic language with two first-class authoring surfaces: visual
authoring in BDL Studio and textual authoring in ordinary editors. Both consume
**one** language service. This document describes that service — its crates, its
state model, its query model, and how each surface plugs in — and the rules that
keep the two surfaces from growing separate semantics.

```text
                    Project / Sources
                          │
                    committed state
                          │
                ┌─────────▼─────────┐
                │      IdeHost      │   bdl-ide-db
                │ ground state      │
                │  ProjectSnapshot  │
                │  OverlaySet       │
                │  documents        │
                │  in-flight reqs   │
                └─────────┬─────────┘
                          │  snapshot()
                ┌─────────▼─────────┐
                │ AnalysisSnapshot  │   immutable, stamped (revision, generation)
                │  effective design │
                │  ProjectAnalysis  │ ◀── bdl-compiler (semantic truth)
                │  ProjectionMap    │
                │  EntityIndex      │
                └─────────┬─────────┘
                          │
                ┌─────────▼─────────┐
                │      bdl-ide      │   diagnostics · hover · explain · completion
                │ semantic queries  │   references · rename · actions · edit plans
                └──────┬─────┬──────┘   invalidation preview · symbols · tokens · draft verdict
                       │     │
          ┌────────────┘     └────────────┐
   ┌──────▼──────┐                 ┌──────▼──────┐
   │   bdl-lsp   │                 │    bdld     │
   │ LSP adapter │                 │  protobuf   │
   └──────┬──────┘                 └──────┬──────┘
      text editors                     BDL Studio
```

## Why LSP is an adapter

LSP is a wire protocol for text editors. Its vocabulary — `Uri`, `Position`,
`Range`, `CompletionItem`, `CodeAction`, `WorkspaceEdit` — is the vocabulary of
_text_, and every one of those types is a projection of something BDL knows more
precisely: a byte range inside a document that projects an entity, an item that
names an input concept, an action that is a semantic edit plan. Defining the
language service in LSP terms would put text at the centre of a language whose
canvas surface has no text at all, and would force Studio either to speak LSP
(and translate node clicks into fake positions) or to grow a second semantic
layer.

So the internal API is BDL-owned (`bdl-ide`, `bdl-ide-db`), and `bdl-lsp` is a
thin translation at the edge: transport, position encoding, and type-to-type
rendering. It holds no semantic logic and no state the host does not already
hold. `bdld` is the same kind of edge for Studio, over protobuf. See ADR-0017.

## Crates

| crate             | owns                                                                                                                                                                                                    | never does                                                                                               |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `bdl-ide-db`      | mutable ground state (`IdeHost`), overlays, documents, entity refs and roles, projections (text and visual), the entity/reference index, immutable `AnalysisSnapshot`s, stamps, cancellation            | decide meaning; parse or type-check (it _calls_ `bdl-syntax` to bind text and `bdl-compiler` to analyse) |
| `bdl-ide`         | semantic queries over a snapshot: diagnostics, hover, explain, completion, references, rename planning, semantic actions, edit plans, invalidation preview, symbols, semantic tokens, the draft verdict | push results; know about LSP or Studio types; re-implement any compiler pass                             |
| `bdl-lsp`         | JSON-RPC over `lsp-server`, byte-offset ↔ position conversion for the negotiated encoding, rendering to `lsp-types`                                                                                     | keep its own buffer store; search text; compute meaning                                                  |
| `bdld` (existing) | the Studio session; now also the project's `IdeHost`                                                                                                                                                    | speak LSP                                                                                                |

Dependency direction stays strict:
`model → … → compiler → ide-db → ide → {lsp, daemon}`.

## Ground state: `IdeHost`

The host is the one mutable place. It owns:

- the **committed** `ProjectSnapshot` (the last revision `bdld` committed, or
  the project loaded from disk by `bdl-lsp`);
- the **`OverlaySet`** — uncommitted authoring state (below);
- the **document registry** (`DocumentUri ↔ DocumentId`; ids are never reused,
  so a closed-and-reopened document is a new document);
- the **in-flight request tracker** for cancellation;
- a cache of the snapshot for the current stamp.

Every mutation moves the stamp and cancels the requests it makes obsolete.
`set_committed` also prunes definition drafts that the commit made redundant
(the draft's text is now the committed definition — the commit succeeded) or
orphaned (the mapping is gone). Nothing else clears a draft behind the client's
back.

Threading: the host is `Send`; `bdld` owns it on its single coordinator loop,
`bdl-lsp` shares it behind a mutex. Queries never touch the host — they take an
`Arc<AnalysisSnapshot>` and run anywhere.

## Immutable snapshots: `AnalysisSnapshot`

`host.snapshot()` composes **committed + overlays** into the _effective_ design,
runs `bdl_compiler::analyze` over it (the only semantic authority), builds the
projection map and the entity index, and stamps the result with
`(revision, generation)`. The snapshot is logically immutable; the host builds a
new one when anything changes, and a query holding an old one keeps seeing the
world it started in.

Consequences that the design leans on:

- a completion result and a diagnostic result from one snapshot describe the
  same world, always;
- a query can never observe a half-applied overlay or a half-committed edit,
  because it never observes the host;
- the committed snapshot is available _inside_ the analysis snapshot
  (`snapshot.committed()`), so a client can ask "what would the commit change?"
  without a second host.

## Overlays

```text
committed ProjectSnapshot  +  OverlaySet  =  what analysis sees
```

One mechanism serves every surface:

| overlay                                      | key              | who sets it                                |
| -------------------------------------------- | ---------------- | ------------------------------------------ |
| `MappingDefinitionDraft { mapping, source }` | one per mapping  | Studio's definition editor, through `bdld` |
| `TextDocument { document, source }`          | one per document | `bdl-lsp` on `didOpen` / `didChange`       |

Overlays never create a revision. A text document overlay is bound to the model
by the textual projection (below); a definition draft is placed into the
effective design as if it were attached. Each overlay entry records the set
generation at which its content was set; the set's generation is monotonic over
every insert, update and removal.

Faults are results, not panics: a draft for a mapping that does not exist is an
`AppliedOverlay` with an `OverlayFault`, reported as a diagnostic.

## Entity references and roles

```rust
enum EntityRef { Project, Concept(ConceptId), Mapping(DeclId), Clock(ClockId),
                 Output(OutputId), Device(DeviceId), Requirement { device, index } }

enum EntityRole { Declaration, Name, Reference, Input { index }, Output, Signature,
                  Representation, Definition, DriveEdge, ClockBinding, DeviceBinding,
                  Requirement, Description }
```

Identity is the model's stable id — never a name, never a position. A role says
_which aspect_ of an entity something is about, so a diagnostic about a
contested sink lands on the drive edge, not on the mapping's name, on every
surface.

## Projections

A `ProjectionAnchor` places `(entity, role)` on one surface:

- `ProjectionId::Text { document }` + `TextRange` (UTF-8 bytes), built by
  **binding** a parsed document (`bdl-ide-db::textual`): each `concept` /
  `mapping` item is bound to the committed entity with that declared name — the
  identity-resolution pass for the textual surface — or to a fresh id allocated
  for the overlay. From then on everything speaks `EntityRef`. Anchors are
  recorded for names, signature inputs/outputs, definitions, and every name
  inside a body that the elaborator's own resolution rule maps to an input
  concept.
- `ProjectionId::Visual` + `VisualElementRef` (`MappingNode`, `InputPort`,
  `DriveEdge`, `OutputTerminal`, `DefinitionField`, …), derived from the model
  alone (`bdl-ide-db::visual`). No geometry: positions are layout, never
  semantics (ADR-0003).

`ProjectionMap` answers `entity → anchors`, `document → anchors by position`,
and `(document, offset) → innermost anchor`. It is the only way text positions
and canvas elements are ever related to meaning; there is no string search
anywhere in the service.

The inverse direction exists too: `render_module(design)` renders the model's
concepts and mappings as a canonical `.bdl` module, so the same `DeclId` can be
shown, navigated and renamed on both surfaces (this is the basis of the
shared-identity test).

## Semantic diagnostics

```rust
SemanticDiagnostic { code, severity: Error | Warning | Open,
                     primary: SemanticAnchor { entity, role, source? },
                     related: Vec<(SemanticAnchor, note)>,
                     message, explanation, technical, fixes, actions }
```

`bdl-ide::diagnostics` lifts the compiler's `Diagnostic`s once per snapshot. The
lift is the one place that knows which role a code is about (`output.*` →
`DriveEdge`, `clock.*` → `ClockBinding`, `formula.*` → `Definition`, …) and
which other entities are involved (the other claimants of a contested sink, the
members of a cycle, the concepts still without a representation).
`output.multiple_drivers` is emitted **once per sink**, whichever claimant the
compiler reported it on, with the other claimants and the sink as related
anchors.

The same diagnostic then projects two ways without any second logic:

- `project_to_document` → `TextDiagnostic { range, related[] }`, using the
  anchor's source span (a formula span is offset into the document that holds
  the body) or the projection map (`role`, then `Name`, then `Declaration`);
- `project_to_visual` → `VisualDiagnostic { highlights[] }`: the two edges and
  the terminal.

**Open is not an error.** The compiler's `Info` severity — an unresolved
mapping, an unbound concept, an undriven or clockless sink — becomes
`SemanticSeverity::Open`; the LSP adapter renders it as _information_ with the
_unnecessary_ tag (faded, not red); Studio renders it as the design's state. No
surface ever shows "missing body" as an error.

## Semantic actions and edit plans

```rust
SemanticAction { id, title, kind: QuickFix | Refactor,
                 applicability: Ready | NeedsChoice { options } | Blocked { reason },
                 plan: Option<SemanticEditPlan>, explanation, addresses[] }

SemanticEditPlan { title, operations: [ Model { EditOp } | Text { document, edits } | DraftText { mapping, edits } ],
                   affected_entities, invalidation: InvalidationPreview, preconditions: [Revision, Generation] }
```

Plans speak two vocabularies only: the model's own `EditOp`s (the only way a
project changes) and byte-range text edits for text not yet in the model. Studio
applies model operations through `bdld`; the LSP adapter turns text operations
into a `WorkspaceEdit`. Neither invents mutation logic.

Actions implemented now: detach a conflicting driver; create an upstream
combination mapping (declared, unresolved, not yet connected — a legal state);
move a mapping to the sink's domain or the sink to the mapping's; choose a
domain for an open sink; connect a driver to a required sink; choose a
representation; replace a misspelt formula name by a current input name; insert
an explicit sync (blocked, with the reason: no surface phrase yet, DI-3/DI-17);
add the value that applies a rule nothing applies (`rule.apply:<rule>`, for
`reactive.rule_unapplied`: `EditOp::CreateMapping` with a definition —
`<lowerCamel rule> : () -> <output> = Rule(arg, …)`, a fresh non-colliding name,
the domain the rule or its arguments fix — ready when every read concept has
exactly one producing value, one option per argument combination when a concept
has several, blocked when one has none or the arguments update in two domains;
`crates/bdl-ide/tests/rule_apply.rs`). **Meaning-changing fixes are never
applied implicitly**; `NeedsChoice` exists precisely so the tool does not guess.

Completion in a `concept …` position also offers the Standard Library's Concept
items (`CompletionKind::Template`, from `bdl-library` — the same data Studio's
Library tab shows), writing ordinary declarations such as
`AmbientLight : Illuminance`; see `docs/spec/concept-library.md`.

## Rename and references by identity

`references(EntityRef)` reads the entity index — the mappings whose signatures
mention a concept, the sinks that accept it, the formula bodies that read it
(resolved by `bdl_elab::names`, the elaborator's rule) — and places each
reference through the projection map. `plan_rename(entity, new_name)` produces
one model rename plus the text edits at every name and reference site in open
documents, plus, for a concept, rewrites of the committed formulas that read it
(ADR-0013 makes input names display names). A comment that says `Tilt`, or a
mapping named `tiltGuard`, is untouched because nothing compares spellings.

A bare `RenameConcept` is a refinement in the kernel's sense and yet, under
ADR-0013, would make a formula stop resolving. `preview_change` reports both
facts (`is_refinement()` and a `status_change` to `Invalid`); the rename _plan_
rewrites the formula so the designer never has to discover it.

## Invalidation preview

`preview_change(snapshot, &EditOp)` applies the edit to a copy of the effective
project, reads the model's own `EditOutcome` (kind, categories, origins — the
same `Invalidation` taxonomy `apply_edit` reports; there is no second one),
follows the dependency graph to the dependents that would re-validate,
re-analyses the candidate for status changes, and lists what is preserved —
identities always. The result is a product feature (Studio can show "changing
Tilt's representation reopens dimByTilt; nothing else") and the payload of every
action's plan.

## Cancellation and staleness

Two mechanisms, deliberately redundant:

- **Cancellation** is co-operative. `host.begin_request(scope)` hands out a
  `CancellationToken`; changing an overlay cancels every request scoped to it
  (and every project-scoped one); a new committed revision cancels everything.
  `snapshot_cancellable(&token)` polls between phases. `bdl_compiler::analyze`
  itself is one uninterruptible call today (it takes ~2 ms on a 400-mapping
  design, so it is not worth splitting yet).
- **Stamps** are the backstop. Every snapshot and every result carries
  `SnapshotStamp { revision, generation }`; `ResultGate` accepts only strictly
  newer stamps (`offer`) or only the current world (`offer_current`). Deliver
  generations `2, 1, 3` in that order and only `3` is ever the last thing shown.
  A query that never polled its token still cannot publish a stale answer.

## Studio integration

The Studio definition editor is the first consumer of overlays:

```text
type  ──▶  DefinitionDraftChanged (Studio state, generation++)
      ──▶  AnalyzeDefinitionDraft { revision, mapping_id, generation, source }   (bdld, protocol 0.4)
      ──▶  session.draft_verdict(mapping, source)
              = host.set_definition_draft(mapping, source)   (overlay)
              ; snapshot = host.snapshot()
              ; bdl_ide::draft_verdict(&snapshot, mapping)   (stamped verdict)
      ──▶  DefinitionDraftAnalysis { revision, mapping_id, generation, parse_ok, analysis }
      ──▶  Studio keeps it iff generation == draft.generation && revision == held revision
save  ──▶  ApplyEdit { AttachDefinition | ReplaceDefinition }   (one ordinary edit)
      ──▶  session.apply → host.set_committed → the redundant draft overlay is dropped
revert / reload / detach / text back to committed
      ──▶  DiscardDefinitionDraft { mapping_id }  →  host.clear_definition_draft
complete / hover in the field
      ──▶  CompleteDefinitionDraft | HoverDefinitionDraft { revision, mapping_id, source, offset }
              = same overlay + snapshot; bdl_ide::completion / entity_at_formula + hover
```

The project is untouched while the designer types; the compiler is the only
judge; the overlay clears when the commit lands (`set_committed` prunes a draft
equal to the committed text or orphaned by a deletion) or when Studio says the
draft is gone (`DiscardDefinitionDraft`), so a reverted draft never lingers as
the effective definition for a later LSP or hover query. Every draft query is
one request scoped to the mapping's overlay key
(`begin_request(CancelScope::Overlay)` → `snapshot_cancellable` →
`end_request`): setting the overlay cancels the previous request for the same
mapping; the coordinator is serial today, so the token never trips mid-flight,
and the client's generation check remains the correctness backstop. Studio keeps
presentation only — no parser, no type checker, no dimension logic in Dart.
`bdld`'s `RunAnalysis` and `AnalysisReady` also read the host's cached committed
analysis, so the committed verdict is computed once per revision.

`crates/bdl-ide/tests/surface_equivalence.rs` holds the two surfaces to one
verdict: the same formula as a `MappingDefinitionDraft` overlay and inside a
`TextDocument` overlay yields the same semantic diagnostics (code, severity,
entity, role) and the same ladder status.

Studio does not consume LSP. Its projection is visual; its transport is
protobuf; both read the same `bdl-ide` results as text editors do.

Highlighting is the same pattern with a text instead of a verdict
(`docs/architecture/syntax-highlighting.md`, ADR-0035): the Code view and the
definition editor send the text as typed —
`SemanticTokens { revision, generation, text, path | formula }` (protocol 0.21)
— the daemon sets it as the file's overlay on a text workspace over the sources
as written back (`SystemState::text_ide`), or as the relationship's draft
overlay in its scope, and answers `bdl_ide::semantic_tokens` / `formula_tokens`
with the legend; Studio keeps only the latest generation's answer, shifts the
spans while typing, and maps the legend's names to a theme. No class is decided
in Dart.

The Code view's completion, hover, navigation and formatting take the same path
(protocol 0.22): `SourceCompletion`, `SourceHover`, `SourceDefinition`,
`SourceReferences` and `FormatSource` carry the text as typed, the daemon sets
it as the file's overlay once per distinct text (`Session::source_snapshot`) and
answers `completion(Document)`, `hover_at`, `definition_at`, `references_at` and
`format_document` — the functions the language server calls for
`textDocument/completion`, `hover`, `definition`, `references` and `formatting`.
`bdl_ide::navigation` is the one answer to _what is at a position_: a name site
in the projection map (the authored entity among a site's — a port over the
flattened copies it backs — and a copy presented by its authored name), an
equation callee inside a formula body, and, for the formula field, the
elaborator's own input environment (`formula_name_at`), so the field's hover
names a relationship, an equation and a parameter's concept as the Code view
does.

### The canonical type of a relationship

Hover carries `type: () -> RoomTemp` / `Angle -> Brightness` /
`(Angle, Time) -> Speed` (`pretty::mapping_type`) beside the surface line that
keeps the declared shorthand; Explain adds `canonical type`, the empty-product
sentence for a relationship without inputs, and the kernel `interface`
(`() -> B` encoded as `B`; ADR-0029). Reference candidates, completion, the
output-driver action and the simulation-input readiness ask
`Signature::is_unit_domain` — the one predicate for "read as a value" — never a
separate kind of mapping.

### The relationship role (ADR-0032)

`bdl_ide::relationship_role(snapshot, decl)` answers what a relationship _is_
where the designer reads it: `Port(kind)` when the host says the declaration
backs a port (`IdeHost::set_port_backed`, fed by the daemon from the component's
interface; a text workspace reads it off the flattened origins), else `Source`
when it has no definition and the unit domain, else `Mapping`. Nothing stores
the role; a realised base relationship has a reference definition in the design
and is therefore a `Mapping`. Hover puts `role: Source` first and words the
missing definition as _none — provided by the environment, observed once per
activation_; Explain adds `role`, `provision: environment` and a `reading` line
naming `refForms_agree` (a Source is observed, not called) or
`resolved_not_source` (a resolved `() -> A` computes internally).
`RelationshipRole::word` is the English the daemon carries; Studio words its own
surfaces from the same rule (`relationshipRole` in `app/state.dart`), so the two
never disagree on a given projection.

### The Formula Composer (protocol 0.12, 0.13, 0.27)

`bdl-ide::formula` gives the Studio definition editor its structured projection,
on the same overlay path (docs/architecture/studio-ui.md §4b, ADR-0028):

```text
FormulaProjection      formula_projection(&snapshot, mapping)
                       = trace_formula_in (bdl-elab: the surface tree and the type of every
                         sub-expression, on success and on failure)
                       → FormulaNode tree: tree-path ids, byte ranges, kinds, actual types,
                         expected types by local dimension inference (solve), diagnostics
                         placed on the innermost node, the slots in source order
SlotInfo               formula_slot(&snapshot, mapping, node)
                       = the projection's expectation at `node` + units_for(dim) +
                         reference candidates by type + equations by scheme match and capability
ComposeResult          compose(&snapshot, mapping, source, ComposeOp)
                       = the text a structured action makes: byte-range edits of the draft,
                         parenthesised by precedence, a unit switched with the value kept
```

The projection is a _view_: it is rebuilt from the text on every request and
never stored; a slot is the surface's `?` (docs/spec/textual-syntax.md §16), so
an incomplete structured formula is ordinary draft text that does not check.
`bdld` carries the projection with every `DefinitionDraftAnalysis` (one round
trip per keystroke, the same generation) and answers `GetFormulaProjection` (the
committed definition, no overlay), `GetFormulaSlot` and `ComposeFormula` (both
set the overlay to the request's source, like completion). Local inference is
`Composer.lean`'s `solve`: `+ −` propagate the result to both sides, `×` gives
the unknown side `result − known`, `÷` the numerator `result + denominator` and
the denominator `numerator − result`; two unknown operands are _insufficient
information_, never searched. Candidate units are the registered atoms of the
solved dimension (`unitsFor`) followed by the curated composites of its named
quantity (`units::candidates_for`, ADR-0040); for a literal, of its own
dimension (a switch keeps the quantity — by atom id or by canonical spelling,
`180 deg per s` → `30 turn per min`). Tests:
`crates/bdl-ide/tests/formula_composer.rs`.

The natural forms (docs/spec/textual-syntax.md §17) reach the projection as
their own nodes, never desugared there:
`NodeKind::Binder { form, param, param_type }` with the collection and the body
as children, `NodeKind::Range` with the two ends, `??` as a `Compare` node. The
builder keeps a scope of binder locals so a `Reference` bound by an enclosing
binder carries `local: true` and no entity; `param_type` is the collection's
element (`TypeView.element`, from the list type or a concept's list
representation). `solve` reads a binder as the elaborator types it: the
collection expects a collection, the body of `all`/`any`/`filter` expects
`true`/`false`, the body of `map` expects one element of what the position
expects; `x in lo .. hi` passes the subject's actual kind — nominal when it is a
concept value — to both ends. Two actions: `ComposeOp::Binder { node, form }`
makes `form local in node: ?` with a fresh local (`fresh_local`: the
collection's name without its plural `s`, lowercased, when that is free —
`readings` → `reading`, `Angles` → `angle` — else `item`, `item2`, …; never a
name of the design, a local of the formula, a word of the language or an
equation), `ComposeOp::Range { node }` makes `node in ? .. ?`; grouping treats a
binder as weaker than every operator (an operand only in parentheses) and its
collection position as the range level (a comparison, a range or another binder
there is parenthesised), `..` between `in` and `??`, `??` between `..` and
`+ −`. Completion offers the locals in scope at the point
(`CompletionKind::Local`: a binder's element inside its body, a rule's
parameters, a pattern's names) above the design's names, and the four binder
templates only when a collection is in sight (an input or a nullary relationship
of a list kind), with the one collection filled in when there is exactly one.
The legacy output-only shorthand `mapping f : A` is `text.legacy_unit_domain` —
`SemanticSeverity::Hint`, the one severity that is neither wrong nor unfinished
(LSP `Hint` tagged `Deprecated`; the daemon's formula adapter maps it to `Info`)
— raised from `bdl_syntax::migrate::legacy_unit_domain_signatures` over the
document (a `mapping`'s bare signature, nothing else), anchored to the bound
mapping with `EntityRole::Signature`; `actions_for` answers it with _Make empty
domain explicit_, a `SemanticOperation::Text` inserting `() ->`.
`hover::declared_spelling` reads the anchored signature text so hover and
Explain can show the declared spelling beside the canonical type. The legacy
drive spelling `drive o = m` (preferred: `drive o by m`) is `text.legacy_drive`
on the same pattern — `bdl_syntax::migrate::legacy_drive_decls`, anchored to the
driving mapping with `EntityRole::DriveEdge` at the `=` token; `actions_for`
answers it with _Write `by`_, one `TextEdit` replacing that token. The Studio
Code view (fed by `bdl-text` load faults, not this service) shows no hint today.

References, rename and semantic tokens never take a local for an entity:
`ast::NameExpr::local_binding` (the nearest enclosing binder body, rule, match
arm or earlier `let` that binds the spelling) is consulted by the index, the
textual anchors and the token classifier — the binder word is a keyword only in
the head position, the local a parameter (declaration and uses), `..` an
operator.

**The Formula view's structure (0.27).** The projection carries, on every node,
what a client needs to render and navigate without reparsing: `role` — what the
node is to its parent (`condition` / `then` / `else`, `numerator` /
`denominator`, `argument 1`, `collection` / `body`, `subject` / `arm 1`, `value`
/ `result`, `item 1`, `initial` / `value`, `domain`, `from` / `to`); `locals` —
the formula's own names in scope there (a binder's or rule's parameters, an
arm's pattern names, the `let`s before), computed while building; `append_at` —
the byte before a call's, list's, tuple's or match's closing delimiter. The
forms that were opaque are structured: `NodeKind::Match` (the subject, then
`Arm { pattern, binds }` nodes over their bodies), `Block`
(`Let { pattern, binds }` nodes over their values, then the result),
`Rule { params }`, `List`, `Tuple`, `Delay` (initial, value), `Sync` (domain,
initial, value); `Opaque` remains for `()`. `solve` propagates the position's
expectation to every arm's body, a block's result, a delay's and a sync's
values, a list's items (its element). A composite unit literal (ADR-0040) is a
`Quantity` node with the unit as written, its canonical spelling (`unit_source`)
and its rendering (`unit_display`); `unit_id` names the atom only when the unit
is one.

A **structural caret** is a `(node, Side)` pair — `Before` or `After` a node;
both sides of a slot are the slot itself. `caret_offset` gives its byte offset;
`navigate(root, node, side, Motion)` is pure over the tree: `Left` / `Right`
walk the caret sequence in source order (before a node, its children's carets,
after it), `Up` keeps the side on the parent, `Down` takes the first child's
`Before` (or the last child's `After`), `Exit` is `After` the parent, `NextSlot`
/ `PreviousSlot` wrap. The daemon keeps no caret and no second tree: every
request names the draft source and a caret in its projection.

**Keyboard insertion** is `ComposeOp::Insert { node, side, text }`, interpreted
by the grammar and answered with a text edit like every other action: a
two-sided operator makes `node op ?` (or `? op node` before), `!` the prefix
form, `(` groups the node (or a slot), a word into a slot becomes its canonical
form (`canonical_fill`: an equation with one slot per argument, a relationship
with inputs as a call with slots, `if ? then ? else ?`, a binder with a fresh
local, `delay(?, ?)`), a number or `true` / `false` fills as written; after a
quantity literal a unit word becomes its unit, `per` and `*` extend the unit
(`180 deg per` — a draft state the parser names until a unit follows; `*` gives
`180 deg * ?`, whose slot filled with an atom the parser reads as one composite
literal) and `^` opens a power. What cannot stand at the caret is refused with
the reason (`insert an operator first`). The Formula view and the text field
edit one draft; neither side has a grammar of its own.

**Completion at a caret**
(`CompletionContext::FormulaCaret { mapping, node, side, prefix }`) runs the
formula completion at the caret's offset with the projection's expectation for
the position and its `locals` — the text is not scanned for either; the prefix
is the client's. Every item carries `structured_insert` where its Formula-view
form differs from the text insert (`clamp(?, ?, ?)` beside `clamp(`; `spin(?)`;
`if ? then ? else ?`; a binder template with a slot; a composite unit's whole
spelling), so a client never synthesises slots. Units: after a number every atom
is offered with the ones of the expected dimension first, and the curated
composites of the expected dimension by their spelling (`10 d` → `deg`,
`deg per s` for an angular velocity); after `per` or `*` of a unit being
written, the atoms are ranked by the dimension they would complete the unit to
(`10 deg per` → `s` first).

**Signature help** (`signature(projection, design, node)`) finds the innermost
call enclosing a node: an equation's parameter names with what each takes (the
scheme instantiated by the arguments already written, from the projection's
expectations), a relationship's inputs by concept and its result, and the
argument the caret is in.

**The render** (`render(projection)`) flattens the tree into fragments in source
order — a leaf's own text as `reference` / `local` / `number` / `unit` (its text
the rendering, `m/s²`) / `bool` / `slot`, the words and marks between a node's
children as `operator` / `keyword` / `punctuation` / `pattern`, a call's head as
`reference` or `equation` — with the compact one-line source, the result's
description, the diagnostic counts and the references; the daemon serves it for
the **committed** formula (`IdeHost::committed_snapshot`: the world with no
overlay, so a canvas node shows the saved formula while the inspector drafts
another). No second rendering grammar exists. Tests:
`crates/bdl-ide/tests/formula_structure.rs`.

### Component-scoped drafts (system projects)

A component's body is an ordinary design, so the same service serves it — in the
body's own scope. The daemon keeps one `IdeHost` per component on demand
(`SystemState::component_ide`, created from `ProjectSnapshot { revision, body }`
at first use and re-seated with the body on every commit; a host whose component
is gone is dropped). The four draft requests carry an optional `component`;
`Session::ide_in(scope)` routes to that host, and completion, hover and verdicts
inside a component's source therefore see the body's names (`dimByTilt`, not
`lampA.dimByTilt`) and the body's standalone analysis (required ports open).
Studio stashes its drafts per context (`draftKey(root, context)`) so switching
between the system and a component's source loses nothing typed. Entity hover
and semantic actions stay flat-entity services: not offered inside a component's
source (DI-40). Group edits do not touch any host: they are not commits (DI-38),
and `ApplyGroupEdit` reads the host's cached committed analysis for the
boundaries it answers with.

## LSP integration

`bdl-lsp` (`lsp-server` + `lsp-types` 0.97, LSP 3.17 baseline):

| LSP                                    | `bdl-ide`                                                                                                                                                                                                                                                                           |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `initialize`                           | negotiate position encoding (UTF-8 if offered, else UTF-16, else UTF-32); load the project at `initializationOptions.projectRoot` / the first workspace folder if it holds `bdl.toml`, else start empty                                                                             |
| `didOpen` / `didChange` / `didClose`   | `host.set_text_document` / `close_text_document` (overlay)                                                                                                                                                                                                                          |
| `textDocument/hover`                   | `hover_at` (an entity's card, or an equation's words, with its range)                                                                                                                                                                                                               |
| `textDocument/definition`              | `definition_at` (name sites, in every document of the workspace)                                                                                                                                                                                                                    |
| `textDocument/references`              | `references_at` (reference sites, the declaration when asked)                                                                                                                                                                                                                       |
| `textDocument/prepareRename`, `rename` | `plan_rename` → `WorkspaceEdit` (text operations; model operations are reported through `bdl/previewEdit`)                                                                                                                                                                          |
| `textDocument/completion`              | `completion(Document { offset })` — inputs, relationships, units, keywords, and the equation library (`CompletionKind::Equation`, documented in the designer's words; a relationship of the design with the same name wins)                                                         |
| `textDocument/diagnostic` (pull)       | `diagnostics(Document)` → `project_to_document`                                                                                                                                                                                                                                     |
| `textDocument/documentSymbol`          | `document_symbols`                                                                                                                                                                                                                                                                  |
| `textDocument/semanticTokens/full`     | `semantic_tokens` → `tokens::encode` in the negotiated position encoding; the legend is `bdl_ide::legend()` — LSP standard types plus `unit` / `slot`, modifiers `declaration`, `defaultLibrary`, `source`, `output`, `device`, `instance`, `unresolved` (`syntax-highlighting.md`) |
| `textDocument/codeAction`              | `actions_for` on the diagnostics in range, `actions_at` on the entity                                                                                                                                                                                                               |
| `$/cancelRequest`                      | `host.cancel_request`                                                                                                                                                                                                                                                               |
| `bdl/explainEntity`                    | `explain` (+ a Markdown rendering)                                                                                                                                                                                                                                                  |
| `bdl/invalidationPreview`              | `preview_change`                                                                                                                                                                                                                                                                    |
| `bdl/previewEdit`                      | `plan_rename` as a full `SemanticEditPlan`                                                                                                                                                                                                                                          |

Pull diagnostics are the model. Push (`publishDiagnostics`) exists only for
clients that do not advertise `textDocument.diagnostic`, is confined to the
notification path, and computes the same placed diagnostics.

Position encoding lives in one type (`bdl_lsp::position::LineIndex`); nothing
else in the server does the arithmetic. Tests round-trip every character
boundary of a document with combining marks, an astral-plane character and CJK
text in all three encodings.

Requests run on worker threads over one snapshot each; the host is locked only
to take the snapshot (which runs the compiler when the cached stamp is stale)
and to record cancellation.

## Text workspaces

Every project is a text workspace (ADR-0023; the mechanism is ADR-0020's): the
language server opens a project directory as the host's _ground_
(`bdl-ide-db::workspace::TextGround`): the sources under `src/**/*.bdl` read in
path order and the identity table from `.bdl/identities.json`. Composing a
snapshot substitutes open buffers for files, builds the system with
`bdl_text::load_workspace` — the same loader `bdld` and the CLI use — flattens
it, and anchors every entity on the authored source: a component body's
declaration maps to its per-instance flat ids, a shared concept to the system's,
so diagnostics, navigation and rename land on the text a person wrote. Names
inside formula bodies are resolved through the elaborator's own environment
(plain or pinned scope) to concepts and mapping calls; textual parameter names
are lexical and are not references to the concept.

Edits from Studio or semantic actions go back through `bdl-text`'s item-level
splice: only a changed item is re-rendered, comments and trivia outside it stay.
Completion reads scope from the authored system (a component body offers the
body's names). The adapter reloads the ground on save and on watched-file
changes and writes the reconciled identity table back. User-defined enums parse
and are reported open (ISS-0005).

## Virtual documents

`DocumentUri::kind()` distinguishes authored documents from generated ones
(`bdl-core://`, `bdl-explain://`, `bdl-generated://`). `bdl_ide::virtual_docs`
renders three read-only documents — the explanation of an entity or of the
project as Markdown, the kernel Core of the design, and the generated Rust core
(or why nothing is generated yet) — and `bdl-lsp` serves them through
`bdl/virtualDocument`. Generated text is never a source of truth and is never
edited.

## Semantic and presentation operations

Semantic (source) operations and presentation (layout) operations are kept
apart, as before (ADR-0003):

| semantic — an `EditOp`, a revision, an invalidation                                                                 | presentation — never a revision                                            |
| ------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| connect a concept to a mapping input · set a representation · attach a definition · bind an output · assign a clock | move a node · resize a panel · change zoom · collapse an inspector section |

GLSP's idea worth keeping is structure-aware operations over a shared source
model: our `SemanticEditPlan` is that operation model, with the model's
`EditOp`s as the vocabulary. GLSP itself is not adopted (the product stack is
Flutter + `bdld`).

## Performance baseline

`cargo run --release -p bdl-ide --example perf_baseline` (best of N, one core,
2026-09-15, Apple silicon; the token and document-query rows measured
2026-09-20):

| query                                      | small (3/3/1) | medium (60/80/10) | large (300/400/40) |
| ------------------------------------------ | ------------- | ----------------- | ------------------ |
| committed snapshot (compile + index)       | 0.05 ms       | 0.75 ms           | 2.2 ms             |
| formula overlay → snapshot + draft verdict | 0.05 ms       | 0.73 ms           | 2.1 ms             |
| diagnostics (project)                      | <0.01 ms      | 0.03 ms           | 0.11 ms            |
| completion (formula)                       | <0.01 ms      | <0.01 ms          | <0.01 ms           |
| hover / references / rename plan           | <0.01 ms      | <0.01 ms          | <0.01 ms           |
| text overlay → snapshot (bind + compile)   | 0.09 ms       | 0.80 ms           | 4.7 ms             |
| semantic tokens (whole document)           | 0.03 ms       | 0.50 ms           | 1.5 ms             |
| semantic tokens → LSP data (UTF-16)        | <0.01 ms      | 0.07 ms           | 0.21 ms            |
| formula tokens (one draft)                 | 0.01 ms       | 0.01 ms           | 0.01 ms            |
| completion (document, formula body)        | 0.06 ms       | 0.07 ms           | 0.22 ms            |
| hover (document)                           | 0.02 ms       | 0.14 ms           | 0.63 ms            |
| definition / references (document)         | <0.01 ms      | <0.01 ms          | <0.01 ms           |
| format (document)                          | 0.03 ms       | 0.28 ms           | 1.0 ms             |

(concepts/mappings/outputs). Every keystroke recomputes the whole project; at
these sizes that is well under a frame. Debug builds are roughly an order of
magnitude slower, still within a keystroke.

## Where recomputation is coarse, and the future path

Today one snapshot = one full `analyze` + one full index + one full projection
map. There is no memoisation below the snapshot. The data above says this is
fine for the foreseeable project sizes; the API is shaped so that finer
recomputation can be added without changing any query:

- inputs are explicit (committed snapshot, overlay set, document set);
- snapshots are immutable and queries are pure functions of one snapshot;
- the invalidation domains an edit reports
  (`Interface | Realization | Semantic | Reactive | Clock | Output | Deployment`)
  are the same categories an incremental engine would key on — there is no
  second taxonomy to reconcile.

When profiling on real projects says otherwise, the options are a
dependency-keyed cache inside `AnalysisSnapshot::compose` (per-mapping
elaboration keyed by `(signature, definition, concept representations)`), a
custom memoisation layer over the compiler passes, or Salsa. None is implemented
now; see `docs/background/ide-service-research.md` for the precedents.

## Tests

| scenario                                                                                                                                           | test                                                                                                                     |
| -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| unresolved mapping: no error anywhere; hover/references/rename/definition/explain work                                                             | `bdl-ide/tests/acceptance.rs::unresolved_mapping_is_legal_and_fully_queryable`                                           |
| the same `DeclId` from a text position and from a canvas node is one `EntityRef`; the rename plan is identical from either                         | `…::the_same_mapping_is_one_entity_on_both_surfaces_and_rename_hits_both`                                                |
| two drivers for one sink: one semantic diagnostic, projected to text (with related info) and to two edges + a terminal; explicit actions           | `…::one_conflict_diagnostic_projects_to_text_and_canvas`                                                                 |
| formula draft: committed stays unresolved, overlay is judged, completion sees it, one edit commits, the overlay clears only after the commit lands | `…::formula_draft_is_an_overlay_until_the_commit_lands`, `bdld session::drafts_are_overlays_over_the_committed_revision` |
| unsaved document: disk model unchanged, analysis sees the overlay, close reverts                                                                   | `…::unsaved_document_is_an_overlay_and_close_reverts`                                                                    |
| generations delivered 2, 1, 3 → only 3 visible                                                                                                     | `…::out_of_order_verdicts_cannot_overwrite_the_newest`, `bdl-ide-db stamp::tests`                                        |
| a long-running query is cancelled by an overlay change and cannot publish                                                                          | `…::changing_the_overlay_cancels_the_running_query_and_its_result_is_rejected`                                           |
| byte ↔ LSP position round-trips in UTF-8/16/32 on non-ASCII text                                                                                   | `bdl-lsp position::tests`                                                                                                |
| rename by identity ignores comments and look-alike names                                                                                           | `…::rename_follows_identity_not_spelling`, `bdl-lsp/tests/e2e.rs`                                                        |
| the LSP adapter end to end over an in-memory connection                                                                                            | `bdl-lsp/tests/e2e.rs`                                                                                                   |
| no query panics on missing entities, malformed overlays, garbage documents                                                                         | `…::queries_never_panic_on_missing_or_malformed_input`                                                                   |
