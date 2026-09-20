---
id: ADR-0041
status: accepted
date: 2026-09-21
area: studio
supersedes: []
superseded-by: []
related: [ADR-0040, ADR-0032, ADR-0031, ADR-0028, ISS-0019]
fv:
  [
    "engineering choice: an authoring-surface decision; nothing formal is
    claimed",
    "production-tested: `crates/bdl-library` unit tests,
    `crates/bdl-daemon/tests/{stdio_e2e,text_e2e,system_e2e}.rs`,
    `apps/studio/test/{library_test,library_e2e_test,concept_sheet_test}.dart`",
  ]
---

# ADR-0041: The Standard Library offers value categories, not product concepts — a concept is created from a category with the name it has in this product, and the units it may be written in are the compiler's fact, not a choice

## Status

Accepted (the authoring UX slice of 2026-09-21). Extends ADR-0040 (the
compiler-owned value categories on the wire) and ADR-0032 (a Source is a role
over a chosen concept); changes nothing in the kernel, the project model or the
protocol's library messages.

## Context

Until this record the Standard Library was 36 named concept presets in eight
groups (_Motor Angle_, _Servo Position_, _Fan Speed_, _Heater Power_, _Valve
Opening_, _Tilt_, _Dial Position_, …) plus eight Source presets, and inserting
one created a concept with the preset's name that the designer then renamed on
the canvas. Three problems were observed:

- **The presets pre-named the product.** _Motor Angle_, _Servo Position_ and
  _Tilt_ are one thing to the compiler — an angle — and _Fan Speed_, _Heater
  Power_, _Valve Opening_ and _Brightness_ are one thing — a level; they differ
  only in a name the library invented for a product it does not know. A designer
  looking for the angle of a lid found no _Lid Angle_ and took _Tilt_, then
  renamed it; the preset's name was never right and always had to be changed
  (create-then-rename was the routine, not the exception).
- **The list was not organised by what a value is.** Eight product-domain groups
  (_actuation_, _human_, _visual_, …) put the same value category in several
  groups and put values of different categories side by side, so the question
  the compiler cares about — what dimension, what unit — was answered by reading
  each row's right-hand column.
- **The unit column looked like a choice.** A preset carried one unit (`K`,
  `lx`, `rad`), and the row, the menu and the Source sheet showed it as if
  creating the concept chose it. A concept stores its **dimension** and no unit
  (`Representation::Quantity { dim }`, `docs/spec/kernel.md`): every unit of the
  dimension is legal in every formula over it, and there is no per-concept
  display unit in the project model. What the compiler does know — since
  ADR-0040 — is every unit a dimension may be written in, composites included,
  served as `ListValueCategories` with the preferred one first.

The constraints: the Library adds no type and no compiler rule
(`docs/spec/concept-library.md`); Studio decides nothing about dimensions or
units (ADR-0040, ADR-0028); a Source is created over a concept the designer
chooses (ADR-0032); item text is localized by the client (ADR-0031); the textual
completion `concept Ang…` keeps working from the same data.

## Decision

1. **A Standard Library item is a value category.** `library/std/concepts.toml`
   (version 0.3) holds 22 `[[template]]` items in two groups: the **value
   forms** — _On / off_ (`boolean`), _Count_, _Level_ (a dimensionless
   quantity), _Decide later_ (`open`) — and one **quantity** per named quantity
   of the shared vocabulary (`bdl_model::quantity`), in its order: _Angle_,
   _Length_, _Time_, _Mass_, _Current_, _Temperature_, _Amount of substance_,
   _Luminous intensity_, _Speed_, _Acceleration_, _Angular velocity_,
   _Frequency_, _Force_, _Pressure_, _Torque_, _Power_, _Voltage_,
   _Illuminance_. No item names a product concept; the old product names
   (_tilt_, _servo_, _encoder_, _heater_, _fan_, _dimmer_, _battery_,
   _humidity_) live on as **search synonyms** (`keywords`) of the category they
   were, so what a designer used to look for still finds it. There is **no
   Source item**: a Source is created on the Source sheet over a concept the
   designer chooses, existing or new from these same categories (ADR-0032). The
   schema (2), the loader, the fragment model and `InstantiateLibraryItem` are
   unchanged: a category is a Concept item whose `default_name` is the
   category's own word, used only by the legacy path and by the textual
   completion.
