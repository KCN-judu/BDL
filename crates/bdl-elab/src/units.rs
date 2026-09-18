//! The unit registry (FV Phase 10, `Surface/Units.lean`): a unit is a
//! stable id, a symbol, a dimension and a chart onto the canonical
//! magnitude.  A literal `90 deg` elaborates to a dimensioned literal in
//! SI base units (`q[rad]` with value 90·π/180) — `withUnit`, kernel
//! multiplication against the unit's constant; a coordinate in a unit is
//! the division; nothing about a unit reaches `Ty`, `Value` or the
//! runtime.  Linear charts only today (ISS-0004 / DI-7: °C and °F are
//! affine and are not registered); [`convert`] is the one conversion
//! operation and hides the chart's shape, so an affine chart is a new
//! variant of [`Chart`], not a new API.
//!
//! The symbol is presentation: identity is the id.  `in` (inch) is
//! spelled `inch` because `in` is the membership keyword.

use bdl_model::Dim;

/// How a coordinate in the unit maps onto the canonical magnitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Chart {
    /// `canonical = coordinate × factor`.
    Linear { factor: f64 },
}

pub struct UnitDef {
    /// Stable identity, never shown: `angle.deg`, `length.inch`.
    pub id: &'static str,
    /// The symbol as written in a formula.
    pub name: &'static str,
    pub dim: Dim,
    /// Multiply the written value by this to get the SI base value.
    pub factor: f64,
}

