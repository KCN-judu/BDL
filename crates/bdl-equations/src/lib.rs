//! The equation library: the production counterpart of `BDL/Surface/Poly.lean`
//! and `BDL/Surface/Stdlib.lean` (FV Phases 9b and 9c).
//!
//! A designer-facing equation such as `min`, `any` or `map` is *not* a kernel
//! primitive and *not* a declaration of the design.  It is a **definitional
//! family**: a closed Core term indexed by the types (and dimensions) it is
//! used at.  What this crate holds for each entry is
//!
//! * its [`Scheme`] — a type pattern with type variables, dimension
//!   variables and the capability each variable needs ({Data, Eq, Ord},
//!   the closed vocabulary; nothing is user-definable);
//! * its **builder** — the closed de Bruijn term at one instance
//!   (`Stdlib.lean`'s `minF`, `anyF`, `mapF`, … in the designer's argument
//!   order).
//!
//! Instantiation is first-order [`match_ty`] of the scheme against the
//! *closed* argument types the elaborator already knows (`matchTy_sound`,
//! `matchTy_complete`): no unification of open types, no generalisation,
//! no principal-type search.  The kernel never sees a type variable: the
//! elaborator inlines the built term at the use site, and `bdl-check` types
//! the result like any other Core term (`lib_expansion`: an inlined
//! combinator adds no privilege — typing, construction and clocking are
//! those of its arguments).
//!
//! Ordering (Phase 9c): `Ord` holds for a quantity, or for a concept the
//! designer *declared* ordered that is represented by a quantity; an
//! ordered concept compares through its representation and `min`/`max`/
//! `clamp` return the original concept value.  Nothing else has an order.
//! Equality (`Eq`) coincides with `Data` on this type grammar.

#![forbid(unsafe_code)]

use bdl_ir::{Expr, Prim, Scalar, Ty};
use bdl_model::{ConceptId, Dim};
use std::collections::BTreeMap;

/// A dimension pattern: a constant, or a dimension variable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PDim {
    Const(Dim),
    Var(u8),
}

/// A type pattern: the kernel's type formers over type variables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PTy {
    Var(u8),
    Bool,
    Nat,
    Sem(ConceptId),
    Q(PDim),
    Opt(Box<PTy>),
    List(Box<PTy>),
    Prod(Box<PTy>, Box<PTy>),
    Arr(Box<PTy>, Box<PTy>),
}

impl PTy {
    pub fn opt(t: PTy) -> PTy {
        PTy::Opt(Box::new(t))
    }
    pub fn list(t: PTy) -> PTy {
        PTy::List(Box::new(t))
    }
    pub fn prod(a: PTy, b: PTy) -> PTy {
        PTy::Prod(Box::new(a), Box::new(b))
    }
    pub fn arr(a: PTy, b: PTy) -> PTy {
        PTy::Arr(Box::new(a), Box::new(b))
    }
    /// `a₁ → … → aₙ → b`.
    pub fn arrows(inputs: impl IntoIterator<Item = PTy>, output: PTy) -> PTy {
        let inputs: Vec<PTy> = inputs.into_iter().collect();
        inputs
            .into_iter()
            .rev()
            .fold(output, |cod, dom| PTy::arr(dom, cod))
    }
    /// The domains and the final codomain of an arrow pattern.
    pub fn uncurry(&self) -> (Vec<&PTy>, &PTy) {
        let mut doms = Vec::new();
        let mut t = self;
        while let PTy::Arr(a, b) = t {
            doms.push(a.as_ref());
            t = b;
        }
        (doms, t)
    }
}

/// The closed capability vocabulary (`Poly.Cap`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap {
    /// May be delayed or transported: no function inside.
    Data,
    /// Has structural equality — coincides with `Data` today, kept as a
    /// separate name because it answers a different question.
    Eq,
    /// Has a designer-meaningful order: a quantity, or a concept declared
    /// ordered and represented by one.
    Ord,
}

/// A scheme: the parameter patterns (in the designer's order), the result
/// pattern, and the capability each type variable needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheme {
    pub params: Vec<PTy>,
    pub result: PTy,
    pub caps: Vec<(u8, Cap)>,
}

/// A substitution accumulated by matching: type variables to closed types,
/// dimension variables to dimensions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Subst {
    pub tys: BTreeMap<u8, Ty>,
    pub dims: BTreeMap<u8, Dim>,
}

/// Why a pattern did not match a closed type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchError {
    /// The type has a different shape from the pattern.
    Shape { pattern: PTy, found: Ty },
    /// A type variable was already bound to a different type.
    TyConflict { var: u8, bound: Ty, found: Ty },
    /// A dimension variable was already bound to a different dimension.
    DimConflict { var: u8, bound: Dim, found: Dim },
}

