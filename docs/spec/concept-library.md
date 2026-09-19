---
kind: specification
area: language
status: current
---

# Standard Library

A shared, designer-facing catalogue of _items_: reusable authoring fragments
built from ordinary BDL structures, so that "I need a temperature concept" or "I
need a temperature the environment provides" is one drag and one rename, not
five dialogs. One library serves the canvas (right-click, Library panel,
drag-and-drop), the textual surface (`concept Amb…` completion) and any future
authoring client, from one data file.

```text
library/std/concepts.toml   ──▶  bdl-library  ──┬──▶  bdld  ──▶  Studio (Library tab, right-click menu, unit picker)
       (data, versioned)         (load, validate,  └──▶  bdl-ide completion  ──▶  bdl-lsp
                                  search, plan)
```

## Purpose, and what it is not

The library removes repetitive setup. It is **not** the kernel (it adds no type
and no compiler rule), **not** a device catalogue (a `BH1750` is hardware that
_provides_ an `AmbientLight : Illuminance`; it is not the concept), **not** a
source of global identity (nothing anywhere is "the" Temperature), and **not** a
place where a role lives (a Source is a Source because of its shape, ADR-0032).
It provides good defaults that instantiate ordinary BDL objects.

## Items, fragments, categories

A **`LibraryItem`** is
`{ id, category, display_name, description, group, keywords, icon, fragment }`.
Its **`Fragment`** is the ordered list of objects instantiating it creates, each
with a fragment-local **key**:

- `FragmentObject::Concept { key, default_name, description, representation, unit, role_hint }`
  — one `CreateConcept`;
- `FragmentObject::Mapping { key, default_name, description, inputs, output }` —
  one `CreateMapping` whose signature names other objects of the same fragment
  by key.

Two **categories** exist today (`ItemCategory`): a **Concept** item is a
fragment of exactly one concept; a **Source** item is a fragment of one concept
(`value`) and one relationship without inputs over it (`source`, written
`() -> Value`), with no definition — an environment-provided value entering the
behavior model. The category is a browsing section (_Concepts_, _Sources_) and a
shape constraint the loader checks; it is not a flag anything downstream reads.
The _Sources_ section is not _Sensors_: a button state, an analog level and a
value a host provides are Sources and not sensors.

`group` (`environment`, `motion`, `human`, `external`, …) orders rows inside a
section and the quick-insert menu; `role_hint` (`input | output | either`, on a
concept) groups that menu and nothing else — input/output is not in the kernel;
`icon` is a generic word a client may map to a glyph, never identity.

## Item vs semantic identity

A fragment is a recipe for edits. Instantiating it allocates fresh `SemanticId`s
like any other creation, and from then on the objects are independent copies of
the defaults:

- instantiating `std.environment.temperature` twice yields two concepts
  (`Temperature`, `Temperature2`) with distinct identities — rename them
  `RoomTemperature` and `MotorTemperature` and they stay two; instantiating
  `std.source.temperature` twice yields `RoomTemp`/`TempSensor` and
  `RoomTemp2`/`TempSensor2`;
- every field is editable afterwards: name, description, representation, the
  relationship's definition;
- the project never records which item an object came from, and it loads without
  the library present — name, identity, representation and dimension all live in
  `design/project.bdl.json`; the fragment keys reach nothing;
- a later library version cannot reinterpret an existing object; the tests
  change an item's default and check the project does not move.

Item ids (`std.environment.temperature`, `std.source.temperature`) are
**library** identities: stable for menus, recents and completion; never a
`SemanticId`, never a `DeclId`, never a key the model uses.

## Schema (`schema_version = 2`)

```toml
[library]
id = "std"                      # prefixes every item id
name = "BDL Standard Library"
schema_version = 2
version = "0.2"                 # informational; projects never depend on it

[[template]]                    # a Concept item
id = "std.environment.ambient_light"
display_name = "Ambient Light"  # menus and rows (canonical English)
default_name = "AmbientLight"   # the concept's name on creation (an identifier)
description = "How much light falls on the product from its surroundings."
category = "environment"        # the group
role_hint = "input"             # input | output | either — discovery only
representation = { quantity = "illuminance" }   # or "boolean" | "count" | "open"
unit = "lx"                     # a symbol from the shared unit table; the default to show
keywords = ["light", "lux", "brightness", "daylight", "dark", "photocell"]
icon = "light"                  # presentation hint; never semantic

[[source]]                      # a Source item
id = "std.source.temperature"
display_name = "Temperature Sensor"
description = "The temperature of a room, a surface or the air, as the environment provides it."
category = "environment"        # the group
keywords = ["temp", "thermal", "thermometer", "sensor", "source"]
icon = "temperature"
[source.value]                  # the concept (fragment key `value`)
default_name = "RoomTemp"
description = "The measured temperature."
representation = { quantity = "temperature" }
unit = "K"
[source.relationship]           # `mapping TempSensor : () -> RoomTemp` (key `source`)
default_name = "TempSensor"
description = "Supplies the measured temperature, one reading per activation."
```

