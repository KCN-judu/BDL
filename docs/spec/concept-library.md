---
kind: specification
area: language
status: current
---

# Standard Concept Library

A shared, designer-facing vocabulary of _templates_ for the concepts products
author again and again — `Temperature`, `AmbientLight`, `ButtonPressed`,
`MotorSpeed` — so that "I need a temperature concept" is one drag and one
rename, not five dialogs. One library serves the canvas (right-click, Library
panel, drag-and-drop), the textual surface (`concept Amb…` completion) and any
future authoring client, from one data file.

```text
library/std/concepts.toml   ──▶  bdl-library  ──┬──▶  bdld  ──▶  Studio (Library tab, right-click menu, unit picker)
       (data, versioned)         (load, validate,  └──▶  bdl-ide completion  ──▶  bdl-lsp
                                  search, instantiate)
```

## Purpose, and what it is not

The library removes repetitive setup. It is **not** the kernel (it adds no type
and no compiler rule), **not** a device catalogue (a `BH1750` is hardware that
_provides_ an `AmbientLight : Illuminance`; it is not the concept), and **not**
a source of global identity (nothing anywhere is "the" Temperature). It provides
good defaults that instantiate ordinary BDL concepts.

## Template vs semantic identity

A template is a recipe for one `CreateConcept` edit. Instantiating it allocates
a fresh `SemanticId` like any other creation, and from then on the concept is an
independent copy of the defaults:

- instantiating `std.environment.temperature` twice yields two concepts
  (`Temperature`, `Temperature2`) with distinct identities — rename them
  `RoomTemperature` and `MotorTemperature` and they stay two;
- every field is editable afterwards: name, description, representation;
- the project never records which template a concept came from, and it loads
  without the library present — name, identity, representation and dimension all
  live in `design/project.bdl.json`;
- a later library version cannot reinterpret an existing concept; the tests
  change a template's default and check the project does not move.

Template ids (`std.environment.temperature`) are **library** identities: stable
for menus, recents and completion; never a `SemanticId`, never a `DeclId`, never
a key the model uses.

## Schema (`schema_version = 1`)

```toml
[library]
id = "std"                      # prefixes every template id
name = "BDL Standard Concept Library"
schema_version = 1
version = "0.1"                 # informational; concepts never depend on it

[[template]]
id = "std.environment.ambient_light"
display_name = "Ambient Light"  # menus and rows
default_name = "AmbientLight"   # the concept's name on creation (an identifier)
description = "How much light falls on the product from its surroundings."
category = "environment"        # browsing group
role_hint = "input"             # input | output | either — discovery only
representation = { quantity = "illuminance" }   # or "boolean" | "count" | "open"
unit = "lx"                     # a symbol from the shared unit table; the default to show
keywords = ["light", "lux", "brightness", "daylight", "dark", "photocell"]
icon = "light"                  # presentation hint; never semantic

[[template]]
id = "std.source.temperature"
display_name = "Temperature Sensor"
default_name = "RoomTemp"       # the concept
description = "The temperature of a room, a surface or the air, as the environment provides it."
category = "sources"
role_hint = "input"
representation = { quantity = "temperature" }
unit = "K"
keywords = ["temp", "thermal", "sensor", "source"]
icon = "temperature"
source = { default_name = "TempSensor" }   # also creates `mapping TempSensor : () -> RoomTemp`
i18n = { "zh-Hans" = { display_name = "温度传感器", description = "…" }, ja = { display_name = "温度センサー", description = "…" } }
```

A **Source template** (`source` present) creates two ordinary items in one
commit: the concept, and the relationship
`<source.default_name> : () -> <the concept>` with no definition — a value the
environment provides (ADR-0032). The library carries nothing else about it: the
canvas shows the relationship as a Source because of its shape and state, and a
third-party library's `source` field gets the same treatment.
`source.default_name` must be an identifier different from `default_name`.

**`i18n`** carries the display name and description per locale tag (`zh-Hans`,
`ja`); a locale absent from it shows the English fields. Identifiers
(`default_name`, `source.default_name`, `id`) never change with the locale
(ADR-0031).

Loading validates every template: the quantity must exist in the shared
vocabulary, the unit must exist in the shared unit table and measure that
quantity, the default name must be an identifier, ids must be unique and
prefixed by the library id; an unsupported `schema_version` is refused. The
standard library is embedded in `bdl-library` at build time
(`Library::standard()`); the same loader reads any file (`Library::from_toml`).

`role_hint` groups the quick-insert menu and nothing else — a concept is a
semantic value concept; input/output is not in the kernel. `icon` is a generic
word (`temperature`, `motor`) a client may map to a glyph; it is not identity
and not used today.

## Categories (43 templates)

| category    | templates                                                                                                                     |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------- |
| environment | Temperature, Ambient Light, Humidity, Air Pressure, Sound Level                                                               |
| human       | Button Pressed, Touch, Switch State, Dial Position, Slider Position                                                           |
| motion      | Distance, Position, Angle, Tilt, Speed, Acceleration, Angular Velocity, Orientation                                           |
| mechanical  | Force, Pressure, Torque                                                                                                       |
| electrical  | Voltage, Current, Battery Level                                                                                               |
| visual      | Brightness, Color, Display Value                                                                                              |
| actuation   | Motor Speed, Motor Angle, Servo Position, Vibration Intensity, Heater Power, Fan Speed, Valve Opening                         |
| audio       | Volume, Pitch                                                                                                                 |
| sources     | Temperature Sensor, Tilt Sensor, Distance Sensor, Ambient Light Sensor, Button / Switch State, Encoder Position, Analog Input |