/// First-order matching of a pattern against a closed type, extending the
/// substitution.  Deterministic: a successful match yields the unique
/// substitution on the pattern's variables (`matchTy_sound`,
/// `matchTy_complete`).
pub fn match_ty(p: &PTy, t: &Ty, s: &mut Subst) -> Result<(), MatchError> {
    match (p, t) {
        (PTy::Var(v), t) => match s.tys.get(v) {
            Some(bound) if bound == t => Ok(()),
            Some(bound) => Err(MatchError::TyConflict {
                var: *v,
                bound: bound.clone(),
                found: t.clone(),
            }),
            None => {
                s.tys.insert(*v, t.clone());
                Ok(())
            }
        },
        (PTy::Bool, Ty::Bool) | (PTy::Nat, Ty::Nat) => Ok(()),
        (PTy::Sem(a), Ty::Sem { id }) if a == id => Ok(()),
        (PTy::Q(PDim::Const(d)), Ty::Q { dim }) if d == dim => Ok(()),
        (PTy::Q(PDim::Var(v)), Ty::Q { dim }) => match s.dims.get(v) {
            Some(bound) if bound == dim => Ok(()),
            Some(bound) => Err(MatchError::DimConflict {
                var: *v,
                bound: *bound,
                found: *dim,
            }),
            None => {
                s.dims.insert(*v, *dim);
                Ok(())
            }
        },
        (PTy::Opt(a), Ty::Opt { inner }) => match_ty(a, inner, s),
        (PTy::List(a), Ty::List { elem }) => match_ty(a, elem, s),
        (PTy::Prod(a, b), Ty::Prod { fst, snd }) => {
            match_ty(a, fst, s)?;
            match_ty(b, snd, s)
        }
        (PTy::Arr(a, b), Ty::Arr { dom, cod }) => {
            match_ty(a, dom, s)?;
            match_ty(b, cod, s)
        }
        _ => Err(MatchError::Shape {
            pattern: p.clone(),
            found: t.clone(),
        }),
    }
}

/// The closed type a pattern denotes under a substitution; `None` while a
/// variable it mentions is unbound.
pub fn instantiate(p: &PTy, s: &Subst) -> Option<Ty> {
    Some(match p {
        PTy::Var(v) => s.tys.get(v)?.clone(),
        PTy::Bool => Ty::Bool,
        PTy::Nat => Ty::Nat,
        PTy::Sem(id) => Ty::sem(*id),
        PTy::Q(PDim::Const(d)) => Ty::q(*d),
        PTy::Q(PDim::Var(v)) => Ty::q(*s.dims.get(v)?),
        PTy::Opt(a) => Ty::opt(instantiate(a, s)?),
        PTy::List(a) => Ty::list(instantiate(a, s)?),
        PTy::Prod(a, b) => Ty::prod(instantiate(a, s)?, instantiate(b, s)?),
        PTy::Arr(a, b) => Ty::arr(instantiate(a, s)?, instantiate(b, s)?),
    })
}

/// `Stdlib.Ordered`: evidence that a type has a designer-meaningful order —
/// a quantity of dimension `d`, or a concept declared ordered whose
/// representation is a quantity of dimension `d`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ordered {
    Q(Dim),
    Sem(ConceptId, Dim),
}

impl Ordered {
    pub fn dim(self) -> Dim {
        match self {
            Ordered::Q(d) | Ordered::Sem(_, d) => d,
        }
    }
}

/// One instance of a scheme: the substitution and, for every variable that
/// needed `Ord`, its evidence.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Instance {
    pub subst: Subst,
    pub ordered: BTreeMap<u8, Ordered>,
}

impl Instance {
    /// The type a variable stands for (`bool` for a variable the scheme
    /// never binds, which the builders never read).
    pub fn ty(&self, v: u8) -> Ty {
        self.subst.tys.get(&v).cloned().unwrap_or(Ty::Bool)
    }
    pub fn dim(&self, v: u8) -> Dim {
        self.subst.dims.get(&v).copied().unwrap_or(Dim::ZERO)
    }
    fn ord(&self, v: u8) -> Ordered {
        self.ordered
            .get(&v)
            .copied()
            .unwrap_or(Ordered::Q(Dim::ZERO))
    }
}

/// A library entry: its designer-facing name, its scheme, what it means,
/// and the closed Core term at any instance.
pub struct Entry {
    pub name: &'static str,
    /// The parameter names, for signature help and hover.
    pub params: &'static [&'static str],
    pub scheme: Scheme,
    /// One sentence in the designer's vocabulary.
    pub summary: &'static str,
    /// The designer-facing category ("Any", "In range", …).
    pub intent: &'static str,
    /// The combinator (`Stdlib.lean`'s name) this entry is, for the
    /// explanation view and the correspondence table.
    pub combinator: &'static str,
    pub build: fn(&Instance) -> Expr,
}

impl Entry {
    /// `min(a, b)`, for messages.
    pub fn shape(&self) -> String {
        format!("{}({})", self.name, self.params.join(", "))
    }
}

// ---- builders: closed de Bruijn terms ---------------------------------------

const A: u8 = 0;
const B: u8 = 1;
const D: u8 = 0;

fn a() -> PTy {
    PTy::Var(A)
}
fn b() -> PTy {
    PTy::Var(B)
}
fn qd() -> PTy {
    PTy::Q(PDim::Var(D))
}
fn q0() -> PTy {
    PTy::Q(PDim::Const(Dim::ZERO))
}

