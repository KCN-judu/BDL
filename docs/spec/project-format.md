---
kind: specification
area: persistence
status: current
---

# Project format

A BDL project is a directory. There is one kind of project (ADR-0023): its
semantic content is the source text under `src/`, owned by the Rust model
(`crates/bdl-text` reads it into `bdl-system::BehaviorSystem`), never by the
Flutter graph. Design, Code and Split are views of it, not kinds.

```text
project/
├── bdl.toml                 manifest: schema_version = 2, name, compiler_version — no kind
├── src/**/*.bdl             the authored design and system — the semantic source
├── .bdl/identities.json     source key → stable id, allocators, flat ids — tool-owned
├── .bdl/authoring.json      behavior groups (authoring metadata, ADR-0019) — tool-owned
├── ui/layout.json           canvas positions, viewports, group boxes — presentation
├── components/              supplied Rust components (planned)
└── Bdl.lock                 pinned toolchain / runtime versions (planned)
```

Which information lives where is a per-fact rule (ADR-0023 §2):

| Kind of fact                                                                                                                                                               | Lives in               | Never in                      |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ----------------------------- |
| semantic — concepts, relationships and their signatures, formulas, timing domains, outputs, drives, devices, components, ports and contracts, instances, bindings, exports | `src/**/*.bdl`         | layout; the authoring sidecar |
| identity — the stable id behind each entity; the allocators; the flat-id table                                                                                             | `.bdl/identities.json` | the source text               |
| lexical — comments, whitespace, formatting, item order, the partition into files                                                                                           | `src/**/*.bdl`         | the model; the identity table |
| authoring metadata — behavior groups and membership                                                                                                                        | `.bdl/authoring.json`  | source; layout                |
| presentation — positions, viewports, group boxes, component-body canvases                                                                                                  | `ui/layout.json`       | source; the revision          |

