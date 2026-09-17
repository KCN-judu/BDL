# Project format

A BDL project is a directory. The canonical content is owned by the Rust
model (`crates/bdl-model`), never by the Flutter graph.

```
project/
├── bdl.toml                  manifest: schema_version, name, compiler_version, kind
├── design/
│   ├── project.bdl.json      kind = flat:   the authored flat design (concepts, mappings, …, id allocator)
│   └── system.bdl.json       kind = system: the authored behaviour system — the ONLY truth of a system project
├── ui/
│   └── layout.json           canvas positions keyed by stable id — NOT semantics
├── components/               supplied Rust components (planned)
└── Bdl.lock                  pinned toolchain / runtime versions (planned)
```

`kind` is absent in every project written before behaviour systems and means
`flat`. A JSON project has exactly one of the two design files: a system
project's flat design is *derived* on open and on every commit
(`bdl-system::flatten`) and is never written — two files claiming to be the
truth of one project would be two authorities.

A text project instead has this canonical shape (ADR-0020):

```text
project/
├── bdl.toml                  kind = "text"
├── src/**/*.bdl             canonical authored semantics
├── .bdl/identities.json     stable source identities and allocators
├── .bdl/authoring.json      behavior groups (authoring metadata)
└── ui/layout.json           presentation only
```

`bdl-text::load_workspace` discovers source files in sorted path order,
reconciles source keys with the identity sidecar, and builds the same
`BehaviorSystem` used by JSON system projects. A text project has no authored
`design/*.json`; derived flat designs, analyses, traces, and generated Rust are
never source. CLI/LSP/bdld/Studio use this loader rather than separate parsers.
Save refuses an external-disk change unless the caller explicitly chooses the
force-overwrite path; reload is explicit.

## Rules that do not change

* `semantics != UI layout`. A design opens without any layout file; a layout
  never changes what a design means. Moving a node creates no revision.
* Every persisted structured file carries `schema_version`. Newer schemas are
  refused; supported older schemas are migrated by the owning persistence
  layer.
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

## `design/system.bdl.json` (schema 2)

```json
{
  "schema_version": 1,
  "system": {
    "base": { "name": "rover", "concepts": { "0": { "id": 0, "name": "Tilt", … } }, "clocks": { … }, "outputs": { … }, "devices": { … }, "mappings": { … }, "ids": { … } },
    "components": { "0": { "id": 0, "name": "AdaptiveLamp", "body": { …an ordinary design over local ids… },
                           "interface": { "ports": { "0": { "id": 0, "name": "tiltValue", "kind": "required", "decl": 0,
                                                            "contract": { "signature": { "inputs": [], "output": 0 }, "clock": { "kind": "parameter", "clock": 0 } } } },
                                          "clock_params": [0] },
                           "shared_concepts": { "0": 0 }, "external_outputs": {}, "body_stamp": 7, "interface_stamp": 3 } },
    "instances":  { "1": { "id": 1, "component": 0, "name": "lampA", "clock_bindings": { "0": 0 }, "parameter_bindings": {} } },
    "bindings":   { "0": { "id": 0, "source": { "instance": 0, "port": 2 }, "destination": { "instance": 1, "port": 0 } } },
    "exports":    {},
    "groups":     { "0": { "id": 0, "scope": { "kind": "system_base" }, "name": "Adaptive lamp", "description": "dims with tilt", "members": [4, 5] },
                    "1": { "id": 1, "scope": { "kind": "component", "component": 0 }, "name": "Dimming", "members": [1, 2] } },
    "flat_ids":   { "entries": [ { "instance": 1, "local": { "sort": "decl", "id": 0 }, "flat": 3 }, … ] },
    "ids":        { "next_component": 2, "next_instance": 3, "next_port": 3, "next_binding": 2, "next_export": 0, "next_group": 1 }
  }
}
```

`base` is the system's own flat design; its `ids` allocator issues every
flat identity, the freshened ones in `flat_ids` included, so the two can
never collide and a flat id is never reused. `flat_ids` is completed by
the edit model and read by flattening (docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md
§5). A binding's `transport`, when present, is `{ "init": "45 deg" }`. The
schema is refused when newer, as for every other file.

A port's `contract` is the public promise, stored on its own (schema 2);
`decl` is only the body declaration meant to realize it. **Schema 1**
(ports without `contract`, one `stamp`) is migrated on load — the
contract is derived from the backing declaration *once*, then persisted
on the next save; nothing keeps deriving it from the body
(`bdl-system::persist::migrate_v1_to_v2`).

**Groups** (still schema 2, an optional section added by the Studio
behaviour-authoring milestone, ADR-0019): `groups` holds each behaviour
group's identity, scope, name, description and member list — relationship
ids of the scope's design, in authoring order — and nothing else; collapse
state, position and size are layout (`ui/layout.json`). `scope` is
`{ "kind": "system_base" }` (the system's own design) or `{ "kind":
"component", "component": <id> }` (that component's body, with
component-local member ids); a group without `scope` reads as
base-scoped. A file without `groups` reads as a system without groups.
Boundary results are never persisted. A binding end may also be a base relationship:
`"source": { "decl": 7 }` (a base relationship feeding a required port) or
`"destination": { "decl": 7 }` (a provided port realising an open base
relationship); port ends are `{ "instance", "port" }` as before.

## `ui/layout.json` (schema 1)

```json
{ "schema_version": 1,
  "layout": { "concepts": { "0": { "x": 10, "y": 20 } }, "mappings": {}, "outputs": {},
              "instances": { "1": { "x": 300, "y": 200 } },
              "groups": { "0": { "x": 30, "y": 40, "width": 200, "height": 100, "collapsed": true } },
              "viewport": { "x": 0, "y": 0, "zoom": 1 },
              "components": { "0": { "concepts": {}, "mappings": { "4": { "x": 5, "y": 6 } }, "outputs": {},
                                     "groups": { "1": { "x": 5, "y": 6, "width": 208, "height": 92, "collapsed": false } },
                                     "viewport": { "x": -40, "y": 0, "zoom": 0.75 } } } } }
```

Positions for concepts, mappings and physical outputs (sinks are canvas
nodes too), each keyed by the object's stable id; every section defaults to
empty. Timing domains and devices have no canvas position. A system
project adds (all optional, schema unchanged): `instances` — the
component-instance nodes of the system canvas, by raw instance id;
`groups` — each behaviour group's collapsed box and whether it is
collapsed (the expanded region is computed from its members' positions);
`components` — one layout per component body, by raw component id, in
the body's own ids (an extracted component's canvas starts from the
members' positions in the system canvas), with its own `groups` boxes
and `viewport`: coordinates are never shared between canvases. `viewport`
— where the designer left a canvas (pan and zoom).

## Revisions are not persisted

`Revision` is a session counter starting at 0 on open. Project history and
semantic undo across sessions are future work (ROADMAP); the `EditOp` type is
already serializable so a log can be added without a new vocabulary.
