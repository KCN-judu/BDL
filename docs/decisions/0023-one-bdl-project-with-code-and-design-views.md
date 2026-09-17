---
id: ADR-0023
status: accepted
date: 2026-09-17
area: persistence
supersedes: [ADR-0020]
superseded-by: []
related: [ADR-0003, ADR-0008, ADR-0014, ADR-0017, ADR-0019]
fv: []
---
# ADR-0023: There is one BDL project; Design, Code and Split are views of it

## Status

Accepted (unified-project milestone). Supersedes ADR-0020, which was
right until the circumstances changed: it made text a real authored
surface with a stable-identity sidecar, but it did so by adding a third
project *kind* (`text`) beside the JSON kinds and an explicit conversion
between them. Every mechanism ADR-0020 introduced — the loader, the
source-identity table, reconciliation, item-level write-back, the
authoring sidecar — is kept unchanged and is named below where it is
relied on. What is withdrawn is the choice that a project *is* text *or*
JSON.

## Context

A designer opened "a Studio project" or "a text project" and had to
convert to move between them. Internally the product already had the
right shape — one semantic model (`bdl-model`, `bdl-system`), stable
identities (ADR-0008), layout apart from semantics (ADR-0003), the LSP as
an adapter over the language-owned IDE service (ADR-0017), and text as a
canonical source with an identity sidecar (ADR-0020). The split survived
only in persistence and in the project UX: the manifest's `kind`, three
*New Project* commands, `PROJECT_KIND_TEXT` in the projection, and the
idea of converting. That split is a product statement the language does
not make, and two persistence forms for one meaning are two authorities
that drift.

The constraint that bounds every alternative is identity: layout, hues,
bindings, contracts, provenance and references depend on stable integer
ids; text carries only names. Whatever is canonical must keep identity
across renames, moves between files, formatting, and reopening.

## Decision

### 1. One project, one canonical semantic source

A BDL project is one directory:

```text
project/
├── bdl.toml                 name, schema, compiler version — no kind
├── src/**/*.bdl             the authored design and system — the semantic source
├── .bdl/identities.json     source key → stable id, allocators, flat ids — tool-owned
├── .bdl/authoring.json      behavior groups (authoring metadata, ADR-0019) — tool-owned
└── ui/layout.json           canvas positions, viewports, group boxes — presentation
```

The sources plus the identity sidecar are the semantic truth of *every*
project. There is no other authored semantic file. The manifest has no
`kind`; a manifest that still carries one names a legacy project (§6).

### 2. Which information lives where

| Kind of fact | Lives in | Never in |
|---|---|---|
| **Semantic**: concepts, declarations and their signatures, realizations (formulas), clocks, outputs, drive edges, devices, components, ports and contracts, instances, bindings, exports | `src/**/*.bdl` | layout, authoring sidecar |
| **Identity**: the stable id behind each semantic entity; id allocators; the flat-id freshening table | `.bdl/identities.json` (keyed by kind + qualified name, ADR-0020 §3) | source text (raw ids never appear in normal source) |
| **Lexical**: comments, whitespace, formatting, the order of items where meaning does not depend on it, the partition into files | `src/**/*.bdl` only | the model; the identity table (keys are not positions) |
| **Authoring metadata**: behavior groups and their membership | `.bdl/authoring.json` by identity | source; layout |
| **Presentation**: node positions, per-canvas viewports and zoom, group boxes and collapse state, component-body and system canvases | `ui/layout.json` by identity | source; the semantic revision |

"Source of truth" is therefore a per-fact statement, not a slogan: the
semantic model never depends on geometry; the graph never stores a
semantic fact; the text never stores a position. Erasing both sidecars
and the layout changes no semantic fact; erasing the sources loses the
design.

### 3. Design, Code and Split are views, not kinds

Studio shows the same open project as a graphical behavior model
(*Design*), as its source files (*Code*), or both side by side (*Split*).
Switching is a view change: no conversion, import, export, or project
type. The CLI, the language server and external editors open the same
directory through the same loader (`bdl-text::load_workspace`) and see
the same identities.

An entity authored first in Code and one authored first in Design are
indistinguishable once synchronised; there are no textual and graphical
entities as categories.

### 4. Edits flow through the semantic model

*Text → graph*: a text change is parsed losslessly, its declarations are
bound to identities by reconciliation against the working identity table
(ADR-0020 §4 — retained, allocated, dropped, renamed, ambiguous,
duplicate — unchanged), the semantic project is updated as a new
revision, and the graph projection follows. Nodes keep identity, layout,
references and selection.

*Graph → text*: a graph operation is a semantic edit (`EditOp`,
`SystemEditOp`) on the model; the textual projection is then a minimal
item-level splice of the sources (ADR-0020 §5, unchanged): a renamed
declaration's name and references, one inserted declaration, one replaced
formula body. The whole workspace is not reprinted, comments and
formatting outside the touched item survive.