fn var(i: u32) -> Expr {
    Expr::var(i)
}
fn app2(f: Expr, x: Expr, y: Expr) -> Expr {
    Expr::apps(f, [x, y])
}
fn app3(f: Expr, x: Expr, y: Expr, z: Expr) -> Expr {
    Expr::apps(f, [x, y, z])
}
fn ite(ty: Ty, c: Expr, x: Expr, y: Expr) -> Expr {
    app3(Expr::prim(Prim::Ite { ty }), c, x, y)
}
fn and(x: Expr, y: Expr) -> Expr {
    app2(Expr::prim(Prim::And), x, y)
}
fn or(x: Expr, y: Expr) -> Expr {
    app2(Expr::prim(Prim::Or), x, y)
}
fn not(x: Expr) -> Expr {
    Expr::app(Expr::prim(Prim::Not), x)
}
fn cons(ty: Ty, x: Expr, l: Expr) -> Expr {
    app2(Expr::prim(Prim::Cons { ty }), x, l)
}
fn nil(ty: Ty) -> Expr {
    Expr::prim(Prim::Nil { ty })
}
fn pair(fst: Ty, snd: Ty, x: Expr, y: Expr) -> Expr {
    app2(Expr::prim(Prim::Pair { fst, snd }), x, y)
}
fn fst(fst: Ty, snd: Ty, p: Expr) -> Expr {
    Expr::app(Expr::prim(Prim::Fst { fst, snd }), p)
}
fn snd(fst: Ty, snd: Ty, p: Expr) -> Expr {
    Expr::app(Expr::prim(Prim::Snd { fst, snd }), p)
}
fn some(ty: Ty, x: Expr) -> Expr {
    Expr::app(Expr::prim(Prim::Some { ty }), x)
}
fn none(ty: Ty) -> Expr {
    Expr::prim(Prim::None { ty })
}
fn to_list(ty: Ty, o: Expr) -> Expr {
    Expr::app(Expr::prim(Prim::ToList { ty }), o)
}
fn take(ty: Ty, k: Expr, l: Expr) -> Expr {
    app2(Expr::prim(Prim::Take { ty }), k, l)
}
fn drop(ty: Ty, k: Expr, l: Expr) -> Expr {
    app2(Expr::prim(Prim::Drop { ty }), k, l)
}
fn rev(ty: Ty, l: Expr) -> Expr {
    Expr::app(Expr::prim(Prim::Reverse { ty }), l)
}
fn lit(dim: Dim, v: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim,
        value: Scalar(v),
    })
}

/// `a < b` at an ordered type: directly on quantities, through the
/// representation on an ordered concept (`Stdlib.ltAt`).
fn lt_at(o: Ordered, x: Expr, y: Expr) -> Expr {
    match o {
        Ordered::Q(d) => app2(Expr::prim(Prim::Lt { dim: d }), x, y),
        Ordered::Sem(_, d) => app2(Expr::prim(Prim::Lt { dim: d }), Expr::rep(x), Expr::rep(y)),
    }
}

