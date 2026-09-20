---
id: ADR-0040
status: accepted
date: 2026-09-21
area: language
supersedes: []
superseded-by: []
related: [ADR-0028, ADR-0013, ISS-0004]
fv:
  [
    "informed by FV: the composite is again a `Unit K = ⟨id, dim, scale⟩` of FV
    Phase 10 (`Surface/Units.lean`), so every stated law (`withUnit_dim`,
    `inUnit_withUnit`, `convert_eq`, `convert_trans`, `unitsFor_sound`) holds of
    it; the composition equalities `dim(U * V) = dim U + dim V`, `scale(U per V)
    = s_U / s_V`, `dim(U^n) = n · dim U` are definitional over `Dim` (an abelian
    group, `Core/Base.lean`) and `Scalars` and are not stated as named theorems",
    "production-tested: `crates/bdl-model/tests/unit_expr.rs` (properties),
    `crates/bdl-elab/tests/dimension_algebra.rs` (the pipeline),
    `crates/bdl-syntax/tests/units.rs` (the grammar); the parser, formatter and
    normaliser are not Lean-verified",
  ]
---

# ADR-0040: Composite units are surface coordinates over registered linear atoms — `per`, `*` and `^n`, normalised to a dimension and a scale, never a type

## Status

Accepted (the composite-unit surface for authoring, 2026-09-21). Extends
ADR-0028's unit registry and charts; changes nothing in the kernel, the
dimension algebra, the chart semantics or the affine model (ISS-0004 stays as it
is).

## Context

The dimension algebra already existed end to end before this record:
`q[d₁] * q[d₂] : q[d₁ + d₂]`, `q[d₁] / q[d₂] : q[d₁ − d₂]` in the elaborator,
the checker and the evaluator, over `Dim` as a vector of signed integer
exponents — `Length * Length` is `L²`, `Force * Length` is a torque,
`Angle / Angle` cancels, and every named derived quantity of the vocabulary
equals its decomposition from base dimensions (audited first, systematically:
`crates/bdl-elab/tests/dimension_algebra.rs`). What was missing was only the
**unit surface**: a designer could write `90 deg` but not `180 deg per s` or
`9.81 m per s^2`; the vocabulary's preferred units were display strings (`m/s`,
`N·m`) with no meaning; a Source's concept was created from a preset. BDL_FV's
`Unit K` is closed under multiplication, division and integer powers by
construction, so the extension is a conservative surface over the existing model
— the formal gate passed without a new FV result, and this record says exactly
what is proved and what is tested.

## Decision

1. **A unit expression is a finite product of registered linear atoms with
   signed integer exponents**, normalised: `bdl_model::units::UnitExpr` —
   factors sorted in a canonical order (derived atoms first, then mass, length,
   time, current, temperature, amount, luminous, angle; the registry's order
   within one dimension), exponents of one atom summed, zeros dropped. It
   denotes a dimension `Σ eᵢ · dim(atomᵢ)` and a scale `Π scale(atomᵢ)^eᵢ` from
   the authored coordinate to the canonical magnitude. It is **never** a
   semantic type, a concept, a kernel primitive or a runtime value: a literal
   `9.81 m per s^2` elaborates to the same dimensioned literal an atom would
   (`withUnit`, ADR-0028).
2. **Semantics** for linear atoms `U = (dim_U, s_U)`, `V = (dim_V, s_V)`:
   `U * V = (dim_U + dim_V, s_U · s_V)`, `U per V = (dim_U − dim_V, s_U / s_V)`,
   `U^n = (n · dim_U, s_U^n)`; `U per U` is dimensionless with scale 1. The
   registry stays finite: no static row for `rad per s`, `mm per s`,
   `m per s^2`; composites are derived, and `N * m` and `kg * m^2 per s^2` are
   two factorisations of one physical unit (`UnitExpr::equivalent`: same
   dimension, same scale within ulps) without being the same value.
3. **The canonical textual spelling** is `per` for the quotient, `*` for the
   product and `^n` for an integer power: `180 deg per s`, `9.81 m per s^2`,
   `1 N * m`, `1 kg * m per s^2`, `2 s^-1`. The formatter writes one space
   around `per` and `*` and glues `^`. `/` remains the value-level division
   operator and is never a unit spelling: `Distance / Duration` is arithmetic,
   `10 m per s` is one literal. A tool may render `rad/s`, `m/s²`, `kg·m/s²`
   (`UnitExpr::display`), which is presentation and never source.
4. **Grammar**, after a number and only there (docs/spec/textual-syntax.md §6):
   `UnitSuffix ::= UnitProduct ("per" UnitProduct)?`,
   `UnitProduct ::= UnitFactor ("*" UnitFactor)*`,
   `UnitFactor ::= Ident ("^" "-"? Number)?`. `per` is a **contextual keyword**:
   an identifier everywhere else (a relationship named `per` stays legal), the
   quotient only inside a unit suffix, where a name can never be a reference.
   Everything after `per` is unit syntax. `*` continues the unit **only when the
   word after it is a registered atom symbol** (`1 N * m`); otherwise it is the
   value multiplication it always was (`90 deg * gain`, `2 m * width`) — the one
   place the parser consults the registry, so that no existing formula changes
   meaning. **One `per`**: `m per s per s` is refused with _write a power
   instead: `m per s^2`_ (the power spells the result more clearly;
   left-association would have been the only alternative and is not needed).
   Exponents are whole numbers; `^1.5`, `^` alone, `^^`, `^` beyond ±127 are
   `syntax.malformed_unit` with the designer's words.
