//! The runtime vocabulary every generated BDL semantic core is written
//! against.  `no_std`, allocation-free, `unsafe`-free, and ignorant of any
//! board, device kind, transport or editor: it knows *slots* (clock, input,
//! state, output), *active domains*, *checked numerics* and *structured
//! runtime errors* — nothing else.
//!
//! The numeric helpers implement exactly the reference evaluator's policy
//! (`docs/spec/runtime-semantics.md`, DI-15): IEEE `f64`; division by zero and
//! any non-finite result are errors that fail the tick.  Generated code
//! calls these helpers instead of bare operators so that the policy lives
//! in one place on both sides of the differential tests.

#![no_std]
#![forbid(unsafe_code)]

/// Dense index of a clock domain inside one generated program.  The
/// nominal `ClockId` it stands for is recorded in the program's manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockSlot(pub u16);

/// Which domains are active at the current global tick.  A bitset over
/// clock slots; at most [`ActiveDomains::CAPACITY`] domains per program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ActiveDomains(u64);

impl ActiveDomains {
    pub const CAPACITY: u16 = 64;

    pub const fn none() -> ActiveDomains {
        ActiveDomains(0)
    }

    pub const fn from_bits(bits: u64) -> ActiveDomains {
        ActiveDomains(bits)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Activate `slot`.  Slots at or beyond the capacity are ignored; a
    /// program with that many domains is refused by the compiler.
    pub const fn with(self, slot: ClockSlot) -> ActiveDomains {
        if slot.0 >= Self::CAPACITY {
            self
        } else {
            ActiveDomains(self.0 | (1u64 << slot.0))
        }
    }

    pub const fn is_active(self, slot: ClockSlot) -> bool {
        slot.0 < Self::CAPACITY && (self.0 >> slot.0) & 1 == 1
    }

    /// Whether any domain is active — what a domain-agnostic declaration
    /// asks before it is evaluated.
    pub const fn any(self) -> bool {
        self.0 != 0
    }
}

/// A structured failure of one tick.  `decl` is the raw `DeclId` of the
/// declaration being computed when it happened — the same attribution the
/// reference evaluator makes.  The global tick is added by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    /// An unresolved declaration was due this tick and no input was given.
    MissingInput {
        decl: u64,
    },
    DivisionByZero {
        decl: u64,
    },
    /// A primitive produced NaN or ±∞.
    NonFinite {
        decl: u64,
        op: &'static str,
    },
    /// A declaration was read before it was evaluated this tick.  Clock
    /// consistency rules this out; it is here so generated code never
    /// panics.
    NotEvaluated {
        decl: u64,
    },
}

impl core::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RuntimeError::MissingInput { decl } => {
                write!(f, "no input value for unresolved declaration decl#{decl}")
            }
            RuntimeError::DivisionByZero { decl } => write!(f, "division by zero in decl#{decl}"),
            RuntimeError::NonFinite { decl, op } => {
                write!(f, "{op} produced a non-finite number in decl#{decl}")
            }
            RuntimeError::NotEvaluated { decl } => {
                write!(f, "decl#{decl} read before it was evaluated this tick")
            }
        }
    }
}

/// Checked arithmetic: the reference evaluator's `apply_prim`, operator by
/// operator.  Every result is checked for finiteness; division checks the
/// divisor for exact zero first.
pub mod num {
    use super::RuntimeError;

    #[inline]
    fn finite(x: f64, decl: u64, op: &'static str) -> Result<f64, RuntimeError> {
        if x.is_finite() {
            Ok(x)
        } else {
            Err(RuntimeError::NonFinite { decl, op })
        }
    }

    #[inline]
    pub fn add(a: f64, b: f64, decl: u64) -> Result<f64, RuntimeError> {
        finite(a + b, decl, "add")
    }
    #[inline]
    pub fn sub(a: f64, b: f64, decl: u64) -> Result<f64, RuntimeError> {
        finite(a - b, decl, "sub")
    }
    #[inline]
    pub fn mul(a: f64, b: f64, decl: u64) -> Result<f64, RuntimeError> {
        finite(a * b, decl, "mul")
    }
    #[inline]
    pub fn div(a: f64, b: f64, decl: u64) -> Result<f64, RuntimeError> {
        if b == 0.0 {
            return Err(RuntimeError::DivisionByZero { decl });
        }
        finite(a / b, decl, "div")
    }
}

/// Primitive helpers whose only job is to keep the reference evaluator's
/// *strict* argument evaluation: `if`, `and`, `or` evaluate every operand
/// before choosing, because the reference applies primitives to already
/// evaluated arguments.  Rust evaluates call arguments left to right, so a
/// call to these is exactly that.
pub mod prim {
    #[inline]
    pub fn ite<T>(c: bool, x: T, y: T) -> T {
        if c {
            x
        } else {
            y
        }
    }
    #[inline]
    pub fn and(a: bool, b: bool) -> bool {
        a && b
    }
    #[inline]
    pub fn or(a: bool, b: bool) -> bool {
        a || b
    }
    #[inline]
    pub fn get_d<T>(x: Option<T>, dflt: T) -> T {
        match x {
            Some(v) => v,
            None => dflt,
        }
    }
}

/// Read a declaration evaluated earlier this tick.
#[inline]
pub fn read_decl<T: Copy>(v: Option<T>, decl: u64) -> Result<T, RuntimeError> {
    v.ok_or(RuntimeError::NotEvaluated { decl })
}

/// Read the input supplied for an unresolved declaration due this tick.
#[inline]
pub fn read_input<T: Copy>(v: Option<T>, decl: u64) -> Result<T, RuntimeError> {
    v.ok_or(RuntimeError::MissingInput { decl })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_domains_is_a_bitset_with_capacity() {
        let a = ActiveDomains::none().with(ClockSlot(0)).with(ClockSlot(63));
        assert!(a.is_active(ClockSlot(0)) && a.is_active(ClockSlot(63)));
        assert!(!a.is_active(ClockSlot(1)) && !a.is_active(ClockSlot(64)));
        assert!(a.any() && !ActiveDomains::none().any());
        assert_eq!(a.with(ClockSlot(64)), a);
    }

    #[test]
    fn numerics_follow_the_reference_policy() {
        assert_eq!(
            num::div(1.0, 0.0, 7),
            Err(RuntimeError::DivisionByZero { decl: 7 })
        );
        assert_eq!(
            num::mul(1e308, 1e308, 7),
            Err(RuntimeError::NonFinite { decl: 7, op: "mul" })
        );
        assert_eq!(num::add(1.0, 2.0, 0), Ok(3.0));
        assert_eq!(num::div(1.0, 3.0, 0), Ok(1.0 / 3.0));
        assert_eq!(
            read_input::<f64>(None, 3),
            Err(RuntimeError::MissingInput { decl: 3 })
        );
    }
}