/// `id(x) : α → α`
fn id_f(i: &Instance) -> Expr {
    Expr::lam(i.ty(A), var(0))
}
/// `const(x, y) : α → β → α`
fn const_f(i: &Instance) -> Expr {
    Expr::lam(i.ty(A), Expr::lam(i.ty(B), var(1)))
}
/// `swap(p) : α × β → β × α`
fn swap_f(i: &Instance) -> Expr {
    let (ta, tb) = (i.ty(A), i.ty(B));
    Expr::lam(
        Ty::prod(ta.clone(), tb.clone()),
        pair(
            tb.clone(),
            ta.clone(),
            snd(ta.clone(), tb.clone(), var(0)),
            fst(ta, tb, var(0)),
        ),
    )
}
/// `min(a, b) : α → α → α` for ordered `α` (`minF`)
fn min_f(i: &Instance) -> Expr {
    let t = i.ty(A);
    Expr::lam(
        t.clone(),
        Expr::lam(
            t.clone(),
            ite(t, lt_at(i.ord(A), var(1), var(0)), var(1), var(0)),
        ),
    )
}
/// `max(a, b)` (`maxF`)
fn max_f(i: &Instance) -> Expr {
    let t = i.ty(A);
    Expr::lam(
        t.clone(),
        Expr::lam(
            t.clone(),
            ite(t, lt_at(i.ord(A), var(1), var(0)), var(0), var(1)),
        ),
    )
}
/// `clamp(x, lo, hi) = max(lo, min(x, hi))` (`clampF`)
fn clamp_f(i: &Instance) -> Expr {
    let t = i.ty(A);
    Expr::lam(
        t.clone(),
        Expr::lam(
            t.clone(),
            Expr::lam(t, app2(max_f(i), var(1), app2(min_f(i), var(2), var(0)))),
        ),
    )
}
/// `inRange(x, lo, hi) = lo ≤ x ∧ x ≤ hi` (`inRangeF`)
fn in_range_f(i: &Instance) -> Expr {
    let t = i.ty(A);
    let o = i.ord(A);
    Expr::lam(
        t.clone(),
        Expr::lam(
            t.clone(),
            Expr::lam(
                t,
                and(not(lt_at(o, var(2), var(1))), not(lt_at(o, var(0), var(2)))),
            ),
        ),
    )
}
/// `inInterval(x, interval)`: an interval is a pair `(lo, hi)` (`inIntervalF`)
fn in_interval_f(i: &Instance) -> Expr {
    let t = i.ty(A);
    let iv = Ty::prod(t.clone(), t.clone());
    Expr::lam(
        t.clone(),
        Expr::lam(
            iv,
            app3(
                in_range_f(i),
                var(1),
                fst(t.clone(), t.clone(), var(0)),
                snd(t.clone(), t, var(0)),
            ),
        ),
    )
}
/// `minBy(a, b, less)`: the comparator escape hatch (`minByF`)
fn min_by_f(i: &Instance) -> Expr {
    let t = i.ty(A);
    let cmp = Ty::arr(t.clone(), Ty::arr(t.clone(), Ty::Bool));
    Expr::lam(
        t.clone(),
        Expr::lam(
            t.clone(),
            Expr::lam(cmp, ite(t, app2(var(0), var(2), var(1)), var(2), var(1))),
        ),
    )
}
/// `maxBy(a, b, less)` (`maxByF`)
fn max_by_f(i: &Instance) -> Expr {
    let t = i.ty(A);
    let cmp = Ty::arr(t.clone(), Ty::arr(t.clone(), Ty::Bool));
    Expr::lam(
        t.clone(),
        Expr::lam(
            t.clone(),
            Expr::lam(cmp, ite(t, app2(var(0), var(2), var(1)), var(1), var(2))),
        ),
    )
}
/// `foldr(xs, init, step)`: the recursor as an equation (`foldrF`)
fn foldr_f(i: &Instance) -> Expr {
    let (ta, tb) = (i.ty(A), i.ty(B));
    let step = Ty::arr(ta.clone(), Ty::arr(tb.clone(), tb.clone()));
    Expr::lam(
        Ty::list(ta),
        Expr::lam(tb, Expr::lam(step, Expr::fold(var(0), var(1), var(2)))),
    )
}
/// `any(xs, p)` (`anyF`)
fn any_f(i: &Instance) -> Expr {
    let ta = i.ty(A);
    Expr::lam(
        Ty::list(ta.clone()),
        Expr::lam(
            Ty::arr(ta.clone(), Ty::Bool),
            Expr::fold(
                Expr::lam(
                    ta,
                    Expr::lam(Ty::Bool, or(Expr::app(var(2), var(1)), var(0))),
                ),
                Expr::BoolLit { value: false },
                var(1),
            ),
        ),
    )
}
/// `all(xs, p)` (`allF`)
fn all_f(i: &Instance) -> Expr {
    let ta = i.ty(A);
    Expr::lam(
        Ty::list(ta.clone()),
        Expr::lam(
            Ty::arr(ta.clone(), Ty::Bool),
            Expr::fold(
                Expr::lam(
                    ta,
                    Expr::lam(Ty::Bool, and(Expr::app(var(2), var(1)), var(0))),
                ),
                Expr::BoolLit { value: true },
                var(1),
            ),
        ),
    )
}
/// `contains(x, xs)` for data `α` (`containsF`)
fn contains_f(i: &Instance) -> Expr {
    let ta = i.ty(A);
    Expr::lam(
        ta.clone(),
        Expr::lam(
            Ty::list(ta.clone()),
            Expr::fold(
                Expr::lam(
                    ta.clone(),
                    Expr::lam(
                        Ty::Bool,
                        or(
                            app2(Expr::prim(Prim::Eq { ty: ta }), var(3), var(1)),
                            var(0),
                        ),
                    ),
                ),
                Expr::BoolLit { value: false },
                var(0),
            ),
        ),
    )
}
/// `map(xs, f)` (`mapF`)
fn map_f(i: &Instance) -> Expr {
    let (ta, tb) = (i.ty(A), i.ty(B));
    Expr::lam(
        Ty::list(ta.clone()),
        Expr::lam(
            Ty::arr(ta.clone(), tb.clone()),
            Expr::fold(
                Expr::lam(
                    ta,
                    Expr::lam(
                        Ty::list(tb.clone()),
                        cons(tb.clone(), Expr::app(var(2), var(1)), var(0)),
                    ),
                ),
                nil(tb),
                var(1),
            ),
        ),
    )
}
/// `filter(xs, p)` (`filterF`)
fn filter_f(i: &Instance) -> Expr {
    let ta = i.ty(A);
    Expr::lam(
        Ty::list(ta.clone()),
        Expr::lam(
            Ty::arr(ta.clone(), Ty::Bool),
            Expr::fold(
                Expr::lam(
                    ta.clone(),
                    Expr::lam(
                        Ty::list(ta.clone()),
                        ite(
                            Ty::list(ta.clone()),
                            Expr::app(var(2), var(1)),
                            cons(ta.clone(), var(1), var(0)),
                            var(0),
                        ),
                    ),
                ),
                nil(ta),
                var(1),
            ),
        ),
    )
}
/// `append(xs, ys)` (`appendF`)
fn append_f(i: &Instance) -> Expr {
    let ta = i.ty(A);
    Expr::lam(
        Ty::list(ta.clone()),
        Expr::lam(
            Ty::list(ta.clone()),
            Expr::fold(Expr::prim(Prim::Cons { ty: ta }), var(0), var(1)),
        ),
    )
}
/// `sum(xs) : list (q δ) → q δ` — dimension-generic (`sumF`)
fn sum_f(i: &Instance) -> Expr {
    let d = i.dim(D);
    Expr::lam(
        Ty::list(Ty::q(d)),
        Expr::fold(Expr::prim(Prim::Add { dim: d }), lit(d, 0.0), var(0)),
    )
}
/// `optElim(o, default, f)`: options are lists of length ≤ 1 (`optElimF`)
fn opt_elim_f(i: &Instance) -> Expr {
    let (ta, tb) = (i.ty(A), i.ty(B));
    Expr::lam(
        Ty::opt(ta.clone()),
        Expr::lam(
            tb.clone(),
            Expr::lam(
                Ty::arr(ta.clone(), tb.clone()),
                Expr::fold(
                    Expr::lam(ta.clone(), Expr::lam(tb, Expr::app(var(2), var(1)))),
                    var(1),
                    to_list(ta, var(2)),
                ),
            ),
        ),
    )
}
/// `mapOpt(o, f)` (`mapOptF`)
fn map_opt_f(i: &Instance) -> Expr {
    let (ta, tb) = (i.ty(A), i.ty(B));
    Expr::lam(
        Ty::opt(ta.clone()),
        Expr::lam(
            Ty::arr(ta.clone(), tb.clone()),
            Expr::fold(
                Expr::lam(
                    ta.clone(),
                    Expr::lam(
                        Ty::opt(tb.clone()),
                        some(tb.clone(), Expr::app(var(2), var(1))),
                    ),
                ),
                none(tb),
                to_list(ta, var(1)),
            ),
        ),
    )
}
/// `getOrElse(o, default)` is the registered `getD`
fn get_or_else_f(i: &Instance) -> Expr {
    Expr::prim(Prim::GetD { ty: i.ty(A) })
}
/// `zip(xs, ys) : list α → list β → list (α × β)` — a fold over `reverse xs`
/// with the remaining `ys` and the result accumulated in a pair (`zipF`).
fn zip_f(i: &Instance) -> Expr {
    let (ta, tb) = (i.ty(A), i.ty(B));
    let tab = Ty::prod(ta.clone(), tb.clone());
    let acc = Ty::prod(Ty::list(tb.clone()), Ty::list(tab.clone()));
    let one = || lit(Dim::ZERO, 1.0);
    let rest = |p: Expr| fst(Ty::list(tb.clone()), Ty::list(tab.clone()), p);
    let done = |p: Expr| snd(Ty::list(tb.clone()), Ty::list(tab.clone()), p);
    // λx. λacc. (drop 1 (fst acc), fold cons (snd acc) (fold (λy. λl. cons (x, y) l) [] (take 1 (fst acc))))
    let step = Expr::lam(
        ta.clone(),
        Expr::lam(
            acc.clone(),
            pair(
                Ty::list(tb.clone()),
                Ty::list(tab.clone()),
                drop(tb.clone(), one(), rest(var(0))),
                Expr::fold(
                    Expr::prim(Prim::Cons { ty: tab.clone() }),
                    done(var(0)),
                    Expr::fold(
                        Expr::lam(
                            tb.clone(),
                            Expr::lam(
                                Ty::list(tab.clone()),
                                cons(
                                    tab.clone(),
                                    pair(ta.clone(), tb.clone(), var(3), var(1)),
                                    var(0),
                                ),
                            ),
                        ),
                        nil(tab.clone()),
                        take(tb.clone(), one(), rest(var(0))),
                    ),
                ),
            ),
        ),
    );
    Expr::lam(
        Ty::list(ta.clone()),
        Expr::lam(
            Ty::list(tb.clone()),
            rev(
                tab.clone(),
                done(Expr::fold(
                    step,
                    pair(
                        Ty::list(tb.clone()),
                        Ty::list(tab.clone()),
                        var(0),
                        nil(tab.clone()),
                    ),
                    rev(ta, var(1)),
                )),
            ),
        ),
    )
}
fn length_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Length { ty: i.ty(A) })
}
fn head_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Head { ty: i.ty(A) })
}
fn reverse_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Reverse { ty: i.ty(A) })
}
fn take_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Take { ty: i.ty(A) })
}
fn drop_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Drop { ty: i.ty(A) })
}
fn cons_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Cons { ty: i.ty(A) })
}
fn to_list_f(i: &Instance) -> Expr {
    Expr::prim(Prim::ToList { ty: i.ty(A) })
}
fn pair_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Pair {
        fst: i.ty(A),
        snd: i.ty(B),
    })
}
fn first_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Fst {
        fst: i.ty(A),
        snd: i.ty(B),
    })
}
fn second_f(i: &Instance) -> Expr {
    Expr::prim(Prim::Snd {
        fst: i.ty(A),
        snd: i.ty(B),
    })
}

