---
kind: specification
area: language
status: current
---

# Standard Library

A shared, designer-facing catalogue of _items_: the **value categories** a
concept is created from — a value form (_on / off_, _count_, a _level_, _decide
later_) or a named physical quantity of the shared vocabulary (_Angle_,
_Temperature_, _Angular velocity_, …) — so that "I need the lid's angle" is one
choice and one name (`LidAngle`), not a search through product presets and a
rename (ADR-0041). One library serves the canvas (right-click, Library panel,
drag-and-drop), the textual surface (`concept Ang…` completion) and any future
authoring client, from one data file.

```text
library/std/concepts.toml   ──▶  bdl-library  ──┬──▶  bdld  ──▶  Studio (Library tab, right-click menu, concept sheet)
       (data, versioned)         (load, validate,  └──▶  bdl-ide completion  ──▶  bdl-lsp
                                  search, plan)
```

## Purpose, and what it is not

The library removes repetitive setup. It is **not** the kernel (it adds no type
and no compiler rule), **not** a device catalogue (a `BH1750` is hardware that
_provides_ an `AmbientLight : Illuminance`; it is not the concept), **not** a
source of global identity (nothing anywhere is "the" Temperature), **not** a
place where a role lives (a Source is a Source because of its shape, ADR-0032),
and — since ADR-0041 — **not** a list of product concepts: it names no _Motor
Angle_, _Servo Position_ or _Fan Speed_, because to the compiler those are an
angle and a level, and the name is the product's to give. It provides the
categories that instantiate ordinary BDL objects.

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

Two **categories** exist (`ItemCategory`): a **Concept** item is a fragment of
exactly one concept; a **Source** item is a fragment of one concept (`value`)
and one relationship without inputs over it (`source`, written `() -> Value`),
with no definition — an environment-provided value entering the behavior model.
A Source item is read as a **preset** (`LibraryItem::preset`, `SourcePreset`):
what the Source creation flow prefills — a suggested concept name, value form
and unit, a suggested Source name — and ranks existing concepts by. It decides
no identity: the concept a Source provides is the designer's choice, an existing
concept of the design or a new one created in the same transaction (§ Creating a
Source). The category is a browsing section and a shape constraint the loader
checks; it is not a flag anything downstream reads. **The standard library ships
no Source item** (ADR-0041): a Source is created on the Source sheet over a
concept the designer chooses, existing or new from the value categories; a
third-party library may still ship Source presets and they are served and shown
as before.

In the standard library a Concept item is a **value category**: its
`representation` is the category (`boolean`, `count`, `open`, or one quantity of
the vocabulary), its `display_name` the category's word, its `default_name` the
category's type name — used only where no name is asked for (the legacy path,
the textual completion). `group` is `form` (the four value forms) or `quantity`
(the vocabulary, in its order) and orders the sections; `role_hint` is `either`
throughout; `keywords` are the search synonyms, including the product words the
presets used to be (_tilt_, _servo_, _encoder_, _heater_, _fan_, _dimmer_,
_battery_, _humidity_) and the units' spellings; `icon` is a generic word a
client may map to a glyph, never identity. A third-party library may still use
product groups (`environment`, `motion`, …) and `input` / `output` hints; they
order its own section and nothing else — input/output is not in the kernel.

## Item vs concept identity

A fragment is a recipe for edits. Instantiating it allocates fresh `ConceptId`s
like any other creation, and from then on the objects are independent copies of
the defaults:

- creating two concepts from `std.quantity.temperature` — `RoomTemperature` and
  `MotorTemperature`, named on the sheet — yields two concepts with distinct
  identities that share nothing but a dimension (the legacy path without a name
  makes `Temperature`, then `Temperature2`); two Sources created from the
  Temperature preset with new concepts are `Temperature`/`temperatureInput` and
  `Temperature2`/`temperatureInput2`, and one created over an existing
  `RoomTemperature` adds no concept at all;
- every field is editable afterwards: name, description, representation, the
  relationship's definition;
- the project never records which item an object came from, and it loads without
  the library present — name, representation and dimension are in the `.bdl`
  sources, identity in `.bdl/identities.json`; the fragment keys reach nothing;
- a later library version cannot reinterpret an existing object; the tests
  change an item's default and check the project does not move.

