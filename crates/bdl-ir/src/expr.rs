//! The Reactive Core IR: the kernel's terms (`Base.lean`).
//!
//! Deliberately small.  Two temporal forms only — `delay` and `sync` — and
//! every dimension rule lives in [`Prim::ty`].

use crate::ty::Ty;
use bdl_model::{ClockId, DeclId, Dim, SemanticId};
use serde::{Deserialize, Serialize};

/// A numeric literal.  The formal development uses `Nat`; the production
/// implementation uses IEEE doubles and records that as a deliberate,
/// documented deviation (`docs/archive/design-issues-ledger.md`, DI-1).  `Scalar` gives
/// literals total equality and ordering (by bit pattern) so IR values remain
/// hashable and deterministic.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scalar(pub f64);

impl PartialEq for Scalar {
    fn eq(&self, o: &Scalar) -> bool {
        self.0.to_bits() == o.0.to_bits()
    }
}
impl Eq for Scalar {}
impl std::hash::Hash for Scalar {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.0.to_bits().hash(h)
    }
}
impl PartialOrd for Scalar {
    fn partial_cmp(&self, o: &Scalar) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Scalar {
    fn cmp(&self, o: &Scalar) -> std::cmp::Ordering {
        self.0.to_bits().cmp(&o.0.to_bits())
    }
}

/// Registered pure operators.  Typing an application of a primitive is
/// ordinary function application against [`Prim::ty`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "prim", rename_all = "snake_case")]
pub enum Prim {
    Lit { dim: Dim, value: Scalar },
    Add { dim: Dim },
    Sub { dim: Dim },
    Mul { d1: Dim, d2: Dim },
    Div { d1: Dim, d2: Dim },
    Lt { dim: Dim },
    Eq { dim: Dim },
    Not,
    And,
    Or,
    Ite { ty: Ty },
    None { ty: Ty },
    Some { ty: Ty },
    IsSome { ty: Ty },
    GetD { ty: Ty },
}

impl Prim {
    /// `Prim.ty` — the dimension algebra lives here and nowhere else.
    pub fn ty(&self) -> Ty {
        use Ty::*;
        match self {
            Prim::Lit { dim, .. } => Q { dim: *dim },
            Prim::Add { dim } | Prim::Sub { dim } => {
                Ty::arrows([Q { dim: *dim }, Q { dim: *dim }], Q { dim: *dim })
            }
            Prim::Mul { d1, d2 } => {
                Ty::arrows([Q { dim: *d1 }, Q { dim: *d2 }], Q { dim: *d1 + *d2 })
            }
            Prim::Div { d1, d2 } => {
                Ty::arrows([Q { dim: *d1 }, Q { dim: *d2 }], Q { dim: *d1 - *d2 })
            }
            Prim::Lt { dim } | Prim::Eq { dim } => {
                Ty::arrows([Q { dim: *dim }, Q { dim: *dim }], Bool)
            }
            Prim::Not => Ty::arr(Bool, Bool),
            Prim::And | Prim::Or => Ty::arrows([Bool, Bool], Bool),
            Prim::Ite { ty } => Ty::arrows([Bool, ty.clone(), ty.clone()], ty.clone()),
            Prim::None { ty } => Ty::opt(ty.clone()),
            Prim::Some { ty } => Ty::arr(ty.clone(), Ty::opt(ty.clone())),
            Prim::IsSome { ty } => Ty::arr(Ty::opt(ty.clone()), Bool),
            Prim::GetD { ty } => Ty::arrows([Ty::opt(ty.clone()), ty.clone()], ty.clone()),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Prim::Lit { .. } | Prim::None { .. } => 0,
            Prim::Not | Prim::IsSome { .. } | Prim::Some { .. } => 1,
            Prim::Add { .. }
            | Prim::Sub { .. }
            | Prim::Mul { .. }
            | Prim::Div { .. }
            | Prim::Lt { .. }
            | Prim::Eq { .. }
            | Prim::And
            | Prim::Or
            | Prim::GetD { .. } => 2,
            Prim::Ite { .. } => 3,
        }
    }
}

/// Core terms.  Variables are de Bruijn indices.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "expr", rename_all = "snake_case")]
pub enum Expr {
    Var {
        index: u32,
    },
    BoolLit {
        value: bool,
    },
    NatLit {
        value: u64,
    },
    Lam {
        dom: Ty,
        body: Box<Expr>,
    },
    App {
        f: Box<Expr>,
        a: Box<Expr>,
    },
    /// Reference to a declaration by identity; typed through the type view.
    DeclRef {
        id: DeclId,
    },
    /// Observe a semantic value's representation (free everywhere).
    Rep {
        e: Box<Expr>,
    },
    /// Construct a semantic value (only under a grant for `s`).
    Mk {
        s: SemanticId,
        e: Box<Expr>,
    },
    Prim {
        p: Prim,
    },
    /// The value of `e` one activation ago in the own domain; `init` at the
    /// first activation.
    Delay {
        init: Box<Expr>,
        e: Box<Expr>,
    },
    /// The value of `e`, evaluated in domain `src`, at the last activation of
    /// `src` strictly before now; `init` if there was none.
    Sync {
        src: ClockId,
        init: Box<Expr>,
        e: Box<Expr>,
    },
}

impl Expr {
    pub fn app(f: Expr, a: Expr) -> Expr {
        Expr::App {
            f: Box::new(f),
            a: Box::new(a),
        }
    }
    pub fn apps(f: Expr, args: impl IntoIterator<Item = Expr>) -> Expr {
        args.into_iter().fold(f, Expr::app)
    }
    pub fn prim(p: Prim) -> Expr {
        Expr::Prim { p }
    }
    pub fn decl(id: DeclId) -> Expr {
        Expr::DeclRef { id }
    }
    pub fn rep(e: Expr) -> Expr {
        Expr::Rep { e: Box::new(e) }
    }
    pub fn mk(s: SemanticId, e: Expr) -> Expr {
        Expr::Mk { s, e: Box::new(e) }
    }
    pub fn delay(init: Expr, e: Expr) -> Expr {
        Expr::Delay {
            init: Box::new(init),
            e: Box::new(e),
        }
    }
    pub fn sync(src: ClockId, init: Expr, e: Expr) -> Expr {
        Expr::Sync {
            src,
            init: Box::new(init),
            e: Box::new(e),
        }
    }

