//! The unit registry (FV Phase 10 `Surface/Units.lean`, Phase 10b
//! `Surface/Charts.lean`) and the composite unit expressions over it
//! (`docs/spec/textual-syntax.md` §5.2): a unit **atom** is a stable id, a
//! symbol, a dimension and a **chart** onto the canonical magnitude; a
//! **unit expression** is a finite product of linear atoms with signed
//! integer exponents — `m per s^2`, `kg * m per s^2`, `N * m` — whose
//! dimension and scale are computed, never registered ([`UnitExpr`]).  A literal `90 deg` elaborates
//! to a dimensioned literal in SI base units (`q[rad]` with value 90·π/180)
//! — `reconstruct`, the chart applied to the coordinate; a coordinate in a
//! unit is `coord`, the chart's inverse; nothing about a unit reaches `Ty`,
//! `Value` or the runtime.
//!
//! The chart owns the conversion semantics.  A linear chart is a scale; an
//! affine chart a scale and an offset (°C, °F against kelvin).  [`convert`]
//! is the one conversion operation — `coord_v (reconstruct_u x)` — and the
//! Phase-10b laws hold for both shapes: identity, composition, inverse,
//! the display switch preserving the quantity, the difference map linear
//! (`charts::tests`, within a few ulps; the formal laws are exact).  A
//! coordinate is a bare scalar: the chart that interprets it is supplied
//! where it is reinterpreted, never carried at runtime.
//!
//! The affine charts exist as tested infrastructure and are **not** in
//! [`UNITS`]: they are not offered to formulas or the Composer's pickers
//! (ISS-0004, ADR-0028 amendment) — conversion and display of an absolute
//! temperature are safe, arithmetic on absolute temperatures is a separate
//! validation concern (FV Phase 10 §10) — and they may never be a factor
//! of a unit expression ([`UnitExpr::atom`] refuses one: an affine chart
//! has no multiplicative algebra).  The symbol is presentation; identity
//! is the id.  `in` (inch) is spelled `inch` because `in` is the
//! membership keyword.
//!
//! Where the mathematics stands (docs/project/formal-correspondence.md):
//! the formal `Unit K = ⟨id, dim, scale⟩` is closed under the operations
//! below — `U * V`, `U per V` and `U^n` are each again a unit with the
//! dimension the group `Dim` gives and the scale the `Scalars` laws give —
//! and every Phase-10 law is stated for an arbitrary `Unit K`, so it holds
//! of a composite; the composition equalities themselves are definitional
//! and are **production-tested** here, not named theorems.

use crate::dim::Dim;

/// How a coordinate in the unit maps onto the canonical magnitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Chart {
    /// `canonical = coordinate × factor`.
    Linear { factor: f64 },
    /// `canonical = coordinate × scale + offset` (FV `Chart K = ⟨scale,
    /// offset⟩`, valid when `scale ≠ 0`).
    Affine { scale: f64, offset: f64 },
}

impl Chart {
    /// `reconstruct u x`: the canonical magnitude of coordinate `x`.
    pub fn reconstruct(self, x: f64) -> f64 {
        match self {
            Chart::Linear { factor } => x * factor,
            Chart::Affine { scale, offset } => x * scale + offset,
        }
    }
    /// `coord u q`: the coordinate of canonical magnitude `q`.
    pub fn coord(self, q: f64) -> f64 {
        match self {
            Chart::Linear { factor } => q / factor,
            Chart::Affine { scale, offset } => (q - offset) / scale,
        }
    }
    /// The linear part `L(u, v) = s_u / s_v`: what a *difference* of
    /// coordinates scales by between two charts (FV `difference_map`).
    pub fn linear_part(self, to: Chart) -> f64 {
        self.scale() / to.scale()
    }
    fn scale(self) -> f64 {
        match self {
            Chart::Linear { factor } => factor,
            Chart::Affine { scale, .. } => scale,
        }
    }
    pub fn is_affine(self) -> bool {
        matches!(self, Chart::Affine { .. })
    }
}

pub struct UnitDef {
    /// Stable identity, never shown: `angle.deg`, `length.inch`.
    pub id: &'static str,
    /// The symbol as written in a formula.
    pub symbol: &'static str,
    pub dim: Dim,
    pub chart: Chart,
}