Item ids (`std.environment.temperature`, `std.source.temperature`) are
**library** identities: stable for menus, recents and completion; never a
`ConceptId`, never a `DeclId`, never a key the model uses.

## Schema (`schema_version = 2`)

```toml
[library]
id = "std"                      # prefixes every item id
name = "BDL Standard Library"
schema_version = 2
version = "0.3"                 # informational; projects never depend on it

[[template]]                    # a Concept item: in the standard library, a value category
id = "std.quantity.illuminance"
display_name = "Illuminance"    # menus and rows (canonical English)
default_name = "Illuminance"    # the name where none is asked for (legacy path, textual completion)
description = "How much light falls on a surface: ambient light, daylight, a photocell's reading."
category = "quantity"           # the group: form | quantity (a third-party library: any word)
role_hint = "either"            # input | output | either — discovery only
representation = { quantity = "illuminance" }   # or "boolean" | "count" | "open"
unit = "lx"                     # a symbol from the shared unit table, for a row to show; may be omitted
keywords = ["illuminance", "light", "lux", "lx", "ambient", "daylight", "dark", "photocell", "light sensor"]
icon = "light"                  # presentation hint; never semantic

[[source]]                      # a Source item (third-party libraries): a preset for the Source sheet
id = "team.source.temperature"
display_name = "Temperature Input"
description = "A temperature the environment provides — a room, a surface, the air — read once per activation."
category = "environment"        # the group
keywords = ["temp", "thermal", "thermometer", "sensor", "input", "source"]
icon = "temperature"
[source.value]                  # the new concept it suggests (fragment key `value`)
default_name = "Temperature"
description = "How warm something is."
representation = { quantity = "temperature" }   # ranks existing concepts of this form first
unit = "K"
[source.relationship]           # the Source it suggests: `temperatureInput : () -> <chosen>` (key `source`)
default_name = "temperatureInput"
description = "Provides the temperature, one reading per activation."
```

Schema 1 files (`[[template]]` only) still load; `[[source]]` needs schema 2.
The schema did not change for the Source sheet: a `[[source]]` entry is read
both as a preset (`LibraryItem::preset`) and, on the legacy path, as the
fragment it always was — no parameterized fragment language was added for one
dialog. Loading validates every item: the quantity must exist in the shared
vocabulary, the unit — when given — must exist in the shared unit table and
measure that quantity (a category whose preferred unit is a composite,
`rad per s`, gives none and the row shows the vocabulary's rendering), every
default name must be an identifier and the names inside one fragment distinct, a
mapping's `inputs`/`output` keys must name concepts of the same fragment, ids
must be unique and prefixed by the library id; an unsupported `schema_version`
is refused with the reason. The standard library is embedded in `bdl-library` at
build time (`Library::standard()`); the same loader reads any file
(`Library::from_toml`). The daemon serves the standard library and every file
named in **`BDL_LIBRARIES`** (path-separated, `LibrarySet::from_environment`); a
file that does not load is reported on stderr and skipped, never fatal.

**Text is canonical English.** The file, the crate and the protocol carry
`display_name` and `description` in English only; a locale is presentation
(ADR-0031), so Studio localizes by item id (below). Identifiers (`default_name`,
`id`) never change with the locale.

## Items (22)

| section    | group    | items                                                                                                                                                                                            |
| ---------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Values     | form     | On / off (`boolean`), Count (`count`), Level (`scalar`), Decide later (`open`)                                                                                                                   |
| Quantities | quantity | Angle, Length, Time, Mass, Current, Temperature, Amount of substance, Luminous intensity, Speed, Acceleration, Angular velocity, Frequency, Force, Pressure, Torque, Power, Voltage, Illuminance |

One item per value form and one per named quantity of `bdl_model::quantity`, in
the vocabulary's order (the base dimensions, then the derived ones) — nothing
else, and nothing added when the vocabulary is not. Ids are `std.value.<form>`
and `std.quantity.<quantity id>`. A category is not a product concept: _Angle_
becomes `LidAngle`, `ShaftAngle` or `Heading` on the concept sheet, and the
words a designer used to find the presets by — _tilt_, _servo_, _encoder_,
_dial_, _heater_, _fan_, _dimmer_, _battery_, _humidity_, _brightness_ — are the
categories' search synonyms. _Level_ is the dimensionless quantity (`scalar`: a
ratio, a percentage, a 0–1 setting) and is listed with the forms because that is
how a designer thinks of it. Until 2026-09-21 the file held 36 product presets
in eight groups and 8 Source presets (the ids `std.environment.*`,
`std.motion.*`, `std.source.*`, …); an object created from one is an ordinary
concept and is untouched.

