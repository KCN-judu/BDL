//! The storage-order audit: the list operators over the reversed `Vec`
//! agree with a reference model written in *list order* on generated
//! lists, so nothing observable depends on how the core stores a list.
//! Also the cost evidence: how many element clones the recursor and the
//! operators perform (docs/evidence/testing.md, "collections cost").

#![cfg(feature = "collections")]
#![allow(clippy::unwrap_used, clippy::bool_assert_comparison)]

use bdl_runtime_core::list;
use bdl_runtime_core::RuntimeError;
use std::cell::Cell;

/// Deterministic xorshift generator: the audit needs many lists, not
/// randomness.
struct Gen(u64);

impl Gen {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn list(&mut self) -> Vec<i64> {
        let n = self.below(12) as usize;
        (0..n).map(|_| self.below(7) as i64 - 3).collect()
    }
}

/// The reference model: a list *is* its elements in order, every
/// operator is the textbook one (`fold f z [x₁…xₙ] = f x₁ (… (f xₙ z))`).
mod model {
    pub fn take<T: Clone>(k: usize, xs: &[T]) -> Vec<T> {
        xs.iter().take(k).cloned().collect()
    }
    pub fn drop<T: Clone>(k: usize, xs: &[T]) -> Vec<T> {
        xs.iter().skip(k).cloned().collect()
    }
    pub fn fold<T: Clone, A>(xs: &[T], z: A, f: impl Fn(T, A) -> A) -> A {
        xs.iter().rev().cloned().fold(z, |acc, x| f(x, acc))
    }
    pub fn map<T: Clone, U>(xs: &[T], f: impl Fn(T) -> U) -> Vec<U> {
        xs.iter().cloned().map(f).collect()
    }
    pub fn filter<T: Clone>(xs: &[T], p: impl Fn(&T) -> bool) -> Vec<T> {
        xs.iter().filter(|x| p(x)).cloned().collect()
    }
    pub fn append<T: Clone>(xs: &[T], ys: &[T]) -> Vec<T> {
        xs.iter().chain(ys).cloned().collect()
    }
    pub fn zip<T: Clone, U: Clone>(xs: &[T], ys: &[U]) -> Vec<(T, U)> {
        xs.iter().cloned().zip(ys.iter().cloned()).collect()
    }
}

/// The library's definitions over the core's operators, exactly as the
/// generated core spells them (`docs/spec/equation-library.md`): folds
/// that `cons` onto the accumulator.
mod core_lib {
    use super::list;
    use bdl_runtime_core::RuntimeError;
    type R<T> = Result<T, RuntimeError>;

    pub fn map<T, U>(xs: Vec<T>, f: impl Fn(T) -> U) -> R<Vec<U>> {
        list::fold(xs, list::nil(), |x, acc| Ok(list::cons(f(x), acc)))
    }
    pub fn filter<T>(xs: Vec<T>, p: impl Fn(&T) -> bool) -> R<Vec<T>> {
        list::fold(xs, list::nil(), |x, acc| {
            Ok(if p(&x) { list::cons(x, acc) } else { acc })
        })
    }
    pub fn append<T>(xs: Vec<T>, ys: Vec<T>) -> R<Vec<T>> {
        list::fold(xs, ys, |x, acc| Ok(list::cons(x, acc)))
    }
    /// `zip xs ys = snd (fold (λx (rest, out). (drop 1 rest,
    /// append (map (λy. (x, y)) (take 1 rest)) out)) (ys, []) (reverse xs))`,
    /// then reversed — the library's expansion.
    pub fn zip<T: Clone, U: Clone>(xs: Vec<T>, ys: Vec<U>) -> R<Vec<(T, U)>> {
        let folded = list::fold(list::reverse(xs), (ys, list::nil()), |x, acc| {
            let (rest, out) = acc;
            let firsts = map(list::take(1.0, rest.clone()), |y| (x.clone(), y))?;
            Ok((list::drop(1.0, rest), append(firsts, out)?))
        })?;
        Ok(list::reverse(folded.1))
    }
}

fn of(xs: &[i64]) -> Vec<i64> {
    list::from_ordered(xs.to_vec())
}
fn ordered<T>(v: Vec<T>) -> Vec<T> {
    list::into_ordered(v)
}