impl UnitDef {
    /// The canonical magnitude of `coordinate` in this unit (`withUnit`,
    /// `reconstruct`).
    pub fn to_canonical(&self, coordinate: f64) -> f64 {
        self.chart.reconstruct(coordinate)
    }
    /// The coordinate of a canonical magnitude in this unit (`inUnit`,
    /// `coord`).
    pub fn from_canonical(&self, canonical: f64) -> f64 {
        self.chart.coord(canonical)
    }
}

const fn linear(id: &'static str, symbol: &'static str, dim: Dim, factor: f64) -> UnitDef {
    UnitDef {
        id,
        symbol,
        dim,
        chart: Chart::Linear { factor },
    }
}

/// Every registered unit of a dimension, in registry order
/// (`unitsFor`: sound and complete relative to the registry).
pub fn units_for(dim: Dim) -> Vec<&'static UnitDef> {
    UNITS.iter().filter(|u| u.dim == dim).collect()
}

pub fn by_id(id: &str) -> Option<&'static UnitDef> {
    UNITS.iter().find(|u| u.id == id)
}

/// The coordinate in `to` of the quantity whose coordinate in `from` is
/// `x` — the same physical quantity, re-expressed (`convert`: `coord_v ∘
/// reconstruct_u`; FV `display_switch_preserves_quantity`,
/// `convert_is_affine`).  `None` when the units measure different
/// dimensions.
pub fn convert(x: f64, from: &UnitDef, to: &UnitDef) -> Option<f64> {
    if from.dim != to.dim {
        return None;
    }
    Some(to.from_canonical(from.to_canonical(x)))
}

/// The affine temperature charts, kelvin canonical: `celsius = ⟨1,
/// 273.15⟩`, `fahrenheit = ⟨5/9, 459.67·5/9⟩` (FV `Charts.lean`).  Tested
/// infrastructure only — not in [`UNITS`], so a formula cannot write
/// `20 degC` and no picker offers it (see the module note).
pub const AFFINE_CHARTS: &[UnitDef] = &[
    UnitDef {
        id: "temperature.celsius",
        symbol: "°C",
        dim: Dim::TEMPERATURE,
        chart: Chart::Affine {
            scale: 1.0,
            offset: 273.15,
        },
    },
    UnitDef {
        id: "temperature.fahrenheit",
        symbol: "°F",
        dim: Dim::TEMPERATURE,
        chart: Chart::Affine {
            scale: 5.0 / 9.0,
            offset: 459.67 * 5.0 / 9.0,
        },
    },
];

pub const UNITS: &[UnitDef] = &[
    linear("angle.rad", "rad", Dim::ANGLE, 1.0),
    linear("angle.deg", "deg", Dim::ANGLE, std::f64::consts::PI / 180.0),
    linear("angle.turn", "turn", Dim::ANGLE, 2.0 * std::f64::consts::PI),
    linear("time.s", "s", Dim::TIME, 1.0),
    linear("time.ms", "ms", Dim::TIME, 1e-3),
    linear("time.min", "min", Dim::TIME, 60.0),
    linear("time.h", "h", Dim::TIME, 3600.0),
    linear("length.m", "m", Dim::LENGTH, 1.0),
    linear("length.mm", "mm", Dim::LENGTH, 1e-3),
    linear("length.cm", "cm", Dim::LENGTH, 1e-2),
    linear("length.km", "km", Dim::LENGTH, 1e3),
    linear("length.inch", "inch", Dim::LENGTH, 0.0254),
    linear("length.ft", "ft", Dim::LENGTH, 0.3048),
    linear("mass.kg", "kg", Dim::MASS, 1.0),
    linear("mass.g", "g", Dim::MASS, 1e-3),
    linear("current.A", "A", Dim::CURRENT, 1.0),
    linear("current.mA", "mA", Dim::CURRENT, 1e-3),
    linear("temperature.K", "K", Dim::TEMPERATURE, 1.0),
    linear("luminous.cd", "cd", Dim::LUMINOUS, 1.0),
    linear("amount.mol", "mol", Dim::AMOUNT, 1.0),
    // Derived units, dimensions from the shared quantity vocabulary
    // (`bdl_model::quantity`): one symbol per quantity that has one.
    linear(
        "frequency.Hz",
        "Hz",
        Dim {
            time: -1,
            ..Dim::ZERO
        },
        1.0,
    ),
    linear(
        "force.N",
        "N",
        Dim {
            mass: 1,
            length: 1,
            time: -2,
            ..Dim::ZERO
        },
        1.0,
    ),
    linear(
        "pressure.Pa",
        "Pa",
        Dim {
            mass: 1,
            length: -1,
            time: -2,
            ..Dim::ZERO
        },
        1.0,
    ),
    linear(
        "pressure.kPa",
        "kPa",
        Dim {
            mass: 1,
            length: -1,
            time: -2,
            ..Dim::ZERO
        },
        1000.0,
    ),
    linear(
        "power.W",
        "W",
        Dim {
            mass: 1,
            length: 2,
            time: -3,
            ..Dim::ZERO
        },
        1.0,
    ),
    linear(
        "voltage.V",
        "V",
        Dim {
            mass: 1,
            length: 2,
            time: -3,
            current: -1,
            ..Dim::ZERO
        },
        1.0,
    ),
    linear(
        "voltage.mV",
        "mV",
        Dim {
            mass: 1,
            length: 2,
            time: -3,
            current: -1,
            ..Dim::ZERO
        },
        0.001,
    ),
    linear(
        "illuminance.lx",
        "lx",
        Dim {
            luminous: 1,
            angle: 2,
            length: -2,
            ..Dim::ZERO
        },
        1.0,
    ),
];