## Instantiation is one transaction

`bdl_library::plan(design, item, names)` turns a fragment into ordered
`PlannedStep`s against the target design: a `Concept` step carries its
`CreateConcept` edit, a `Mapping` step its name, description and the
`inputs`/`output` **keys**. A default name is made free among the design's
concepts _and_ mappings and the steps before it (`free_name`: `Temperature`,
then `Temperature2`); a name the caller chooses by key is used exactly as given
— a taken or unspellable name is refused by the edit it becomes, never silently
renamed — and a key the item does not create is refused before anything is
planned (`library.invalid_plan`). Planning is pure and deterministic; the loader
has already refused a fragment whose mapping names a concept the fragment does
not create before it.

The daemon (`Session::apply_library_item`) applies the steps in one
**transaction**: each step's edit goes through the ordinary `SystemEditOp` path
(name check, rename expansion, `apply_system_edit`) on a working copy, a mapping
step resolves its keys to the `ConceptId`s the earlier steps allocated, and the
working copy becomes the new revision only when every step succeeded — **exactly
one revision** (`base + 1`, however many edits the fragment was), one history
entry (one Undo removes the whole fragment, a Redo recreates it with the same
identities), one re-derivation, one merged outcome (`created_concept` and
`created_mapping` both set for a Source). When any step is refused (a stale
revision, a taken or invalid chosen name, an unknown component) nothing is
applied: no object, no layout placement, no history entry, no dirtiness, and no
identity consumed — the next creation gets the id it would have got anyway. The
plan never leaves the daemon; fragment keys are never a project fact. Inside a
component body the keys resolve to the body's own identities: a body mapping
never binds to a system concept that shares its local number.

Instantiating an item this way (`InstantiateLibraryItem`) is the **legacy
path**: it creates the objects from the item's data — a concept named as the
item suggests (`Angle`, then `Angle2`), or a Source item's concept and the
Source over it — and stays supported for third-party schema-2 libraries and
older clients. Studio takes it for neither: a concept is created on the concept
sheet with the name the designer gives (§ Creating a concept), a Source on the
Source sheet over the concept the designer chooses (§ Creating a Source). Either
way the result is ordinary objects; the canvas shows the relationship as a
Source because it reads nothing, has no definition and backs no port (ADR-0032).
`mapping temperatureInput : () -> Temperature` is the preferred spelling of the
Unit-domain relationship (ADR-0029), written by whoever generates it; no item
privileges a Source.

## Creating a concept

A concept is `Name : Category` — an identity the product cares about and the
value form it is represented by — and the Library provides only the second half.
**A concept is created with its name** (ADR-0041): choosing a category anywhere
in Studio opens the concept sheet with the category preset; the sheet asks for
the name (required; an identifier; free among the design's concepts and
relationships — said on the sheet before the daemon would refuse it with
`edit.duplicate_concept` / `edit.invalid_name`), lets the category be changed in
one pop-up over the same items, shows the meaning, a live preview of the node
and the declaration the Code view will write, and — for a physical quantity —
the units the category is **measured in**. _Create_ sends one ordinary
`CreateConcept { name, description, representation }`
(`CreateConceptRequested`): one revision, one history entry, the concept placed
where the sheet was asked for (layout, never a revision), selected and named.
Cancel commits nothing. A name is never suggested and never made free by the
daemon: what was typed is what is created.

**The units are a fact, not a choice.** A concept stores its dimension and no
unit; every unit of the dimension is legal in every formula over it. The sheet
lists the compiler's candidates for the category — `ValueCategoryView.units`
(`ListValueCategories`, 0.27), preferred first, rendered as
`UnitExprView.display` (`rad`, `deg`, `turn`; `rad/s`, `deg/s`) — as _what a
formula over this concept may write_, and sends none of them. A per-concept
display unit is ISS-0019.

## Creating a Source