Schema 1 files (`[[template]]` only) still load; `[[source]]` needs schema 2.
Loading validates every item: the quantity must exist in the shared vocabulary,
the unit must exist in the shared unit table and measure that quantity, every
default name must be an identifier and the names inside one fragment distinct, a
mapping's `inputs`/`output` keys must name concepts of the same fragment, ids
must be unique and prefixed by the library id; an unsupported `schema_version`
is refused with the reason. The standard library is embedded in `bdl-library` at
build time (`Library::standard()`); the same loader reads any file
(`Library::from_toml`).

**Text is canonical English.** The file, the crate and the protocol carry
`display_name` and `description` in English only; a locale is presentation
(ADR-0031), so Studio localizes by item id (below). Identifiers (`default_name`,
`id`) never change with the locale.

## Items (44)

| section  | group       | items                                                                                                 |
| -------- | ----------- | ----------------------------------------------------------------------------------------------------- |
| Concepts | environment | Temperature, Ambient Light, Humidity, Air Pressure, Sound Level                                       |
|          | human       | Button Pressed, Touch, Switch State, Dial Position, Slider Position                                   |
|          | motion      | Distance, Position, Angle, Tilt, Speed, Acceleration, Angular Velocity, Orientation                   |
|          | mechanical  | Force, Pressure, Torque                                                                               |
|          | electrical  | Voltage, Current, Battery Level                                                                       |
|          | visual      | Brightness, Color, Display Value                                                                      |
|          | actuation   | Motor Speed, Motor Angle, Servo Position, Vibration Intensity, Heater Power, Fan Speed, Valve Opening |
|          | audio       | Volume, Pitch                                                                                         |
| Sources  | environment | Temperature Sensor (`RoomTemp` / `TempSensor`), Ambient Light Sensor (`AmbientLight` / `LightSensor`) |
|          | motion      | Tilt Sensor (`Tilt` / `TiltSensor`), Distance Sensor (`Distance` / `DistanceSensor`)                  |
|          | human       | Button State (`ButtonHeld` / `ButtonInput`), Encoder Position (`ShaftAngle` / `EncoderPosition`)      |
|          | electrical  | Analog Input (`AnalogValue` / `AnalogInput`)                                                          |
|          | external    | External Value (`ExternalValue` / `ExternalSource`)                                                   |

Deliberately small: nothing is added to inflate the count. The eight Sources are
the environment-provided values a first product needs, named for what a designer
looks for. _Analog Input_ and _External Value_ leave the concept's
representation **open** (_decide later_): the library does not know what an
analog reading or a host-provided value measures, and the designer chooses the
quantity once they do — no `Scalar` or voltage is presumed. _External Value_ is
there so the section says what it means: a Source is a boundary, not a sensor.

## Instantiation is one transaction

`bdl_library::plan(design, item, names)` turns a fragment into ordered
`PlannedStep`s against the target design: a `Concept` step carries its
`CreateConcept` edit with a name free among the design's concepts _and_ mappings
(`free_name`: `RoomTemp`, then `RoomTemp2`), or the caller's chosen name by key;
a `Mapping` step carries its name, description and the `inputs`/`output`
**keys**. The daemon (`Session::apply_library_item`) applies the steps in one
**transaction**: each step's edit goes through the ordinary `SystemEditOp` path
(name check, rename expansion, `apply_system_edit`) on a working copy, a mapping
step resolves its keys to the `SemanticId`s the earlier steps allocated, and the
working copy becomes the new revision only when every step succeeded — one
history entry (one Undo removes the whole fragment, a Redo recreates it with the
same identities), one re-derivation, one merged outcome (`created_concept` and
`created_mapping` both set for a Source). When any step is refused (a stale
revision, an invalid chosen name, an unknown component) nothing is applied and
the project stays at `base_revision`. The plan never leaves the daemon; fragment
keys are never a project fact.

A Source item's result is two ordinary objects; the canvas shows the
relationship as a Source because it reads nothing, has no definition and backs
no port (ADR-0032), and a third-party library's `[[source]]` gets the same
treatment. `mapping TempSensor : () -> RoomTemp` is the preferred spelling of
the Unit-domain relationship (ADR-0029), written by whoever generates it; no
item privileges a Source.

## Presentation-owned localization

