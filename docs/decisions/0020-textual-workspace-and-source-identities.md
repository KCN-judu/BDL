---
id: ADR-0020
status: superseded
date: 2026-09-17
area: textual
supersedes: []
superseded-by: [ADR-0023]
related: []
fv: []
---

# ADR-0020: A textual project is canonical source plus a source-identity sidecar

## Status

Accepted (textual authoring milestone). Amends the project-format rules of
`docs/spec/project-format.md`; extends ADR-0008 (stable ids), ADR-0014 (textual
syntax infrastructure) and ADR-0017 (LSP is an adapter).

## Context

Textual BDL existed as an _overlay_: an open `.bdl` buffer was bound by name to
a JSON project and forgotten on close. That made the text surface a viewer, not
an authoring surface. The requirement is that text becomes a real authored
project — usable by the CLI, the language server, `bdld` and Studio — while
there remains **one semantic model** (`bdl-model`, `bdl-system`) and one
pipeline behind both surfaces.

The hard constraint is identity (ADR-0008): every concept, relationship, domain,
output, device, component, port, instance and binding has a stable integer
identity that layout, hues, bindings, contracts, provenance and persisted
references depend on. Names are labels. Text has only names.

## Decision

### 1. One canonical semantic source per project

A project's manifest names its kind. `flat` and `system` keep their JSON design
files. A new kind, **`text`**, makes the source tree the only semantic truth:

```text
project/
├── bdl.toml                 kind = "text"
├── src/**/*.bdl             the authored design and system — canonical
├── .bdl/identities.json     source-identity table + id allocators — tool-owned
├── .bdl/authoring.json      behavior groups (authoring metadata) — tool-owned
└── ui/layout.json           canvas positions, viewports, group boxes — as before
```

No `design/*.json` exists in a text project. The flat design is derived by
`bdl-system::flatten` exactly as for a system project. A project is text _or_
JSON, never both (option A of the milestone brief: text canonical, sidecars for
everything that is not semantics). Converting between kinds is an explicit
operation, not a mode.

### 2. What each tool opens

- **CLI, LSP, bdld, Studio** all open a text project through one loader
  (`bdl-text::load_workspace`): discover `src/**/*.bdl` in sorted path order,
  parse every file (lossless CST), lower, reconcile identities against the
  sidecar, build a `BehaviorSystem`, then hand it to the existing `flatten` /
  `analyze_system` / simulation / deployment / backend. There is no CLI-specific
  or LSP-specific parser path.
- Studio treats a text project as a system project (a flat text project is the
  degenerate system) and never rewrites source merely by opening it. The layout
  sidecar is read as for any project.

### 3. Identity lives in a sidecar keyed by source identity, not in source

Raw ids never appear in normal source (`mapping @decl(48291) …` is rejected).
`.bdl/identities.json` maps a **source key** to a stable id and carries the id
allocators so an id is never reused:

```text
concept:Tilt                      → SemanticId 0
mapping:dimByTilt                 → DeclId 2
clock:interaction                 → ClockId 0
output:light                      → OutputId 0
device:pwmLight                   → DeviceId 0
component:AdaptiveLamp            → ComponentId 0
component:AdaptiveLamp/concept:X  → local SemanticId (the body's allocator)
component:AdaptiveLamp/port:tiltValue → PortId 0
instance:lampA                    → ComponentInstanceId 0
binding:lampA.tiltValue<-tiltValue → BindingId 0
export:tiltIn                     → ExportId 0
```

A key is _kind + qualified name_, never a file path or a position, so a
declaration keeps its identity when it moves between files or within a file,
when files are split or merged, and when a formatter runs.

### 4. Reconciliation is deterministic and reported

`bdl-text::reconcile(previous_table, lowered_items) → (table, report)`:

| Situation                                                                                                                                                                    | Result                                                                                                      |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| key present in both                                                                                                                                                          | **retained** — the same id                                                                                  |
| key only in the new items                                                                                                                                                    | **allocated** — a fresh id from the table's allocator                                                       |
| key only in the previous table                                                                                                                                               | **dropped** — the id is retired (never reused); a later item with the same name is a new identity           |
| exactly one key of a kind dropped and exactly one allocated _in the same file_ with the same shape (same signature concept names for a relationship, same kind for the rest) | **renamed** — the id is carried over; the report says so                                                    |
| more than one candidate on either side                                                                                                                                       | **ambiguous** — fresh ids, and a loader finding names the items and says to use the editor's rename         |
| the same key twice                                                                                                                                                           | **duplicate** — a loader fault; the first declaration binds, the second is ignored (no id is minted for it) |

Rename through the language server or Studio is never heuristic: the tool knows
the identity and rewrites the key. The same-file swap rule exists so that a raw
rename in an external editor does not lose identity in the common case, and it
is refused (fresh ids, reported) the moment it could be wrong.
Copy/paste-then-rename yields a new key and a new id. Delete, save, recreate
later with the same name yields a new id.

Within one editing session the language server's working table keeps keys whose
items are transiently absent from a dirty buffer; keys are dropped only when the
sidecar is written, so a half-typed line never retires an identity.

### 5. Text edits persist as text

Every edit of a text project — from an editor, from the language server's rename
or code actions, or from Studio — ends as a change to `src/**/*.bdl` (and, when
identities were created or renamed, to the sidecar). Studio's model edits are
written back **surgically**: each authored item owns a source span; a changed
item is re-rendered and spliced over its span, a new item is appended to the
file that owns its scope (a component body's block, or the file the designer's
related items live in — the first source file by default), a deleted item's span
is removed. Trivia between items — comments, blank lines — and every untouched
item's spelling survive. Formula text is authored text and is never reprinted.
Comments _inside_ a re-rendered item are not preserved; the round-trip promise
is exactly this and no more (no byte-identical formatting after a semantic
edit).

The language server's rename, references and code actions produce text edits
through the same item spans and anchors (ADR-0017: the LSP is an adapter;
`bdl-text` owns the printer and splicer).

### 6. Multiple files compose one design

Files are read in sorted relative-path order; the loader composes one
`BehaviorSystem` from all of them. Names are resolved project-wide after all
files are read (concepts first, then clocks, outputs, mappings, components,
instances, bindings), so a file may reference a declaration in another file
regardless of order. A duplicate key across files is the same fault as within
one file. Diagnostics carry the file and range of the authored item they
concern.

### 7. Groups and layout stay outside the source

Behavior groups are authoring metadata (ADR-0019); they are stored in
`.bdl/authoring.json` by identity and never appear in source. Layout stays in
`ui/layout.json`. Erasing both sidecars changes no semantic fact.

### 8. Derived artefacts are excluded

The flat design, analyses, simulation traces and generated Rust are derived and
never written into the project as source. Nothing under `target/` or
`.bdl/cache/` (reserved) is read by the loader.

### 9. Concurrent editing: explicit reload, never silent loss

Studio and an editor may hold the same text project. Studio (through `bdld`)
records the modification time of every source file and sidecar it loaded. A save
from Studio refuses when a source file changed on disk since it was loaded
(`project.changed_on_disk`), and Studio offers _Reload_ (drop the in-memory
design and reload from disk) or _Save anyway_ (overwrite). Reload preserves
identities through the sidecar. Live merging of concurrent edits is out of
scope; nothing dirty is discarded without a choice.

## Consequences

- `bdl-text` is a new crate (`model → syntax → text → {daemon, ide-db, cli}`);
  it never depends on the IDE or protocol crates.
- `bdl-model::MappingBlock` gains `parameters: Vec<String>` (default empty): the
  textual parameter names, a real lexical binding layer for formula bodies
  (DI-30 closed); Studio-created relationships have none and keep resolving by
  concept name.
- The syntax grows to v0.2 (`docs/spec/textual-syntax.md` §14) to cover clocks,
  outputs, drives, devices, components, ports, instances, bindings and exports,
  since text cannot be canonical for a model it cannot spell.
- `bdl-ide-db` gains a workspace mode: the committed state of a text project is
  the loader's result over the files on disk with open buffers substituted, and
  anchors are placed on the authored source of every entity, including the body
  declarations behind flattened ones.
- Existing flat and system JSON projects are unchanged; the migration to text is
  `bdl convert --to text` (renders every item once), never automatic.