A Source is `name : () -> C` with no definition: a value entering the behavior
model from the environment, observed once per activation. **`C` is a concept the
designer chooses**, and a committed Source always has one — a concrete
`ConceptId` in `Signature.output`. There is no `() -> ?` in the project model,
in a source file or on the wire; the unmade choice exists only on the creation
sheet, and cancelling it leaves the project untouched. Three decisions are kept
apart: which concept the Source provides (concept identity), the abstract Source
over it (this section), and what realizes it at deployment (FV Phase 13,
PRP-0001: a device's raw input and a checked transducer, which make the Source a
Value — `docs/architecture/relationship-roles.md` § Phase 13). Authoring makes
the first two; nothing here touches the third.

A Source is created in one of two ways, and no other (`CreateSource`, protocol
0.23):

- **over an existing concept**, by identity —
  `roomTemperatureInput : () -> RoomTemperature` — one ordinary `CreateMapping`
  edit: one revision, one history entry, no new concept; one Undo removes the
  Source alone. Identity is nominal: `RoomTemperature : Temperature` and
  `MotorTemperature : Temperature` are two choices, and a choice is never
  inferred from a name, a value form, a dimension or a unit. An open value form
  (_decide later_) is a legal choice: a Source is about identity, not about
  closing the representation.
- **with a new concept**, created in the same transaction — `CreateConcept` then
  `CreateMapping` over it, the same two planned steps a Source item runs through
  `Session::apply_library_item`: one revision, one history entry, all or
  nothing. When either edit is refused (a taken or unspellable name, a stale
  revision) nothing is applied and no identity is consumed. The representation
  may be left open, as `CreateConcept` allows.

Names are the designer's: the sheet suggests one (`<concept>Input` in
lowerCamel, made free in the design; the preset's, made free by the daemon) and
sends what was typed as typed — a taken name is the ordinary edit refusal, never
a silent rename. Inside a component body the concept is one of the body's, by
the body's identities; a system concept that shares the local number is not a
candidate (`edit.unknown_concept`).

The daemon lists the candidates (`ListSourceCandidates`): every concept of the
design in scope, in id order, with the ones whose value form is the preset's
first (`preferred`) when a Source item of a served library is named — an
authoring convenience the library computes (`bdl_library::rank_concepts`), not a
rule, and never a filter: nothing is hidden, Studio infers nothing from
dimensions or names. The same answer carries the preset and its names made free
in the design. The _New concept_ half of the sheet is the concept sheet's form:
a category from the same items and a name, with the category's units shown as a
fact.

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
and tags, the English name and synonyms, the category's type name, the group and
section words and the display and source spellings of the category's units —
`温度` finds _Temperature_ in zh-Hans and ja, `lux` and `lx` find _Illuminance_,
`deg` and `rotation` find _Angle_ (and `rotation` _Angular velocity_),
`brightness` finds _Level_, `button` finds _On / off_. Changing the locale
changes what the rows say and never what a creation makes.
`just studio-l10n-check` (preflight `l10n`) fails when the generated files are
behind the catalogue.

## Concepts and quantities

