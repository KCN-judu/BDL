//! Surface AST: exactly what the designer wrote, with spans.

use bdl_diagnostics::Span;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceExpr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    /// An input reference, resolved by the elaborator.
    Name(String),
    /// A numeric literal, optionally with a unit written after it (`90 deg`).
    Number {
        text: String,
        value: f64,
        unit: Option<Unit>,
    },
    Bool(bool),
    Unary {
        op: UnaryOp,
        expr: Box<SurfaceExpr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<SurfaceExpr>,
        rhs: Box<SurfaceExpr>,
    },
    If {
        cond: Box<SurfaceExpr>,
        then: Box<SurfaceExpr>,
        els: Box<SurfaceExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

impl BinaryOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}

impl SurfaceExpr {
    /// Every `Name` in the expression, left to right, with spans.
    pub fn names(&self) -> Vec<(&str, Span)> {
        let mut out = Vec::new();
        self.walk(&mut |e| {
            if let ExprKind::Name(n) = &e.kind {
                out.push((n.as_str(), e.span));
            }
        });
        out
    }

    pub fn walk<'a>(&'a self, f: &mut dyn FnMut(&'a SurfaceExpr)) {
        f(self);
        match &self.kind {
            ExprKind::Name(_) | ExprKind::Number { .. } | ExprKind::Bool(_) => {}
            ExprKind::Unary { expr, .. } => expr.walk(f),
            ExprKind::Binary { lhs, rhs, .. } => {
                lhs.walk(f);
                rhs.walk(f);
            }
            ExprKind::If { cond, then, els } => {
                cond.walk(f);
                then.walk(f);
                els.walk(f);
            }
        }
    }
}
