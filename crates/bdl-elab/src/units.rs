//! Surface units: a literal `90 deg` elaborates to a dimensioned literal in
//! SI base units (`q[rad]` with value 90·π/180).  Linear scaling only
//! (DESIGN_ISSUES DI-7: affine units such as °C are not modelled).

use bdl_model::Dim;

pub struct UnitDef {
    pub name: &'static str,
    pub dim: Dim,
    /// Multiply the written value by this to get the SI base value.
    pub factor: f64,
}

pub const UNITS: &[UnitDef] = &[
    UnitDef {
        name: "rad",
        dim: Dim::ANGLE,
        factor: 1.0,
    },
    UnitDef {
        name: "deg",
        dim: Dim::ANGLE,
        factor: std::f64::consts::PI / 180.0,
    },
    UnitDef {
        name: "s",
        dim: Dim::TIME,
        factor: 1.0,
    },
    UnitDef {
        name: "ms",
        dim: Dim::TIME,
        factor: 1e-3,
    },
    UnitDef {
        name: "min",
        dim: Dim::TIME,
        factor: 60.0,
    },
    UnitDef {
        name: "h",
        dim: Dim::TIME,
        factor: 3600.0,
    },
    UnitDef {
        name: "m",
        dim: Dim::LENGTH,
        factor: 1.0,
    },
    UnitDef {
        name: "mm",
        dim: Dim::LENGTH,
        factor: 1e-3,
    },
    UnitDef {
        name: "cm",
        dim: Dim::LENGTH,
        factor: 1e-2,
    },
    UnitDef {
        name: "km",
        dim: Dim::LENGTH,
        factor: 1e3,
    },
    UnitDef {
        name: "kg",
        dim: Dim::MASS,
        factor: 1.0,
    },
    UnitDef {
        name: "g",
        dim: Dim::MASS,
        factor: 1e-3,
    },
    UnitDef {
        name: "A",
        dim: Dim::CURRENT,
        factor: 1.0,
    },
    UnitDef {
        name: "mA",
        dim: Dim::CURRENT,
        factor: 1e-3,
    },
    UnitDef {
        name: "K",
        dim: Dim::TEMPERATURE,
        factor: 1.0,
    },
    UnitDef {
        name: "cd",
        dim: Dim::LUMINOUS,
        factor: 1.0,
    },
    UnitDef {
        name: "mol",
        dim: Dim::AMOUNT,
        factor: 1.0,
    },
    // Derived units, dimensions from the shared quantity vocabulary
    // (`bdl_model::quantity`): one symbol per quantity that has one.
    UnitDef {
        name: "Hz",
        dim: Dim {
            time: -1,
            ..Dim::ZERO
        },
        factor: 1.0,
    },
    UnitDef {
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