#[test]
fn every_operator_agrees_with_the_list_order_model_on_generated_lists() {
    let mut g = Gen(0x9E37_79B9_7F4A_7C15);
    for _ in 0..500 {
        let xs = g.list();
        let ys = g.list();
        let k = g.below(14) as f64 - 1.0; // −1 … 12, off both ends
        let kk = list::count(k);
        let l = of(&xs);
        // the boundary round-trip
        assert_eq!(ordered(of(&xs)), xs);
        assert_eq!(ordered(list::cons(9, l.clone())), model::append(&[9], &xs));
        assert_eq!(list::length(l.clone()), xs.len() as f64);
        assert_eq!(list::head(l.clone()), xs.first().copied());
        assert_eq!(ordered(list::take(k, l.clone())), model::take(kk, &xs));
        assert_eq!(ordered(list::drop(k, l.clone())), model::drop(kk, &xs));
        assert_eq!(
            ordered(list::reverse(l.clone())),
            xs.iter().rev().copied().collect::<Vec<_>>()
        );
        assert_eq!(
            ordered(list::to_list(xs.first().copied())),
            model::take(1, &xs)
        );
        // fold: a non-commutative step tells the order apart
        let step = |x: i64, acc: i64| x - 2 * acc;
        assert_eq!(
            list::fold(l.clone(), 1, |x, acc| Ok(step(x, acc))),
            Ok(model::fold(&xs, 1, step))
        );
        assert_eq!(
            ordered(core_lib::map(l.clone(), |x| x * 3).unwrap()),
            model::map(&xs, |x| x * 3)
        );
        assert_eq!(
            ordered(core_lib::filter(l.clone(), |x| x % 2 == 0).unwrap()),
            model::filter(&xs, |x| x % 2 == 0)
        );
        assert_eq!(
            ordered(core_lib::append(l.clone(), of(&ys)).unwrap()),
            model::append(&xs, &ys)
        );
        assert_eq!(
            ordered(core_lib::zip(l.clone(), of(&ys)).unwrap()),
            model::zip(&xs, &ys)
        );
        // equality is elementwise in list order: the same list stored
        // twice is equal, a reversed one only when it is a palindrome
        assert_eq!(l == of(&xs), true);
        assert_eq!(l == list::reverse(l.clone()), xs.iter().eq(xs.iter().rev()));
        assert_eq!(l == of(&ys), xs == ys);
    }
}

#[test]
fn an_erroring_step_stops_the_fold_at_that_element() {
    let l = of(&[1, 2, 0, 4]);
    let r: Result<i64, RuntimeError> = list::fold(l, 0, |x, acc| {
        if x == 0 {
            Err(RuntimeError::DivisionByZero { decl: 3 })
        } else {
            Ok(acc + x)
        }
    });
    assert_eq!(r, Err(RuntimeError::DivisionByZero { decl: 3 }));
}

// ---- cost evidence -------------------------------------------------------------

thread_local! {
    static CLONES: Cell<usize> = const { Cell::new(0) };
}

/// An element whose clones are counted.
#[derive(Debug, PartialEq)]
struct Counted(i64);

impl Clone for Counted {
    fn clone(&self) -> Counted {
        CLONES.with(|c| c.set(c.get() + 1));
        Counted(self.0)
    }
}

fn clones_during<T>(f: impl FnOnce() -> T) -> (T, usize) {
    CLONES.with(|c| c.set(0));
    let r = f();
    (r, CLONES.with(|c| c.get()))
}

fn counted(n: i64) -> Vec<Counted> {
    list::from_ordered((0..n).map(Counted).collect())
}

/// The recursor moves each element into the step and the accumulator
/// through it: `map`, `filter` and `append` written as folds that `cons`
/// onto the moved accumulator clone **nothing** — the linear cost is one
/// `push` per element, and the accumulator is one growing `Vec`.
#[test]
fn map_filter_append_over_a_moved_accumulator_clone_no_element() {
    for n in [0, 1, 100, 1000] {
        let (_, c) = clones_during(|| core_lib::map(counted(n), |x| Counted(x.0 + 1)).unwrap());
        assert_eq!(c, 0, "map of {n}");
        let (_, c) = clones_during(|| core_lib::filter(counted(n), |x| x.0 % 2 == 0).unwrap());
        assert_eq!(c, 0, "filter of {n}");
        let (_, c) = clones_during(|| core_lib::append(counted(n), counted(n)).unwrap());
        assert_eq!(c, 0, "append of {n}");
    }
}

/// A step that keeps the accumulator beside a `cons` of it — the strict
/// conditional of ISS-0013 as the generator emitted it before the lazy
/// emission — clones the whole accumulator per element: quadratically
/// many element clones.  The test pins the number so the cost the
/// generator now avoids stays visible.
#[test]
fn a_cloned_accumulator_per_element_is_quadratic() {
    let quad = |n: i64| {
        clones_during(|| {
            list::fold(counted(n), list::nil(), |x, acc| {
                Ok(if x.0 % 2 == 0 {
                    list::cons(x, acc.clone())
                } else {
                    acc
                })
            })
            .unwrap()
        })
        .1
    };
    // n/2 kept elements, each `cons` cloning the accumulator so far
    let expected: i64 = (0..100).filter(|x| x % 2 == 0).map(|x| x / 2).sum();
    assert_eq!(quad(100), expected as usize);
    assert!(quad(400) > 10 * quad(100));
}

/// `take`, `drop`, `reverse`, `head`, `length` and `cons` over an owned
/// list clone no element either: they move or truncate.
#[test]
fn owned_operators_clone_no_element() {
    let (_, c) = clones_during(|| {
        let l = counted(1000);
        let l = list::cons(Counted(-1), l);
        let l = list::reverse(l);
        let n = list::length(l);
        let l = list::drop(3.0, counted(1000));
        let l = list::take(500.0, l);
        let h = list::head(l);
        (n, h)
    });
    assert_eq!(c, 0);
}