2. **A concept is created from a category with a required, product-specific
   name.** Every concept entry point in Studio — the canvas's _Add Concept ▸_,
   the Library tab's rows (double-click, Return, drag) and the Project tab's `+`
   — opens the **concept sheet** (`ui/concept_sheet.dart`) with the category
   preset: the **name** (required, an identifier, free in the design — refused
   before the daemon would refuse it), the **category** (one pop-up over the
   same items), the units it is **measured in**, the meaning, and a live preview
   of the node and the declaration the Code view will write. _Create_ sends one
   ordinary `CreateConcept` edit (`CreateConceptRequested`); the concept lands
   where the sheet was asked for, selected and named — never opened for
   renaming, because the name was given first. Cancel commits nothing.
   Create-then-rename is no longer a concept flow.
3. **The units a quantity is measured in are shown as a fact, never chosen.**
   The sheet's _Measured in_ row lists the compiler's unit candidates for the
   category's dimension (`ValueCategoryView.units`, `UnitExprView.display` —
   `rad`, `deg`, `turn`; `rad/s`, `deg/s`), preferred first, and the Library
   row's right column the vocabulary's preferred unit, as _what a formula over
   this concept may write_. Nothing is stored, nothing is sent: `CreateConcept`
   carries the representation and no unit. A per-concept display unit is a real
   need and an open question — ISS-0019 — not something Studio pretends to
   persist.
4. **Search is over what the compiler knows.** The Library tab's search
   (`searchItems`) matches the localized name and tags, the English name and
   synonyms, the category's type name, and the display and source spellings of
   its units (`deg` finds _Angle_; `lux` finds _Illuminance_; `rotation` finds
   _Angle_ and _Angular velocity_; `brightness` finds _Level_; `button` finds
   _On / off_). Nothing is inferred from a name: search ranks rows, it decides
   nothing.
5. **Libraries are discovered from the environment.** `LibrarySet::shared()`
   loads the embedded standard library and every file named in `BDL_LIBRARIES`
   (`std::env::split_paths`); a file that does not load is reported on stderr
   and skipped, never fatal. This is the discovery rule the spec's _Future
   custom libraries_ asked for, and it is how the daemon's tests keep their
   product presets (`crates/bdl-daemon/tests/fixtures/presets.toml`, library
   `fx`) after the standard library stopped carrying any.

## Alternatives

- **Keep the presets and add a category filter.** Rejected: a preset's name is
  always replaced, so it is not a default but a chore; and two lists (presets
  and categories) for one act of creation would have to be kept consistent by
  hand — the categories are already the compiler's (ADR-0040).
- **Let the sheet choose a unit and store it on the concept.** Rejected for this
  record: the project model has no such field and the Lean kernel has no such
  notion; storing it would be a schema change with a migration and a question
  about what a display unit means for a formula that writes another (ISS-0019).
  Showing the compiler's units as a fact answers the question the designer
  actually has — _what may I write?_ — without inventing a fact.
- **Drop `default_name` and the legacy `InstantiateLibraryItem`.** Rejected:
  third-party schema-2 libraries and the textual completion (`concept Ang…` →
  `Angle : Angle`) use them, and a protocol removal is not this record's
  business.
- **Organise categories by product domain after all** (_environment_, _motion_,
  …). Rejected: a category belongs to several domains; the compiler's
  organisation — value forms, then the vocabulary's order — is the one that is
  never wrong.

## Consequences

- The Library is 22 rows in two sections (_Values_, _Quantities_) plus a
  _Sources_ row that opens the Source sheet; the canvas's _Add Concept ▸_ menu
  is _Recent_, the four forms, _Quantities ▸_ and _More…_.
- Every concept a designer creates has the name they gave it; no project
  contains a `Temperature2` nobody asked for.
- `docs/spec/concept-library.md` (the items, the schema example, the Studio
  section, the tests), `docs/architecture/studio-ui.md` §2, the user guide's
  _Library_ page, `docs/project/status.md` and the change fragment
  `2026-09-authoring-ux.md` change with this record; `locale/library/std.json`
  is regenerated for the 22 ids. Screenshots of the old presets are recaptured.
- Third-party libraries keep working unchanged; a library that still ships
  product presets is shown under its own section, as before.
