//! The runtime vocabulary every generated BDL semantic core is written
//! against.  `no_std`, `unsafe`-free, allocation-free unless the
//! `collections` feature is on, and ignorant of any board, device kind,
//! transport or editor: it knows *slots* (clock, input, state, output),
//! *active domains*, *checked numerics*, *structured runtime errors* and —
//! with `collections` — the list operators and the list recursor over
//! `alloc::vec::Vec` — nothing else.
//!
//! The numeric helpers implement exactly the reference evaluator's policy
//! (`docs/spec/runtime-semantics.md`, DI-15): IEEE `f64`; division by zero and
//! any non-finite result are errors that fail the tick.  Generated code
//! calls these helpers instead of bare operators so that the policy lives
//! in one place on both sides of the differential tests.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "collections")]
extern crate alloc;

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

/// The list operators and the recursor (`collections`).  Each is the
/// reference evaluator's `apply_prim` case, operator by operator.
///
/// A list is a `Vec` stored **last element first**: `cons` pushes, the
/// recursor consumes from the front, so the library's `map`, `filter` and
/// `append` (folds that `cons` onto the accumulator) stay linear, and
/// equality is elementwise as in the list's order.  The host bridge
/// reverses at the boundary; nothing inside the core observes the storage
/// order.  A count argument (`take`, `drop`) is a dimensionless `f64` read
/// as a whole number towards zero, never below zero
/// (`bdl_reactive::eval::count`).
#[cfg(feature = "collections")]
pub mod list {
    use super::RuntimeError;
    use alloc::vec::Vec;

    #[inline]
    pub fn count(k: f64) -> usize {
        if k.is_finite() && k > 0.0 {
            k as usize
        } else {
            0
        }
    }
    /// A list from its elements in list order.
    pub fn from_ordered<T>(items: Vec<T>) -> Vec<T> {
        let mut v = items;
        v.reverse();
        v
    }
    /// The elements in list order.
    pub fn into_ordered<T>(xs: Vec<T>) -> Vec<T> {
        let mut v = xs;
        v.reverse();
        v
    }
    #[inline]
    pub fn nil<T>() -> Vec<T> {
        Vec::new()
    }
    #[inline]
    pub fn cons<T>(x: T, mut xs: Vec<T>) -> Vec<T> {
        xs.push(x);
        xs
    }
    #[inline]
    pub fn length<T>(xs: Vec<T>) -> f64 {
        xs.len() as f64
    }
    /// `length` of a list the caller keeps: no copy.
    #[inline]
    pub fn length_of<T>(xs: &[T]) -> f64 {
        xs.len() as f64
    }
    /// `head` of a list the caller keeps: one element cloned.
    #[inline]
    pub fn head_of<T: Clone>(xs: &[T]) -> Option<T> {
        xs.last().cloned()
    }
    /// `take` from a list the caller keeps: the `k` kept elements are
    /// copied, nothing else.
    #[inline]
    pub fn take_of<T: Clone>(k: f64, xs: &[T]) -> Vec<T> {
        let keep = count(k).min(xs.len());
        xs[xs.len() - keep..].to_vec()
    }
    /// The first `k` elements of the list.
    #[inline]
    pub fn take<T>(k: f64, mut xs: Vec<T>) -> Vec<T> {
        let keep = count(k).min(xs.len());
        xs.split_off(xs.len() - keep)
    }
    /// All but the first `k` elements of the list.
    #[inline]
    pub fn drop<T>(k: f64, mut xs: Vec<T>) -> Vec<T> {
        let keep = xs.len().saturating_sub(count(k));
        xs.truncate(keep);
        xs
    }
    #[inline]
    pub fn reverse<T>(mut xs: Vec<T>) -> Vec<T> {
        xs.reverse();
        xs
    }
    #[inline]
    pub fn head<T>(mut xs: Vec<T>) -> Option<T> {
        xs.pop()
    }
    #[inline]
    pub fn to_list<T>(x: Option<T>) -> Vec<T> {
        match x {
            Some(v) => alloc::vec![v],
            None => Vec::new(),
        }
    }
    /// `fold f z [x₁, …, xₙ] = f x₁ (… (f xₙ z))`: one step per element,
    /// from the last; a step may fail like any primitive.
    #[inline]
    pub fn fold<T, A>(
        xs: Vec<T>,
        init: A,
        mut step: impl FnMut(T, A) -> Result<A, RuntimeError>,
    ) -> Result<A, RuntimeError> {
        let mut acc = init;
        for x in xs {
            acc = step(x, acc)?;
        }
        Ok(acc)
    }
}

/// Read a declaration evaluated earlier this tick.
#[inline]
pub fn read_decl<T: Clone>(v: &Option<T>, decl: u64) -> Result<T, RuntimeError> {
    v.clone().ok_or(RuntimeError::NotEvaluated { decl })
}

/// Borrow a declaration evaluated earlier this tick — for an operator
/// that only inspects its operand (`length`, `head`, `take`, `==`, `<`).
#[inline]
pub fn read_decl_ref<T>(v: &Option<T>, decl: u64) -> Result<&T, RuntimeError> {
    v.as_ref().ok_or(RuntimeError::NotEvaluated { decl })
}

/// Read the input supplied for an unresolved declaration due this tick.
#[inline]
pub fn read_input<T: Clone>(v: &Option<T>, decl: u64) -> Result<T, RuntimeError> {
    v.clone().ok_or(RuntimeError::MissingInput { decl })
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

    #[cfg(feature = "collections")]
    #[test]
    fn list_operators_follow_the_reference_evaluator() {
        use alloc::vec;
        // the list [1, 2, 3], stored last first
        let xs = list::from_ordered(vec![1.0, 2.0, 3.0]);
        let ordered = |v: alloc::vec::Vec<f64>| list::into_ordered(v);
        assert_eq!(
            ordered(list::cons(0.0, xs.clone())),
            vec![0.0, 1.0, 2.0, 3.0]
        );
        assert_eq!(list::length(xs.clone()), 3.0);
        assert_eq!(ordered(list::take(2.7, xs.clone())), vec![1.0, 2.0]);
        assert_eq!(ordered(list::take(9.0, xs.clone())), vec![1.0, 2.0, 3.0]);
        assert_eq!(ordered(list::drop(-1.0, xs.clone())), vec![1.0, 2.0, 3.0]);
        assert_eq!(ordered(list::drop(2.0, xs.clone())), vec![3.0]);
        assert_eq!(ordered(list::reverse(xs.clone())), vec![3.0, 2.0, 1.0]);
        assert_eq!(list::head(xs.clone()), Some(1.0));
        assert_eq!(list::head(list::nil::<f64>()), None);
        assert_eq!(ordered(list::to_list(Some(4.0))), vec![4.0]);
        // foldr: 1 - (2 - (3 - 0))
        let r = list::fold(xs.clone(), 0.0, |x, acc| num::sub(x, acc, 0));
        assert_eq!(r, Ok(2.0));
        // map through fold keeps the order: cons onto the accumulator
        let mapped = list::fold(xs, list::nil(), |x, acc| Ok(list::cons(x * 2.0, acc)));
        assert_eq!(ordered(mapped.unwrap()), vec![2.0, 4.0, 6.0]);
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
            read_input::<f64>(&None, 3),
            Err(RuntimeError::MissingInput { decl: 3 })
        );
    }
}