Layout edits — moving a node, changing a viewport, collapsing a group —
change the layout file and never the semantic revision (ADR-0003,
unchanged). Reordering declarations in text never moves a node; moving a
node never reorders text.

### 5. Invalid text keeps the last known good semantics

Free text editing means text is temporarily invalid. The open project
then holds the **committed semantic project** (the last revision whose
sources built), plus the **text draft** exactly as typed, plus the
loader's faults as diagnostics on the file and range they concern. The
graph shows the committed entities, marked out of sync with the text;
nothing is invented and nothing is discarded. When the text builds again,
reconciliation binds it to the same identities (the working identity
table keeps keys whose items are transiently absent, ADR-0020 §4) and the
project moves to a new revision. Malformed text cannot erase the graph.

### 6. Legacy JSON projects migrate on open, once, and are not converted back

A manifest with `kind = flat` or `kind = system` and a `design/*.json`
file is a legacy project. Opening it (Studio, `bdld`, the CLI) migrates
it in place before anything else runs:

1. load the JSON model as before;
2. render its textual projection once into `src/main.bdl`;
3. seed `.bdl/identities.json` from the ids the model already has, so
   every concept, relationship, domain, output, device, component and
   binding keeps the identity that `ui/layout.json`, hues and references
   are keyed by; allocators carry over so no id is ever reused;
4. write `.bdl/authoring.json` from the model's groups;
5. rename the JSON design file to `<name>.migrated` in place (recoverable,
   never read again) and write a manifest without `kind`;
6. verify that the sources read back as the same model; refuse the open
   with a fault naming the difference if they do not.

Layout is untouched by migration. There is no `convert` command and no
path back to JSON; the JSON persistence layer remains only to *read* a
legacy project for migration.

### 7. First graphical projection and auto-layout

A project may have sources and identities but no layout for some or all
entities (a hand-written project, an item added in Code or by an external
editor). On every open and every commit the daemon places exactly the
entities that have no position, deterministically, near what they read
and produce, avoiding overlap, and never moves an entity that has a
position. Placement is a layout service (input: semantic graph, existing
positions, entities to place; output: layout updates only) and is
persisted with the layout. Whole-graph relayout happens only on explicit
request. This is the first graphical projection of the sources, not an
import.

### 8. Conflicts are revisions, not merges

Studio, the language server and external editors edit one project. A
semantic edit is sent against the revision the client holds and is
refused when the project moved on (ADR-0009); a text edit from Studio's
Code view is sent against the revision it was read at; a source file
changed on disk since it was loaded is reported, and a save over it
refuses unless the designer chooses reload or overwrite (ADR-0020 §9).
Nothing is last-write-wins; nothing dirty is discarded without a choice.

## Alternatives

* **Keep the `text` kind and add a Code view to JSON projects.** Two
  persistence forms for one meaning; every feature would be written twice
  (write-back for one, JSON edits for the other) and "which kind is this"
  stays a question a designer must answer. Lost on the drift argument.
* **JSON canonical, text derived.** Comments, formatting and file
  partitioning cannot be derived from JSON; every text edit would have to
  be re-imported, which is the import the product must not have. Lost on
  the lexical-information argument.
* **Ids inline in source (`@id` annotations).** Loses the readability of
  the source and turns every copy/paste into an identity hazard; the
  sidecar keyed by qualified name with deterministic reconciliation keeps
  identity where a person expects it and makes the hazards explicit
  (copy → fresh id, delete-and-recreate → fresh id). Rejected in ADR-0020
  §3 and not reopened.
* **Migrating legacy projects lazily (keep reading JSON until the first
  save).** Leaves two authorities open at once; a session that never
  saves would read one and the language server the other. Lost.

## Consequences

* `bdl-model::persist::ProjectKind` and the manifest `kind` are read only
  by the migration path; `PROJECT_KIND_*` in the protocol becomes a
  compatibility field that always reports the unified kind
  (`docs/PROTOCOL.md`); `InitSystemProject` and `InitTextProject` behave
  as `InitProject`.
* `docs/PROJECT_FORMAT.md` describes one layout; the JSON schemas move to
  a *legacy, migrated on open* section kept for the migration reader.
* The daemon gains a layout service and runs it on open and commit
  (`docs/ARCHITECTURE.md`); Studio's own auto-placement at render time is
  removed.
* Studio gains Code and Split views over the same open project
  (`docs/STUDIO_UI.md`); the welcome screen offers one *New Project*.
* The user guide says *Code view* and *Design view* of *the same BDL
  project*; "text project" and "Studio project" are retired
  (`docs/user-guide`).
* Status, roadmap and a change fragment record what of this is
  implemented at each step; this record does not.