// ---- the table -------------------------------------------------------------

fn scheme(params: Vec<PTy>, result: PTy, caps: Vec<(u8, Cap)>) -> Scheme {
    Scheme {
        params,
        result,
        caps,
    }
}

fn entry(
    name: &'static str,
    params: &'static [&'static str],
    scheme: Scheme,
    summary: &'static str,
    intent: &'static str,
    combinator: &'static str,
    build: fn(&Instance) -> Expr,
) -> Entry {
    Entry {
        name,
        params,
        scheme,
        summary,
        intent,
        combinator,
        build,
    }
}

fn table() -> Vec<Entry> {
    let pred = || PTy::arr(a(), PTy::Bool);
    vec![
        entry(
            "id",
            &["x"],
            scheme(vec![a()], a(), vec![]),
            "The value itself.",
            "Value",
            "idF",
            id_f,
        ),
        entry(
            "const",
            &["x", "y"],
            scheme(vec![a(), b()], a(), vec![]),
            "Always the first value, whatever the second.",
            "Value",
            "constF",
            const_f,
        ),
        entry(
            "swap",
            &["pair"],
            scheme(vec![PTy::prod(a(), b())], PTy::prod(b(), a()), vec![]),
            "The grouped value with its two parts exchanged.",
            "Grouped value",
            "swapF",
            swap_f,
        ),
        entry(
            "min",
            &["a", "b"],
            scheme(vec![a(), a()], a(), vec![(A, Cap::Ord)]),
            "The smaller of two values of the same ordered kind.",
            "Smallest",
            "minF",
            min_f,
        ),
        entry(
            "max",
            &["a", "b"],
            scheme(vec![a(), a()], a(), vec![(A, Cap::Ord)]),
            "The larger of two values of the same ordered kind.",
            "Largest",
            "maxF",
            max_f,
        ),
        entry(
            "clamp",
            &["x", "low", "high"],
            scheme(vec![a(), a(), a()], a(), vec![(A, Cap::Ord)]),
            "The value held between a low and a high bound.",
            "Clamp",
            "clampF",
            clamp_f,
        ),
        entry(
            "inRange",
            &["x", "low", "high"],
            scheme(vec![a(), a(), a()], PTy::Bool, vec![(A, Cap::Ord)]),
            "Whether the value lies between the bounds, inclusive.",
            "In range",
            "inRangeF",
            in_range_f,
        ),
        entry(
            "inInterval",
            &["x", "interval"],
            scheme(
                vec![a(), PTy::prod(a(), a())],
                PTy::Bool,
                vec![(A, Cap::Ord)],
            ),
            "Whether the value lies within an interval given as (low, high).",
            "In range",
            "inIntervalF",
            in_interval_f,
        ),
        entry(
            "minBy",
            &["a", "b", "less"],
            scheme(
                vec![a(), a(), PTy::arrows([a(), a()], PTy::Bool)],
                a(),
                vec![],
            ),
            "The value a rule ranks lower: `less(x, y)` says x comes first.",
            "Smallest by rule",
            "minByF",
            min_by_f,
        ),
        entry(
            "maxBy",
            &["a", "b", "less"],
            scheme(
                vec![a(), a(), PTy::arrows([a(), a()], PTy::Bool)],
                a(),
                vec![],
            ),
            "The value a rule ranks higher.",
            "Largest by rule",
            "maxByF",
            max_by_f,
        ),
        entry(
            "foldr",
            &["collection", "start", "step"],
            scheme(
                vec![PTy::list(a()), b(), PTy::arrows([a(), b()], b())],
                b(),
                vec![],
            ),
            "Combine a collection from its last element to its first, starting from a value.",
            "Combine",
            "foldrF",
            foldr_f,
        ),
        entry(
            "any",
            &["collection", "condition"],
            scheme(vec![PTy::list(a()), pred()], PTy::Bool, vec![]),
            "Whether at least one element satisfies the condition.",
            "Any",
            "anyF",
            any_f,
        ),
        entry(
            "all",
            &["collection", "condition"],
            scheme(vec![PTy::list(a()), pred()], PTy::Bool, vec![]),
            "Whether every element satisfies the condition (true of an empty collection).",
            "All",
            "allF",
            all_f,
        ),
        entry(
            "contains",
            &["x", "collection"],
            scheme(vec![a(), PTy::list(a())], PTy::Bool, vec![(A, Cap::Eq)]),
            "Whether the collection holds the value; `x in [a, b]` is the same.",
            "One of",
            "containsF",
            contains_f,
        ),
        entry(
            "oneOf",
            &["x", "options"],
            scheme(vec![a(), PTy::list(a())], PTy::Bool, vec![(A, Cap::Eq)]),
            "Whether the value is one of the options; `x in [a, b]` is the same.",
            "One of",
            "oneOfE",
            contains_f,
        ),
        entry(
            "map",
            &["collection", "rule"],
            scheme(
                vec![PTy::list(a()), PTy::arr(a(), b())],
                PTy::list(b()),
                vec![],
            ),
            "The collection with the rule applied to every element.",
            "Each",
            "mapF",
            map_f,
        ),
        entry(
            "filter",
            &["collection", "condition"],
            scheme(vec![PTy::list(a()), pred()], PTy::list(a()), vec![]),
            "The elements that satisfy the condition, in order.",
            "Those",
            "filterF",
            filter_f,
        ),
        entry(
            "append",
            &["first", "second"],
            scheme(vec![PTy::list(a()), PTy::list(a())], PTy::list(a()), vec![]),
            "One collection followed by another.",
            "Collection",
            "appendF",
            append_f,
        ),
        entry(
            "sum",
            &["quantities"],
            scheme(vec![PTy::list(qd())], qd(), vec![]),
            "The total of a collection of quantities, in their dimension.",
            "Total",
            "sumF",
            sum_f,
        ),
        entry(
            "zip",
            &["first", "second"],
            scheme(
                vec![PTy::list(a()), PTy::list(b())],
                PTy::list(PTy::prod(a(), b())),
                vec![],
            ),
            "The two collections paired element by element, as long as the shorter.",
            "Pair up",
            "zipF",
            zip_f,
        ),
        entry(
            "optElim",
            &["optional", "default", "rule"],
            scheme(vec![PTy::opt(a()), b(), PTy::arr(a(), b())], b(), vec![]),
            "The rule applied to the value when present, the default when absent.",
            "Optional value",
            "optElimF",
            opt_elim_f,
        ),
        entry(
            "mapOpt",
            &["optional", "rule"],
            scheme(
                vec![PTy::opt(a()), PTy::arr(a(), b())],
                PTy::opt(b()),
                vec![],
            ),
            "The rule applied inside an optional value; absent stays absent.",
            "Optional value",
            "mapOptF",
            map_opt_f,
        ),
        entry(
            "getOrElse",
            &["optional", "default"],
            scheme(vec![PTy::opt(a()), a()], a(), vec![]),
            "The value when present, the default when absent.",
            "Optional value",
            "getOrElseF",
            get_or_else_f,
        ),
        entry(
            "length",
            &["collection"],
            scheme(vec![PTy::list(a())], q0(), vec![]),
            "How many elements the collection has.",
            "Collection",
            "length",
            length_f,
        ),
        entry(
            "head",
            &["collection"],
            scheme(vec![PTy::list(a())], PTy::opt(a()), vec![]),
            "The first element, if there is one.",
            "Collection",
            "head",
            head_f,
        ),
        entry(
            "reverse",
            &["collection"],
            scheme(vec![PTy::list(a())], PTy::list(a()), vec![]),
            "The elements in the opposite order.",
            "Collection",
            "reverse",
            reverse_f,
        ),
        entry(
            "take",
            &["count", "collection"],
            scheme(vec![q0(), PTy::list(a())], PTy::list(a()), vec![]),
            "The first elements, as many as the count.",
            "Collection",
            "take",
            take_f,
        ),
        entry(
            "drop",
            &["count", "collection"],
            scheme(vec![q0(), PTy::list(a())], PTy::list(a()), vec![]),
            "The collection without its first elements, as many as the count.",
            "Collection",
            "drop",
            drop_f,
        ),
        entry(
            "cons",
            &["x", "collection"],
            scheme(vec![a(), PTy::list(a())], PTy::list(a()), vec![]),
            "The collection with a value put in front.",
            "Collection",
            "cons",
            cons_f,
        ),
        entry(
            "toList",
            &["optional"],
            scheme(vec![PTy::opt(a())], PTy::list(a()), vec![]),
            "An optional value as a collection of zero or one element.",
            "Collection",
            "toList",
            to_list_f,
        ),
        entry(
            "pair",
            &["first", "second"],
            scheme(vec![a(), b()], PTy::prod(a(), b()), vec![]),
            "Two values grouped together; `(a, b)` is the same.",
            "Grouped value",
            "pair",
            pair_f,
        ),
        entry(
            "first",
            &["pair"],
            scheme(vec![PTy::prod(a(), b())], a(), vec![]),
            "The first part of a grouped value.",
            "Grouped value",
            "fst",
            first_f,
        ),
        entry(
            "second",
            &["pair"],
            scheme(vec![PTy::prod(a(), b())], b(), vec![]),
            "The second part of a grouped value.",
            "Grouped value",
            "snd",
            second_f,
        ),
    ]
}

