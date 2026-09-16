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
    "concepts": { "0": { "id": 0, "name": "Tilt", "representation": { "kind": "quantity", "dim": { "length": 0, "mass": 0, "time": 0, "current": 0, "temperature": 0, "amount": 0, "luminous": 0, "angle": 1 } } },
                  "1": { "id": 1, "name": "Brightness" } },
    "mappings": {
      "0": { "id": 0, "name": "tilt", "signature": { "inputs": [], "output": 0 }, "clock": 0 },
      "1": { "id": 1, "name": "dimByTilt", "signature": { "inputs": [0], "output": 1 },
             "definition": { "kind": "formula", "source": "Tilt / 90 deg" }, "clock": 0, "drives": 0 }
    },
    "clocks":  { "0": { "id": 0, "name": "interaction" } },
    "outputs": { "0": { "id": 0, "name": "Light Output", "accepts": 1, "clock": 0, "required": true } },
    "devices": { "0": { "id": 0, "name": "PWM light", "kind": "pwm_channel", "output": 0 } },
    "ids": { "next_semantic": 2, "next_decl": 2, "next_clock": 1, "next_output": 1, "next_device": 1 }
  }
}
```

Every section is a map keyed by the object's own id (as a string), in id
order, so the file is stable across saves. `examples/smart_lamp` is a
complete checked-in instance.

| Section | Object | Fields (`bdl-model::surface`) |
|---|---|---|
| `concepts` | `Concept` | `id`, `name`, `description` (omitted when empty), `representation` (omitted while not chosen: `{"kind":"quantity","dim":{…}}` with the eight integer exponents, `{"kind":"boolean"}` or `{"kind":"count"}`) |
| `mappings` | `MappingBlock` | `id`, `name`, `description`, `signature { inputs[], output }` (concept ids), `definition` (omitted while unresolved — that absence *is* the unresolved declaration; `{"kind":"formula","source":"…"}` is the only authored form), `clock` (its timing domain, omitted while domain-agnostic), `drives` (the output it is the final driver of, omitted while it drives none) |
| `clocks` | `ClockDomain` | `id`, `name` — a domain is identity plus name; it has no frequency (the period is a simulation schedule, never project data) |
| `outputs` | `PhysicalOutput` | `id`, `name`, `description`, `accepts` (concept id), `clock` (omitted while open), `required` (default `true`) |
| `devices` | `DeviceBinding` | `id`, `name`, `kind` one of `pwm_channel`, `digital_output`, `h_bridge_channel`, `i2c_sensor`, `quadrature_encoder`, `uart`, `output` (omitted for a device bound to no sink), `fixed_pins` (map from the device's requirement index to a board-relative pin name, omitted when empty) |
| `ids` | `IdAllocator` | the next free id per identity kind; `next_device` defaults to 0 when absent (files written before devices existed still load) |

`clocks`, `outputs` and `devices` are omitted when empty, so a design with
concepts and mappings only reads exactly as it did before those sections
existed — the additions were made without a schema bump. Layout, analysis
results, deployment results and simulation traces are never in this file.

## `ui/layout.json` (schema 1)

```json
{ "schema_version": 1,
  "layout": { "concepts": { "0": { "x": 10, "y": 20 } }, "mappings": {}, "outputs": {} } }
```

Positions for concepts, mappings and physical outputs (sinks are canvas
nodes too), each keyed by the object's stable id; every section defaults to
empty. Timing domains and devices have no canvas position.

## Revisions are not persisted

`Revision` is a session counter starting at 0 on open. Project history and
semantic undo across sessions are future work (ROADMAP); the `EditOp` type is
already serializable so a log can be added without a new vocabulary.