/// The unit written with this symbol, if registered.
pub fn lookup(symbol: &str) -> Option<&'static UnitDef> {
    UNITS.iter().find(|u| u.symbol == symbol)
}

pub fn names() -> Vec<&'static str> {
    UNITS.iter().map(|u| u.symbol).collect()
}

// ---------------------------------------------------------------------------
// Composite unit expressions

/// One factor of a unit expression: a linear atom of the registry raised
/// to a signed integer power.
#[derive(Clone, Copy)]
pub struct UnitFactor {
    pub atom: &'static UnitDef,
    pub exponent: i8,
}

impl PartialEq for UnitFactor {
    fn eq(&self, other: &Self) -> bool {
        self.atom.id == other.atom.id && self.exponent == other.exponent
    }
}
impl Eq for UnitFactor {}

impl std::fmt::Debug for UnitFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}^{}", self.atom.symbol, self.exponent)
    }
}

/// A unit expression: a finite product of **linear** atoms with signed
/// integer exponents, normalised — factors sorted by atom id, exponents
/// of one atom summed, zero exponents dropped — so that two spellings of
/// one physical unit (`m * s per s`, `m`; `m^2 per m`, `m`) are one value,
/// while `N * m` and `kg * m^2 per s^2` stay two values that are
/// **physically equivalent** ([`UnitExpr::equivalent`]: same dimension,
/// same scale) without being the same factorisation.
///
/// Semantics, for linear atoms `U = (dim_U, s_U)`:
///
/// ```text
/// dim(U * V) = dim U + dim V        scale(U * V) = s_U · s_V
/// dim(U per V) = dim U − dim V      scale(U per V) = s_U / s_V
/// dim(U^n) = n · dim U              scale(U^n) = s_U ^ n
/// ```
///
/// The identity `U per U` is dimensionless with scale 1.  A unit
/// expression denotes a dimension and a scale from the authored coordinate
/// to the canonical magnitude — never a semantic type, a concept or a
/// kernel primitive: a literal `9.81 m per s^2` elaborates to the same
/// dimensioned literal `(9.81 · scale) : q[L T⁻²]` an atom would.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitExpr {
    factors: Vec<UnitFactor>,
}

/// Why a unit expression could not be built (the registry-level faults;
/// the parser has its own for malformed spellings).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitError {
    /// No registered unit has this symbol.
    UnknownAtom { symbol: String },
    /// The symbol names an affine chart (`°C`, `°F`): no multiplicative
    /// algebra exists for it.
    AffineAtom { symbol: String },
    /// An exponent, or a combined dimension exponent, leaves the range
    /// the dimension representation carries (`i8`).
    ExponentRange { symbol: String, exponent: i64 },
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitError::UnknownAtom { symbol } => write!(f, "`{symbol}` is not a unit"),
            UnitError::AffineAtom { symbol } => write!(
                f,
                "`{symbol}` is an absolute temperature unit and cannot be multiplied, divided or raised to a power"
            ),
            UnitError::ExponentRange { symbol, exponent } => write!(
                f,
                "`{symbol}^{exponent}` is beyond the unit powers that can be represented (±{})",
                i8::MAX
            ),
        }
    }
}