/// Every entry, in table order (deterministic).
pub fn entries() -> &'static [Entry] {
    static TABLE: std::sync::OnceLock<Vec<Entry>> = std::sync::OnceLock::new();
    TABLE.get_or_init(table)
}

/// The entry for a designer-facing name.
pub fn lookup(name: &str) -> Option<&'static Entry> {
    entries().iter().find(|e| e.name == name)
}

/// Every designer-facing name, in table order.
pub fn names() -> Vec<&'static str> {
    entries().iter().map(|e| e.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_check::{infer, Grant};
    use bdl_ir::DesignIr;

    fn q(d: Dim) -> Ty {
        Ty::q(d)
    }

    #[test]
    fn matching_is_first_order_and_unique() {
        let mut s = Subst::default();
        let p = PTy::arrows([PTy::list(a()), PTy::arr(a(), b())], PTy::list(b()));
        let t = Ty::arrows(
            [Ty::list(q(Dim::ZERO)), Ty::arr(q(Dim::ZERO), Ty::Bool)],
            Ty::list(Ty::Bool),
        );
        match_ty(&p, &t, &mut s).unwrap();
        assert_eq!(s.tys[&A], q(Dim::ZERO));
        assert_eq!(s.tys[&B], Ty::Bool);
        assert_eq!(instantiate(&p, &s), Some(t));
        // a conflict on a bound variable is reported, never resolved by search
        let mut s = Subst::default();
        match_ty(&a(), &q(Dim::LENGTH), &mut s).unwrap();
        assert!(matches!(
            match_ty(&a(), &q(Dim::TIME), &mut s),
            Err(MatchError::TyConflict { var: 0, .. })
        ));
        // dimension variables
        let mut s = Subst::default();
        match_ty(&PTy::list(qd()), &Ty::list(q(Dim::LENGTH)), &mut s).unwrap();
        assert_eq!(s.dims[&D], Dim::LENGTH);
        assert!(matches!(
            match_ty(&qd(), &q(Dim::TIME), &mut s),
            Err(MatchError::DimConflict { .. })
        ));
        // shapes
        assert!(matches!(
            match_ty(&PTy::list(a()), &q(Dim::ZERO), &mut Subst::default()),
            Err(MatchError::Shape { .. })
        ));
        // an unbound variable has no instance yet
        assert_eq!(instantiate(&b(), &Subst::default()), None);
    }

    /// Every entry, instantiated at representative closed types, types at
    /// its scheme's instance (`*_typed` at every instance).
    #[test]
    fn every_entry_types_at_its_scheme() {
        let ir = DesignIr::default();
        let cases: Vec<(Ty, Ty, Dim)> = vec![
            (q(Dim::ZERO), Ty::Bool, Dim::ZERO),
            (Ty::Bool, q(Dim::LENGTH), Dim::LENGTH),
            (
                Ty::prod(q(Dim::ZERO), Ty::Nat),
                Ty::opt(Ty::Bool),
                Dim::TIME,
            ),
            (Ty::list(Ty::Bool), q(Dim::ANGLE), Dim::ANGLE),
        ];
        for e in entries() {
            for (ta, tb, d) in &cases {
                let mut i = Instance::default();
                i.subst.tys.insert(A, ta.clone());
                i.subst.tys.insert(B, tb.clone());
                i.subst.dims.insert(D, *d);
                let needs_ord = e.scheme.caps.iter().any(|(_, c)| *c == Cap::Ord);
                if needs_ord {
                    // ordered instances: a quantity, and an ordered concept
                    i.subst.tys.insert(A, q(*d));
                    i.ordered.insert(A, Ordered::Q(*d));
                }
                let want = Ty::arrows(
                    e.scheme
                        .params
                        .iter()
                        .map(|p| instantiate(p, &i.subst).unwrap()),
                    instantiate(&e.scheme.result, &i.subst).unwrap(),
                );
                let term = (e.build)(&i);
                let got = infer(&ir, &Grant::None, &[], &term)
                    .unwrap_or_else(|err| panic!("{}: {err:?}", e.name));
                assert_eq!(got, want, "{} at {:?}", e.name, (ta, tb, d));
            }
        }
    }

    #[test]
    fn ordered_concepts_compare_through_their_representation() {
        let mut ir = DesignIr::default();
        let bright = ConceptId::from_raw(7);
        ir.concepts.insert(
            bright,
            bdl_ir::ConceptBinding {
                id: bright,
                name: "Brightness".into(),
                representation: Some(q(Dim::ZERO)),
                ordered: true,
            },
        );
        let mut i = Instance::default();
        i.subst.tys.insert(A, Ty::sem(bright));
        i.ordered.insert(A, Ordered::Sem(bright, Dim::ZERO));
        let term = (lookup("min").unwrap().build)(&i);
        // min at Brightness : Brightness → Brightness → Brightness
        assert_eq!(
            infer(&ir, &Grant::None, &[], &term).unwrap(),
            Ty::arrows([Ty::sem(bright), Ty::sem(bright)], Ty::sem(bright))
        );
        assert!(ir.is_ordered(&Ty::sem(bright)));
        assert!(!ir.is_ordered(&Ty::Bool));
        assert!(!ir.is_ordered(&Ty::list(q(Dim::ZERO))));
        assert!(!ir.is_ordered(&Ty::prod(q(Dim::ZERO), q(Dim::ZERO))));
        assert!(!ir.is_ordered(&Ty::opt(q(Dim::ZERO))));
    }

    #[test]
    fn names_are_unique_and_the_brief_is_covered() {
        let names = names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
        for required in [
            "id",
            "const",
            "swap",
            "min",
            "max",
            "clamp",
            "inRange",
            "inInterval",
            "foldr",
            "map",
            "filter",
            "append",
            "sum",
            "zip",
            "any",
            "all",
            "contains",
            "optElim",
            "mapOpt",
            "getOrElse",
            "minBy",
            "maxBy",
            "oneOf",
        ] {
            assert!(lookup(required).is_some(), "{required}");
        }
        // the weakest capability: Eq for membership, Ord for order, none otherwise
        let caps = |n: &str| lookup(n).unwrap().scheme.caps.clone();
        assert_eq!(caps("contains"), vec![(A, Cap::Eq)]);
        assert_eq!(caps("min"), vec![(A, Cap::Ord)]);
        assert_eq!(caps("clamp"), vec![(A, Cap::Ord)]);
        for free in [
            "map", "filter", "any", "all", "zip", "foldr", "sum", "minBy",
        ] {
            assert!(caps(free).is_empty(), "{free}");
        }
    }
}
