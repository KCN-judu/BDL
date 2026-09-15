# Project format

A BDL project is a directory. The canonical content is owned by the Rust
model (`crates/bdl-model`), never by the Flutter graph.

```
project/
├── bdl.toml                  manifest: schema_version, name, compiler_version
├── design/
│   └── project.bdl.json      semantic content (concepts, mappings, id allocator)
├── ui/
│   └── layout.json           canvas positions keyed by stable id — NOT semantics
├── components/               supplied Rust components (planned)
└── Bdl.lock                  pinned toolchain / runtime versions (planned)
```

## Rules that do not change

* `semantics != UI layout`. A design opens without any layout file; a layout
  never changes what a design means. Moving a node creates no revision.
* Every persisted file carries `schema_version`. Newer schemas are refused,
  older ones are migrated forward in `persist::check_schema` (none exist yet).
* Identity is a stable integer id allocated per project and never reused
  (`IdAllocator` is persisted). Display names are mutable documentation.
* Writes are crash-safe: temporary file in the same directory → flush → fsync →
  atomic rename. The design file is written before the manifest so a manifest
  never points at a design that failed to write.

## `design/project.bdl.json` (schema 1)

```json
{
  "schema_version": 1,
  "design": {
    "name": "lamp",
    "concepts": { "0": { "id": 0, "name": "Tilt" }, "1": { "id": 1, "name": "Brightness" } },
    "mappings": {
      "0": { "id": 0, "name": "dimByTilt", "signature": { "inputs": [0], "output": 1 } }
    },
    "ids": { "next_semantic": 2, "next_decl": 1, "next_clock": 0, "next_output": 0 }
  }
}
```

An unresolved mapping simply has no `definition` key. Ordered maps keep the
file stable across saves.

`Concept.representation` is `{"kind":"quantity","dim":{…}}`, `{"kind":"boolean"}`
or `{"kind":"count"}`; absent means not yet chosen.
`MappingBlock.definition` is `{"kind":"formula","source":"…"}` for now.

## `ui/layout.json` (schema 1)

```json
{ "schema_version": 1, "layout": { "concepts": { "0": { "x": 10, "y": 20 } }, "mappings": {} } }
```

## Revisions are not persisted

`Revision` is a session counter starting at 0 on open. Project history and
semantic undo across sessions are future work (ROADMAP); the `EditOp` type is
already serializable so a log can be added without a new vocabulary.