Erasing both sidecars and the layout changes no semantic fact (identities and
positions are re-derived: fresh ids, the layout service's placement); erasing
the sources loses the design.

`bdl-text::load_workspace` discovers source files in sorted path order,
reconciles source keys with the identity sidecar (ADR-0020 §4), and builds the
`BehaviorSystem` every tool works on; the flat design is derived by
`bdl-system::flatten` on open and on every commit and is never written. `bdld`,
the CLI, the language server and Studio open a project through this one loader
(`bdl-text::load_project`). Derived flat designs, analyses, traces and generated
Rust are never source. A save writes the changed source files as item-level
edits (ADR-0020 §5), then the sidecars, the layout and the manifest; it refuses
when a source file changed on disk since it was read unless the caller chooses
to overwrite; reload is explicit.

## `bdl.toml` (schema 2)

```toml
schema_version = 2
name = "smart_lamp"
compiler_version = "0.1.0"
```

A manifest with `schema_version = 1` names a **legacy project** (below). A newer
schema is refused.

## `.bdl/identities.json` (schema 1)

```json
{
  "schema_version": 1,
  "keys": {
    "concept:Tilt": { "id": 0, "file": "src/main.bdl" },
    "mapping:dimByTilt": { "id": 1, "file": "src/main.bdl", "shape": "1" },
    "component:AdaptiveLamp": { "id": 0, "file": "src/lamp.bdl" },
    "component:AdaptiveLamp/port:tiltValue": { "id": 0, "file": "src/lamp.bdl" }
  },
  "base_ids": {
    "next_semantic": 3,
    "next_decl": 5,
    "next_clock": 1,
    "next_output": 1,
    "next_device": 1
  },
  "system_ids": {
    "next_component": 1,
    "next_instance": 2,
    "next_port": 1,
    "next_binding": 1,
    "next_export": 0,
    "next_group": 0
  },
  "flat_ids": {
    "entries": [
      { "instance": 1, "local": { "sort": "decl", "id": 0 }, "flat": 3 }
    ]
  }
}
```

`keys` maps a source key — `<kind>:<name>`, a component body's entity as
`component:<Component>/<kind>:<name>` — to the id the model gives that entity,
the file it was last seen in, and for a relationship its `shape` (the number of
inputs, which tells two declarations with one name apart). `base_ids` and
`system_ids` are the allocators (`IdAllocator`, `SystemIdAllocator`): an id is
never reused. `flat_ids` is the table that gives each instance its own flat
identities (docs/architecture/behavior-systems.md §5). The table is written on
open when the sources needed identities it did not have, on every save, and by
the language server after a save.

## `.bdl/authoring.json` (schema 1)

```json
{
  "schema_version": 1,
  "groups": {
    "0": {
      "id": 0,
      "scope": { "kind": "system_base" },
      "name": "Adaptive lamp",
      "description": "dims with tilt",
      "members": [4, 5]
    },
    "1": {
      "id": 1,
      "scope": { "kind": "component", "component": 0 },
      "name": "Dimming",
      "members": [1, 2]
    }
  }
}
```

Each behavior group's identity, scope, name, description and member list —
relationship ids of the scope's design, in authoring order — and nothing else
(ADR-0019); collapse state, position and size are layout. `scope` is
`{ "kind": "system_base" }` or `{ "kind": "component", "component": <id> }`
(component-local member ids). A member whose relationship the sources no longer
declare is dropped on load.

## Rules that do not change

- `semantics != UI layout`. A design opens without any layout file; a layout
  never changes what a design means. Moving a node creates no revision.
- Every persisted structured file carries `schema_version`. Newer schemas are
  refused; supported older schemas are migrated by the owning persistence layer.
- Identity is a stable integer id allocated per project and never reused
  (`IdAllocator` is persisted). Display names are mutable documentation.

- Names are identifiers (`docs/spec/textual-syntax.md` §2.5): the text is the
  semantic source, so a name it cannot spell is refused on every surface
  (`edit.invalid_name`).
- Writes are crash-safe: temporary file in the same directory → flush → fsync →
  atomic rename. The sources and sidecars are written before the manifest so a
  manifest never points at a design that failed to write.

## Legacy projects (schema 1 manifest), migrated on open

Projects written before ADR-0023 have `schema_version = 1` in `bdl.toml` and one
of the JSON design files below (`kind = "flat"`, absent, or `kind = "system"`);
a `kind = "text"` manifest names a project that already has sources and needs
only its manifest rewritten. Opening one — in Studio, `bdld`, the CLI or the
language server — migrates it in place before anything else runs
(`bdl-text::migrate_legacy`, ADR-0023 §6):

1. the JSON model is loaded as it always was;
2. its textual projection is rendered once into `src/main.bdl` — a display name
   the syntax cannot spell becomes an identifier deterministically
   (`Light Output` → `Light_Output`);
3. `.bdl/identities.json` is seeded from the ids the model already has, so every
   concept, relationship, domain, output, device, component, instance and
   binding keeps the identity `ui/layout.json` and hues are keyed by; the
   allocators carry over;
4. `.bdl/authoring.json` is written from the model's groups;
5. the sources are read back and compared with the model; a difference refuses
   the open with a fault naming it;
6. the JSON design file is renamed `<name>.migrated` (recoverable, never read
   again) and the manifest is rewritten at schema 2.

Layout is untouched. A migrated project cannot be written back as JSON; the JSON
readers below remain only for this step. A project whose `src/` already holds
files beside a JSON design is refused rather than guessed at.

### `design/project.bdl.json` (legacy, schema 1)

```json
{
  "schema_version": 1,
  "design": {
    "name": "lamp",
    "concepts": {
      "0": {
        "id": 0,
        "name": "Tilt",
        "representation": {
          "kind": "quantity",
          "dim": {
            "length": 0,
            "mass": 0,
            "time": 0,
            "current": 0,
            "temperature": 0,
            "amount": 0,
            "luminous": 0,
            "angle": 1
          }
        }
      },
      "1": { "id": 1, "name": "Brightness" }
    },
    "mappings": {
      "0": {
        "id": 0,
        "name": "tilt",
        "signature": { "inputs": [], "output": 0 },
        "clock": 0
      },
      "1": {
        "id": 1,
        "name": "dimByTilt",
        "signature": { "inputs": [0], "output": 1 },
        "definition": { "kind": "formula", "source": "Tilt / 90 deg" },
        "clock": 0,
        "drives": 0
      }
    },
    "clocks": { "0": { "id": 0, "name": "interaction" } },
    "outputs": {
      "0": {
        "id": 0,
        "name": "Light Output",
        "accepts": 1,
        "clock": 0,
        "required": true
      }
    },
    "devices": {
      "0": { "id": 0, "name": "PWM light", "kind": "pwm_channel", "output": 0 }
    },
    "ids": {
      "next_semantic": 2,
      "next_decl": 2,
      "next_clock": 1,
      "next_output": 1,
      "next_device": 1
    }
  }
}
```

Every section is a map keyed by the object's own id (as a string), in id order.
`examples/smart_lamp` is the unified form of this design;
`crates/bdl-compiler/tests/examples.rs` migrates a legacy copy and compares.

| Section    | Object           | Fields (`bdl-model::surface`)                                                                                                                                                                                                                                                                                                                                               |
| ---------- | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `concepts` | `Concept`        | `id`, `name`, `description` (omitted when empty), `representation` (omitted while not chosen: `{"kind":"quantity","dim":{…}}` with the eight integer exponents, `{"kind":"boolean"}` or `{"kind":"count"}`)                                                                                                                                                                 |
| `mappings` | `MappingBlock`   | `id`, `name`, `description`, `signature { inputs[], output }` (concept ids), `definition` (omitted while unresolved — that absence _is_ the unresolved declaration; `{"kind":"formula","source":"…"}` is the only authored form), `clock` (its timing domain, omitted while domain-agnostic), `drives` (the output it is the final driver of, omitted while it drives none) |
| `clocks`   | `ClockDomain`    | `id`, `name` — a domain is identity plus name; it has no frequency (the period is a simulation schedule, never project data)                                                                                                                                                                                                                                                |
| `outputs`  | `PhysicalOutput` | `id`, `name`, `description`, `accepts` (concept id), `clock` (omitted while open), `required` (default `true`)                                                                                                                                                                                                                                                              |
| `devices`  | `DeviceBinding`  | `id`, `name`, `kind` one of `pwm_channel`, `digital_output`, `h_bridge_channel`, `i2c_sensor`, `quadrature_encoder`, `uart`, `output` (omitted for a device bound to no sink), `fixed_pins` (map from the device's requirement index to a board-relative pin name, omitted when empty)                                                                                      |
| `ids`      | `IdAllocator`    | the next free id per identity kind; `next_device` defaults to 0 when absent (files written before devices existed still load)                                                                                                                                                                                                                                               |

`clocks`, `outputs` and `devices` are omitted when empty, so a design with
concepts and mappings only reads exactly as it did before those sections existed
— the additions were made without a schema bump. Layout, analysis results,
deployment results and simulation traces are never in this file.

### `design/system.bdl.json` (legacy, schema 2)

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

`base` is the system's own flat design; its `ids` allocator issues every flat
identity, the freshened ones in `flat_ids` included, so the two can never
collide and a flat id is never reused. `flat_ids` is completed by the edit model
and read by flattening (docs/architecture/behavior-systems.md §5). A binding's
`transport`, when present, is `{ "init": "45 deg" }`. The schema is refused when
newer, as for every other file.

A port's `contract` is the public promise, stored on its own (schema 2); `decl`
is only the body declaration meant to realize it. **Schema 1** (ports without
`contract`, one `stamp`) is migrated on load — the contract is derived from the
backing declaration _once_, then persisted on the next save; nothing keeps
deriving it from the body (`bdl-system::persist::migrate_v1_to_v2`).

**Groups** (still schema 2, an optional section added by the Studio
behaviour-authoring milestone, ADR-0019): `groups` holds each behaviour group's
identity, scope, name, description and member list — relationship ids of the
scope's design, in authoring order — and nothing else; collapse state, position
and size are layout (`ui/layout.json`). `scope` is `{ "kind": "system_base" }`
(the system's own design) or `{ "kind": "component", "component": <id> }` (that
component's body, with component-local member ids); a group without `scope`
reads as base-scoped. A file without `groups` reads as a system without groups.
Boundary results are never persisted. A binding end may also be a base
relationship: `"source": { "decl": 7 }` (a base relationship feeding a required
port) or `"destination": { "decl": 7 }` (a provided port realising an open base
relationship); port ends are `{ "instance", "port" }` as before.

## `ui/layout.json` (schema 1)

Every entity the sources declare has a position: the layout service
(`crates/bdl-layout`, ADR-0023 §7) places what has none on open and on every
commit — deterministically, in the column of its kind, beside what it reads or
produces, never moving what is placed — and `bdld` writes the result on open. A
project opens without this file; it is then created.

```json
{
  "schema_version": 1,
  "layout": {
    "concepts": { "0": { "x": 10, "y": 20 } },
    "mappings": {},
    "outputs": {},
    "instances": { "1": { "x": 300, "y": 200 } },
    "groups": {
      "0": { "x": 30, "y": 40, "width": 200, "height": 100, "collapsed": true }
    },
    "viewport": { "x": 0, "y": 0, "zoom": 1 },
    "components": {
      "0": {
        "concepts": {},
        "mappings": { "4": { "x": 5, "y": 6 } },
        "outputs": {},
        "groups": {
          "1": {
            "x": 5,
            "y": 6,
            "width": 208,
            "height": 92,
            "collapsed": false
          }
        },
        "viewport": { "x": -40, "y": 0, "zoom": 0.75 }
      }
    }
  }
}
```

Positions for concepts, mappings and physical outputs (sinks are canvas nodes
too), each keyed by the object's stable id; every section defaults to empty.
Timing domains and devices have no canvas position. A system project adds (all
optional, schema unchanged): `instances` — the component-instance nodes of the
system canvas, by raw instance id; `groups` — each behaviour group's collapsed
box and whether it is collapsed (the expanded region is computed from its
members' positions); `components` — one layout per component body, by raw
component id, in the body's own ids (an extracted component's canvas starts from
the members' positions in the system canvas), with its own `groups` boxes and
`viewport`: coordinates are never shared between canvases. `viewport` — where
the designer left a canvas (pan and zoom).

## Revisions are not persisted

`Revision` is a session counter starting at 0 on open. Project history and
semantic undo across sessions are an open design issue (ISS-0009); the `EditOp`
type is already serializable so a log can be added without a new vocabulary.
