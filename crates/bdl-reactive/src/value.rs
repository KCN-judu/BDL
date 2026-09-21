//! Runtime values.  Concept identity and dimension are kept at runtime so a
//! trace can be read in the design's own terms and so two values that merely
//! share a number are never confused (`Tilt` 0.5 rad is not `MotorAngle`
//! 0.5 rad).
//!
//! Numeric policy (docs/spec/runtime-semantics.md, DESIGN_ISSUES DI-15): IEEE
//! `f64`; a primitive whose result is not finite (division by zero,
//! overflow, NaN) is a runtime error that fails the tick — the reference
//! evaluator never propagates NaN/∞ as a value.  Equality is exact bitwise
//! `f64` equality; serialization is the JSON number.

use bdl_ir::{Expr, Prim};
use bdl_model::{ConceptId, Dim};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Value {
    Bool {
        value: bool,
    },
    Nat {
        value: u64,
    },
    Quantity {
        dim: Dim,
        value: f64,
    },
    /// A Sem value (`sem C v`): a value of concept `id`, the instance level
    /// of the concept ladder — never the concept, which is a type.
    Semantic {
        id: ConceptId,
        repr: Box<Value>,
    },
    None,
    Some {
        value: Box<Value>,
    },
    /// A lambda closed over its de Bruijn environment (innermost first).
    Closure {
        env: Vec<Value>,
        body: Expr,
    },
    /// A partially applied primitive.
    Prim {
        prim: Prim,
        args: Vec<Value>,
    },
    /// An ordinary list (Phase 9a).
    List {
        items: List,
    },
    /// A pair (Phase 9b).
    Pair {
        fst: Box<Value>,
        snd: Box<Value>,
    },
}

/// A finite list of values: an immutable, shared cons list.  `cons` and
/// `clone` are constant-time and share the tail, so the recursor's
/// accumulator, the closures that capture a list and the memoised
/// declaration values never copy it; `map`, `filter` and `append` through
/// `fold` stay linear.  Every observable operation — iteration, equality,
/// serialisation, rendering — is in the list's own order.
#[derive(Clone, Default)]
pub struct List(Option<Arc<Node>>);

struct Node {
    head: Value,
    tail: List,
    len: usize,
}

impl Drop for Node {
    // Unlink iteratively: a long list must not drop by recursion.
    fn drop(&mut self) {
        let mut next = self.tail.0.take();
        while let Some(arc) = next {
            match Arc::try_unwrap(arc) {
                Ok(mut node) => next = node.tail.0.take(),
                Err(_) => break,
            }
        }
    }
}

impl List {
    pub fn new() -> List {
        List(None)
    }
    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |n| n.len)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
    /// The elements in the list's order.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            next: self.0.as_deref(),
        }
    }
    /// The elements from the last to the first: the recursor's order.
    pub fn iter_from_last(&self) -> impl Iterator<Item = &Value> {
        let mut all: Vec<&Value> = self.iter().collect();
        all.reverse();
        all.into_iter()
    }
    pub fn first(&self) -> Option<&Value> {
        self.0.as_ref().map(|n| &n.head)
    }
    /// `cons x xs`: constant time, sharing `xs`.
    pub fn cons(x: Value, xs: List) -> List {
        let len = xs.len() + 1;
        List(Some(Arc::new(Node {
            head: x,
            tail: xs,
            len,
        })))
    }
    /// The first `k` elements.
    pub fn take(&self, k: usize) -> List {
        self.iter().take(k).cloned().collect()
    }
    /// All but the first `k` elements: shares the tail.
    pub fn drop(&self, k: usize) -> List {
        let mut cur = self.clone();
        for _ in 0..k {
            let Some(node) = cur.0.as_ref() else { break };
            let next = node.tail.clone();
            cur = next;
        }
        cur
    }
    pub fn reversed(&self) -> List {
        self.iter()
            .fold(List::new(), |acc, x| List::cons(x.clone(), acc))
    }
}

/// Iteration in the list's order.
pub struct Iter<'a> {
    next: Option<&'a Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Value;
    fn next(&mut self) -> Option<&'a Value> {
        let node = self.next?;
        self.next = node.tail.0.as_deref();
        Some(&node.head)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.next.map_or(0, |n| n.len);
        (n, Some(n))
    }
}

impl ExactSizeIterator for Iter<'_> {}

impl FromIterator<Value> for List {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> List {
        let items: Vec<Value> = iter.into_iter().collect();
        items
            .into_iter()
            .rev()
            .fold(List::new(), |acc, x| List::cons(x, acc))
    }
}

impl PartialEq for List {
    fn eq(&self, other: &List) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl std::fmt::Debug for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Serialize for List {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(self.iter())
    }
}

impl<'de> Deserialize<'de> for List {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<List, D::Error> {
        let items: Vec<Value> = Vec::deserialize(d)?;
        Ok(items.into_iter().collect())
    }
}

impl Value {
    pub fn q(dim: Dim, value: f64) -> Value {
        Value::Quantity { dim, value }
    }
    pub fn scalar(value: f64) -> Value {
        Value::Quantity {
            dim: Dim::ZERO,
            value,
        }
    }
    pub fn boolean(value: bool) -> Value {
        Value::Bool { value }
    }
    pub fn sem(id: ConceptId, repr: Value) -> Value {
        Value::Semantic {
            id,
            repr: Box::new(repr),
        }
    }
    pub fn some(value: Value) -> Value {
        Value::Some {
            value: Box::new(value),
        }
    }
    pub fn list(items: impl IntoIterator<Item = Value>) -> Value {
        Value::List {
            items: items.into_iter().collect(),
        }
    }
    pub fn pair(fst: Value, snd: Value) -> Value {
        Value::Pair {
            fst: Box::new(fst),
            snd: Box::new(snd),
        }
    }