/// The largest source exponent a factor may carry: the dimension
/// representation is `i8`, and a factor's own exponent is checked against
/// it before any dimension arithmetic, so nothing wraps or saturates.
pub const MAX_EXPONENT: i64 = i8::MAX as i64;

impl UnitExpr {
    /// The dimensionless unit with scale 1 (the empty product).
    pub fn one() -> UnitExpr {
        UnitExpr {
            factors: Vec::new(),
        }
    }

    /// A single linear atom.  An affine chart is refused: it has no place
    /// in a multiplicative algebra (docs/spec/textual-syntax.md §5.2).
    pub fn atom(def: &'static UnitDef) -> Result<UnitExpr, UnitError> {
        if def.chart.is_affine() {
            return Err(UnitError::AffineAtom {
                symbol: def.symbol.to_owned(),
            });
        }
        Ok(UnitExpr {
            factors: vec![UnitFactor {
                atom: def,
                exponent: 1,
            }],
        })
    }

    /// The atom written with `symbol`, as a unit expression.
    pub fn symbol(symbol: &str) -> Result<UnitExpr, UnitError> {
        match lookup(symbol) {
            Some(def) => UnitExpr::atom(def),
            None => match AFFINE_CHARTS.iter().find(|u| u.symbol == symbol) {
                Some(_) => Err(UnitError::AffineAtom {
                    symbol: symbol.to_owned(),
                }),
                None => Err(UnitError::UnknownAtom {
                    symbol: symbol.to_owned(),
                }),
            },
        }
    }

    /// The factors, normalised: sorted by atom id, no zero exponent.
    pub fn factors(&self) -> &[UnitFactor] {
        &self.factors
    }