    /// `Expr.refs`: every declaration referenced (with multiplicity).
    pub fn refs(&self) -> Vec<DeclId> {
        let mut out = Vec::new();
        self.collect_refs(false, &mut out);
        out
    }

    /// `Expr.instRefs`: declarations referenced *instantaneously* — not under
    /// the delayed operand of a `delay` or the transported operand of a
    /// `sync`.  Initial values are read at the first activation and count as
    /// instantaneous.
    pub fn inst_refs(&self) -> Vec<DeclId> {
        let mut out = Vec::new();
        self.collect_refs(true, &mut out);
        out
    }

    fn collect_refs(&self, instantaneous_only: bool, out: &mut Vec<DeclId>) {
        match self {
            Expr::Var { .. } | Expr::BoolLit { .. } | Expr::NatLit { .. } | Expr::Prim { .. } => {}
            Expr::Lam { body, .. } => body.collect_refs(instantaneous_only, out),
            Expr::App { f, a } => {
                f.collect_refs(instantaneous_only, out);
                a.collect_refs(instantaneous_only, out);
            }
            Expr::DeclRef { id } => out.push(*id),
            Expr::Rep { e } | Expr::Mk { e, .. } => e.collect_refs(instantaneous_only, out),
            Expr::Delay { init, e } | Expr::Sync { init, e, .. } => {
                init.collect_refs(instantaneous_only, out);
                if !instantaneous_only {
                    e.collect_refs(instantaneous_only, out);
                }
            }
        }
    }

    /// `Expr.DelayFree`: the timeless fragment.
    pub fn is_delay_free(&self) -> bool {
        match self {
            Expr::Delay { .. } | Expr::Sync { .. } => false,
            Expr::Lam { body, .. } => body.is_delay_free(),
            Expr::App { f, a } => f.is_delay_free() && a.is_delay_free(),
            Expr::Rep { e } | Expr::Mk { e, .. } => e.is_delay_free(),
            Expr::Var { .. }
            | Expr::BoolLit { .. }
            | Expr::NatLit { .. }
            | Expr::Prim { .. }
            | Expr::DeclRef { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_algebra_in_prim_types() {
        let mul = Prim::Mul {
            d1: Dim::LENGTH,
            d2: Dim::TIME,
        };
        assert_eq!(
            mul.ty(),
            Ty::arrows(
                [Ty::q(Dim::LENGTH), Ty::q(Dim::TIME)],
                Ty::q(Dim::LENGTH + Dim::TIME)
            )
        );
        let div = Prim::Div {
            d1: Dim::LENGTH,
            d2: Dim::TIME,
        };
        assert_eq!(
            div.ty(),
            Ty::arrows(
                [Ty::q(Dim::LENGTH), Ty::q(Dim::TIME)],
                Ty::q(Dim::LENGTH - Dim::TIME)
            )
        );
    }

    #[test]
    fn inst_refs_skip_delayed_operand() {
        let a = DeclId::from_raw(0);
        let b = DeclId::from_raw(1);
        // A := delay B (init 0) — a self-delayed cycle shape: init reads
        // nothing, e reads B.
        let e = Expr::delay(Expr::NatLit { value: 0 }, Expr::decl(b));
        assert_eq!(e.refs(), vec![b]);
        assert_eq!(e.inst_refs(), Vec::<DeclId>::new());
        let e2 = Expr::delay(Expr::decl(a), Expr::decl(b));
        assert_eq!(e2.inst_refs(), vec![a]);
        assert!(!e2.is_delay_free());
    }

    #[test]
    fn scalar_is_total() {
        assert_eq!(Scalar(0.5), Scalar(0.5));
        assert_ne!(Scalar(0.0), Scalar(-0.0)); // bit-pattern equality, documented
    }
}