Deliberately small: nothing is added to inflate the count. The Sources are the
environment-provided values a first product needs, named for what a designer
looks for; two of them (a button state, an analog level) are not sensors, which
is why the category is _Sources_ and not _Sensors_.

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
    Temperature Sensor · Tilt Sensor · … · Analog Input   (the Source templates)
    New source…                 (a `() -> C` over a concept already in the design)
```

A Source template's answer carries `created_concept` and `created_mapping`; the
concept lands where the pointer was and opens for renaming, the Source lands a
node width to its left, and one Undo removes both.

**Create-then-rename**: choosing a template instantiates it through `bdld`
(`InstantiateConceptTemplate`); when the projection with the new concept
arrives, the node is placed where the pointer was (layout only, never a
revision), selected, and its name opens for editing on the canvas with the
default name selected — Return commits a rename edit, Esc keeps the default,
leaving the field commits what was typed. Add Temperature → type
`MotorTemperature` → keep designing.

**Library tab** (left sidebar, beside _Project_) — browsing and discovery: a
search field (display name in English and in the current locale, default name,
the Source's relationship name, keywords, category, unit — `lux` finds Ambient
Light and Ambient Light Sensor, `motor` finds Motor Speed and Motor Angle,
`温度` finds the Temperature Sensor in zh-Hans), category sections (_Sources_
last, its rows wearing the Source silhouette instead of the socket glyph, the
tooltip naming what the relationship provides), rows with a grey socket glyph
(filled = representation chosen, hollow = decide later; grey because the hue is
the identity the compiler will allocate), the name, and what the value is
measured as in the contract's words (`K`, `lx`, `no unit`, `on–off`, `count`,
`decide later`); the description on hover. **Drag** a row onto the canvas to
insert at the drop point; double-click or Return inserts auto-placed.

Every entry point dispatches the same `InsertConceptTemplateRequested` and the
daemon performs the one instantiation; the results differ only in `SemanticId`,
layout position and whatever the designer renames.

The **unit picker** (New Concept sheet, inspector) lists the served quantity
vocabulary, with a built-in fallback for the base dimensions until the daemon
has answered.

## Textual surface and LSP

`bdl-ide` reads the same `LibrarySet` for completion: in a `concept Amb…`
position it offers `AmbientLight : Illuminance` (every template whose name or
keyword matches; a name already taken in the project ranks lower), as
`CompletionKind::Template` carrying the template id. Accepting one writes
ordinary syntax — no metadata, no marker; the source stays plain BDL. The
library holds concept templates only; a relationship that supplies one of them
from outside is an ordinary relationship without inputs, written in the
preferred spelling — `mapping tempSensor : () -> Temperature`,
`mapping tilt : () -> Tilt` — by whoever generates it (Studio's write-back, the
migration, an example); no template privileges a source. The LSP adapter renders
it as a class item labelled with the library id; the test asserts that every
template of the embedded library is offered from the same data, never from a
second list.

## Protocol

| request                                                                                      | response                                                                                                    | notes                                                                                                                                                                                                                                                                                                   |
| -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ListConceptTemplates`                                                                       | `ConceptTemplatesResponse { libraries[] { id, name, schema_version, version, templates[] }, quantities[] }` | independent of any project; Studio asks once per connection                                                                                                                                                                                                                                             |
| `InstantiateConceptTemplate { base_revision, template_id, name?, component?, source_name? }` | `SystemEditApplied`                                                                                         | the daemon builds the `CreateConcept` (defaults + a free name, or `name`) and applies it exactly like `ApplyEdit`; for a Source template also `CreateMapping { source_name or the template's default, () -> the concept }` in the same commit (0.16); `edit.stale_revision`, `library.unknown_template` |

`ConceptTemplateView` (0.16) adds `source_default_name` (empty for a plain
concept template), `display_names` and `descriptions` (by locale tag). Protocol
0.5; 0.16 for the Source fields.

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

| scenario                                                                                                              | where                                                                                                                                                   |
| --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| the embedded library loads, is 30–50 templates, every quantity/unit resolves; `Brightness` is a level, not luminance  | `bdl-library` unit tests                                                                                                                                |
| two instantiations → two `SemanticId`s, free names, independent defaults after renaming and rebinding                 | `bdl-library`, `bdld` stdio e2e, Studio e2e                                                                                                             |
| a project persists and reloads without the library; a changed template default does not reach an existing concept     | `bdl-library`                                                                                                                                           |
| invalid libraries are refused with the reason (unit of the wrong dimension, unknown quantity, schema)                 | `bdl-library`                                                                                                                                           |
| every template is a textual completion, from the same data                                                            | `bdl-ide/tests/acceptance.rs`                                                                                                                           |
| the daemon serves the embedded library template for template                                                          | `bdld/tests/stdio_e2e.rs`                                                                                                                               |
| right-click and drag are one creation path; create-then-rename; recents; inline rename; search; refused insertion     | `apps/studio/test/concept_library_test.dart`                                                                                                            |
| Studio against the real `bdld`: insert twice, rename, save, reopen                                                    | `apps/studio/test/concept_library_e2e_test.dart`                                                                                                        |
| a Source template is two edits in one commit, written `mapping S : () -> C`, one undo removes both, reopened the same | `bdl-library` (`source_templates_create_a_concept_and_an_explicit_unit_domain_relationship`), `bdld/tests/text_e2e.rs`, `concept_library_e2e_test.dart` |
| the Sources category, localized names and search, the row glyph                                                       | `apps/studio/test/source_role_test.dart`                                                                                                                |
