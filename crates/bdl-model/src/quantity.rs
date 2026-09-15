//! The surface vocabulary of physical quantities: the names a designer
//! writes (`concept Tilt : Angle`, `concept AmbientLight : Illuminance`),
//! each with its dimension and its canonical unit symbol.
//!
//! This is the one table.  The textual syntax resolves representation
//! names here, the Standard Concept Library references quantities by their
//! id, Studio's unit picker and the protocol read it, and the unit table in
//! `bdl-elab` names its units against these dimensions.  A quantity name is
//! *representation* vocabulary — what a value is measured as — never a
//! semantic concept: `AmbientLight` is a concept, `Illuminance` is what it
//! is represented by.
//!
//! Angle is a base dimension in BDL (`Dim::ANGLE`), so a steradian is
//! `rad²` and illuminance is `cd·rad²·m⁻²`; torque and energy share `N·m`.

use crate::dim::Dim;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuantityDef {
    /// Stable lowercase id (`"illuminance"`), as referenced by libraries.
    pub id: &'static str,
    /// The textual type name (`Illuminance`).
    pub type_name: &'static str,
    /// The canonical unit symbol, for display (`lx`, `m/s`); empty for a
    /// dimensionless quantity.
    pub unit: &'static str,
    pub dim: Dim,
}

macro_rules! dim {
    ($($field:ident : $e:expr),* $(,)?) => {
        Dim { $($field: $e,)* ..Dim::ZERO }
    };
}

/// Every named quantity, base dimensions first, then the derived ones the
/// authoring vocabulary needs.
pub const QUANTITIES: &[QuantityDef] = &[
    QuantityDef {
        id: "scalar",
        type_name: "Scalar",
        unit: "",
        dim: Dim::ZERO,
    },
    QuantityDef {
        id: "angle",
        type_name: "Angle",
        unit: "rad",
        dim: Dim::ANGLE,
    },
    QuantityDef {
        id: "length",
        type_name: "Length",
        unit: "m",
        dim: Dim::LENGTH,
    },
    QuantityDef {
        id: "time",
        type_name: "Time",
        unit: "s",
        dim: Dim::TIME,
    },
    QuantityDef {
        id: "mass",
        type_name: "Mass",
        unit: "kg",
        dim: Dim::MASS,
    },
    QuantityDef {
        id: "current",
        type_name: "Current",
        unit: "A",
        dim: Dim::CURRENT,
    },
    QuantityDef {
        id: "temperature",
        type_name: "Temperature",
        unit: "K",
        dim: Dim::TEMPERATURE,
    },
    QuantityDef {
        id: "amount",
        type_name: "Amount",
        unit: "mol",
        dim: Dim::AMOUNT,
    },
    QuantityDef {
        id: "luminous_intensity",
        type_name: "Luminous",
        unit: "cd",
        dim: Dim::LUMINOUS,
    },
    QuantityDef {
        id: "speed",
        type_name: "Speed",
        unit: "m/s",
        dim: dim!(length: 1, time: -1),
    },
    QuantityDef {
        id: "acceleration",
        type_name: "Acceleration",
        unit: "m/s²",
        dim: dim!(length: 1, time: -2),
    },
    QuantityDef {
        id: "angular_velocity",
        type_name: "AngularVelocity",
        unit: "rad/s",
        dim: dim!(angle: 1, time: -1),
    },
    QuantityDef {
        id: "frequency",
        type_name: "Frequency",
        unit: "Hz",
        dim: dim!(time: -1),
    },
    QuantityDef {
        id: "force",
        type_name: "Force",
        unit: "N",
        dim: dim!(mass: 1, length: 1, time: -2),
    },
    QuantityDef {
        id: "pressure",
        type_name: "Pressure",
        unit: "Pa",
        dim: dim!(mass: 1, length: -1, time: -2),
    },
    QuantityDef {
        id: "torque",
        type_name: "Torque",
        unit: "N·m",
        dim: dim!(mass: 1, length: 2, time: -2),
    },
    QuantityDef {
        id: "power",
        type_name: "Power",
        unit: "W",
        dim: dim!(mass: 1, length: 2, time: -3),
    },
    QuantityDef {
        id: "voltage",
        type_name: "Voltage",
        unit: "V",
        dim: dim!(mass: 1, length: 2, time: -3, current: -1),
    },
    QuantityDef {
        id: "illuminance",
        type_name: "Illuminance",
        unit: "lx",
        dim: dim!(luminous: 1, angle: 2, length: -2),
    },
];

/// By library id (`"illuminance"`).
pub fn lookup(id: &str) -> Option<&'static QuantityDef> {
    QUANTITIES.iter().find(|q| q.id == id)
}

/// By textual type name (`"Illuminance"`).
pub fn by_type_name(name: &str) -> Option<&'static QuantityDef> {
    QUANTITIES.iter().find(|q| q.type_name == name)
}

/// The named quantity with exactly this dimension, if one exists.
pub fn by_dim(dim: Dim) -> Option<&'static QuantityDef> {
    QUANTITIES.iter().find(|q| q.dim == dim)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn ids_names_and_dims_are_unique() {
        let ids: BTreeSet<_> = QUANTITIES.iter().map(|q| q.id).collect();
        let names: BTreeSet<_> = QUANTITIES.iter().map(|q| q.type_name).collect();
        let dims: BTreeSet<_> = QUANTITIES.iter().map(|q| q.dim).collect();
        assert_eq!(ids.len(), QUANTITIES.len());
        assert_eq!(names.len(), QUANTITIES.len());
        assert_eq!(dims.len(), QUANTITIES.len(), "one name per dimension");
        assert_eq!(lookup("illuminance").map(|q| q.unit), Some("lx"));
        assert_eq!(by_type_name("Angle").map(|q| q.dim), Some(Dim::ANGLE));
        assert_eq!(by_dim(Dim::ZERO).map(|q| q.type_name), Some("Scalar"));
    }

    #[test]
    fn derived_dimensions_compose_from_base_ones() {
        let speed = lookup("speed").expect("speed").dim;
        assert_eq!(speed, Dim::LENGTH - Dim::TIME);
        let accel = lookup("acceleration").expect("acceleration").dim;
        assert_eq!(accel, speed - Dim::TIME);
        let force = lookup("force").expect("force").dim;
        assert_eq!(force, Dim::MASS + accel);
        let pressure = lookup("pressure").expect("pressure").dim;
        assert_eq!(pressure, force - Dim::LENGTH - Dim::LENGTH);
        let torque = lookup("torque").expect("torque").dim;
        assert_eq!(torque, force + Dim::LENGTH);
        let power = lookup("power").expect("power").dim;
        assert_eq!(power, torque - Dim::TIME);
        let voltage = lookup("voltage").expect("voltage").dim;
        assert_eq!(voltage, power - Dim::CURRENT);
        let lux = lookup("illuminance").expect("illuminance").dim;
        assert_eq!(
            lux,
            Dim::LUMINOUS + Dim::ANGLE + Dim::ANGLE - Dim::LENGTH - Dim::LENGTH
        );
    }
}