A concept's identity and its representation are different things, and the
library keeps them apart by offering only the second: `AmbientLight` is a
_concept_ (what the product cares about, named on the sheet); `Illuminance` is a
_quantity_ (what a value of it is measured as, the Library's item). Likewise a
_brightness_, a _humidity_, a _battery level_ or a _motor speed setting_ is a
_Level_ (`Scalar`, 0–1) and **not** a photometric luminance or an angular
velocity — which is why those words are _Level_'s synonyms.

The quantity vocabulary is one table, `bdl_model::quantity`: each entry has a
library id (`illuminance`), the textual type name (`Illuminance`), the canonical
unit symbol (`lx`) and the dimension. The textual syntax resolves
`concept X : Illuminance` there; the library references quantities by id;
`bdl_model::units` names its units against the same dimensions; `bdld` serves
the table with the libraries and the value categories with their unit candidates
(`ListValueCategories`, ADR-0040) so Studio's concept sheet and unit pop-ups
list the same kinds. There is no second dimension table anywhere. Angle is a
base dimension in BDL, so illuminance is `cd·rad²·m⁻²` and torque shares `N·m`
with energy — a consequence of the kernel's `Dim`, not of the library.

## Studio

**Right-click on the canvas** — compact quick insertion, at the pointer:

```text
Rename · Delete                 (when over a node)
Add Concept ▸
    Recent ▸                    (up to 6, most recent first — a preference, not project state)
    On / off · Count · Level · Decide later
    Quantities ▸                (Angle · Length · … · Illuminance)
    <a third-party library's groups> ▸
    More…                       (opens the Library tab)
Add Source ▸
    New Source…                 (choose or create the concept)
    <a third-party library's Source presets>   (the same sheet, prefilled)
```

**The concept sheet** (`ui/concept_sheet.dart`, § Creating a concept): every
Concept entry point — a menu item, a Library row's double-click or Return, a
drag of a row onto the canvas, the Project tab's `+` — dispatches
`NewConceptRequested(presetId?, position?)` and the sheet opens over the design
with the category preset (none when asked for from the Project tab: the category
pop-up says _choose a category_). _Create_ dispatches
`CreateConceptRequested(name, description, representation, presetId, position)`
— one `CreateConcept` edit; when the projection with the new concept arrives it
lands where the pointer was, selected, named as typed — nothing opens for
renaming. Cancel dispatches `ConceptSheetDismissed`. The old create-then-rename
(`InsertLibraryItemRequested` → `InstantiateLibraryItem`) remains in the code
for a third-party item a client chooses to insert without a name, and no
standard entry point takes it.

**The Source sheet** (`ui/source_sheet.dart`): every Source entry point — _New
Source…_, a third-party preset in the menu, the Library's _Sources_ row's
double-click or Return, a drag of it onto the canvas — dispatches
`NewSourceRequested(presetId?, position?)`; the daemon ranks the concepts
(`ListSourceCandidates`) and the sheet opens over the design with one choice at
its centre — **Existing concept** (a pop-up of the design's concepts, each with
its value form beside it so two of one form stay two) or **New concept** (the
concept sheet's form: name, category, the units as a fact, meaning) — then the
**Source name** and its meaning, a preview node, and the exact objects that will
be committed (`concept RoomTemperature : Temperature` /
`mapping roomTemperatureInput : () -> RoomTemperature`). _Create Source_ is
enabled once the choice is complete and dispatches `CreateSourceRequested`;
cancel dispatches nothing but `SourceSheetDismissed`. When the answer arrives, a
new concept lands where the pointer was and is selected, with its Source a node
width to the left; a Source over an existing concept lands where the pointer was
and is selected.

**Library tab** (left sidebar, beside _Project_) — browsing and discovery: a
search field (above), then _Recent_, **Values** (the four forms), **Quantities**
(the vocabulary's order), a **Sources** row (the Source silhouette,
`MappingGlyph(source:)`; _a value the environment provides, over a concept you
choose_; opens the Source sheet) and one section per third-party library served.
A category row carries a grey socket glyph (filled = a value form, hollow =
_decide later_; grey because the hue is the identity the compiler will
allocate), the name, and in the right column the unit a value of it is written
in by default — the vocabulary's preferred unit, `rad`, `K`, `rad/s`, a fact —
or the form's word (`on–off`, `count`, `no unit`, `decide later`); the
description on hover; the full list of units is the sheet's. **Drag** a row onto
the canvas to open the sheet for the drop point; double-click or Return opens it
for an auto-placed concept.

Every Concept entry point dispatches the same `NewConceptRequested` and the
sheet performs the one creation; every Source entry point dispatches the same
`NewSourceRequested` and the Source sheet performs the one choice. The results
differ only in `ConceptId`, layout position and the name the designer gave.

## Textual surface and LSP

`bdl-ide` reads the same `LibrarySet` for completion: in a `concept Ang…`
position it offers `Angle : Angle` (every Concept item whose name or keyword
matches; a name already taken in the project ranks lower), as
`CompletionKind::Template` carrying the item id — the category's type name as
the default, for the designer to replace with the product's word. Accepting one
writes ordinary syntax — no metadata, no marker; the source stays plain BDL. A
Source item is not a concept completion: its relationship is ordinary syntax
too, written in the preferred spelling —
`mapping temperatureInput : () -> Temperature` — by whoever generates it
(Studio's write-back, the migration, an example). The LSP adapter renders a
completion as a class item labelled with the library id; the test asserts that
every Concept item of the embedded library is offered from the same data, never
from a second list.

## Protocol

| request                                                                                                                                                      | response                                                                                                                                | notes                                                                                                                                                                                                                                                                                                                                                        |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `ListLibraryItems` (0.17)                                                                                                                                    | `LibraryItemsResponse { libraries[] { id, name, schema_version, version, items[] LibraryItemView }, quantities[] }`                     | every item of every served library, `creates[] LibraryObjectView { kind, key, name, type_name, signature, description, representation?, unit }` in creation order with the default names, `concept?` the `ConceptTemplateView` of a Concept item; independent of any project; Studio asks once per connection                                                |
| `InstantiateLibraryItem { base_revision, item_id, names{key → name}, component? }` (0.17)                                                                    | `SystemEditApplied`                                                                                                                     | the daemon plans the fragment (free names, or `names` by key) and applies every step in one transaction — one revision, one history entry, the outcome with every `created_*`; nothing is applied when any step is refused; `edit.stale_revision`, `library.unknown_item`, `library.invalid_plan` (a key no earlier step created), `edit.invalid_name`       |
| `ListConceptTemplates` (0.5)                                                                                                                                 | `ConceptTemplatesResponse { libraries[] { …, templates[] }, quantities[] }`                                                             | the Concept items as `ConceptTemplateView`s — the legacy projection; a Source item is not listed                                                                                                                                                                                                                                                             |
| `InstantiateConceptTemplate { base_revision, template_id, name?, component? }` (0.5)                                                                         | `SystemEditApplied`                                                                                                                     | the same transaction with `names = { concept: name }`; `library.unknown_template`                                                                                                                                                                                                                                                                            |
| `CreateSource { base_revision, component?, source_name, source_description, existing_concept \| new_concept { name, description, representation? } }` (0.23) | `SystemEditApplied`                                                                                                                     | a Source over the chosen concept: one `CreateMapping` for an existing concept (by identity, `edit.unknown_concept` when not in the design in scope); `CreateConcept` then `CreateMapping` in one transaction for a new one — one revision, one history entry, nothing when either is refused; `edit.duplicate_*`, `edit.invalid_name`, `edit.stale_revision` |
| `ListSourceCandidates { revision, component?, item_id }` (0.23)                                                                                              | `SourceCandidatesResponse { revision, candidates[] { concept_id, preferred }, preset?, suggested_concept_name, suggested_source_name }` | the concepts of the design in scope a Source may be created over, the preset's value form first when `item_id` names a Source item, every concept in id order otherwise; the preset (`SourcePresetView`) and its names made free; `draft.stale_revision`, `library.unknown_item`                                                                             |

`LibraryItemView.preset` (0.23, a `SourcePresetView`: the suggested concept name
and description, its representation and type name, the unit, the suggested
Source name and description) is the preset view of a Source item, beside
`creates` (what the legacy path would make); the standard library serves none
since 2026-09-21 (ADR-0041); a third-party library may. The value categories a
concept is created from — with their unit candidates — are `ListValueCategories`
(0.27, `docs/spec/protocol.md`), asked once per connection beside
`ListLibraryItems`. `ConceptTemplateView.source_default_name`, `display_names`,
`descriptions` and `InstantiateConceptTemplateRequest.source_name` (0.16) are
**deprecated since 0.17** and never set: a Source is an item, not a template
field, and text is localized by the client. Protocol 0.5 for the template
surface; 0.16 for the deprecated fields; 0.17 for items; 0.23 for the Source
sheet.

## Future custom libraries

`LibrarySet` holds several libraries, searched together; ids stay unique because
each library prefixes its own (`team.`, `proj.`, a package name). Loading is one
call per file (`Library::from_toml`), and the one discovery rule today is the
environment: `BDL_LIBRARIES` names the files `bdld` serves beside the standard
library (`LibrarySet::from_environment`). A team library in the workspace, a
project library under `project/library/`, or an installed package library needs
a further discovery rule and nothing else — no package manager is implemented
and nothing assumes there is only one library.

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

| scenario                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | where                                                                                                                       |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| the embedded library loads: 22 items — the four forms, then one per named quantity in the vocabulary's order, ids `std.value.*` / `std.quantity.*`, groups `form` / `quantity`, no Source item; every quantity/unit resolves; search by synonym and unit (`lux`, `deg`, `rotation`, `brightness`, `button`); schema 1 still loads; a file from `BDL_LIBRARIES` is served beside it                                                                                                                                                                                                      | `bdl-library` unit tests                                                                                                    |
| (fixture library) a Source item plans a concept and a `() -> value` mapping; names free across concepts and mappings; chosen names by key; the template projection                                                                                                                                                                                                                                                                                                                                                                                                                      | `bdl-library` unit tests                                                                                                    |
| (fixture library) a Source item is a preset: names, value form and unit suggested, the design's concepts ranked with the preset's value form first, two of one form two candidates, an open one listed, nothing hidden; the item owns no identity                                                                                                                                                                                                                                                                                                                                       | `bdl-library` unit tests                                                                                                    |
| two instantiations → two `ConceptId`s, free names, independent defaults after renaming and rebinding                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | `bdl-library`, `bdld` stdio e2e, Studio e2e                                                                                 |
| a project persists and reloads without the library; a changed item default does not reach an existing object                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | `bdl-library`                                                                                                               |
| invalid libraries are refused with the reason (unit of the wrong dimension, unknown quantity, schema, a key no concept of the fragment has)                                                                                                                                                                                                                                                                                                                                                                                                                                             | `bdl-library`                                                                                                               |
| every Concept item is a textual completion, from the same data (`concept Ang…` → `Angle : Angle`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | `bdl-ide/tests/acceptance.rs`                                                                                               |
| the daemon serves the embedded library and the `BDL_LIBRARIES` fixture (`crates/bdl-daemon/tests/fixtures/presets.toml`, library `fx`) item for item, the legacy surface lists Concept items only with the 0.16 fields empty                                                                                                                                                                                                                                                                                                                                                            | `bdld/tests/stdio_e2e.rs`, `bdld/tests/text_e2e.rs`                                                                         |
| (legacy path) a Source item is one transaction: exactly one revision and one history entry; a refused step leaves no object, layout, history entry, dirtiness or consumed identity; a taken chosen name and an unknown key are refused whole; keys resolve inside a component body; the role is Source only while unresolved; the text view                                                                                                                                                                                                                                             | `bdld/tests/stdio_e2e.rs` (`library_items_over_stdio`, `a_library_transaction_is_all_or_nothing`), `bdld/tests/text_e2e.rs` |
| a Source is created over a chosen concept: existing — one edit, no new concept, the chosen identity never its twin, one Undo removes the Source alone; an open value form accepted; an unknown concept refused; new — one revision, both created, one Undo removes both, a taken name refuses the pair with no identity consumed; the candidates ranked and the suggestions free; the Source drives a sink (`DriveWF`) and is a simulation input; in a component body the body's concept, never the system's with the same number; saved as ordinary text, no `?`, reopened as a Source | `bdld/tests/system_e2e.rs` (`a_source_is_created_over_a_chosen_concept`)                                                    |
| right-click, a row and a drag are one creation path through the concept sheet; the sheet requires an identifier that is free, offers the category's units as a fact, previews the declaration, and creates one concept with the typed name where asked, selected and not renaming; cancel creates nothing; recents; localized search over synonyms and units; the Source placed left                                                                                                                                                                                                    | `apps/studio/test/library_test.dart`, `apps/studio/test/concept_sheet_test.dart`, `apps/studio/test/source_role_test.dart`  |
| the Source sheet: an existing concept describes one object and names the Source after it; an explicit name is sent as typed; the preset prefills the new concept and the preview lists both objects exactly; the preset never forces a new concept; cancel creates nothing; a suggested name is made free; a preset opens the sheet without a request to create; a Source over an existing concept lands where pointed, selected, not renamed                                                                                                                                           | `apps/studio/test/source_role_test.dart`, `apps/studio/test/library_test.dart`                                              |
| Studio against the real `bdld`: the 22 categories and the value categories with their composite units arrive; two concepts created from one category with two names, save, reopen; a Source over an existing concept and one with a new concept, cancel, undo/redo, a taken name refused, the text, save and reopen                                                                                                                                                                                                                                                                     | `apps/studio/test/library_e2e_test.dart`                                                                                    |
| the generated presentation strings follow the catalogue; an item without translations, an orphan entry or an empty field is refused; `--check` is read-only                                                                                                                                                                                                                                                                                                                                                                                                                             | `scripts/gen_library_l10n.py --check`, `scripts/test_gen_library_l10n.py` (preflight `l10n`)                                |