5. **Bounds.** `Dim` keeps `i8` exponents; a factor's exponent and every
   combined exponent are checked against them (`MAX_EXPONENT = 127`,
   `UnitError::ExponentRange`); nothing wraps, saturates or truncates.
6. **Affine charts never enter the algebra.** `UnitExpr::atom` refuses an affine
   chart and `UnitExpr::symbol("°C")` names it (`UnitError::AffineAtom`,
   `formula.unit.affine`: _an absolute temperature unit cannot be multiplied,
   divided or raised to a power_) — a mathematical restriction, tested at the
   constructor before any affine symbol is surface-addressable. A temperature
   _difference_ representation, whose units would be linear, is a separate
   future question (ISS-0004).
7. **Angle stays a base dimension**: `rad per s` is `ANGLE − TIME`, never
   `TIME⁻¹`; `Hz` and `rad per s` are not equivalent; `deg per s` scales by
   π/180 against `rad per s`.
8. **The vocabulary's preferred units are canonical spellings**
   (`QuantityDef.unit = "m per s"`, `"rad per s"`, `"N * m"`), parsed by the
   same normaliser; a curated, bounded list of composites per named quantity
   (`quantity::preferred_units`: `rad per s`, `deg per s`, `turn per s`,
   `deg per min` for an angular velocity; `m per s`, `mm per s`, `km per h`;
   `m per s^2`, `mm per s^2`; `N * m`; `s^-1`) joins the registered atoms as the
   candidates for a dimension (`units::candidates_for`) — never the product of
   every compatible atom.
9. **The registry moves to `bdl-model::units`** (re-exported from
   `bdl-elab::units`) so the parser can consult the atom symbols; ids, symbols,
   charts and conversions are unchanged.
10. **The authoring vocabulary is compiler-owned on the wire**
    (`ListValueCategories`, protocol 0.27): every named quantity plus the truth
    value and the count, each with its id, display name, type name,
    representation, dimension, preferred unit and unit candidates as
    `UnitExprView` descriptors — type vocabulary, never a concept (`Angle` here;
    `LidAngle` in a design with its own identity). A new Source's concept may be
    created by category id (`NewConcept.category_id`); the daemon resolves the
    representation; a chosen unit is presentation and is never part of the
    concept (FVD-0105).

## Alternatives

- **`/` as the unit quotient** (`180 deg/s`) — rejected: `/` is the value
  division everywhere else, and `10 m / s` with a relationship named `s` would
  be ambiguous or need a whitespace rule; `per` reads as the designer's word and
  has no other meaning after a number.
- **A globally reserved `per`** — rejected: it would reinterpret an existing
  identifier; the contextual reading costs nothing because a name cannot follow
  a number.
- **Implicit juxtaposition `kg m`** — rejected: a second word after a unit is
  today an error a designer is told about; silence there would hide mistakes.
- **A static registry row per composite** — rejected: unbounded and meaningless;
  the algebra is finite over atoms.
- **Raw syntax-tree equality as physical equality** — rejected: `N * m` and
  `kg * m^2 per s^2` are one unit physically and two factorisations; the
  normalised factor list is the identity, the dimension and scale the
  equivalence.
- **A symbolic unit algebra (fractional or symbolic exponents, cancellation
  rules beyond exponent arithmetic)** — rejected: `Dim` is integral by decision,
  and nothing asked for more.
- **Changing `Dim` to a wider integer** to allow `^1000` — rejected: a power
  that large has no physical reading; the bound is diagnosed.

## Consequences

- `180 deg per s`, `1 turn per s`, `60 deg per min`, `1000 mm per s`,
  `36 km per h`, `60 m per min`, `9.81 m per s^2` elaborate to the dimensions
  and canonical magnitudes the algebra gives, through the parser, the
  elaborator, the checker and the evaluator (tested); `90 deg`, `2.5 s`,
  `300 lx`, `25.4 mm` mean exactly what they did.
- New:
  `bdl_model::units::{UnitExpr, UnitFactor, UnitError, MAX_EXPONENT, candidates_for, preferred_for}`,
  `quantity::{preferred_units, QuantityDef::display_name}`,
  `SyntaxKind::{Caret, UnitFactor}`, `SyntaxErrorCode::MalformedUnit`,
  `bdl_syntax::SurfaceUnitFactor` and `Unit::{numerator, denominator}`,
  diagnostics `formula.unit.affine`, `formula.unit.exponent`,
  `formula.unit.dimension`; `pretty::describe_dim` names every derived dimension
  of the vocabulary.
- The Formula view's structure (roles, locals, carets, keyboard insertion,
  completion at a caret, signature help, the render) lands in the same change
  and is recorded in `docs/architecture/ide-service.md`; it decides nothing
  mathematical.
- Not decided here: affine composition, a temperature-difference representation,
  fractional exponents, a preferred unit stored on a concept.
- Records: `docs/spec/textual-syntax.md` §6, `docs/spec/protocol.md`,
  `docs/architecture/ide-service.md`, `docs/project/formal-correspondence.md`,
  `docs/project/status.md`, a change fragment, the user guide's formula
  reference.
