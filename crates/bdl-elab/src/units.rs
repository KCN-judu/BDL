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
];

pub fn lookup(name: &str) -> Option<&'static UnitDef> {
    UNITS.iter().find(|u| u.name == name)
}

pub fn names() -> Vec<&'static str> {
    UNITS.iter().map(|u| u.name).collect()
}