Item names, descriptions and search tags in zh-Hans and ja live in Studio's
ordinary l10n pipeline, never in the library, the crate or the protocol:
`locale/library/std.json`
(`{ "<item id>": { "<locale>": { name, description, tags[] } } }`) is generator
input only — `scripts/gen_library_l10n.py` reads it with `concepts.toml` and
writes `libItem_<id>_name` / `_description` / `_tags` into
`apps/studio/lib/l10n/app_{en,zh,ja}.arb` and the lookup
`apps/studio/lib/l10n/library_strings.dart` (`libraryItemStrings(l10n, id)`),
which `flutter gen-l10n` then compiles like every other string. The panel shows
the localized name and description for a standard item and the daemon's English
for an item the catalogue does not know; **search** matches the localized name
and tags, the English name and keywords, the default names, the group and
section words and the unit — `温度` and `传感器` find the Temperature Sensor in
zh-Hans, `温度` and `センサー` in ja, `lux` finds Ambient Light and Ambient
Light Sensor. Changing the locale changes what the rows say and never what an
instantiation creates. `just studio-l10n-check` (preflight `l10n`) fails when
the generated files are behind the catalogue.

## Concepts and quantities

A concept's identity and its representation are different things, and the
library keeps them apart: `AmbientLight` is a _concept_ (what the product cares
about); `Illuminance` is a _quantity_ (what a value of it is measured as).
Likewise `Brightness` defaults to a dimensionless level (`Scalar`, 0–1) and does
**not** mean photometric luminance; `Humidity`, `BatteryLevel`, `MotorSpeed` are
levels too.

The quantity vocabulary is one table, `bdl_model::quantity`: each entry has a
library id (`illuminance`), the textual type name (`Illuminance`), the canonical
unit symbol (`lx`) and the dimension. The textual syntax resolves
`concept X : Illuminance` there; the library references quantities by id;
`bdl-elab`'s unit table names its units against the same dimensions; `bdld`
serves the table with the libraries so Studio's unit picker lists the same
kinds. There is no second dimension table anywhere. Angle is a base dimension in
BDL, so illuminance is `cd·rad²·m⁻²` and torque shares `N·m` with energy — a
consequence of the kernel's `Dim`, not of the library.

## Studio

**Right-click on the canvas** — compact quick insertion, at the pointer:

```text
Rename · Delete                 (when over a node)
Add Concept ▸
    Recent ▸                    (up to 6, most recent first — a preference, not project state)
    Input ▸  ·  Output ▸        (by role hint)
    Environment ▸ · Geometry & motion ▸ · Human interaction ▸
    More…                       (opens the Library tab)
Add Source ▸
    Temperature Sensor · Tilt Sensor · … · External Value   (the Source items)
    New source…                 (a `() -> C` over a concept already in the design)
```

**Create-then-rename**: choosing an item instantiates it through `bdld`
(`InstantiateLibraryItem`); when the projection with the new objects arrives,
the concept is placed where the pointer was (layout only, never a revision),
selected, and its name opens for editing on the canvas with the default name
selected — Return commits a rename edit, Esc keeps the default, leaving the
field commits what was typed. Add Temperature → type `MotorTemperature` → keep
designing. For a Source item the answer carries `created_concept` and
`created_mapping`: the Source lands a node width to the concept's left (the
environment side), and one Undo removes both.

**Library tab** (left sidebar, beside _Project_) — browsing and discovery: a
search field (above), two sections — _Concepts_, then _Sources_ — each with its
groups, rows with a grey socket glyph (filled = representation chosen, hollow =
decide later; grey because the hue is the identity the compiler will allocate)
or, in _Sources_, the Source silhouette (`MappingGlyph(source:)`), the name, and
what the value is measured as in the contract's words (`K`, `lx`, `no unit`,
`on–off`, `count`, `decide later`); the description on hover, and for a Source
what the item creates (`RoomTemp`, `TempSensor : () -> RoomTemp`). **Drag** a
row onto the canvas to insert at the drop point; double-click or Return inserts
auto-placed.

Every entry point dispatches the same `InsertLibraryItemRequested` and the
daemon performs the one instantiation; the results differ only in `SemanticId`,
layout position and whatever the designer renames. _New source…_ is not a
library path: it is the New relationship sheet without its _Reads_ row, an
ordinary `CreateMappingRequested` with no inputs.

The **unit picker** (New Concept sheet, inspector) lists the served quantity
vocabulary, with a built-in fallback for the base dimensions until the daemon
has answered.

## Textual surface and LSP