    /// The one atom of a single-atom expression with exponent 1 (`deg`,
    /// `m`): what the registry-keyed operations (`SetUnit` by id, the
    /// registry's `convert`) accept.
    pub fn as_atom(&self) -> Option<&'static UnitDef> {
        match self.factors.as_slice() {
            [UnitFactor { atom, exponent: 1 }] => Some(atom),
            _ => None,
        }
    }

    pub fn is_dimensionless(&self) -> bool {
        self.dim().is_dimensionless()
    }

    fn normalise(mut factors: Vec<UnitFactor>) -> Result<UnitExpr, UnitError> {
        factors.sort_by_key(|f| canonical_rank(f.atom));
        let mut out: Vec<UnitFactor> = Vec::new();
        for f in factors {
            match out.last_mut() {
                Some(last) if last.atom.id == f.atom.id => {
                    last.exponent = last.exponent.checked_add(f.exponent).ok_or_else(|| {
                        UnitError::ExponentRange {
                            symbol: f.atom.symbol.to_owned(),
                            exponent: i64::from(last.exponent) + i64::from(f.exponent),
                        }
                    })?;
                }
                _ => out.push(f),
            }
        }
        out.retain(|f| f.exponent != 0);
        Ok(UnitExpr { factors: out })
    }

    /// `U * V`.
    pub fn mul(&self, other: &UnitExpr) -> Result<UnitExpr, UnitError> {
        let mut factors = self.factors.clone();
        factors.extend(other.factors.iter().copied());
        UnitExpr::normalise(factors)
    }

    /// `U per V`.
    pub fn per(&self, other: &UnitExpr) -> Result<UnitExpr, UnitError> {
        self.mul(&other.pow(-1)?)
    }

    /// `U^n`, `n` a signed integer within [`MAX_EXPONENT`].
    pub fn pow(&self, n: i64) -> Result<UnitExpr, UnitError> {
        if n == 0 {
            return Ok(UnitExpr::one());
        }
        let mut factors = Vec::with_capacity(self.factors.len());
        for f in &self.factors {
            let e = i64::from(f.exponent) * n;
            if e.abs() > MAX_EXPONENT {
                return Err(UnitError::ExponentRange {
                    symbol: f.atom.symbol.to_owned(),
                    exponent: e,
                });
            }
            factors.push(UnitFactor {
                atom: f.atom,
                exponent: e as i8,
            });
        }
        UnitExpr::normalise(factors)
    }

    /// The dimension: `Σ eᵢ · dim(atomᵢ)`, in the group `Dim`.  Every
    /// atom's dimension has exponents in `−3 ..= 3`, and a factor's own
    /// exponent is bounded, so the sum is computed with checked arithmetic
    /// and a range fault is a [`UnitError::ExponentRange`] at construction
    /// — `dim` itself cannot fail once an expression exists.
    pub fn dim(&self) -> Dim {
        self.factors
            .iter()
            .fold(Dim::ZERO, |acc, f| acc + scale_dim(f.atom.dim, f.exponent))
    }

    /// The scale from a coordinate in this unit to the canonical
    /// magnitude: `Π scale(atomᵢ) ^ eᵢ`.
    pub fn scale(&self) -> f64 {
        self.factors
            .iter()
            .map(|f| f.atom.chart.linear_factor().powi(i32::from(f.exponent)))
            .product()
    }

    /// The canonical magnitude of `coordinate` (`reconstruct`).
    pub fn to_canonical(&self, coordinate: f64) -> f64 {
        coordinate * self.scale()
    }

    /// The coordinate of a canonical magnitude (`coord`).
    pub fn from_canonical(&self, canonical: f64) -> f64 {
        canonical / self.scale()
    }

    /// Physical equivalence: the same dimension and the same scale — `N *
    /// m` and `kg * m^2 per s^2`, `Hz` and `s^-1` — whatever the
    /// factorisation.  Scales compare within a few ulps of their
    /// magnitude (the production `f64` boundary).
    pub fn equivalent(&self, other: &UnitExpr) -> bool {
        if self.dim() != other.dim() {
            return false;
        }
        let (a, b) = (self.scale(), other.scale());
        (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
    }

    /// The canonical BDL spelling (`docs/spec/textual-syntax.md` §5.2):
    /// the positive factors joined by ` * `, then ` per ` and the negative
    /// ones with their exponents made positive; a factor's power as `^n`;
    /// an expression with no positive factor spells its negative powers
    /// (`s^-1`); the empty product is `1`.  What the formatter and every
    /// generated source write; what [`UnitExpr::parse_canonical`] reads
    /// back.
    pub fn source(&self) -> String {
        let (num, den): (Vec<_>, Vec<_>) = self.factors.iter().partition(|f| f.exponent > 0);
        let factor = |f: &UnitFactor, e: i8| {
            if e == 1 {
                f.atom.symbol.to_owned()
            } else {
                format!("{}^{e}", f.atom.symbol)
            }
        };
        if num.is_empty() && den.is_empty() {
            return "1".into();
        }
        if num.is_empty() {
            return den
                .iter()
                .map(|f| factor(f, f.exponent))
                .collect::<Vec<_>>()
                .join(" * ");
        }
        let mut s = num
            .iter()
            .map(|f| factor(f, f.exponent))
            .collect::<Vec<_>>()
            .join(" * ");
        if !den.is_empty() {
            s.push_str(" per ");
            s.push_str(
                &den.iter()
                    .map(|f| factor(f, -f.exponent))
                    .collect::<Vec<_>>()
                    .join(" * "),
            );
        }
        s
    }

    /// The mathematical rendering for a display — presentation, never
    /// source: `·` between factors, `/` before the denominator,
    /// superscript digits (`rad/s`, `m/s²`, `kg·m/s²`, `s⁻¹`).
    pub fn display(&self) -> String {
        fn sup(n: i8) -> String {
            let digits = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
            let mut s = String::new();
            if n < 0 {
                s.push('⁻');
            }
            for c in n.unsigned_abs().to_string().chars() {
                s.push(digits[c.to_digit(10).unwrap_or(0) as usize]);
            }
            s
        }
        let (num, den): (Vec<_>, Vec<_>) = self.factors.iter().partition(|f| f.exponent > 0);
        let factor = |f: &UnitFactor, e: i8| {
            if e == 1 {
                f.atom.symbol.to_owned()
            } else {
                format!("{}{}", f.atom.symbol, sup(e))
            }
        };
        if num.is_empty() && den.is_empty() {
            return "1".into();
        }
        if num.is_empty() {
            return den
                .iter()
                .map(|f| factor(f, f.exponent))
                .collect::<Vec<_>>()
                .join("·");
        }
        let mut s = num
            .iter()
            .map(|f| factor(f, f.exponent))
            .collect::<Vec<_>>()
            .join("·");
        if !den.is_empty() {
            s.push('/');
            s.push_str(
                &den.iter()
                    .map(|f| factor(f, -f.exponent))
                    .collect::<Vec<_>>()
                    .join("·"),
            );
        }
        s
    }

    /// Read a canonical spelling (`m per s^2`, `kg * m per s^2`, `N * m`,
    /// `s^-1`, `deg`): one `per` at most, `*` between factors, `^n` for a
    /// power.  The registry's own parser, for the vocabulary's preferred
    /// units and the wire; the textual language's parser is `bdl-syntax`,
    /// which builds the same value through [`UnitExpr::build`].
    pub fn parse_canonical(text: &str) -> Result<UnitExpr, UnitError> {
        let text = text.trim();
        if text == "1" || text.is_empty() {
            return Ok(UnitExpr::one());
        }
        let mut sides = text.split(" per ");
        let num = sides.next().unwrap_or("");
        let den = sides.next();
        if sides.next().is_some() {
            return Err(UnitError::UnknownAtom {
                symbol: text.to_owned(),
            });
        }
        let product = |side: &str| -> Result<UnitExpr, UnitError> {
            let mut acc = UnitExpr::one();
            for factor in side.split(" * ") {
                let (symbol, exponent) = match factor.split_once('^') {
                    Some((s, e)) => (
                        s.trim(),
                        e.trim()
                            .parse::<i64>()
                            .map_err(|_| UnitError::UnknownAtom {
                                symbol: factor.to_owned(),
                            })?,
                    ),
                    None => (factor.trim(), 1),
                };
                acc = acc.mul(&UnitExpr::symbol(symbol)?.pow(exponent)?)?;
            }
            Ok(acc)
        };
        let n = product(num)?;
        match den {
            Some(d) => n.per(&product(d)?),
            None => Ok(n),
        }
    }

    /// Build from parsed factors: `(symbol, exponent)` pairs for the
    /// numerator and the denominator, each checked against the registry
    /// and the exponent bound; the first fault is returned with the index
    /// of the factor it is about (numerator factors first).
    pub fn build(
        numerator: &[(String, i64)],
        denominator: &[(String, i64)],
    ) -> Result<UnitExpr, (usize, UnitError)> {
        let mut acc = UnitExpr::one();
        for (i, (symbol, exponent)) in numerator
            .iter()
            .map(|f| (f, 1i64))
            .chain(denominator.iter().map(|f| (f, -1i64)))
            .enumerate()
            .map(|(i, ((s, e), sign))| (i, (s, e * sign)))
        {
            if exponent.abs() > MAX_EXPONENT {
                return Err((
                    i,
                    UnitError::ExponentRange {
                        symbol: symbol.clone(),
                        exponent,
                    },
                ));
            }
            let f = UnitExpr::symbol(symbol)
                .and_then(|u| u.pow(exponent))
                .map_err(|e| (i, e))?;
            acc = acc.mul(&f).map_err(|e| (i, e))?;
        }
        Ok(acc)
    }
}

