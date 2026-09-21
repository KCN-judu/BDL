//! The Reactive Core IR: the kernel's terms (`Base.lean`).
//!
//! Deliberately small.  Two temporal forms only — `delay` and `sync` — and
//! every dimension rule lives in [`Prim::ty`].

use crate::ty::Ty;
use bdl_model::{ClockId, ConceptId, DeclId, Dim};
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
    Lit {
        dim: Dim,
        value: Scalar,
    },
    Add {
        dim: Dim,
    },
    Sub {
        dim: Dim,
    },
    Mul {
        d1: Dim,
        d2: Dim,
    },
    Div {
        d1: Dim,
        d2: Dim,
    },
    /// Ordering is a *quantity* comparison only (Phase 9c): a magnitude of
    /// dimension `dim`.  Concepts, pairs, lists, options and truth values
    /// have no order in the kernel; an ordered concept compares through its
    /// representation at elaboration.
    Lt {
        dim: Dim,
    },
    /// Structural equality at any *data* type (Phase 9b).  The kernel
    /// carries the proof `τ.Data` in the syntax; here the checker refuses
    /// `Eq` at a function type instead (`TypeErrorKind::EqualityNotData`).
    Eq {
        ty: Ty,
    },
    Not,
    And,
    Or,
    Ite {
        ty: Ty,
    },
    None {
        ty: Ty,
    },
    Some {
        ty: Ty,
    },
    IsSome {
        ty: Ty,
    },
    GetD {
        ty: Ty,
    },
    // Phase 9a: list data — constructors and a first-order eliminator set.
    Nil {
        ty: Ty,
    },
    Cons {
        ty: Ty,
    },
    /// `list τ → q 0`: the length as a dimensionless quantity.
    Length {
        ty: Ty,
    },
    /// `q 0 → list τ → list τ`: the first `k` elements.
    Take {
        ty: Ty,
    },
    /// `q 0 → list τ → list τ`: all but the first `k` elements (Phase 9b).
    Drop {
        ty: Ty,
    },
    Reverse {
        ty: Ty,
    },
    /// `list τ → opt τ`.
    Head {
        ty: Ty,
    },
    /// `opt τ → list τ`: an option is a list of length at most one, so
    /// `fold` eliminates options too (Phase 9b).
    ToList {
        ty: Ty,
    },
    // Phase 9b: products — construction and the two projections.
    Pair {
        fst: Ty,
        snd: Ty,
    },
    Fst {
        fst: Ty,
        snd: Ty,
    },
    Snd {
        fst: Ty,
        snd: Ty,
    },
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
            Prim::Lt { dim } => Ty::arrows([Q { dim: *dim }, Q { dim: *dim }], Bool),
            Prim::Eq { ty } => Ty::arrows([ty.clone(), ty.clone()], Bool),
            Prim::Not => Ty::arr(Bool, Bool),
            Prim::And | Prim::Or => Ty::arrows([Bool, Bool], Bool),
            Prim::Ite { ty } => Ty::arrows([Bool, ty.clone(), ty.clone()], ty.clone()),
            Prim::None { ty } => Ty::opt(ty.clone()),
            Prim::Some { ty } => Ty::arr(ty.clone(), Ty::opt(ty.clone())),
            Prim::IsSome { ty } => Ty::arr(Ty::opt(ty.clone()), Bool),
            Prim::GetD { ty } => Ty::arrows([Ty::opt(ty.clone()), ty.clone()], ty.clone()),
            Prim::Nil { ty } => Ty::list(ty.clone()),
            Prim::Cons { ty } => {
                Ty::arrows([ty.clone(), Ty::list(ty.clone())], Ty::list(ty.clone()))
            }
            Prim::Length { ty } => Ty::arr(Ty::list(ty.clone()), Q { dim: Dim::ZERO }),
            Prim::Take { ty } | Prim::Drop { ty } => Ty::arrows(
                [Q { dim: Dim::ZERO }, Ty::list(ty.clone())],
                Ty::list(ty.clone()),
            ),
            Prim::Reverse { ty } => Ty::arr(Ty::list(ty.clone()), Ty::list(ty.clone())),
            Prim::Head { ty } => Ty::arr(Ty::list(ty.clone()), Ty::opt(ty.clone())),
            Prim::ToList { ty } => Ty::arr(Ty::opt(ty.clone()), Ty::list(ty.clone())),
            Prim::Pair { fst, snd } => Ty::arrows(
                [fst.clone(), snd.clone()],
                Ty::prod(fst.clone(), snd.clone()),
            ),
            Prim::Fst { fst, snd } => Ty::arr(Ty::prod(fst.clone(), snd.clone()), fst.clone()),
            Prim::Snd { fst, snd } => Ty::arr(Ty::prod(fst.clone(), snd.clone()), snd.clone()),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Prim::Lit { .. } | Prim::None { .. } | Prim::Nil { .. } => 0,
            Prim::Not
            | Prim::IsSome { .. }
            | Prim::Some { .. }
            | Prim::Length { .. }
            | Prim::Reverse { .. }
            | Prim::Head { .. }
            | Prim::ToList { .. }
            | Prim::Fst { .. }
            | Prim::Snd { .. } => 1,
            Prim::Add { .. }
            | Prim::Sub { .. }
            | Prim::Mul { .. }
            | Prim::Div { .. }
            | Prim::Lt { .. }
            | Prim::Eq { .. }
            | Prim::And
            | Prim::Or
            | Prim::GetD { .. }
            | Prim::Cons { .. }
            | Prim::Take { .. }
            | Prim::Drop { .. }
            | Prim::Pair { .. } => 2,
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
    /// Observe a Sem value's representation (free everywhere).
    Rep {
        e: Box<Expr>,
    },
    /// Construct a Sem value (only under a grant for `s`).
    Mk {
        s: ConceptId,
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
    /// The list recursor (Phase 9b): `fold f z [x₁, …, xₙ] = f x₁ (… (f xₙ z))`.
    /// The one term former that applies a function value in the course of
    /// evaluation; registered operators never do.  Every collection
    /// operation of the equation library is a definition over it.  It is
    /// not general recursion: a finite list folds in finitely many steps.
    Fold {
        f: Box<Expr>,
        z: Box<Expr>,
        l: Box<Expr>,
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
    pub fn mk(s: ConceptId, e: Expr) -> Expr {
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
    pub fn fold(f: Expr, z: Expr, l: Expr) -> Expr {
        Expr::Fold {
            f: Box::new(f),
            z: Box::new(z),
            l: Box::new(l),
        }
    }
    pub fn lam(dom: Ty, body: Expr) -> Expr {
        Expr::Lam {
            dom,
            body: Box::new(body),
        }
    }
    pub fn var(index: u32) -> Expr {
        Expr::Var { index }
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
            Expr::Fold { f, z, l } => {
                f.collect_refs(instantaneous_only, out);
                z.collect_refs(instantaneous_only, out);
                l.collect_refs(instantaneous_only, out);
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
            Expr::Fold { f, z, l } => f.is_delay_free() && z.is_delay_free() && l.is_delay_free(),
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