`bdl-ide` reads the same `LibrarySet` for completion: in a `concept Amb…`
position it offers `AmbientLight : Illuminance` (every Concept item whose name
or keyword matches; a name already taken in the project ranks lower), as
`CompletionKind::Template` carrying the item id. Accepting one writes ordinary
syntax — no metadata, no marker; the source stays plain BDL. A Source item is
not a concept completion: its relationship is ordinary syntax too, written in
the preferred spelling — `mapping tempSensor : () -> Temperature` — by whoever
generates it (Studio's write-back, the migration, an example). The LSP adapter
renders a completion as a class item labelled with the library id; the test
asserts that every Concept item of the embedded library is offered from the same
data, never from a second list.

## Protocol

| request                                                                                   | response                                                                                                            | notes                                                                                                                                                                                                                                                                                                                                                  |
| ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `ListLibraryItems` (0.17)                                                                 | `LibraryItemsResponse { libraries[] { id, name, schema_version, version, items[] LibraryItemView }, quantities[] }` | every item of every served library, `creates[] LibraryObjectView { kind, key, name, type_name, signature, description, representation?, unit }` in creation order with the default names, `concept?` the `ConceptTemplateView` of a Concept item; independent of any project; Studio asks once per connection                                          |
| `InstantiateLibraryItem { base_revision, item_id, names{key → name}, component? }` (0.17) | `SystemEditApplied`                                                                                                 | the daemon plans the fragment (free names, or `names` by key) and applies every step in one transaction — one revision, one history entry, the outcome with every `created_*`; nothing is applied when any step is refused; `edit.stale_revision`, `library.unknown_item`, `library.invalid_plan` (a key no earlier step created), `edit.invalid_name` |
| `ListConceptTemplates` (0.5)                                                              | `ConceptTemplatesResponse { libraries[] { …, templates[] }, quantities[] }`                                         | the Concept items as `ConceptTemplateView`s — the legacy projection; a Source item is not listed                                                                                                                                                                                                                                                       |
| `InstantiateConceptTemplate { base_revision, template_id, name?, component? }` (0.5)      | `SystemEditApplied`                                                                                                 | the same transaction with `names = { concept: name }`; `library.unknown_template`                                                                                                                                                                                                                                                                      |

`ConceptTemplateView.source_default_name`, `display_names`, `descriptions` and
`InstantiateConceptTemplateRequest.source_name` (0.16) are **deprecated since
0.17** and never set: a Source is an item, not a template field, and text is
localized by the client. Protocol 0.5 for the template surface; 0.16 for the
deprecated fields; 0.17 for items.

## Future custom libraries

`LibrarySet` holds several libraries, searched together; ids stay unique because
each library prefixes its own (`team.`, `proj.`, a package name). Loading is one
call per file (`Library::from_toml`), so a team library in the workspace, a
project library under `project/library/`, or an installed package library needs
a discovery rule and nothing else — no package manager is implemented and
nothing assumes there is only one library.

## Device library (separate, later)

Hardware is a different catalogue: `BME280`, `BH1750`, `MPU6050`, `SG90`,
`WS2812` are _devices_ that provide or consume concepts (or compatible
representations) and generate hardware requirements for the allocator
(`docs/spec/hardware-model.md`). The intended relation is

```text
device package BH1750   provides   AmbientLight : Illuminance
```

without `AmbientLight == BH1750`: the design stays replaceable with another
sensor, and the solver keeps working through device requirements, not through
the concept library.

## Tests

| scenario                                                                                                                                               | where                                                                            |
| ------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| the embedded library loads, is 30–50 items, every quantity/unit resolves; `Brightness` is a level, not luminance; schema 1 still loads                 | `bdl-library` unit tests                                                         |
| a Source item plans a concept and a `() -> value` mapping; names free across concepts and mappings; chosen names by key; the template projection       | `bdl-library` unit tests                                                         |
| two instantiations → two `SemanticId`s, free names, independent defaults after renaming and rebinding                                                  | `bdl-library`, `bdld` stdio e2e, Studio e2e                                      |
| a project persists and reloads without the library; a changed item default does not reach an existing object                                           | `bdl-library`                                                                    |
| invalid libraries are refused with the reason (unit of the wrong dimension, unknown quantity, schema, a key no concept of the fragment has)            | `bdl-library`                                                                    |
| every Concept item is a textual completion, from the same data                                                                                         | `bdl-ide/tests/acceptance.rs`                                                    |
| the daemon serves the embedded library item for item, the legacy surface lists Concept items only with the 0.16 fields empty                           | `bdld/tests/stdio_e2e.rs`, `bdld/tests/text_e2e.rs`                              |
| a Source item is one transaction: one revision, one Undo, a Redo with the same identities, nothing applied when a later step is refused; the text view | `bdld/tests/stdio_e2e.rs` (`library_items_over_stdio`), `bdld/tests/text_e2e.rs` |
| right-click and drag are one creation path; create-then-rename; recents; inline rename; localized search; refused insertion; the Source placed left    | `apps/studio/test/library_test.dart`, `apps/studio/test/source_role_test.dart`   |
| Studio against the real `bdld`: insert twice, rename, save, reopen; a Source item in zh-Hans                                                           | `apps/studio/test/library_e2e_test.dart`                                         |
| the generated presentation strings follow the catalogue                                                                                                | `scripts/gen_library_l10n.py --check` (preflight `l10n`)                         |