/// The order factors are normalised and spelled in: the derived atoms
/// (`N`, `Pa`, …) first, then the base dimensions in SI order — mass,
/// length, time, current, temperature, amount, luminous — with angle
/// last, and within one dimension the registry's order.  So `kg * m per
/// s^2`, `N * m`, `rad per s`.  A total order over atom ids: two
/// expressions with the same factors normalise to the same value.
fn canonical_rank(atom: &UnitDef) -> (u8, usize) {
    let d = atom.dim;
    let base = [
        d.mass,
        d.length,
        d.time,
        d.current,
        d.temperature,
        d.amount,
        d.luminous,
        d.angle,
    ];
    let nonzero = base.iter().filter(|e| **e != 0).count();
    let single = base.iter().position(|e| *e == 1);
    let class = match (nonzero, single) {
        (1, Some(i)) => 1 + i as u8,
        _ => 0,
    };
    let index = UNITS
        .iter()
        .chain(AFFINE_CHARTS)
        .position(|u| u.id == atom.id)
        .unwrap_or(usize::MAX);
    (class, index)
}

/// `n · d` in the group `Dim`; the callers bound `n` and the atoms'
/// exponents so the product fits.
fn scale_dim(d: Dim, n: i8) -> Dim {
    Dim {
        length: d.length * n,
        mass: d.mass * n,
        time: d.time * n,
        current: d.current * n,
        temperature: d.temperature * n,
        amount: d.amount * n,
        luminous: d.luminous * n,
        angle: d.angle * n,
    }
}

