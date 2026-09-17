//! Runtime values.  Semantic identity and dimension are kept at runtime so a
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
use bdl_model::{Dim, SemanticId};
use serde::{Deserialize, Serialize};

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
    Semantic {
        id: SemanticId,
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
    pub fn sem(id: SemanticId, repr: Value) -> Value {
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

    /// The representation carried by a semantic value, one level down.
    pub fn unwrap_semantic(&self) -> Option<&Value> {
        match self {
            Value::Semantic { repr, .. } => Some(repr),
            _ => None,
        }
    }

    /// Compact rendering for traces and diagnostics: `0.5 [rad]`, `Tilt(0.5 [rad])`.
    pub fn render(&self, concept_name: &dyn Fn(SemanticId) -> String) -> String {
        match self {
            Value::Bool { value } => value.to_string(),
            Value::Nat { value } => value.to_string(),
            Value::Quantity { dim, value } => {
                if dim.is_dimensionless() {
                    format!("{value}")
                } else {
                    format!("{value} [{}]", bdl_check::pretty::symbol(*dim))
                }
            }
            Value::Semantic { id, repr } => {
                format!("{}({})", concept_name(*id), repr.render(concept_name))
            }
            Value::None => "none".into(),
            Value::Some { value } => format!("some({})", value.render(concept_name)),
            Value::Closure { .. } => "<function>".into(),
            Value::Prim { prim, .. } => format!("<{prim:?}>"),
        }
    }
}