impl UnitDef {
    pub fn chart(&self) -> Chart {
        Chart::Linear {
            factor: self.factor,
        }
    }
    /// The canonical magnitude of `coordinate` in this unit (`withUnit`).
    pub fn to_canonical(&self, coordinate: f64) -> f64 {
        match self.chart() {
            Chart::Linear { factor } => coordinate * factor,
        }
    }
    /// The coordinate of a canonical magnitude in this unit (`inUnit`).
    pub fn from_canonical(&self, canonical: f64) -> f64 {
        match self.chart() {
            Chart::Linear { factor } => canonical / factor,
        }
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
/// `x` — the same physical quantity, re-expressed (`convert`: the
/// composition of `withUnit` and `inUnit`; FV `display_switch_preserves_quantity`).
/// `None` when the units measure different dimensions.
pub fn convert(x: f64, from: &UnitDef, to: &UnitDef) -> Option<f64> {
    if from.dim != to.dim {
        return None;
    }
    Some(to.from_canonical(from.to_canonical(x)))
}

pub const UNITS: &[UnitDef] = &[
    UnitDef {
        id: "angle.rad",
        name: "rad",
        dim: Dim::ANGLE,
        factor: 1.0,
    },
    UnitDef {
        id: "angle.deg",
        name: "deg",
        dim: Dim::ANGLE,
        factor: std::f64::consts::PI / 180.0,
    },
    UnitDef {
        id: "angle.turn",
        name: "turn",
        dim: Dim::ANGLE,
        factor: 2.0 * std::f64::consts::PI,
    },
    UnitDef {
        id: "time.s",
        name: "s",
        dim: Dim::TIME,
        factor: 1.0,
    },
    UnitDef {
        id: "time.ms",
        name: "ms",
        dim: Dim::TIME,
        factor: 1e-3,
    },
    UnitDef {
        id: "time.min",
        name: "min",
        dim: Dim::TIME,
        factor: 60.0,
    },
    UnitDef {
        id: "time.h",
        name: "h",
        dim: Dim::TIME,
        factor: 3600.0,
    },
    UnitDef {
        id: "length.m",
        name: "m",
        dim: Dim::LENGTH,
        factor: 1.0,
    },
    UnitDef {
        id: "length.mm",
        name: "mm",
        dim: Dim::LENGTH,
        factor: 1e-3,
    },
    UnitDef {
        id: "length.cm",
        name: "cm",
        dim: Dim::LENGTH,
        factor: 1e-2,
    },
    UnitDef {
        id: "length.km",
        name: "km",
        dim: Dim::LENGTH,
        factor: 1e3,
    },
    UnitDef {
        id: "length.inch",
        name: "inch",
        dim: Dim::LENGTH,
        factor: 0.0254,
    },
    UnitDef {
        id: "length.ft",
        name: "ft",
        dim: Dim::LENGTH,
        factor: 0.3048,
    },
    UnitDef {
        id: "mass.kg",
        name: "kg",
        dim: Dim::MASS,
        factor: 1.0,
    },
    UnitDef {
        id: "mass.g",
        name: "g",
        dim: Dim::MASS,
        factor: 1e-3,
    },
    UnitDef {
        id: "current.A",
        name: "A",
        dim: Dim::CURRENT,
        factor: 1.0,
    },
    UnitDef {
        id: "current.mA",
        name: "mA",
        dim: Dim::CURRENT,
        factor: 1e-3,
    },
    UnitDef {
        id: "temperature.K",
        name: "K",
        dim: Dim::TEMPERATURE,
        factor: 1.0,
    },
    UnitDef {
        id: "luminous.cd",
        name: "cd",
        dim: Dim::LUMINOUS,
        factor: 1.0,
    },
    UnitDef {
        id: "amount.mol",
        name: "mol",
        dim: Dim::AMOUNT,
        factor: 1.0,
    },
    // Derived units, dimensions from the shared quantity vocabulary
    // (`bdl_model::quantity`): one symbol per quantity that has one.
    UnitDef {
        id: "frequency.Hz",
        name: "Hz",
        dim: Dim {
            time: -1,
            ..Dim::ZERO
        },
        factor: 1.0,
    },
    UnitDef {
        id: "force.N",
        name: "N",
        dim: Dim {
            mass: 1,
            length: 1,
            time: -2,
            ..Dim::ZERO
        },
        factor: 1.0,
    },
    UnitDef {
        id: "pressure.Pa",
        name: "Pa",
        dim: Dim {
            mass: 1,
            length: -1,
            time: -2,
            ..Dim::ZERO
        },
        factor: 1.0,
    },
    UnitDef {
        id: "pressure.kPa",
        name: "kPa",
        dim: Dim {
            mass: 1,
            length: -1,
            time: -2,
            ..Dim::ZERO
        },
        factor: 1000.0,
    },
    UnitDef {
        id: "power.W",
        name: "W",
        dim: Dim {
            mass: 1,
            length: 2,
            time: -3,
            ..Dim::ZERO
        },
        factor: 1.0,
    },
    UnitDef {
        id: "voltage.V",
        name: "V",
        dim: Dim {
            mass: 1,
            length: 2,
            time: -3,
            current: -1,
            ..Dim::ZERO
        },
        factor: 1.0,
    },
    UnitDef {
        id: "voltage.mV",
        name: "mV",
        dim: Dim {
            mass: 1,
            length: 2,
            time: -3,
            current: -1,
            ..Dim::ZERO
        },
        factor: 0.001,
    },
    UnitDef {
        id: "illuminance.lx",
        name: "lx",
        dim: Dim {
            luminous: 1,
            angle: 2,
            length: -2,
            ..Dim::ZERO
        },
        factor: 1.0,
    },
];

pub fn lookup(name: &str) -> Option<&'static UnitDef> {
    UNITS.iter().find(|u| u.name == name)
}

pub fn names() -> Vec<&'static str> {
    UNITS.iter().map(|u| u.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_conversion_preserves_the_quantity() {
        let mut ids: Vec<&str> = UNITS.iter().map(|u| u.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), UNITS.len());
        let deg = lookup("deg").unwrap();
        let rad = lookup("rad").unwrap();
        let turn = lookup("turn").unwrap();
        assert!((convert(180.0, deg, rad).unwrap() - std::f64::consts::PI).abs() < 1e-12);
        assert!((convert(1.0, turn, deg).unwrap() - 360.0).abs() < 1e-9);
        assert_eq!(convert(1.0, deg, lookup("mm").unwrap()), None);
        // round trips within a few ulps (production is f64: the exact
        // laws hold in the formal model only)
        for u in UNITS {
            for v in units_for(u.dim) {
                let x = 12.5;
                let back = convert(convert(x, u, v).unwrap(), v, u).unwrap();
                assert!(
                    (back - x).abs() <= 1e-9 * x,
                    "{} → {} → {}",
                    u.name,
                    v.name,
                    u.name
                );
            }
        }
        assert_eq!(
            units_for(Dim::ANGLE)
                .iter()
                .map(|u| u.name)
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
                u.name
            );
        }
        for q in bdl_model::quantity::QUANTITIES {
            if q.unit.is_empty() || q.unit.contains('/') || q.unit.contains('·') {
                continue;
            }
            let u = lookup(q.unit).unwrap_or_else(|| panic!("no unit `{}`", q.unit));
            assert_eq!(u.dim, q.dim, "unit `{}` vs quantity `{}`", q.unit, q.id);
        }
    }
}