impl Chart {
    /// The scale of a linear chart; an affine chart's linear part.  Only
    /// linear atoms enter a [`UnitExpr`], where this is the whole chart.
    pub fn linear_factor(self) -> f64 {
        self.scale()
    }
}

/// The unit expressions a designer is offered for a dimension: every
/// registered atom of the dimension, then the curated composites of the
/// named quantity of that dimension (`rad per s`, `deg per s`, `turn per
/// s` for an angular velocity; `m per s`, `mm per s`, `km per h` for a
/// speed) — a bounded list, never the product of every compatible atom.
pub fn candidates_for(dim: Dim) -> Vec<UnitExpr> {
    let mut out: Vec<UnitExpr> = units_for(dim)
        .into_iter()
        .filter_map(|u| UnitExpr::atom(u).ok())
        .collect();
    if let Some(q) = crate::quantity::by_dim(dim) {
        for spelling in crate::quantity::preferred_units(q.id) {
            if let Ok(u) = UnitExpr::parse_canonical(spelling) {
                debug_assert_eq!(u.dim(), dim, "{spelling} is not a {}", q.id);
                if !out.contains(&u) {
                    out.push(u);
                }
            }
        }
    }
    out
}

/// The preferred unit of a dimension: the named quantity's, when the
/// vocabulary names one; else the first registered atom; else none.
pub fn preferred_for(dim: Dim) -> Option<UnitExpr> {
    if let Some(q) = crate::quantity::by_dim(dim) {
        if let Ok(u) = UnitExpr::parse_canonical(q.unit) {
            return Some(u);
        }
    }
    units_for(dim)
        .into_iter()
        .find_map(|u| UnitExpr::atom(u).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-9 * b.abs().max(1.0)
    }

    #[test]
    fn ids_and_symbols_are_unique_and_conversion_preserves_the_quantity() {
        let mut ids: Vec<&str> = UNITS.iter().map(|u| u.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), UNITS.len());
        let mut symbols: Vec<&str> = UNITS.iter().map(|u| u.symbol).collect();
        symbols.sort_unstable();
        symbols.dedup();
        assert_eq!(symbols.len(), UNITS.len());
        assert!(
            UNITS.iter().all(|u| !u.chart.is_affine()),
            "no affine unit is offered"
        );
        let deg = lookup("deg").unwrap();
        let rad = lookup("rad").unwrap();
        let turn = lookup("turn").unwrap();
        assert!(close(
            convert(180.0, deg, rad).unwrap(),
            std::f64::consts::PI
        ));
        assert!(close(convert(1.0, turn, deg).unwrap(), 360.0));
        assert_eq!(convert(1.0, deg, lookup("mm").unwrap()), None);
        assert_eq!(
            units_for(Dim::ANGLE)
                .iter()
                .map(|u| u.symbol)
                .collect::<Vec<_>>(),
            vec!["rad", "deg", "turn"]
        );
    }

    #[test]
    fn every_unit_measures_a_named_quantity() {
        for u in UNITS {
            assert!(
                crate::quantity::by_dim(u.dim).is_some(),
                "unit `{}` has a dimension no quantity names",
                u.symbol
            );
        }
        for q in crate::quantity::QUANTITIES {
            if q.unit.is_empty() {
                continue;
            }
            let u = UnitExpr::parse_canonical(q.unit)
                .unwrap_or_else(|e| panic!("quantity `{}`: {e}", q.id));
            assert_eq!(u.dim(), q.dim, "unit `{}` vs quantity `{}`", q.unit, q.id);
            assert_eq!(u.source(), q.unit, "`{}` is not canonical", q.unit);
        }
    }

    /// The Phase-10b chart laws on production `f64`, for every registered
    /// chart and the affine ones (the formal laws are exact; here within a
    /// few ulps scaled by the magnitude, as `UNITS_NOTE.md` §17.8 prescribes).
    #[test]
    fn chart_laws_hold_within_ulps_for_linear_and_affine_charts() {
        let all: Vec<&UnitDef> = UNITS.iter().chain(AFFINE_CHARTS).collect();
        let samples = [-40.0, -1.5, 0.0, 0.25, 1.0, 12.5, 100.0, 1e4];
        for u in &all {
            for &x in &samples {
                // chart left inverse: coord (reconstruct x) = x
                assert!(
                    close(u.from_canonical(u.to_canonical(x)), x),
                    "{}",
                    u.symbol
                );
                // identity: C(u,u) = id
                assert!(close(convert(x, u, u).unwrap(), x), "{}", u.symbol);
                for v in all.iter().filter(|v| v.dim == u.dim) {
                    let y = convert(x, u, v).unwrap();
                    // inverse: C(v,u) ∘ C(u,v) = id
                    assert!(
                        close(convert(y, v, u).unwrap(), x),
                        "{} → {}",
                        u.symbol,
                        v.symbol
                    );
                    // the display switch preserves the quantity
                    assert!(
                        close(v.to_canonical(y), u.to_canonical(x)),
                        "{} → {}",
                        u.symbol,
                        v.symbol
                    );
                    // composition: C(v,w) ∘ C(u,v) = C(u,w)
                    for w in all.iter().filter(|w| w.dim == u.dim) {
                        assert!(
                            close(convert(y, v, w).unwrap(), convert(x, u, w).unwrap()),
                            "{} → {} → {}",
                            u.symbol,
                            v.symbol,
                            w.symbol
                        );
                    }
                    // the difference map is the linear part: offsets cancel
                    let d = 7.25;
                    assert!(
                        close(
                            convert(x + d, u, v).unwrap() - y,
                            u.chart.linear_part(v.chart) * d
                        ),
                        "{} → {} difference",
                        u.symbol,
                        v.symbol
                    );
                }
            }
        }
    }

    /// FV `CtoF_closed`, `FtoC_closed`, `exA`–`exG`: Celsius and Fahrenheit
    /// against kelvin, exactly the textbook values.
    #[test]
    fn celsius_and_fahrenheit_convert_as_the_formal_charts_say() {
        let c = &AFFINE_CHARTS[0];
        let f = &AFFINE_CHARTS[1];
        let k = lookup("K").unwrap();
        assert!(close(convert(0.0, c, f).unwrap(), 32.0));
        assert!(close(convert(100.0, c, f).unwrap(), 212.0));
        assert!(close(convert(-40.0, c, f).unwrap(), -40.0));
        assert!(close(convert(212.0, f, c).unwrap(), 100.0));
        assert!(close(convert(0.0, c, k).unwrap(), 273.15));
        assert!(close(convert(32.0, f, k).unwrap(), 273.15));
        // C(°C,°F)(x) = 9/5·x + 32; the linear part 9/5; Δ10 °C = Δ18 °F
        for x in [-30.0, 5.5, 37.0] {
            assert!(close(convert(x, c, f).unwrap(), 1.8 * x + 32.0));
        }
        assert!(close(c.chart.linear_part(f.chart), 1.8));
        assert!(close(
            convert(30.0, c, f).unwrap() - convert(20.0, c, f).unwrap(),
            18.0
        ));
        // not additive: C(°C,°F)(a + b) ≠ C(a) + C(b) when the offset is not 0
        assert!(!close(
            convert(10.0 + 10.0, c, f).unwrap(),
            convert(10.0, c, f).unwrap() + convert(10.0, c, f).unwrap()
        ));
        // affine charts are infrastructure: no formula symbol resolves to them
        assert!(lookup("°C").is_none() && by_id("temperature.celsius").is_none());
    }
}
