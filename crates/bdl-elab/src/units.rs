//! The unit registry (FV Phase 10 `Surface/Units.lean`, Phase 10b
//! `Surface/Charts.lean`): a unit is a stable id, a symbol, a dimension and
//! a **chart** onto the canonical magnitude.  A literal `90 deg` elaborates
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
//! validation concern (FV Phase 10 §10).  The symbol is presentation;
//! identity is the id.  `in` (inch) is spelled `inch` because `in` is the
//! membership keyword.

use bdl_model::Dim;

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
                bdl_model::quantity::by_dim(u.dim).is_some(),
                "unit `{}` has a dimension no quantity names",
                u.symbol
            );
        }
        for q in bdl_model::quantity::QUANTITIES {
            if q.unit.is_empty() || q.unit.contains('/') || q.unit.contains('·') {
                continue;
            }
            let u = lookup(q.unit).unwrap_or_else(|| panic!("no unit `{}`", q.unit));
            assert_eq!(u.dim, q.dim, "unit `{}` vs quantity `{}`", u.symbol, q.id);
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