    /// `Value.beq` — structural equality on data values: truth values,
    /// numbers (exact `f64` equality, the numeric policy of `Eq` since
    /// DI-15), Sem values by concept and representation, `none`/`some`,
    /// pairs and lists componentwise.  Closures and partial primitives
    /// compare `false`; typing never asks.  There is *no* structural order
    /// on values (Phase 9c): ordering is a quantity comparison only.
    pub fn structurally_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Bool { value: a }, Value::Bool { value: b }) => a == b,
            (Value::Nat { value: a }, Value::Nat { value: b }) => a == b,
            (Value::Quantity { value: a, .. }, Value::Quantity { value: b, .. }) => a == b,
            (Value::Semantic { id: a, repr: x }, Value::Semantic { id: b, repr: y }) => {
                a == b && x.structurally_equal(y)
            }
            (Value::None, Value::None) => true,
            (Value::Some { value: a }, Value::Some { value: b }) => a.structurally_equal(b),
            (Value::Pair { fst: a, snd: b }, Value::Pair { fst: c, snd: d }) => {
                a.structurally_equal(c) && b.structurally_equal(d)
            }
            (Value::List { items: a }, Value::List { items: b }) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.structurally_equal(y))
            }
            _ => false,
        }
    }

    pub fn as_quantity(&self) -> Option<(Dim, f64)> {
        match self {
            Value::Quantity { dim, value } => Some((*dim, *value)),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool { value } => Some(*value),
            _ => None,
        }
    }

    /// The representation carried by a Sem value, one level down.
    pub fn unwrap_semantic(&self) -> Option<&Value> {
        match self {
            Value::Semantic { repr, .. } => Some(repr),
            _ => None,
        }
    }

    /// The one designer-facing rendering of a value, for traces, probes
    /// and diagnostics (ADR-0033): `0.5 [rad]`, `Tilt(0.5 [rad])`,
    /// `Held(on)`.  Product words, not kernel or syntax words: a truth
    /// value is `on` / `off` — the value form is called *On / off* — and a
    /// number shows six significant digits, the serialised trace keeping
    /// the exact `f64`.  Every host shows this text as it is.
    pub fn render(&self, concept_name: &dyn Fn(ConceptId) -> String) -> String {
        match self {
            Value::Bool { value } => if *value { "on" } else { "off" }.into(),
            Value::Nat { value } => value.to_string(),
            Value::Quantity { dim, value } => {
                if dim.is_dimensionless() {
                    render_number(*value)
                } else {
                    format!(
                        "{} [{}]",
                        render_number(*value),
                        bdl_check::pretty::symbol(*dim)
                    )
                }
            }
            Value::Semantic { id, repr } => {
                format!("{}({})", concept_name(*id), repr.render(concept_name))
            }
            Value::None => "none".into(),
            Value::Some { value } => format!("some({})", value.render(concept_name)),
            Value::Closure { .. } => "<function>".into(),
            Value::Prim { prim, .. } => format!("<{prim:?}>"),
            Value::List { items } => format!(
                "[{}]",
                items
                    .iter()
                    .map(|v| v.render(concept_name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Pair { fst, snd } => {
                format!(
                    "({}, {})",
                    fst.render(concept_name),
                    snd.render(concept_name)
                )
            }
        }
    }
}
/// A number as a designer reads it: an integer as an integer, otherwise six
/// significant digits with trailing zeros dropped (`0.785398`, `1.5e-7`).
/// The exact value stays in the serialised trace; this is text.
pub fn render_number(v: f64) -> String {
    if !v.is_finite() {
        return format!("{v}");
    }
    if v == v.trunc() && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    let magnitude = v.abs().log10().floor() as i32;
    if !(-5..15).contains(&magnitude) {
        let s = format!("{:.5e}", v);
        let (mant, exp) = s.split_once('e').unwrap_or((&s, "0"));
        return format!("{}e{}", trim_zeros(mant), exp);
    }
    let decimals = (5 - magnitude).max(0) as usize;
    trim_zeros(&format!("{v:.decimals$}")).to_string()
}

fn trim_zeros(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn numbers_read_as_a_designer_writes_them() {
        assert_eq!(render_number(0.5), "0.5");
        assert_eq!(render_number(2.0), "2");
        assert_eq!(render_number(-3.0), "-3");
        assert_eq!(render_number(100_000.0), "100000");
        assert_eq!(render_number(std::f64::consts::FRAC_PI_4), "0.785398");
        assert_eq!(render_number(1.0 / 3.0), "0.333333");
        assert_eq!(render_number(123.456789), "123.457");
        assert_eq!(render_number(1.5e-7), "1.5e-7");
        assert_eq!(render_number(2.5e17), "2.5e17");
    }

    #[test]
    fn truth_values_render_in_product_words() {
        let name = |_: ConceptId| "Held".to_string();
        assert_eq!(Value::boolean(true).render(&name), "on");
        assert_eq!(Value::boolean(false).render(&name), "off");
        assert_eq!(
            Value::sem(ConceptId::from_raw(1), Value::boolean(true)).render(&name),
            "Held(on)"
        );
        assert_eq!(
            Value::sem(
                ConceptId::from_raw(1),
                Value::q(Dim::ANGLE, std::f64::consts::FRAC_PI_4)
            )
            .render(&name),
            "Held(0.785398 [rad])"
        );
    }
}
