//! Physical dimensions as exponent vectors.
//!
//! The kernel carries dimensions in the types of primitive operators
//! (`mul : q d₁ → q d₂ → q (d₁ + d₂)`); nothing else in typing mentions
//! them.  The Lean development uses three base dimensions "enough to test
//! the abstraction"; the production model uses the seven SI base
//! dimensions plus plane angle, which the paper keeps as a base dimension so
//! that `Tilt` and `MotorAngle` can share it.  See `docs/IR.md`.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct Dim {
    pub length: i8,
    pub mass: i8,
    pub time: i8,
    pub current: i8,
    pub temperature: i8,
    pub amount: i8,
    pub luminous: i8,
    pub angle: i8,
}

impl Dim {
    pub const ZERO: Dim = Dim {
        length: 0,
        mass: 0,
        time: 0,
        current: 0,
        temperature: 0,
        amount: 0,
        luminous: 0,
        angle: 0,
    };
    pub const LENGTH: Dim = Dim {
        length: 1,
        ..Dim::ZERO
    };
    pub const MASS: Dim = Dim {
        mass: 1,
        ..Dim::ZERO
    };
    pub const TIME: Dim = Dim {
        time: 1,
        ..Dim::ZERO
    };
    pub const CURRENT: Dim = Dim {
        current: 1,
        ..Dim::ZERO
    };
    pub const TEMPERATURE: Dim = Dim {
        temperature: 1,
        ..Dim::ZERO
    };
    pub const AMOUNT: Dim = Dim {
        amount: 1,
        ..Dim::ZERO
    };
    pub const LUMINOUS: Dim = Dim {
        luminous: 1,
        ..Dim::ZERO
    };
    pub const ANGLE: Dim = Dim {
        angle: 1,
        ..Dim::ZERO
    };

    pub const fn is_dimensionless(self) -> bool {
        self.length == 0
            && self.mass == 0
            && self.time == 0
            && self.current == 0
            && self.temperature == 0
            && self.amount == 0
            && self.luminous == 0
            && self.angle == 0
    }
}

impl Add for Dim {
    type Output = Dim;
    fn add(self, o: Dim) -> Dim {
        Dim {
            length: self.length + o.length,
            mass: self.mass + o.mass,
            time: self.time + o.time,
            current: self.current + o.current,
            temperature: self.temperature + o.temperature,
            amount: self.amount + o.amount,
            luminous: self.luminous + o.luminous,
            angle: self.angle + o.angle,
        }
    }
}

impl Sub for Dim {
    type Output = Dim;
    fn sub(self, o: Dim) -> Dim {
        Dim {
            length: self.length - o.length,
            mass: self.mass - o.mass,
            time: self.time - o.time,
            current: self.current - o.current,
            temperature: self.temperature - o.temperature,
            amount: self.amount - o.amount,
            luminous: self.luminous - o.luminous,
            angle: self.angle - o.angle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebra() {
        assert_eq!(
            Dim::LENGTH - Dim::TIME,
            Dim {
                length: 1,
                time: -1,
                ..Dim::ZERO
            }
        );
        assert_eq!((Dim::ANGLE - Dim::ANGLE), Dim::ZERO);
        assert!(Dim::ZERO.is_dimensionless());
        assert!(!Dim::ANGLE.is_dimensionless());
    }
}
