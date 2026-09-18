//! Formula → Core `Expr`.
//!
//! The elaborator types the surface expression at two levels.  A value is
//! either *semantic* — an input (`sem A`), a relationship without inputs,
//! or a relationship applied to semantic values — or a *representation*: a
//! quantity of some dimension, a truth value, a count, or an option of one
//! of those.  Arithmetic, comparisons, `if` conditions and literal patterns
//! work on representations; the kernel's primitives are dimension-indexed,
//! so to emit `add` the elaborator must know *which* `add`.  A semantic
//! value is observed with `rep` wherever a representation is needed; the
//! result of the whole formula is constructed with `mk` under the
//! declaration's own grant.  The checker (`bdl-check`) then re-derives the
//! type of the produced Core term; it is the authority, this pass is the
//! translator.
//!
//! Every surface form desugars into the *existing* Core; no primitive was
//! added for the textual syntax (`docs/spec/textual-syntax.md` §11):
//!
//! | surface | Core |
//! |---|---|
//! | `f(a, b)` | `app (app (declRef f) a) b` — `a`, `b` semantic values |
//! | `level` (relationship without inputs) | `declRef level` |
//! | `{ let x = v; e }` | `app (λx:τ. e) v` |
//! | `if c then a else b` | `ite c a b` (strict, DI-26) |
//! | `Some(e)` / `None` | `some e` / `none` over representations |
//! | `match s { p₁ => e₁, … }` | `app (λs. ite test₁ (lets… e₁) (ite test₂ … eₙ)) s` |
//! | `delay(init, v)` / `sync(d, init, v)` | `delay init v` / `sync d init v` |
//! | `[a, b]` / `(a, b)` | `cons a (cons b nil)` / `pair a b` |
//! | `min(a, b)`, `any(xs, x => p)`, … | the equation library's closed combinator applied: `app (app minF a) b` (`bdl-equations`) |
//! | `x in [a, b]` | `contains x [a, b]` |
//! | `a == b` on concept values | `eq (sem A) a b` — the same concept, or a nominal error |
//! | `a < b` on concept values | `lt d (rep a) (rep b)` — only for a concept declared ordered |
//!
//! `match` binds the scrutinee once, so a temporal form inside it keeps one
//! `ExprPath` (one state cell) whichever arm is taken; arm bindings are
//! `getD` projections of that variable, tests are `isSome` / `eq` / the
//! Boolean itself.  Because `ite` is strict, every arm is evaluated at every
//! tick (DI-26); `getD` on `None` yields the arm's default and is total.
//! Temporal forms are only legal where the kernel allows them — outside
//! every binder — so they may be a `let`'s value or a `match`'s scrutinee
//! but not a block's result, an arm's body or a formula over inputs.

use crate::names::{self, InputEnv, Lookup};
use crate::units;
use bdl_check::pretty;
use bdl_check::ExprPath;
use bdl_diagnostics::{Diagnostic, Entity, Severity, Span};
use bdl_equations::{self as equations, Cap, Entry, Instance, MatchError, Ordered, PDim, PTy};
use bdl_ir::{DesignIr, Expr, Prim, Scalar, Ty};
use bdl_model::surface::{Design, MappingBlock};
use bdl_model::{ClockId, DeclId, Dim, SemanticId};
use bdl_syntax::lower::Ident;
use bdl_syntax::lower::{SurfaceArm, SurfaceLet};
use bdl_syntax::{BinaryOp, ExprKind, PatternKind, SurfaceExpr, SurfacePattern, UnaryOp};
use std::collections::{BTreeMap, BTreeSet};

/// A successfully elaborated realization.
#[derive(Clone, Debug, PartialEq)]
pub struct Realized {
    pub expr: Expr,
    /// Source span of each Core sub-term the surface produced, by path.
    pub spans: BTreeMap<ExprPath, Span>,
}

/// Type of a surface sub-expression.
#[derive(Clone, Debug, PartialEq)]
enum STy {
    Q(Dim),
    Bool,
    Nat,
    /// A semantic value: an input, or a relationship's result.
    Sem(SemanticId),
    /// An optional representation value.
    Opt(Box<STy>),
    /// A collection (`list τ`); elements may be semantic values.
    List(Box<STy>),
    /// A grouped value (`τ × σ`); parts may be semantic values.
    Pair(Box<STy>, Box<STy>),
    /// The payload of a bare `None` (or the elements of `[]`) before
    /// context fixes it.
    Unknown,
    /// Already reported; silences cascades.
    Error,
}

impl STy {
    fn of(ty: &Ty) -> Option<STy> {
        match ty {
            Ty::Q { dim } => Some(STy::Q(*dim)),
            Ty::Bool => Some(STy::Bool),
            Ty::Nat => Some(STy::Nat),
            Ty::Sem { id } => Some(STy::Sem(*id)),
            Ty::Opt { inner } => STy::of(inner).map(|t| STy::Opt(Box::new(t))),
            Ty::List { elem } => STy::of(elem).map(|t| STy::List(Box::new(t))),
            Ty::Prod { fst, snd } => {
                Some(STy::Pair(Box::new(STy::of(fst)?), Box::new(STy::of(snd)?)))
            }
            Ty::Arr { .. } => None,
        }
    }

    fn to_ty(&self) -> Option<Ty> {
        Some(match self {
            STy::Q(d) => Ty::q(*d),
            STy::Bool => Ty::Bool,
            STy::Nat => Ty::Nat,
            STy::Sem(s) => Ty::sem(*s),
            STy::Opt(t) => Ty::opt(t.to_ty()?),
            STy::List(t) => Ty::list(t.to_ty()?),
            STy::Pair(a, b) => Ty::prod(a.to_ty()?, b.to_ty()?),
            STy::Unknown | STy::Error => return None,
        })
    }

    /// Fully determined: no `Unknown`, no `Error`.
    fn is_known(&self) -> bool {
        match self {
            STy::Opt(t) | STy::List(t) => t.is_known(),
            STy::Pair(a, b) => a.is_known() && b.is_known(),
            STy::Unknown | STy::Error => false,
            _ => true,
        }
    }

    fn is_error(&self) -> bool {
        matches!(self, STy::Error)
    }

    /// The common type of two branches, if one exists (`Unknown` yields).
    fn unify(a: &STy, b: &STy) -> Option<STy> {
        match (a, b) {
            (STy::Unknown, t) | (t, STy::Unknown) => Some(t.clone()),
            (STy::Opt(x), STy::Opt(y)) => STy::unify(x, y).map(|t| STy::Opt(Box::new(t))),
            (STy::List(x), STy::List(y)) => STy::unify(x, y).map(|t| STy::List(Box::new(t))),
            (STy::Pair(a1, b1), STy::Pair(a2, b2)) => Some(STy::Pair(
                Box::new(STy::unify(a1, a2)?),
                Box::new(STy::unify(b1, b2)?),
            )),
            _ if a == b => Some(a.clone()),
            _ => None,
        }
    }
}

/// A lexical binding: an input (named through the signature), a `let`, a
/// pattern variable, or an unnamed scrutinee.  Position in the stack is the
/// binding's depth; the de Bruijn index is the distance from the top.
struct Binding {
    name: Option<String>,
    ty: STy,
}

struct Elab<'a> {
    design: &'a Design,
    ir: &'a DesignIr,
    mapping: &'a MappingBlock,
    inputs: InputEnv,
    env: Vec<Binding>,
    diags: Vec<Diagnostic>,
    spans: BTreeMap<ExprPath, Span>,
}

/// Elaborate `mapping`'s formula `source` into `λ x₁ … xₙ. mk B (…)`.
///
/// `Err` carries the diagnostics that stopped elaboration (parse errors,
/// unknown names, unbound representations, dimension and kind errors); a
/// realization is only produced when every sub-expression typed, and comes
/// with its non-error diagnostics (unreachable match arms).
pub fn elaborate_formula(
    design: &Design,
    ir: &DesignIr,
    mapping: &MappingBlock,
    source: &str,
) -> Result<(Realized, Vec<Diagnostic>), Vec<Diagnostic>> {
    let env = InputEnv::for_mapping(design, mapping);
    elaborate_formula_in(design, ir, mapping, source, env)
}

/// [`elaborate_formula`] with an explicit name environment — a pinned
/// scope for `Definition::ScopedFormula`.
pub fn elaborate_formula_in(
    design: &Design,
    ir: &DesignIr,
    mapping: &MappingBlock,
    source: &str,
    names: InputEnv,
) -> Result<(Realized, Vec<Diagnostic>), Vec<Diagnostic>> {
    let entity = Entity::Mapping { id: mapping.id };
    let surface = match bdl_syntax::formula(source) {
        Ok(e) => e,
        Err(errors) => {
            return Err(errors
                .into_iter()
                .map(|e| {
                    let technical = format!("{}: {}", e.code.as_str(), e.technical());
                    let mut d =
                        Diagnostic::error("formula.parse.unexpected_token", entity, e.message)
                            .at(e.span)
                            .technical(technical);
                    if let Some(hint) = e.hint {
                        d = d.fix(hint);
                    }
                    d
                })
                .collect())
        }
    };

    let inputs = &mapping.signature.inputs;
    let mut el = Elab {
        design,
        ir,
        mapping,
        inputs: names,
        env: inputs
            .iter()
            .map(|c| Binding {
                name: None,
                ty: STy::Sem(*c),
            })
            .collect(),
        diags: Vec::new(),
        spans: BTreeMap::new(),
    };

    // Path of the body: under n lambdas, then under `mk`.
    let n = inputs.len();
    let mut path: ExprPath = vec![0; n + 1];
    let (body, body_ty) = el.expr(&surface, &mut path, None);
    let (body, body_ty) = el.observe(body, body_ty, &path, surface.span);

    // The result is constructed as the output concept.
    let out = mapping.signature.output;
    let out_rep = ir.representation_of(out).and_then(STy::of);
    // Concept values inside a grouped value or a collection are observed
    // where the output's representation needs their representations:
    // `(t, h)` for a `Pair<Temperature, Scalar>` concept.
    let (body, body_ty) = match &out_rep {
        Some(want) if !body_ty.is_error() && &body_ty != want => {
            match el.coerce(body.clone(), &body_ty, want) {
                Some(e) => (e, want.clone()),
                None => (body, body_ty),
            }
        }
        _ => (body, body_ty),
    };
    match (&out_rep, &body_ty) {
        (None, _) => el.diags.push(el.unbound(out, surface.span, "produces")),
        (Some(_), STy::Error) => {}
        (Some(expected), found) if expected != found => {
            let out_name = el.concept_name(out);
            el.diags.push(
                Diagnostic::error(
                    "realization.type_mismatch",
                    entity,
                    format!(
                        "{out_name} is {}, but this formula produces {}.",
                        el.describe(expected),
                        el.describe(found)
                    ),
                )
                .at(surface.span)
                .explain(format!(
                    "The mapping's signature promises {out_name}; the whole formula must have {out_name}'s representation."
                ))
                .technical(format!("mk {out} expects {}", pretty::kernel(ir.representation_of(out).unwrap_or(&Ty::Nat)))),
            );
        }
        _ => {}
    }

    let input_unbound = inputs.iter().any(|c| ir.representation_of(*c).is_none());
    if el.diags.iter().any(Diagnostic::is_error) || out_rep.is_none() || input_unbound {
        let mut d = el.diags;
        bdl_diagnostics::sort_diagnostics(&mut d);
        return Err(d);
    }

    // λ x₁ : sem A₁. … λ xₙ : sem Aₙ. mk B body
    let mut expr = Expr::mk(out, body);
    el.spans.insert(vec![0; n], surface.span);
    for c in inputs.iter().rev() {
        expr = Expr::Lam {
            dom: Ty::sem(*c),
            body: Box::new(expr),
        };
    }
    let mut warnings = el.diags;
    bdl_diagnostics::sort_diagnostics(&mut warnings);
    Ok((
        Realized {
            expr,
            spans: el.spans,
        },
        warnings,
    ))
}

// ---- pattern compilation ----------------------------------------------------

/// How a pattern variable or test reaches its value from the scrutinee.
#[derive(Clone, Debug)]
enum Step {
    /// Observe a semantic value's representation.
    Rep,
    /// `getD v default` into an option's payload of this type.
    Unwrap(STy),
}

#[derive(Clone, Debug)]
struct Access {
    /// Depth of the scrutinee binding in the environment.
    scrutinee: usize,
    steps: Vec<Step>,
}

#[derive(Clone, Debug)]
enum Test {
    IsSome(Access, STy),
    IsNone(Access, STy),
    Bool(Access, bool),
    /// Dimensionless quantity equals the literal.
    Eq(Access, f64),
    /// Count equals the whole number.
    EqNat(Access, u64),
}

#[derive(Clone, Debug)]
struct Bind {
    name: String,
    span: Span,
    ty: STy,
    access: Access,
}

#[derive(Default)]
struct Compiled {
    tests: Vec<Test>,
    binds: Vec<Bind>,
    failed: bool,
}

/// One branch of an `if` or `match`, for the join.
struct Branch {
    span: Span,
    path: ExprPath,
    expr: Expr,
    ty: STy,
}

/// Re-elaborates branch `i` with an expected type, at its path.
type ReElab<'e, 'a> = dyn FnMut(&mut Elab<'a>, usize, &STy, &ExprPath) -> (Expr, STy) + 'e;

/// One compiled arm: its test (`None` for a catch-all), body, type, span
/// and the path its body was elaborated at.
struct ArmOut {
    cond: Option<Expr>,
    body: Expr,
    ty: STy,
    span: Span,
    path: ExprPath,
}

impl<'a> Elab<'a> {
    fn entity(&self) -> Entity {
        Entity::Mapping {
            id: self.mapping.id,
        }
    }

    fn concept_name(&self, id: SemanticId) -> String {
        self.design
            .concepts
            .get(&id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| id.to_string())
    }

    fn describe(&self, t: &STy) -> String {
        match t {
            STy::Q(d) => pretty::describe_dim(*d),
            STy::Bool => "true or false".into(),
            STy::Nat => "a count".into(),
            STy::Sem(s) => format!("a {}", self.concept_name(*s)),
            STy::Opt(inner) => match inner.as_ref() {
                STy::Unknown => "an optional value of a kind not yet known".into(),
                inner => format!("an optional value ({})", self.describe(inner)),
            },
            STy::List(inner) => match inner.as_ref() {
                STy::Unknown => "an empty collection".into(),
                inner => format!("a collection of {}", self.describe_plural(inner)),
            },
            STy::Pair(a, b) => format!(
                "a grouped value ({} and {})",
                self.describe(a),
                self.describe(b)
            ),
            STy::Unknown => "a value of a kind not yet known".into(),
            STy::Error => "an erroneous value".into(),
        }
    }

    /// "angles", "Brightness values", "counts": what a collection holds.
    fn describe_plural(&self, t: &STy) -> String {
        match t {
            STy::Q(d) => {
                let one = pretty::describe_dim(*d);
                let noun = one
                    .strip_prefix("an ")
                    .or_else(|| one.strip_prefix("a "))
                    .unwrap_or(&one);
                if noun.starts_with("quantity of") {
                    format!("quantities {}", &noun["quantity ".len()..])
                } else if noun.starts_with("dimensionless") {
                    "dimensionless quantities".into()
                } else {
                    format!("{noun}s")
                }
            }
            STy::Bool => "truth values".into(),
            STy::Nat => "counts".into(),
            STy::Sem(s) => format!("{} values", self.concept_name(*s)),
            other => format!("values ({})", self.describe(other)),
        }
    }

    fn unbound(&self, concept: SemanticId, span: Span, role: &str) -> Diagnostic {
        let name = self.concept_name(concept);
        Diagnostic::info(
            "semantic.unbound_representation",
            self.entity(),
            format!("{name} has no representation yet, so this formula cannot be checked."),
        )
        .at(span)
        .explain(format!(
            "The mapping {role} {name}. Choose what kind of value {name} is (a quantity, on/off, or a count) in its inspector; the formula stays and is checked then."
        ))
        .technical(format!("Θ {concept} = none"))
    }

    fn push(&mut self, d: Diagnostic) {
        self.diags.push(d);
    }

    fn error(&self, code: &'static str, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(code, self.entity(), message).at(span)
    }

    fn record(&mut self, path: &ExprPath, span: Span) {
        self.spans.insert(path.clone(), span);
    }

    fn lit(&self, dim: Dim, value: f64) -> Expr {
        Expr::prim(Prim::Lit {
            dim,
            value: Scalar(value),
        })
    }

    fn placeholder(&self) -> (Expr, STy) {
        (self.lit(Dim::ZERO, 0.0), STy::Error)
    }

    /// A closed term of a representation type, for `getD` defaults.
    fn default_of(&self, ty: &STy) -> Expr {
        match ty {
            STy::Q(d) => self.lit(*d, 0.0),
            STy::Bool => Expr::BoolLit { value: false },
            STy::Nat => Expr::NatLit { value: 0 },
            STy::Opt(inner) => Expr::prim(Prim::None {
                ty: inner.to_ty().unwrap_or(Ty::Nat),
            }),
            STy::List(inner) => Expr::prim(Prim::Nil {
                ty: inner.to_ty().unwrap_or(Ty::Nat),
            }),
            STy::Pair(a, b) => Expr::apps(
                Expr::prim(Prim::Pair {
                    fst: a.to_ty().unwrap_or(Ty::Nat),
                    snd: b.to_ty().unwrap_or(Ty::Nat),
                }),
                [self.default_of(a), self.default_of(b)],
            ),
            // Options carry representations only (`some` observes its
            // operand), so no default of a semantic type is ever needed.
            STy::Sem(_) | STy::Unknown | STy::Error => self.lit(Dim::ZERO, 0.0),
        }
    }

    /// De Bruijn index of the binding at `depth`.
    fn var(&self, depth: usize) -> Expr {
        Expr::Var {
            index: (self.env.len() - 1 - depth) as u32,
        }
    }

    /// Move the spans recorded under `path` one level down (under a new
    /// unary node at `path`), then record `span` for the new node.
    fn shift_under(&mut self, path: &ExprPath, span: Span) {
        let moved: Vec<(ExprPath, Span)> = self
            .spans
            .range(path.clone()..)
            .take_while(|(k, _)| k.starts_with(path))
            .map(|(k, s)| (k.clone(), *s))
            .collect();
        for (k, _) in &moved {
            self.spans.remove(k);
        }
        for (k, s) in moved {
            let mut nk = path.clone();
            nk.push(0);
            nk.extend_from_slice(&k[path.len()..]);
            self.spans.insert(nk, s);
        }
        self.record(path, span);
    }

    /// Every concept value in a type replaced by its representation: what
    /// an equation over collections sees of `Readings : List<Temperature>`.
    fn fully_observed(&self, t: &STy) -> Option<STy> {
        Some(match t {
            STy::Sem(s) => STy::of(self.ir.representation_of(*s)?)?,
            STy::Opt(i) => STy::Opt(Box::new(self.fully_observed(i)?)),
            STy::List(i) => STy::List(Box::new(self.fully_observed(i)?)),
            STy::Pair(a, b) => STy::Pair(
                Box::new(self.fully_observed(a)?),
                Box::new(self.fully_observed(b)?),
            ),
            other => other.clone(),
        })
    }

    /// Rebuild `e : from` as a value of `want`, observing concept values
    /// wherever `want` has their representation — at the top (`rep e`),
    /// inside a pair (through the projections), inside a collection or an
    /// optional value (through a fold).  `None` when the two types differ
    /// in any other way.
    fn coerce(&self, e: Expr, from: &STy, want: &STy) -> Option<Expr> {
        if from == want {
            return Some(e);
        }
        match (from, want) {
            (STy::Sem(s), want) => {
                let rep = STy::of(self.ir.representation_of(*s)?)?;
                if &rep == want {
                    Some(Expr::rep(e))
                } else {
                    let inner = self.coerce(Expr::rep(e), &rep, want)?;
                    Some(inner)
                }
            }
            (STy::Pair(a, b), STy::Pair(wa, wb)) => {
                let (ta, tb) = (a.to_ty()?, b.to_ty()?);
                let fst = Expr::app(
                    Expr::prim(Prim::Fst {
                        fst: ta.clone(),
                        snd: tb.clone(),
                    }),
                    e.clone(),
                );
                let snd = Expr::app(Expr::prim(Prim::Snd { fst: ta, snd: tb }), e);
                let fa = self.coerce(fst, a, wa)?;
                let fb = self.coerce(snd, b, wb)?;
                Some(Expr::apps(
                    Expr::prim(Prim::Pair {
                        fst: wa.to_ty()?,
                        snd: wb.to_ty()?,
                    }),
                    [fa, fb],
                ))
            }
            (STy::List(a), STy::List(wa)) => {
                // fold (λx. λacc. cons (coerce x) acc) nil e
                let (ta, twa) = (a.to_ty()?, wa.to_ty()?);
                let x = self.coerce(Expr::var(1), a, wa)?;
                Some(Expr::fold(
                    Expr::lam(
                        ta,
                        Expr::lam(
                            Ty::list(twa.clone()),
                            Expr::apps(
                                Expr::prim(Prim::Cons { ty: twa.clone() }),
                                [x, Expr::var(0)],
                            ),
                        ),
                    ),
                    Expr::prim(Prim::Nil { ty: twa }),
                    e,
                ))
            }
            (STy::Opt(a), STy::Opt(wa)) => {
                // fold (λx. λacc. some (coerce x)) none (toList e)
                let (ta, twa) = (a.to_ty()?, wa.to_ty()?);
                let x = self.coerce(Expr::var(1), a, wa)?;
                Some(Expr::fold(
                    Expr::lam(
                        ta.clone(),
                        Expr::lam(
                            Ty::opt(twa.clone()),
                            Expr::app(Expr::prim(Prim::Some { ty: twa.clone() }), x),
                        ),
                    ),
                    Expr::prim(Prim::None { ty: twa }),
                    Expr::app(Expr::prim(Prim::ToList { ty: ta }), e),
                ))
            }
            _ => None,
        }
    }

    /// Observe a semantic value where a representation is needed.
    fn observe(&mut self, e: Expr, ty: STy, path: &ExprPath, span: Span) -> (Expr, STy) {
        let STy::Sem(s) = ty else {
            return (e, ty);
        };
        self.shift_under(path, span);
        match self.ir.representation_of(s).and_then(STy::of) {
            Some(rep) => (Expr::rep(e), rep),
            None => {
                let d = self.unbound(s, span, "uses");
                self.push(d);
                (Expr::rep(e), STy::Error)
            }
        }
    }

    // ---- expressions ---------------------------------------------------------

    /// Elaborate at `path`; returns the Core term and its type.  `expect`
    /// is a type the context already knows (it fixes the payload of a bare
    /// `None`); it is never a requirement.
    fn expr(&mut self, e: &SurfaceExpr, path: &mut ExprPath, expect: Option<&STy>) -> (Expr, STy) {
        self.record(path, e.span);
        match &e.kind {
            ExprKind::Bool(b) => (Expr::BoolLit { value: *b }, STy::Bool),
            ExprKind::Number { literal, unit } => {
                // The syntax keeps the exact spelling; the machine number is
                // made here, at the elaboration boundary (DI-18).
                let Some(value) = literal.to_f64() else {
                    let d = self.error(
                        "formula.number.too_large",
                        e.span,
                        format!(
                            "`{}` is too large a number to compute with.",
                            literal.as_str()
                        ),
                    );
                    self.push(d);
                    return self.placeholder();
                };
                match unit {
                    None => (self.lit(Dim::ZERO, value), STy::Q(Dim::ZERO)),
                    Some(u) => match units::lookup(&u.name) {
                        Some(def) => (self.lit(def.dim, value * def.factor), STy::Q(def.dim)),
                        None => {
                            let d = self
                                .error(
                                    "formula.unit.unknown",
                                    u.span,
                                    format!("`{}` is not a unit.", u.name),
                                )
                                .fix(format!("Units available: {}.", units::names().join(", ")));
                            self.push(d);
                            self.placeholder()
                        }
                    },
                }
            }
            ExprKind::Name(name) => self.name(name, e.span, expect),
            ExprKind::Unary { op, expr } => self.unary(*op, expr, e.span, path),
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, e.span, path),
            ExprKind::If { cond, then, els } => self.if_(cond, then, els, e.span, path, expect),
            ExprKind::Call { callee, args } => self.call(callee, args, e.span, path, expect),
            ExprKind::Match { scrutinee, arms } => {
                self.match_(scrutinee, arms, e.span, path, expect)
            }
            ExprKind::Block { lets, tail } => self.block(lets, tail, e.span, path, expect),
            ExprKind::List(items) => self.list(items, e.span, path, expect),
            ExprKind::Tuple(items) => self.tuple(items, e.span, path, expect),
            ExprKind::Lambda { .. } => {
                let d = self
                    .error(
                        "formula.rule.not_a_value",
                        e.span,
                        "A rule (`x => …`) is not a value on its own.",
                    )
                    .explain("A rule is given to an equation that applies it, such as any, all, map, filter or foldr; it cannot be stored, returned or compared.");
                self.push(d);
                self.placeholder()
            }
        }
    }

    fn name(&mut self, name: &str, span: Span, expect: Option<&STy>) -> (Expr, STy) {
        match name {
            "None" => return self.none(expect),
            "Some" => {
                let d = self.error(
                    "formula.constructor.arity",
                    span,
                    "`Some` needs a value: write `Some(x)`.",
                );
                self.push(d);
                return self.placeholder();
            }
            "delay" | "sync" => {
                let d = self.error(
                    "formula.temporal.arity",
                    span,
                    format!(
                        "`{name}` is applied to its arguments, as in `{}`.",
                        temporal_shape(name == "sync")
                    ),
                );
                self.push(d);
                return self.placeholder();
            }
            _ => {}
        }
        // Lexical bindings first, innermost first.
        if let Some(depth) = self
            .env
            .iter()
            .rposition(|b| b.name.as_deref() == Some(name))
        {
            let ty = self.env[depth].ty.clone();
            return (self.var(depth), ty);
        }
        match self.inputs.resolve(self.design, name) {
            Lookup::Input(i) => {
                let ty = self.env[i].ty.clone();
                (self.var(i), ty)
            }
            Lookup::Mapping(id) => {
                let Some(m) = self.design.mappings.get(&id) else {
                    return self.placeholder();
                };
                if !m.signature.inputs.is_empty() {
                    let reads: Vec<String> = m
                        .signature
                        .inputs
                        .iter()
                        .map(|c| self.concept_name(*c))
                        .collect();
                    let d = self
                        .error(
                            "formula.mapping.needs_arguments",
                            span,
                            format!("{} reads {}; give it those values.", m.name, reads.join(", ")),
                        )
                        .explain("A relationship with inputs can only be applied; it cannot be passed along or stored as a value.")
                        .fix(format!("Write {}({}).", m.name, reads.join(", ")));
                    self.push(d);
                    return self.placeholder();
                }
                (Expr::decl(id), STy::Sem(m.signature.output))
            }
            Lookup::Ambiguous(candidates) => {
                let d = self
                    .error(
                        "formula.name.ambiguous",
                        span,
                        format!("`{name}` could mean any of {}.", candidates.join(", ")),
                    )
                    .fix("Write the name exactly as the concept is called, including its capitalisation.");
                self.push(d);
                self.placeholder()
            }
            Lookup::NotAnInput(_, concept_name) => {
                let mapping = self.mapping.name.clone();
                let d = self
                    .error(
                        "formula.name.not_an_input",
                        span,
                        format!("{mapping} does not read {concept_name}."),
                    )
                    .explain("A formula can only use the concepts its mapping reads.")
                    .fix(format!("Connect {concept_name} to {mapping} as an input."));
                self.push(d);
                self.placeholder()
            }
            Lookup::Unknown => {
                let d = self
                    .error(
                        "formula.name.unknown",
                        span,
                        format!("`{name}` is not something this mapping reads or can call."),
                    )
                    .fix(self.available_fix());
                self.push(d);
                self.placeholder()
            }
        }
    }

    fn available_fix(&self) -> String {
        let inputs = self.inputs.names();
        let mappings = names::mapping_names(self.design);
        let mut parts = Vec::new();
        if inputs.is_empty() {
            parts.push("This mapping reads nothing; connect a concept to it first".to_string());
        } else {
            parts.push(format!("Inputs: {}", inputs.join(", ")));
        }
        if !mappings.is_empty() {
            parts.push(format!("relationships: {}", mappings.join(", ")));
        }
        parts.push("constructors: Some(…), None, [a, b], (a, b)".into());
        parts.push(format!("equations: {}", equations::names().join(", ")));
        format!("{}.", parts.join("; "))
    }

    fn none(&self, expect: Option<&STy>) -> (Expr, STy) {
        match expect {
            Some(STy::Opt(inner)) if inner.is_known() => (
                Expr::prim(Prim::None {
                    ty: inner.to_ty().unwrap_or(Ty::Nat),
                }),
                STy::Opt(inner.clone()),
            ),
            // Payload unknown until the context says; a placeholder type
            // that a later re-elaboration with an expectation replaces.
            _ => (
                Expr::prim(Prim::None { ty: Ty::Nat }),
                STy::Opt(Box::new(STy::Unknown)),
            ),
        }
    }

    fn undetermined(&mut self, span: Span) -> (Expr, STy) {
        let d = self
            .error(
                "formula.option.undetermined",
                span,
                "The kind of value this `None` stands for cannot be told here.",
            )
            .explain("`None` takes its kind from the other case it is paired with, as in `if c then Some(x) else None`, or from a `Some(…)` arm of the same `match`.");
        self.push(d);
        self.placeholder()
    }

    // ---- calls -----------------------------------------------------------------

    fn call(
        &mut self,
        callee: &SurfaceExpr,
        args: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        let ExprKind::Name(name) = &callee.kind else {
            let d = self
                .error(
                    "formula.call.not_a_relationship",
                    callee.span,
                    "Only a named relationship can be applied.",
                )
                .explain("A call applies a relationship of the design to its inputs, as in `dimByTilt(tilt)`; the result of a call or an expression cannot be applied again.");
            self.push(d);
            return self.placeholder();
        };
        match name.as_str() {
            "Some" => return self.some(args, span, path, expect),
            "None" => {
                let d = self
                    .error(
                        "formula.constructor.arity",
                        span,
                        "`None` takes no arguments.",
                    )
                    .fix("Write `None` alone for an absent value.");
                self.push(d);
                return self.placeholder();
            }
            "delay" => return self.temporal(false, args, span, path, expect),
            "sync" => return self.temporal(true, args, span, path, expect),
            _ => {}
        }
        if self
            .env
            .iter()
            .any(|b| b.name.as_deref() == Some(name.as_str()))
        {
            let d = self.error(
                "formula.call.not_a_relationship",
                callee.span,
                format!(
                    "`{name}` is a value bound by `let`, not a relationship; it cannot be applied."
                ),
            );
            self.push(d);
            return self.placeholder();
        }
        let resolved = self.inputs.resolve(self.design, name);
        if !matches!(resolved, Lookup::Mapping(_)) {
            if let Some(entry) = equations::lookup(name) {
                return self.equation(entry, args, span, path, expect);
            }
        }
        let id = match resolved {
            Lookup::Mapping(id) => id,
            Lookup::Input(i) => {
                let cname = self.concept_name(self.mapping.signature.inputs[i]);
                let d = self.error(
                    "formula.call.not_a_relationship",
                    callee.span,
                    format!("{cname} is a concept, not a relationship; it cannot be applied."),
                );
                self.push(d);
                return self.placeholder();
            }
            Lookup::NotAnInput(_, concept_name) => {
                let d = self.error(
                    "formula.call.not_a_relationship",
                    callee.span,
                    format!(
                        "{concept_name} is a concept, not a relationship; it cannot be applied."
                    ),
                );
                self.push(d);
                return self.placeholder();
            }
            Lookup::Ambiguous(candidates) => {
                let d = self.error(
                    "formula.name.ambiguous",
                    callee.span,
                    format!("`{name}` could mean any of {}.", candidates.join(", ")),
                );
                self.push(d);
                return self.placeholder();
            }
            Lookup::Unknown => {
                let d = self
                    .error(
                        "formula.name.unknown",
                        callee.span,
                        format!("`{name}` is not a relationship this formula can call."),
                    )
                    .fix(self.available_fix());
                self.push(d);
                return self.placeholder();
            }
        };
        let Some(m) = self.design.mappings.get(&id) else {
            return self.placeholder();
        };
        let params: Vec<SemanticId> = m.signature.inputs.clone();
        let output = m.signature.output;
        let mname = m.name.clone();
        if params.is_empty() && !args.is_empty() {
            let d = self
                .error(
                    "formula.call.arity",
                    span,
                    format!("{mname} reads nothing; it is a value, not something to apply."),
                )
                .fix(format!("Write {mname} without parentheses."));
            self.push(d);
            return self.placeholder();
        }
        if args.len() != params.len() {
            let reads: Vec<String> = params.iter().map(|c| self.concept_name(*c)).collect();
            let d = self
                .error(
                    "formula.call.arity",
                    span,
                    format!(
                        "{mname} reads {} value{} ({}), but {} {} given here.",
                        params.len(),
                        if params.len() == 1 { "" } else { "s" },
                        if reads.is_empty() {
                            "nothing".to_string()
                        } else {
                            reads.join(", ")
                        },
                        args.len(),
                        if args.len() == 1 { "is" } else { "are" }
                    ),
                )
                .fix(format!("Write {mname}({}).", reads.join(", ")));
            self.push(d);
            return self.placeholder();
        }
        // app (… (app (declRef f) a₁) …) aₙ: aᵢ at [0]*(n-1-i) ++ [1].
        let n = args.len();
        let mut failed = false;
        let mut lowered = Vec::with_capacity(n);
        for (i, (arg, want)) in args.iter().zip(&params).enumerate() {
            let mut ap = path.clone();
            ap.extend(std::iter::repeat_n(0u8, n - 1 - i));
            ap.push(1);
            let expect = STy::Sem(*want);
            let (ae, at) = self.expr(arg, &mut ap, Some(&expect));
            match &at {
                STy::Sem(s) if s == want => {}
                STy::Error => failed = true,
                STy::Sem(got) => {
                    failed = true;
                    let (want, got) = (self.concept_name(*want), self.concept_name(*got));
                    let d = self
                        .error(
                            "formula.call.argument_type",
                            arg.span,
                            format!("{mname} reads {want} here, but this is {got}."),
                        )
                        .explain("Concepts are identities: a value of one concept is never a value of another, whatever its representation.")
                        .fix(format!("Pass a {want}: an input of this mapping, or a relationship that produces {want}."));
                    self.push(d);
                }
                other => {
                    failed = true;
                    let wname = self.concept_name(*want);
                    let found = self.describe(other);
                    let d = self
                        .error(
                            "formula.call.argument",
                            arg.span,
                            format!("{mname} reads {wname} here, but this is {found}, not a concept's value."),
                        )
                        .explain("A relationship reads concept values; a computed number has no concept until the relationship that produces it makes one.")
                        .fix(format!("Pass an input or another relationship's value, or move the arithmetic into a relationship that produces {wname}."));
                    self.push(d);
                }
            }
            lowered.push(ae);
        }
        if failed {
            return self.placeholder();
        }
        let mut head_path = path.clone();
        head_path.extend(std::iter::repeat_n(0u8, n));
        self.record(&head_path, callee.span);
        (Expr::apps(Expr::decl(id), lowered), STy::Sem(output))
    }

    fn some(
        &mut self,
        args: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        if args.len() != 1 {
            let d = self.error(
                "formula.constructor.arity",
                span,
                format!("`Some` takes exactly one value, not {}.", args.len()),
            );
            self.push(d);
            return self.placeholder();
        }
        let inner_expect = match expect {
            Some(STy::Opt(inner)) => Some(inner.as_ref().clone()),
            _ => None,
        };
        path.push(1);
        let (ae, at) = self.expr(&args[0], path, inner_expect.as_ref());
        let (ae, at) = self.observe(ae, at, path, args[0].span);
        path.pop();
        if at.is_error() {
            return self.placeholder();
        }
        let ty = at.to_ty().unwrap_or(Ty::Nat);
        (
            Expr::app(Expr::prim(Prim::Some { ty }), ae),
            STy::Opt(Box::new(at)),
        )
    }

    /// `delay(init, value)` and `sync(domain, init, value)`: the kernel's
    /// own memory forms.  Legal only outside every binder — a relationship
    /// without inputs, and not inside a block's result or a match arm —
    /// because the kernel types `delay` in the empty context.
    fn temporal(
        &mut self,
        sync: bool,
        args: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        let word = if sync { "sync" } else { "delay" };
        let wanted = if sync { 3 } else { 2 };
        if args.len() != wanted {
            let d = self.error(
                "formula.temporal.arity",
                span,
                if sync {
                    "`sync` takes a timing domain, an initial value and the value to observe: sync(domain, init, value)."
                } else {
                    "`delay` takes an initial value and the value to remember: delay(init, value)."
                },
            );
            self.push(d);
            return self.placeholder();
        }
        if !self.mapping.signature.inputs.is_empty() {
            let d = self
                .error(
                    "formula.temporal.under_inputs",
                    span,
                    format!("`{word}` can only be used in a relationship without inputs."),
                )
                .explain("Memory belongs to a relationship as a whole — a value per activation — not to a formula over its inputs. Read the inputs through relationships without inputs and remember there.");
            self.push(d);
            return self.placeholder();
        }
        if !self.env.is_empty() {
            let d = self
                .error(
                    "formula.temporal.under_binder",
                    span,
                    format!("`{word}` cannot be used inside a block's result or a match arm."),
                )
                .explain("Memory is a property of the relationship, evaluated once per activation; inside a `let` body or an arm it would depend on names that exist only at this tick.")
                .fix(format!("Bind it first: `let previous = {word}(…);`, or make it the value that `match` looks at."));
            self.push(d);
            return self.placeholder();
        }
        let src: Option<ClockId> = if sync {
            let domain = &args[0];
            let ExprKind::Name(dname) = &domain.kind else {
                let d = self.error(
                    "formula.sync.unknown_domain",
                    domain.span,
                    "The first argument of `sync` must name a timing domain.",
                );
                self.push(d);
                return self.placeholder();
            };
            match self.design.clocks.values().find(|c| &c.name == dname) {
                Some(c) => Some(c.id),
                None => {
                    let known: Vec<&str> = self
                        .design
                        .clocks
                        .values()
                        .map(|c| c.name.as_str())
                        .collect();
                    let d = self
                        .error(
                            "formula.sync.unknown_domain",
                            domain.span,
                            format!("`{dname}` is not a timing domain of this design."),
                        )
                        .fix(if known.is_empty() {
                            "Create a timing domain first.".to_string()
                        } else {
                            format!("Timing domains: {}.", known.join(", "))
                        });
                    self.push(d);
                    return self.placeholder();
                }
            }
        } else {
            None
        };
        let (init, value) = if sync {
            (&args[1], &args[2])
        } else {
            (&args[0], &args[1])
        };
        // Delay { init, e } / Sync { src, init, e }: init at [0], e at [1]
        let mut ip = path.clone();
        ip.push(0);
        let (ie, it) = self.expr(init, &mut ip, expect);
        let mut vp = path.clone();
        vp.push(1);
        let hint = if it.is_known() { Some(&it) } else { expect };
        let (ve, vt) = self.expr(value, &mut vp, hint);
        let mut branches = vec![
            Branch {
                span: init.span,
                path: ip,
                expr: ie,
                ty: it,
            },
            Branch {
                span: value.span,
                path: vp,
                expr: ve,
                ty: vt,
            },
        ];
        let surfaces = [init, value];
        let Some(ty) = self.join(&mut branches, word, span, &mut |el, i, want, p| {
            let mut p = p.clone();
            el.expr(surfaces[i], &mut p, Some(want))
        }) else {
            return self.placeholder();
        };
        let mut it = branches.into_iter();
        let (ie, ve) = match (it.next(), it.next()) {
            (Some(a), Some(b)) => (a.expr, b.expr),
            _ => return self.placeholder(),
        };
        let term = match src {
            None => Expr::delay(ie, ve),
            Some(src) => Expr::sync(src, ie, ve),
        };
        (term, ty)
    }

    // ---- collections, grouped values, equations ----------------------------------

    /// `[a, b, c]`: `cons a (cons b (cons c nil))` — element `i` at
    /// `[1]*i ++ [0, 1]`.  Elements are brought to one type like the
    /// branches of an `if`: concept values of one concept stay concept
    /// values, otherwise every element is observed.  `[]` takes its element
    /// type from the context, like a bare `None`.
    fn list(
        &mut self,
        items: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        let inner_expect = match expect {
            Some(STy::List(inner)) => Some(inner.as_ref().clone()),
            _ => None,
        };
        if items.is_empty() {
            return match inner_expect {
                Some(t) if t.is_known() => (
                    Expr::prim(Prim::Nil {
                        ty: t.to_ty().unwrap_or(Ty::Nat),
                    }),
                    STy::List(Box::new(t)),
                ),
                _ => (
                    Expr::prim(Prim::Nil { ty: Ty::Nat }),
                    STy::List(Box::new(STy::Unknown)),
                ),
            };
        }
        let mut branches = Vec::with_capacity(items.len());
        for (i, item) in items.iter().enumerate() {
            let mut ip = path.clone();
            ip.extend(std::iter::repeat_n(1u8, i));
            ip.extend([0, 1]);
            let (e, t) = self.expr(item, &mut ip, inner_expect.as_ref());
            branches.push(Branch {
                span: item.span,
                path: ip,
                expr: e,
                ty: t,
            });
        }
        let Some(elem) = self.join(&mut branches, "collection", span, &mut |el, i, want, p| {
            let mut p = p.clone();
            el.expr(&items[i], &mut p, Some(want))
        }) else {
            return self.placeholder();
        };
        let Some(elem_ty) = elem.to_ty() else {
            return self.undetermined(span);
        };
        let mut list = Expr::prim(Prim::Nil {
            ty: elem_ty.clone(),
        });
        for b in branches.into_iter().rev() {
            list = Expr::apps(
                Expr::prim(Prim::Cons {
                    ty: elem_ty.clone(),
                }),
                [b.expr, list],
            );
        }
        (list, STy::List(Box::new(elem)))
    }

    /// `(a, b)`: `pair a b` — `a` at `[0, 1]`, `b` at `[1]`; three or more
    /// parts nest to the right, `(a, (b, c))`.  Parts keep their own kinds:
    /// a pair of two concept values keeps both identities.
    fn tuple(
        &mut self,
        items: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        let Some((first, rest)) = items.split_first() else {
            return self.placeholder();
        };
        if rest.is_empty() {
            return self.expr(first, path, expect);
        }
        let (fe, se) = match expect {
            Some(STy::Pair(a, b)) => (Some(a.as_ref().clone()), Some(b.as_ref().clone())),
            _ => (None, None),
        };
        let mut fp = path.clone();
        fp.extend([0, 1]);
        let (a, ta) = self.expr(first, &mut fp, fe.as_ref());
        let mut sp = path.clone();
        sp.push(1);
        let (b, tb) = if rest.len() == 1 {
            self.expr(&rest[0], &mut sp, se.as_ref())
        } else {
            self.tuple(rest, span, &mut sp, se.as_ref())
        };
        if ta.is_error() || tb.is_error() {
            return self.placeholder();
        }
        let (Some(fst), Some(snd)) = (ta.to_ty(), tb.to_ty()) else {
            return self.undetermined(span);
        };
        (
            Expr::apps(Expr::prim(Prim::Pair { fst, snd }), [a, b]),
            STy::Pair(Box::new(ta), Box::new(tb)),
        )
    }

    /// A call of an equation of the library: match the scheme's parameter
    /// patterns against the arguments' closed types in order (a rule
    /// argument is elaborated once its parameter kinds are known), check
    /// the capabilities, then apply the closed combinator built at that
    /// instance: `app (… (app comb a₁) …) aₙ`, `aᵢ` at `[0]*(n-1-i) ++ [1]`.
    fn equation(
        &mut self,
        entry: &'static Entry,
        args: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        let n = entry.scheme.params.len();
        if args.len() != n {
            let d = self
                .error(
                    "formula.equation.arity",
                    span,
                    format!(
                        "{} takes {} value{} ({}), but {} {} given here.",
                        entry.name,
                        n,
                        if n == 1 { "" } else { "s" },
                        entry.params.join(", "),
                        args.len(),
                        if args.len() == 1 { "is" } else { "are" }
                    ),
                )
                .fix(format!("Write {}.", entry.shape()));
            self.push(d);
            return self.placeholder();
        }
        // The result of the whole call may fix a variable the arguments
        // leave open (`[]` given to `append`); seed it from the context.
        let mut seed = equations::Subst::default();
        if let Some(want) = expect.and_then(STy::to_ty) {
            let mut seeded = seed.clone();
            if equations::match_ty(&entry.scheme.result, &want, &mut seeded).is_ok() {
                seed = seeded;
            }
        }
        // Pass 1: the value arguments, in order, each with the expectation
        // the arguments before it fixed (what a bare `None` or `[]` needs).
        struct ArgOut {
            expr: Expr,
            ty: STy,
            path: ExprPath,
        }
        let mut outs: Vec<Option<ArgOut>> = Vec::with_capacity(n);
        let mut failed = false;
        let mut provisional = seed.clone();
        for (i, (arg, pat)) in args.iter().zip(&entry.scheme.params).enumerate() {
            let mut ap = path.clone();
            ap.extend(std::iter::repeat_n(0u8, n - 1 - i));
            ap.push(1);
            match (&arg.kind, pat) {
                (ExprKind::Lambda { .. }, PTy::Arr(..)) => outs.push(None),
                (ExprKind::Lambda { .. }, _) => {
                    let d = self
                        .error(
                            "formula.rule.unexpected",
                            arg.span,
                            format!(
                                "{} reads {} here, not a rule.",
                                entry.name,
                                self.describe_pattern(pat, &Instance::default())
                            ),
                        )
                        .fix(format!("Write {}.", entry.shape()));
                    self.push(d);
                    failed = true;
                    outs.push(None);
                }
                (_, PTy::Arr(..)) => {
                    let (doms, _) = pat.uncurry();
                    let shown = match doms.len() {
                        1 => "x => …".to_string(),
                        2 => "(x, y) => …".to_string(),
                        k => format!(
                            "({}) => …",
                            (0..k)
                                .map(|j| format!("x{}", j + 1))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    };
                    let d = self
                        .error(
                            "formula.rule.expected",
                            arg.span,
                            format!(
                                "{} needs a rule for `{}` here, written `{shown}`.",
                                entry.name,
                                entry.params.get(i).copied().unwrap_or("rule")
                            ),
                        )
                        .explain("A rule names its inputs and gives their result, as in `x => x < 30 deg`.");
                    self.push(d);
                    failed = true;
                    outs.push(None);
                }
                _ => {
                    let want = equations::instantiate(pat, &provisional).and_then(|t| STy::of(&t));
                    let (e, t) = self.expr(arg, &mut ap, want.as_ref());
                    if t.is_error() {
                        failed = true;
                    } else if !t.is_known() {
                        self.undetermined(arg.span);
                        failed = true;
                    } else if let Some(ty) = t.to_ty() {
                        let mut tentative = provisional.clone();
                        if equations::match_ty(pat, &ty, &mut tentative).is_ok() {
                            provisional = tentative;
                        }
                    }
                    outs.push(Some(ArgOut {
                        expr: e,
                        ty: t,
                        path: ap,
                    }));
                }
            }
        }
        if failed {
            return self.placeholder();
        }
        // The instance: match every value argument's closed type.  A concept
        // value met by a plain representation of its own kind (`mode in
        // [1, 2]`, `min(brightness, 0.5)`) is observed, as it is by the
        // operators (ADR-0013); two concept values stay concept values.
        let mut instance = Instance::default();
        loop {
            instance.subst = seed.clone();
            let mut observe: Option<usize> = None;
            for (i, (out, pat)) in outs.iter().zip(&entry.scheme.params).enumerate() {
                let Some(out) = out else { continue };
                let Some(ty) = out.ty.to_ty() else {
                    return self.placeholder();
                };
                match equations::match_ty(pat, &ty, &mut instance.subst) {
                    Ok(()) => {}
                    Err(MatchError::TyConflict { bound, found, .. })
                        if observable(self.ir, &bound, &found)
                            || observable(self.ir, &found, &bound) =>
                    {
                        // Which argument holds the concept value to observe:
                        // this one, or the earlier one that bound the variable.
                        let sem_here = matches!(found, Ty::Sem { .. });
                        let candidate = if sem_here {
                            Some(i)
                        } else {
                            outs.iter().position(|o| {
                                o.as_ref()
                                    .is_some_and(|o| o.ty == STy::of(&bound).unwrap_or(STy::Error))
                            })
                        };
                        match candidate {
                            Some(c) => {
                                observe = Some(c);
                                break;
                            }
                            None => {
                                let err = MatchError::TyConflict {
                                    var: 0,
                                    bound,
                                    found,
                                };
                                self.match_error(entry, i, pat, &err, &instance, args[i].span);
                                return self.placeholder();
                            }
                        }
                    }
                    Err(MatchError::Shape { .. })
                        if mentions_concept(&out.ty)
                            && !matches!(pat, PTy::Var(_) | PTy::Sem(_))
                            && self.fully_observed(&out.ty).is_some_and(|t| t != out.ty) =>
                    {
                        // `Readings : List<Temperature>` where a collection
                        // is expected: the concept value is observed deeply.
                        observe = Some(i);
                        break;
                    }
                    Err(err) => {
                        self.match_error(entry, i, pat, &err, &instance, args[i].span);
                        return self.placeholder();
                    }
                }
            }
            let Some(i) = observe else { break };
            let Some(out) = outs[i].as_mut() else {
                return self.placeholder();
            };
            let (e, t) = if matches!(out.ty, STy::Sem(_)) {
                self.observe(
                    std::mem::replace(&mut out.expr, Expr::NatLit { value: 0 }),
                    out.ty.clone(),
                    &out.path,
                    args[i].span,
                )
            } else {
                let Some(want) = self.fully_observed(&out.ty) else {
                    return self.placeholder();
                };
                match self.coerce(
                    std::mem::replace(&mut out.expr, Expr::NatLit { value: 0 }),
                    &out.ty,
                    &want,
                ) {
                    Some(e) => (e, want),
                    None => return self.placeholder(),
                }
            };
            if t.is_error() {
                return self.placeholder();
            }
            out.expr = e;
            out.ty = t;
        }
        // Pass 2: the rules, now that their parameter kinds are fixed.
        let mut lowered: Vec<Expr> = Vec::with_capacity(n);
        for (i, (arg, pat)) in args.iter().zip(&entry.scheme.params).enumerate() {
            match outs[i].take() {
                Some(out) => lowered.push(out.expr),
                None => {
                    let ExprKind::Lambda { params, body } = &arg.kind else {
                        return self.placeholder();
                    };
                    let mut ap = path.clone();
                    ap.extend(std::iter::repeat_n(0u8, n - 1 - i));
                    ap.push(1);
                    let (e, t) =
                        self.rule(entry, params, body, pat, &mut ap, &mut instance, arg.span);
                    if t.is_error() {
                        failed = true;
                    }
                    lowered.push(e);
                }
            }
        }
        if failed {
            return self.placeholder();
        }
        // Capabilities: the weakest each variable needs.
        for (var, cap) in &entry.scheme.caps {
            let Some(ty) = instance.subst.tys.get(var).cloned() else {
                continue;
            };
            match cap {
                Cap::Data | Cap::Eq => {
                    if !ty.is_data() {
                        let d = self.error(
                            "type.equality_not_data",
                            span,
                            "Rules cannot be compared for equality.",
                        );
                        self.push(d);
                        return self.placeholder();
                    }
                }
                Cap::Ord => match self.ordered_evidence(&ty) {
                    Some(o) => {
                        instance.ordered.insert(*var, o);
                    }
                    None => {
                        self.no_order(entry.name, &ty, span);
                        return self.placeholder();
                    }
                },
            }
        }
        let Some(result) =
            equations::instantiate(&entry.scheme.result, &instance.subst).and_then(|t| STy::of(&t))
        else {
            return self.undetermined(span);
        };
        let comb = (entry.build)(&instance);
        let mut head_path = path.clone();
        head_path.extend(std::iter::repeat_n(0u8, n));
        self.record(&head_path, span);
        (Expr::apps(comb, lowered), result)
    }

    /// A rule `x => e` given to an equation whose parameter is an arrow
    /// pattern: its parameters take the pattern's domains (which the
    /// earlier arguments must have fixed), its body is elaborated under
    /// them and matched against the codomain.  Yields `λx:τ. e`.
    #[allow(clippy::too_many_arguments)]
    fn rule(
        &mut self,
        entry: &'static Entry,
        params: &[Ident],
        body: &SurfaceExpr,
        pat: &PTy,
        path: &mut ExprPath,
        instance: &mut Instance,
        span: Span,
    ) -> (Expr, STy) {
        let (doms, cod) = pat.uncurry();
        if params.len() != doms.len() {
            let d = self
                .error(
                    "formula.rule.arity",
                    span,
                    format!(
                        "This rule names {} value{}, but {} gives it {}.",
                        params.len(),
                        if params.len() == 1 { "" } else { "s" },
                        entry.name,
                        doms.len()
                    ),
                )
                .fix(format!("Write {}.", entry.shape()));
            self.push(d);
            return (self.lit(Dim::ZERO, 0.0), STy::Error);
        }
        let mut dom_tys = Vec::with_capacity(doms.len());
        for (p, dp) in params.iter().zip(&doms) {
            let Some(ty) = equations::instantiate(dp, &instance.subst).and_then(|t| STy::of(&t))
            else {
                let d = self
                    .error(
                        "formula.rule.undetermined",
                        p.span,
                        format!("The kind of `{}` cannot be told here.", p.name),
                    )
                    .explain(format!(
                        "{} decides what its rule reads from the values given before the rule; give it the collection first.",
                        entry.name
                    ));
                self.push(d);
                return (self.lit(Dim::ZERO, 0.0), STy::Error);
            };
            dom_tys.push(ty);
        }
        for (p, ty) in params.iter().zip(&dom_tys) {
            if is_reserved_name(&p.name) {
                let d = self.error(
                    "formula.pattern.constructor_binding",
                    p.span,
                    format!(
                        "`{}` cannot name a rule's input; it is a constructor or a temporal form.",
                        p.name
                    ),
                );
                self.push(d);
                return (self.lit(Dim::ZERO, 0.0), STy::Error);
            }
            self.env.push(Binding {
                name: Some(p.name.clone()),
                ty: ty.clone(),
            });
        }
        let mut bp = path.clone();
        bp.extend(std::iter::repeat_n(0u8, params.len()));
        let expect = equations::instantiate(cod, &instance.subst).and_then(|t| STy::of(&t));
        let (be, bt) = self.expr(body, &mut bp, expect.as_ref());
        self.env.truncate(self.env.len() - params.len());
        if bt.is_error() {
            return (be, STy::Error);
        }
        if !bt.is_known() {
            let (e, _) = self.undetermined(body.span);
            return (e, STy::Error);
        }
        let Some(bty) = bt.to_ty() else {
            return (be, STy::Error);
        };
        if let Err(err) = equations::match_ty(cod, &bty, &mut instance.subst) {
            self.match_error(entry, usize::MAX, cod, &err, instance, body.span);
            return (be, STy::Error);
        }
        let mut lam = be;
        for ty in dom_tys.iter().rev() {
            lam = Expr::lam(ty.to_ty().unwrap_or(Ty::Nat), lam);
        }
        (lam, bt)
    }

    /// "a collection of angles", "a value of the same kind as `a`", …
    fn describe_pattern(&self, p: &PTy, instance: &Instance) -> String {
        match p {
            PTy::Var(v) => match instance.subst.tys.get(v).and_then(STy::of) {
                Some(t) => self.describe(&t),
                None => "a value".into(),
            },
            PTy::Bool => "true or false".into(),
            PTy::Nat => "a count".into(),
            PTy::Sem(s) => format!("a {}", self.concept_name(*s)),
            PTy::Q(PDim::Const(d)) => pretty::describe_dim(*d),
            PTy::Q(PDim::Var(v)) => match instance.subst.dims.get(v) {
                Some(d) => pretty::describe_dim(*d),
                None => "a quantity".into(),
            },
            PTy::Opt(inner) => format!(
                "an optional value ({})",
                self.describe_pattern(inner, instance)
            ),
            PTy::List(inner) => match inner.as_ref() {
                PTy::Var(v) if !instance.subst.tys.contains_key(v) => "a collection".into(),
                inner => format!(
                    "a collection of {}",
                    self.describe_pattern_plural(inner, instance)
                ),
            },
            PTy::Prod(a, b) => format!(
                "a grouped value ({} and {})",
                self.describe_pattern(a, instance),
                self.describe_pattern(b, instance)
            ),
            PTy::Arr(..) => "a rule".into(),
        }
    }

    fn describe_pattern_plural(&self, p: &PTy, instance: &Instance) -> String {
        match p {
            PTy::Var(v) => match instance.subst.tys.get(v).and_then(STy::of) {
                Some(t) => self.describe_plural(&t),
                None => "values".into(),
            },
            other => {
                let one = self.describe_pattern(other, instance);
                format!("values ({one})")
            }
        }
    }

    /// A parameter of an equation did not fit: the shape, the concept or
    /// the dimension differs from what the earlier arguments fixed.
    fn match_error(
        &mut self,
        entry: &'static Entry,
        arg: usize,
        pat: &PTy,
        err: &MatchError,
        instance: &Instance,
        span: Span,
    ) {
        let where_ = if arg == usize::MAX {
            "the rule's result".to_string()
        } else {
            format!("`{}`", entry.params.get(arg).copied().unwrap_or("this"))
        };
        let d = match err {
            MatchError::TyConflict { bound, found, .. } => {
                match (bound, found) {
                    (Ty::Sem { id: a }, Ty::Sem { id: b }) => self
                        .error(
                            "semantic.concept_mismatch",
                            span,
                            format!(
                                "{} and {} are different concepts.",
                                self.concept_name(*a),
                                self.concept_name(*b)
                            ),
                        )
                        .explain(format!(
                            "{} works on values of one kind; a concept's identity is never exchanged for another's, whatever the representation.",
                            entry.name
                        )),
                    (Ty::Q { dim: a }, Ty::Q { dim: b }) => Diagnostic::error(
                        "dimension.mismatch",
                        self.entity(),
                        format!(
                            "{} mixes values with different physical dimensions: {} and {}.",
                            entry.name,
                            pretty::describe_dim(*a),
                            pretty::describe_dim(*b)
                        ),
                    )
                    .at(span)
                    .explain("An equation over quantities keeps one dimension throughout."),
                    (bound, found) => {
                        let (b, f) = (
                            STy::of(bound).map(|t| self.describe(&t)).unwrap_or_else(|| pretty::kernel(bound)),
                            STy::of(found).map(|t| self.describe(&t)).unwrap_or_else(|| pretty::kernel(found)),
                        );
                        self.error(
                            "formula.equation.argument",
                            span,
                            format!("{} expects {b} for {where_}, but this is {f}.", entry.name),
                        )
                    }
                }
            }
            MatchError::DimConflict { bound, found, .. } => Diagnostic::error(
                "dimension.mismatch",
                self.entity(),
                format!(
                    "{} mixes values with different physical dimensions: {} and {}.",
                    entry.name,
                    pretty::describe_dim(*bound),
                    pretty::describe_dim(*found)
                ),
            )
            .at(span),
            MatchError::Shape { found, .. } => {
                let f = STy::of(found)
                    .map(|t| self.describe(&t))
                    .unwrap_or_else(|| pretty::kernel(found));
                self.error(
                    "formula.equation.argument",
                    span,
                    format!(
                        "{} expects {} for {where_}, but this is {f}.",
                        entry.name,
                        self.describe_pattern(pat, instance)
                    ),
                )
                .fix(format!("Write {}.", entry.shape()))
            }
        };
        self.push(d);
    }

    /// `Ty.ordB`: the evidence that a type has a designer-meaningful order.
    fn ordered_evidence(&self, ty: &Ty) -> Option<Ordered> {
        match ty {
            Ty::Q { dim } => Some(Ordered::Q(*dim)),
            Ty::Sem { id } if self.ir.is_ordered(ty) => match self.ir.representation_of(*id) {
                Some(Ty::Q { dim }) => Some(Ordered::Sem(*id, *dim)),
                _ => None,
            },
            _ => None,
        }
    }

    /// The `Ord` capability failed: say what the value is and why it has
    /// no order, in the designer's words (Phase 9c).
    fn no_order(&mut self, what: &str, ty: &Ty, span: Span) {
        let d = match ty {
            Ty::Sem { id } => {
                let name = self.concept_name(*id);
                let numeric = matches!(self.ir.representation_of(*id), Some(Ty::Q { .. }));
                let d = self.error(
                    "semantic.no_order",
                    span,
                    format!("{name} values can be compared for equality, but they have no default order."),
                );
                if numeric {
                    d.explain(format!(
                        "A concept's values are ordered only when the designer says so; a numeric encoding does not make {name} a magnitude."
                    ))
                    .fix(format!(
                        "Declare it ordered (`ordered concept {name} : …`) if its values are magnitudes, or choose with a rule: minBy(a, b, (x, y) => …)."
                    ))
                } else {
                    d.explain(format!(
                        "{name} is not represented by a quantity, so nothing orders its values."
                    ))
                    .fix("Choose with a rule: minBy(a, b, (x, y) => …).")
                }
            }
            Ty::Prod { .. } => self
                .error(
                    "semantic.no_order",
                    span,
                    "A grouped value has no order — compare its parts.",
                )
                .fix(format!("Apply {what} to one part: first(…) or second(…).")),
            Ty::List { .. } => self
                .error(
                    "semantic.no_order",
                    span,
                    "A collection has no order — compare its elements or its length.",
                )
                .fix("Use length(…), or any/all over the elements."),
            Ty::Opt { .. } => self
                .error(
                    "semantic.no_order",
                    span,
                    "An optional value has no order — take it apart with `match` first.",
                )
                .fix("Write `match o { Some(x) => …, None => … }`."),
            Ty::Bool => self.error("semantic.no_order", span, "true and false have no order."),
            Ty::Nat => self
                .error(
                    "semantic.no_order",
                    span,
                    "Counts are not ordered magnitudes here.",
                )
                .fix("Compare quantities instead."),
            Ty::Arr { .. } => self.error("semantic.no_order", span, "Rules have no order."),
            Ty::Q { .. } => self.error("semantic.no_order", span, "This value has no order."),
        };
        self.push(d);
    }

    // ---- blocks ----------------------------------------------------------------

    fn block(
        &mut self,
        lets: &[SurfaceLet],
        tail: &SurfaceExpr,
        _span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        // app (λx:τ. body) v — v at [1], body at [0,0]; nested per `let`.
        let mut p = path.clone();
        let mut bound: Vec<(Ty, Expr)> = Vec::new();
        let mut failed = false;
        for l in lets {
            p.push(1);
            let (ve, vt) = self.expr(&l.value, &mut p, None);
            p.pop();
            let vt = if vt.is_error() {
                failed = true;
                vt
            } else if !vt.is_known() {
                failed = true;
                self.undetermined(l.value.span);
                STy::Error
            } else {
                vt
            };
            let name = match &l.pattern.kind {
                PatternKind::Ident(n) if is_reserved_name(n.as_str()) => {
                    failed = true;
                    let d = self.error(
                        "formula.pattern.constructor_binding",
                        l.pattern.span,
                        format!("`{n}` cannot be bound by `let`; it is a constructor or a temporal form."),
                    );
                    self.push(d);
                    None
                }
                PatternKind::Ident(n) => Some(n.clone()),
                PatternKind::Wildcard => None,
                other => {
                    failed = true;
                    let shown = describe_pattern(other);
                    let d = self
                        .error(
                            "formula.let.refutable",
                            l.pattern.span,
                            format!("A `let` binds a plain name; `{shown}` could fail to match."),
                        )
                        .fix("Bind the value to a name and take it apart with `match`.");
                    self.push(d);
                    // Bind the first name it would have bound, as an error,
                    // so its uses do not cascade.
                    first_binder(&l.pattern)
                }
            };
            let vt = if name.is_some() && failed {
                STy::Error
            } else {
                vt
            };
            self.record(&p, l.span);
            let mut lam = p.clone();
            lam.push(0);
            self.record(&lam, l.span);
            self.env.push(Binding {
                name,
                ty: vt.clone(),
            });
            bound.push((vt.to_ty().unwrap_or(Ty::Nat), ve));
            p.extend([0, 0]);
        }
        let (te, tt) = self.expr(tail, &mut p, expect);
        self.env.truncate(self.env.len() - lets.len());
        if failed || tt.is_error() {
            return self.placeholder();
        }
        let mut result = te;
        for (dom, value) in bound.into_iter().rev() {
            result = Expr::app(
                Expr::Lam {
                    dom,
                    body: Box::new(result),
                },
                value,
            );
        }
        (result, tt)
    }

    // ---- if --------------------------------------------------------------------

    fn if_(
        &mut self,
        c: &SurfaceExpr,
        t: &SurfaceExpr,
        f: &SurfaceExpr,
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        // app (app (app (prim ite) c) t) f — c at [0,0,1], t at [0,1], f at [1]
        let mut cp = path.clone();
        cp.extend([0, 0, 1]);
        let (ce, ct) = self.expr(c, &mut cp, Some(&STy::Bool));
        let (ce, ct) = self.observe(ce, ct, &cp, c.span);
        let mut tp = path.clone();
        tp.extend([0, 1]);
        let (te, tt) = self.expr(t, &mut tp, expect);
        let mut fp = path.clone();
        fp.push(1);
        let hint = if tt.is_known() { Some(&tt) } else { expect };
        let (fe, ft) = self.expr(f, &mut fp, hint);
        if ct.is_error() || tt.is_error() || ft.is_error() {
            return self.placeholder();
        }
        if ct != STy::Bool {
            self.kind_error(
                span,
                "the condition of `if` must be true or false",
                &ct,
                c.span,
            );
            return self.placeholder();
        }
        let mut branches = vec![
            Branch {
                span: t.span,
                path: tp,
                expr: te,
                ty: tt,
            },
            Branch {
                span: f.span,
                path: fp,
                expr: fe,
                ty: ft,
            },
        ];
        let surfaces = [t, f];
        let Some(ty) = self.join(&mut branches, "if", span, &mut |el, i, want, p| {
            let mut p = p.clone();
            el.expr(surfaces[i], &mut p, Some(want))
        }) else {
            return self.placeholder();
        };
        let Some(core_ty) = ty.to_ty() else {
            return self.undetermined(span);
        };
        let mut it = branches.into_iter();
        let (te, fe) = match (it.next(), it.next()) {
            (Some(a), Some(b)) => (a.expr, b.expr),
            _ => return self.placeholder(),
        };
        (
            Expr::apps(Expr::prim(Prim::Ite { ty: core_ty }), [ce, te, fe]),
            ty,
        )
    }

    /// Bring branches to one type: equal semantic results stay semantic,
    /// otherwise every semantic branch is observed and every branch must
    /// have the same representation; a bare `None` takes its payload from
    /// a sibling by re-elaboration.  `None` after reporting a mismatch.
    fn join(
        &mut self,
        branches: &mut [Branch],
        what: &str,
        span: Span,
        re_elab: &mut ReElab<'_, 'a>,
    ) -> Option<STy> {
        if branches.iter().any(|b| b.ty.is_error()) {
            return None;
        }
        let first = branches[0].ty.clone();
        if matches!(first, STy::Sem(_)) && branches.iter().all(|b| b.ty == first) {
            return Some(first);
        }
        for b in branches.iter_mut() {
            if matches!(b.ty, STy::Sem(_)) {
                let (e, t) = self.observe(
                    std::mem::replace(&mut b.expr, Expr::NatLit { value: 0 }),
                    b.ty.clone(),
                    &b.path,
                    b.span,
                );
                b.expr = e;
                b.ty = t;
                if b.ty.is_error() {
                    return None;
                }
            }
        }
        // The common type, letting `Unknown` payloads yield.
        let mut target = branches[0].ty.clone();
        for b in branches.iter().skip(1) {
            match STy::unify(&target, &b.ty) {
                Some(t) => target = t,
                None => {
                    self.branch_mismatch(what, span, &target, &b.ty);
                    return None;
                }
            }
        }
        if !target.is_known() {
            self.undetermined(span);
            return None;
        }
        for (i, b) in branches.iter_mut().enumerate() {
            if !b.ty.is_known() {
                let (e, t) = re_elab(self, i, &target, &b.path);
                if t != target {
                    if !t.is_error() {
                        self.branch_mismatch(what, span, &target, &t);
                    }
                    return None;
                }
                b.expr = e;
                b.ty = t;
            }
        }
        Some(target)
    }

    fn branch_mismatch(&mut self, what: &str, span: Span, a: &STy, b: &STy) {
        let (a, b) = (self.describe(a), self.describe(b));
        let d = match what {
            "delay" | "sync" => self
                .error(
                    "type.temporal_mismatch",
                    span,
                    format!("The initial value is {a} but the remembered value is {b}."),
                )
                .explain("What is remembered and what stands in for it before anything was remembered must be the same kind of value."),
            _ => self
                .error(
                    "type.branch_mismatch",
                    span,
                    format!("The branches of this `{what}` produce different kinds of value: {a} and {b}."),
                )
                .explain(format!(
                    "A `{what}` is one value that depends on a condition; every branch must be the same kind."
                )),
        };
        self.push(d);
    }

    // ---- match -----------------------------------------------------------------

    fn match_(
        &mut self,
        scrutinee: &SurfaceExpr,
        arms: &[SurfaceArm],
        span: Span,
        path: &mut ExprPath,
        expect: Option<&STy>,
    ) -> (Expr, STy) {
        // app (λs:τ. TREE) s — s at [1], TREE at [0,0].
        let mut sp = path.clone();
        sp.push(1);
        let (se, st) = self.expr(scrutinee, &mut sp, None);
        if st.is_error() {
            return self.placeholder();
        }
        if !st.is_known() {
            return self.undetermined(scrutinee.span);
        }
        let mut lam = path.clone();
        lam.push(0);
        self.record(&lam, span);
        self.env.push(Binding {
            name: None,
            ty: st.clone(),
        });
        let depth = self.env.len() - 1;

        // Exhaustiveness and reachability are decided on the patterns alone.
        let patterns: Vec<&SurfacePattern> = arms.iter().map(|a| &a.pattern).collect();
        let exhaustive = covers(&st, &patterns, self.ir);
        let mut keys: BTreeSet<String> = BTreeSet::new();
        let mut first_catch_all: Option<usize> = None;
        for (i, arm) in arms.iter().enumerate() {
            let key = pattern_key(&arm.pattern);
            let unreachable = covers(&st, &patterns[..i], self.ir) || !keys.insert(key);
            if unreachable {
                self.push(
                    Diagnostic::new(
                        "formula.match.unreachable",
                        Severity::Warning,
                        self.entity(),
                        "This arm can never be chosen: the arms above it already cover every value it matches.",
                    )
                    .at(arm.span),
                );
            }
            if first_catch_all.is_none() && is_catch_all(&arm.pattern) {
                first_catch_all = Some(i);
            }
        }
        if !exhaustive {
            let missing = missing_cases(&st, &patterns, self.ir);
            let d = self
                .error(
                    "formula.match.non_exhaustive",
                    span,
                    format!("Not every value is handled: {missing}."),
                )
                .explain("A `match` is one value, so it must say what happens for every possible value it looks at.");
            self.push(d);
        }

        // Elaborate every arm (for diagnostics), keeping those up to and
        // including the first one that always matches; later arms are
        // unreachable and leave the term.
        let last_used = first_catch_all.unwrap_or(arms.len().saturating_sub(1));
        let mut tree_path = path.clone();
        tree_path.extend([0, 0]);
        let mut outs: Vec<ArmOut> = Vec::new();
        let mut failed = !exhaustive;
        for (i, arm) in arms.iter().enumerate() {
            let is_base = i == last_used;
            // ite c body rest at tree_path: c [0,0,1], body [0,1], rest [1]
            let body_path = if is_base {
                tree_path.clone()
            } else {
                let mut b = tree_path.clone();
                b.extend([0, 1]);
                b
            };
            let mut out = self.arm(arm, &st, depth, &body_path, expect);
            if out.ty.is_error() {
                failed = true;
            }
            if is_base {
                out.cond = None;
            }
            if i <= last_used {
                outs.push(out);
            }
            if !is_base {
                tree_path.push(1);
            }
        }
        self.env.pop();
        if failed || outs.is_empty() {
            return self.placeholder();
        }
        let mut branches: Vec<Branch> = outs
            .iter()
            .map(|o| Branch {
                span: o.span,
                path: o.path.clone(),
                expr: o.body.clone(),
                ty: o.ty.clone(),
            })
            .collect();
        let st_again = st.clone();
        let Some(ty) = self.join(&mut branches, "match", span, &mut |el, i, want, p| {
            el.env.push(Binding {
                name: None,
                ty: st_again.clone(),
            });
            let out = el.arm(&arms[i], &st_again, depth, p, Some(want));
            el.env.pop();
            (out.body, out.ty)
        }) else {
            return self.placeholder();
        };
        let Some(core_ty) = ty.to_ty() else {
            return self.undetermined(span);
        };
        // Fold right: the base arm is the innermost alternative.
        let mut tree: Option<Expr> = None;
        for (o, b) in outs.into_iter().zip(branches).rev() {
            tree = Some(match (o.cond, tree) {
                (None, _) | (_, None) => b.expr,
                (Some(c), Some(rest)) => Expr::apps(
                    Expr::prim(Prim::Ite {
                        ty: core_ty.clone(),
                    }),
                    [c, b.expr, rest],
                ),
            });
        }
        let Some(tree) = tree else {
            return self.placeholder();
        };
        let dom = st.to_ty().unwrap_or(Ty::Nat);
        (
            Expr::app(
                Expr::Lam {
                    dom,
                    body: Box::new(tree),
                },
                se,
            ),
            ty,
        )
    }

    /// One arm: its test (`None` for a catch-all), its body wrapped in the
    /// pattern's bindings, and the body's type.
    fn arm(
        &mut self,
        arm: &SurfaceArm,
        st: &STy,
        depth: usize,
        body_path: &ExprPath,
        expect: Option<&STy>,
    ) -> ArmOut {
        let mut c = Compiled::default();
        let access = Access {
            scrutinee: depth,
            steps: Vec::new(),
        };
        self.pattern(&arm.pattern, st, access, &mut c);
        let cond = if c.tests.is_empty() {
            None
        } else {
            let mut tests = c.tests.iter().map(|t| self.test(t));
            let first = tests.next().unwrap_or(Expr::BoolLit { value: true });
            Some(tests.fold(first, |acc, t| Expr::apps(Expr::prim(Prim::And), [acc, t])))
        };
        // Bindings: app (λx₁. app (λx₂. body) v₂) v₁ — vᵢ at [1] of its
        // app, the next app at [0,0].
        let mut p = body_path.clone();
        let mut bound: Vec<(Ty, Expr)> = Vec::new();
        for b in &c.binds {
            let value = self.materialize(&b.access);
            self.record(&p, b.span);
            let mut lam = p.clone();
            lam.push(0);
            self.record(&lam, b.span);
            let mut vp = p.clone();
            vp.push(1);
            self.record(&vp, b.span);
            self.env.push(Binding {
                name: Some(b.name.clone()),
                ty: b.ty.clone(),
            });
            bound.push((b.ty.to_ty().unwrap_or(Ty::Nat), value));
            p.extend([0, 0]);
        }
        let (be, bt) = self.expr(&arm.body, &mut p, expect);
        self.env.truncate(self.env.len() - c.binds.len());
        if c.failed || bt.is_error() {
            return ArmOut {
                cond,
                body: be,
                ty: STy::Error,
                span: arm.body.span,
                path: body_path.clone(),
            };
        }
        let mut body = be;
        for (dom, value) in bound.into_iter().rev() {
            body = Expr::app(
                Expr::Lam {
                    dom,
                    body: Box::new(body),
                },
                value,
            );
        }
        ArmOut {
            cond,
            body,
            ty: bt,
            span: arm.body.span,
            path: body_path.clone(),
        }
    }

    fn materialize(&self, access: &Access) -> Expr {
        let mut e = self.var(access.scrutinee);
        for step in &access.steps {
            e = match step {
                Step::Rep => Expr::rep(e),
                Step::Unwrap(inner) => Expr::apps(
                    Expr::prim(Prim::GetD {
                        ty: inner.to_ty().unwrap_or(Ty::Nat),
                    }),
                    [e, self.default_of(inner)],
                ),
            };
        }
        e
    }

    fn test(&self, t: &Test) -> Expr {
        match t {
            Test::IsSome(a, inner) => Expr::app(
                Expr::prim(Prim::IsSome {
                    ty: inner.to_ty().unwrap_or(Ty::Nat),
                }),
                self.materialize(a),
            ),
            Test::IsNone(a, inner) => Expr::app(
                Expr::prim(Prim::Not),
                Expr::app(
                    Expr::prim(Prim::IsSome {
                        ty: inner.to_ty().unwrap_or(Ty::Nat),
                    }),
                    self.materialize(a),
                ),
            ),
            Test::Bool(a, true) => self.materialize(a),
            Test::Bool(a, false) => Expr::app(Expr::prim(Prim::Not), self.materialize(a)),
            Test::Eq(a, v) => Expr::apps(
                Expr::prim(Prim::Eq {
                    ty: Ty::q(Dim::ZERO),
                }),
                [self.materialize(a), self.lit(Dim::ZERO, *v)],
            ),
            Test::EqNat(a, v) => Expr::apps(
                Expr::prim(Prim::Eq { ty: Ty::Nat }),
                [self.materialize(a), Expr::NatLit { value: *v }],
            ),
        }
    }

    /// Compile a pattern against a value of type `ty` reached by `access`.
    fn pattern(&mut self, p: &SurfacePattern, ty: &STy, access: Access, out: &mut Compiled) {
        match &p.kind {
            PatternKind::Wildcard => {}
            PatternKind::Ident(n) if n == "None" => self.none_pattern(p, ty, access, out),
            PatternKind::Ident(n) if n == "Some" => {
                out.failed = true;
                let d = self.error(
                    "formula.constructor.arity",
                    p.span,
                    "`Some` needs a pattern for its value: write `Some(x)`.",
                );
                self.push(d);
            }
            PatternKind::Ident(n) => {
                if is_reserved_name(n) {
                    out.failed = true;
                    let d = self.error(
                        "formula.pattern.constructor_binding",
                        p.span,
                        format!("`{n}` cannot be used as a pattern name; it is a temporal form."),
                    );
                    self.push(d);
                    return;
                }
                if out.binds.iter().any(|b| &b.name == n) {
                    out.failed = true;
                    let d = self
                        .error(
                            "formula.pattern.duplicate_binding",
                            p.span,
                            format!("`{n}` is bound twice in this pattern."),
                        )
                        .fix("Use a different name for each part, or `_` for a part you do not need.");
                    self.push(d);
                    return;
                }
                out.binds.push(Bind {
                    name: n.clone(),
                    span: p.span,
                    ty: ty.clone(),
                    access,
                });
            }
            PatternKind::Bool(b) => match ty {
                STy::Bool => out.tests.push(Test::Bool(access, *b)),
                STy::Sem(s) => self.through_rep(p, *s, access, out),
                other => self.pattern_kind_error(p, "true or false", other, out),
            },
            PatternKind::Number { negative, literal } => match ty {
                STy::Q(dim) => {
                    if *dim != Dim::ZERO {
                        out.failed = true;
                        let shown = self.describe(ty);
                        let d = self
                            .error(
                                "dimension.mismatch",
                                p.span,
                                format!("This pattern is a plain number, but the value matched is {shown}."),
                            )
                            .fix("Compare with a unit in an `if` instead, e.g. `if x == 10 deg then … else …`.");
                        self.push(d);
                        return;
                    }
                    match literal.to_f64() {
                        Some(v) => out
                            .tests
                            .push(Test::Eq(access, if *negative { -v } else { v })),
                        None => {
                            out.failed = true;
                            let d = self.error(
                                "formula.number.too_large",
                                p.span,
                                format!(
                                    "`{}` is too large a number to compute with.",
                                    literal.as_str()
                                ),
                            );
                            self.push(d);
                        }
                    }
                }
                STy::Nat => {
                    // Structural equality on counts (Phase 9b): a whole,
                    // non-negative number.
                    let whole = literal
                        .decimal()
                        .filter(|d| d.normalized().exponent >= 0)
                        .and_then(|_| literal.to_f64())
                        .filter(|v| !*negative && v.fract() == 0.0 && *v <= u64::MAX as f64);
                    match whole {
                        Some(v) => out.tests.push(Test::EqNat(access, v as u64)),
                        None => {
                            out.failed = true;
                            let d = self.error(
                                "formula.pattern.kind",
                                p.span,
                                "A count is matched against a whole, non-negative number.",
                            );
                            self.push(d);
                        }
                    }
                }
                STy::Sem(s) => self.through_rep(p, *s, access, out),
                other => self.pattern_kind_error(p, "a number", other, out),
            },
            PatternKind::Constructor { name, fields } => match name.as_str() {
                "Some" => match ty {
                    STy::Opt(inner) => {
                        if fields.len() != 1 {
                            out.failed = true;
                            let d = self.error(
                                "formula.constructor.arity",
                                p.span,
                                format!("`Some` carries exactly one value, not {}.", fields.len()),
                            );
                            self.push(d);
                            return;
                        }
                        out.tests
                            .push(Test::IsSome(access.clone(), inner.as_ref().clone()));
                        let mut inner_access = access;
                        inner_access
                            .steps
                            .push(Step::Unwrap(inner.as_ref().clone()));
                        self.pattern(&fields[0], inner, inner_access, out);
                    }
                    STy::Sem(s) => self.through_rep(p, *s, access, out),
                    other => self.pattern_kind_error(p, "an optional value", other, out),
                },
                "None" => {
                    if !fields.is_empty() {
                        out.failed = true;
                        let d = self.error(
                            "formula.constructor.arity",
                            p.span,
                            "`None` carries no value; write `None` alone.",
                        );
                        self.push(d);
                        return;
                    }
                    self.none_pattern(p, ty, access, out);
                }
                other => {
                    out.failed = true;
                    let d = self
                        .error(
                            "formula.constructor.unknown",
                            p.span,
                            format!("`{other}` is not a constructor."),
                        )
                        .explain("Only `Some(…)` and `None` exist; enum types declared in text are not part of the design yet (DI-19).");
                    self.push(d);
                }
            },
        }
    }

    fn none_pattern(&mut self, p: &SurfacePattern, ty: &STy, access: Access, out: &mut Compiled) {
        match ty {
            STy::Opt(inner) => out.tests.push(Test::IsNone(access, inner.as_ref().clone())),
            STy::Sem(s) => self.through_rep(p, *s, access, out),
            other => self.pattern_kind_error(p, "an optional value", other, out),
        }
    }

    /// A literal or constructor pattern on a semantic value matches its
    /// representation.
    fn through_rep(
        &mut self,
        p: &SurfacePattern,
        s: SemanticId,
        access: Access,
        out: &mut Compiled,
    ) {
        match self.ir.representation_of(s).and_then(STy::of) {
            Some(rep) => {
                let mut a = access;
                a.steps.push(Step::Rep);
                self.pattern(p, &rep, a, out);
            }
            None => {
                out.failed = true;
                let d = self.unbound(s, p.span, "matches");
                self.push(d);
            }
        }
    }

    fn pattern_kind_error(
        &mut self,
        p: &SurfacePattern,
        matches: &str,
        found: &STy,
        out: &mut Compiled,
    ) {
        out.failed = true;
        let shown = self.describe(found);
        let d = self.error(
            "formula.pattern.kind",
            p.span,
            format!("This pattern matches {matches}, but the value looked at is {shown}."),
        );
        self.push(d);
    }

    // ---- operators (representation level) --------------------------------------

    fn unary(
        &mut self,
        op: UnaryOp,
        inner: &SurfaceExpr,
        span: Span,
        path: &mut ExprPath,
    ) -> (Expr, STy) {
        // app (prim op) x — x at [1]; `-x` is `sub 0 x`, x at [1] of the outer app
        path.push(1);
        let (x, tx) = self.expr(inner, path, None);
        let (x, tx) = self.observe(x, tx, path, inner.span);
        path.pop();
        match op {
            UnaryOp::Not => match tx {
                STy::Bool => (Expr::app(Expr::prim(Prim::Not), x), STy::Bool),
                STy::Error => (x, STy::Error),
                other => {
                    self.kind_error(span, "`!` needs a true-or-false value", &other, inner.span);
                    (x, STy::Error)
                }
            },
            UnaryOp::Neg => match tx {
                STy::Q(d) => (
                    Expr::apps(Expr::prim(Prim::Sub { dim: d }), [self.lit(d, 0.0), x]),
                    STy::Q(d),
                ),
                STy::Error => (x, STy::Error),
                other => {
                    self.kind_error(span, "`-` needs a quantity", &other, inner.span);
                    (x, STy::Error)
                }
            },
        }
    }

    fn binary(
        &mut self,
        op: BinaryOp,
        l: &SurfaceExpr,
        r: &SurfaceExpr,
        span: Span,
        path: &mut ExprPath,
    ) -> (Expr, STy) {
        use BinaryOp::*;
        // `x in xs` is the library's `contains(x, xs)`; the application
        // shape `app (app f l) r` is the one below, so the paths agree.
        if op == In {
            if let Some(entry) = equations::lookup("contains") {
                return self.equation(entry, &[l.clone(), r.clone()], span, path, None);
            }
        }
        // app (app (prim op) l) r — l at [0,1], r at [1]
        path.push(0);
        path.push(1);
        let lp = path.clone();
        let (le, lt) = self.expr(l, path, None);
        path.pop();
        path.pop();
        path.push(1);
        let rp = path.clone();
        let (re, rt) = self.expr(r, path, None);
        path.pop();
        if lt.is_error() || rt.is_error() {
            return (Expr::apps(Expr::prim(Prim::And), [le, re]), STy::Error);
        }
        // Two concept values compare as concepts: the same concept, or a
        // nominal error — never through their representations.
        // (Arithmetic stays on representations, ADR-0013.)
        if let (STy::Sem(a), STy::Sem(b), Eq | Ne | Lt | Le | Gt | Ge) = (&lt, &rt, op) {
            if a != b {
                let (an, bn) = (self.concept_name(*a), self.concept_name(*b));
                let d = self
                    .error(
                        "semantic.concept_mismatch",
                        span,
                        format!("{an} and {bn} are different concepts."),
                    )
                    .explain("A concept's identity is never exchanged for another's, whatever the representation; only values of one concept compare.")
                    .fix(format!("Compare {an} with {an}, or map one concept to the other through a relationship."));
                self.push(d);
                return (le, STy::Error);
            }
            match op {
                Eq | Ne => {
                    let eq = Expr::apps(Expr::prim(Prim::Eq { ty: Ty::sem(*a) }), [le, re]);
                    return (
                        if op == Ne {
                            Expr::app(Expr::prim(Prim::Not), eq)
                        } else {
                            eq
                        },
                        STy::Bool,
                    );
                }
                Lt | Le | Gt | Ge => {
                    let Some(o) = self.ordered_evidence(&Ty::sem(*a)) else {
                        self.no_order(op.symbol(), &Ty::sem(*a), span);
                        return (le, STy::Error);
                    };
                    let d = o.dim();
                    self.shift_under(&lp, l.span);
                    self.shift_under(&rp, r.span);
                    let (le, re) = (Expr::rep(le), Expr::rep(re));
                    return (compare(op, d, le, re), STy::Bool);
                }
                _ => {}
            }
        }
        let (le, lt) = self.observe(le, lt, &lp, l.span);
        let (re, rt) = self.observe(re, rt, &rp, r.span);
        if lt.is_error() || rt.is_error() {
            return (Expr::apps(Expr::prim(Prim::And), [le, re]), STy::Error);
        }
        match op {
            Add | Sub | Mul | Div | Lt | Le | Gt | Ge => {
                let (STy::Q(dl), STy::Q(dr)) = (&lt, &rt) else {
                    let bad = if !matches!(lt, STy::Q(_)) {
                        (&lt, l.span)
                    } else {
                        (&rt, r.span)
                    };
                    if matches!(op, Lt | Le | Gt | Ge) {
                        let ty = match bad.0 {
                            STy::Opt(_) => Some(bad.0.to_ty().unwrap_or(Ty::opt(Ty::Nat))),
                            STy::List(_) => Some(bad.0.to_ty().unwrap_or(Ty::list(Ty::Nat))),
                            other => other.to_ty(),
                        };
                        if let Some(ty) = ty {
                            if !matches!(ty, Ty::Q { .. }) {
                                self.no_order(op.symbol(), &ty, bad.1);
                                return (le, STy::Error);
                            }
                        }
                    }
                    self.kind_error(
                        span,
                        &format!("`{}` needs quantities on both sides", op.symbol()),
                        bad.0,
                        bad.1,
                    );
                    return (le, STy::Error);
                };
                let (dl, dr) = (*dl, *dr);
                match op {
                    Mul => (
                        Expr::apps(Expr::prim(Prim::Mul { d1: dl, d2: dr }), [le, re]),
                        STy::Q(dl + dr),
                    ),
                    Div => (
                        Expr::apps(Expr::prim(Prim::Div { d1: dl, d2: dr }), [le, re]),
                        STy::Q(dl - dr),
                    ),
                    _ => {
                        if dl != dr {
                            self.dimension_mismatch(op, span, dl, dr);
                            return (le, STy::Error);
                        }
                        match op {
                            Add => (
                                Expr::apps(Expr::prim(Prim::Add { dim: dl }), [le, re]),
                                STy::Q(dl),
                            ),
                            Sub => (
                                Expr::apps(Expr::prim(Prim::Sub { dim: dl }), [le, re]),
                                STy::Q(dl),
                            ),
                            _ => (compare(op, dl, le, re), STy::Bool),
                        }
                    }
                }
            }
            Eq | Ne => {
                // Structural equality at any data kind (Phase 9b): the two
                // sides must be one kind; a bare `None` or `[]` takes the
                // other side's.
                let Some(target) = STy::unify(&lt, &rt) else {
                    let (a, b) = (self.describe(&lt), self.describe(&rt));
                    let d = self
                        .error(
                            "type.operand_kind",
                            span,
                            format!("`{}` compares {a} with {b}.", op.symbol()),
                        )
                        .explain("Both sides of a comparison must be the same kind of value.");
                    self.push(d);
                    return (le, STy::Error);
                };
                if let (STy::Q(dl), STy::Q(dr)) = (&lt, &rt) {
                    if dl != dr {
                        self.dimension_mismatch(op, span, *dl, *dr);
                        return (le, STy::Error);
                    }
                }
                let Some(ty) = target.to_ty() else {
                    return self.undetermined(span);
                };
                let (le, lt2) = if lt.is_known() {
                    (le, lt)
                } else {
                    let mut p = lp.clone();
                    self.expr(l, &mut p, Some(&target))
                };
                let (re, rt2) = if rt.is_known() {
                    (re, rt)
                } else {
                    let mut p = rp.clone();
                    self.expr(r, &mut p, Some(&target))
                };
                if lt2.is_error() || rt2.is_error() {
                    return (le, STy::Error);
                }
                let eq = Expr::apps(Expr::prim(Prim::Eq { ty }), [le, re]);
                (
                    if op == Ne {
                        Expr::app(Expr::prim(Prim::Not), eq)
                    } else {
                        eq
                    },
                    STy::Bool,
                )
            }
            And | Or => {
                if lt != STy::Bool || rt != STy::Bool {
                    let bad = if lt != STy::Bool {
                        (&lt, l.span)
                    } else {
                        (&rt, r.span)
                    };
                    self.kind_error(
                        span,
                        &format!("`{}` needs true-or-false values on both sides", op.symbol()),
                        bad.0,
                        bad.1,
                    );
                    return (le, STy::Error);
                }
                let p = if op == And { Prim::And } else { Prim::Or };
                (Expr::apps(Expr::prim(p), [le, re]), STy::Bool)
            }
            In => self.placeholder(),
        }
    }

    fn kind_error(&mut self, span: Span, need: &str, found: &STy, at: Span) {
        let shown = self.describe(found);
        let d = self
            .error(
                "type.operand_kind",
                at,
                format!("{need}, but this is {shown}."),
            )
            .technical(format!("in expression at {}..{}", span.start, span.end));
        self.push(d);
    }

    fn dimension_mismatch(&mut self, op: BinaryOp, span: Span, l: Dim, r: Dim) {
        let verb = match op {
            BinaryOp::Add => "adds",
            BinaryOp::Sub => "subtracts",
            _ => "compares",
        };
        self.push(
            Diagnostic::error(
                "dimension.mismatch",
                self.entity(),
                format!(
                    "This expression {verb} values with different physical dimensions: {} and {}.",
                    pretty::describe_dim(l),
                    pretty::describe_dim(r)
                ),
            )
            .at(span)
            .explain("Only quantities of the same dimension can be added, subtracted or compared. Multiplying or dividing combines dimensions.")
            .technical(format!("{} : q[{}] → q[{}] → …, found q[{}]", op.symbol(), pretty::symbol(l), pretty::symbol(l), pretty::symbol(r))),
        );
    }
}

/// Whether a type mentions a concept anywhere.
fn mentions_concept(t: &STy) -> bool {
    match t {
        STy::Sem(_) => true,
        STy::Opt(i) | STy::List(i) => mentions_concept(i),
        STy::Pair(a, b) => mentions_concept(a) || mentions_concept(b),
        _ => false,
    }
}

/// Whether `sem` is a concept value whose representation is exactly
/// `rep`: the case an equation observes rather than refuses.
fn observable(ir: &DesignIr, sem: &Ty, rep: &Ty) -> bool {
    match sem {
        Ty::Sem { id } => !matches!(rep, Ty::Sem { .. }) && ir.representation_of(*id) == Some(rep),
        _ => false,
    }
}

/// A quantity comparison of dimension `d`: `lt` directly, the others by
/// swapping and negating — `a > b ≡ b < a`, `a <= b ≡ !(b < a)`, `a >= b ≡ !(a < b)`.
fn compare(op: BinaryOp, d: Dim, l: Expr, r: Expr) -> Expr {
    let lt = |a: Expr, b: Expr| Expr::apps(Expr::prim(Prim::Lt { dim: d }), [a, b]);
    match op {
        BinaryOp::Lt => lt(l, r),
        BinaryOp::Gt => lt(r, l),
        BinaryOp::Le => Expr::app(Expr::prim(Prim::Not), lt(r, l)),
        BinaryOp::Ge => Expr::app(Expr::prim(Prim::Not), lt(l, r)),
        _ => Expr::apps(Expr::prim(Prim::And), [l, r]),
    }
}

fn temporal_shape(sync: bool) -> &'static str {
    if sync {
        "sync(domain, init, value)"
    } else {
        "delay(init, value)"
    }
}

/// Names a `let` or pattern may not bind: constructors and the temporal
/// forms, which are resolved before any binding.
fn is_reserved_name(n: &str) -> bool {
    matches!(n, "Some" | "None" | "delay" | "sync")
}

// ---- pattern predicates -----------------------------------------------------

fn is_catch_all(p: &SurfacePattern) -> bool {
    match &p.kind {
        PatternKind::Wildcard => true,
        PatternKind::Ident(n) => n != "None" && n != "Some",
        _ => false,
    }
}

fn is_none_pattern(p: &SurfacePattern) -> bool {
    matches!(&p.kind, PatternKind::Ident(n) if n == "None")
        || matches!(&p.kind, PatternKind::Constructor { name, fields } if name == "None" && fields.is_empty())
}

fn some_payloads<'a>(pats: &[&'a SurfacePattern]) -> Vec<&'a SurfacePattern> {
    pats.iter()
        .filter_map(|p| match &p.kind {
            PatternKind::Constructor { name, fields } if name == "Some" && fields.len() == 1 => {
                Some(&fields[0])
            }
            _ => None,
        })
        .collect()
}

/// Conservative exhaustiveness: catch-all; `true` and `false`; `None` and
/// an exhaustive `Some(…)`; a semantic value through its representation.
fn covers(ty: &STy, pats: &[&SurfacePattern], ir: &DesignIr) -> bool {
    if pats.iter().any(|p| is_catch_all(p)) {
        return true;
    }
    match ty {
        STy::Bool => {
            let has = |b: bool| pats.iter().any(|p| p.kind == PatternKind::Bool(b));
            has(true) && has(false)
        }
        STy::Opt(inner) => {
            let somes = some_payloads(pats);
            pats.iter().any(|p| is_none_pattern(p))
                && !somes.is_empty()
                && covers(inner, &somes, ir)
        }
        STy::Sem(s) => match ir.representation_of(*s).and_then(STy::of) {
            Some(rep) => covers(&rep, pats, ir),
            None => false,
        },
        _ => false,
    }
}

fn missing_cases(ty: &STy, pats: &[&SurfacePattern], ir: &DesignIr) -> String {
    match ty {
        STy::Bool => {
            let has = |b: bool| pats.iter().any(|p| p.kind == PatternKind::Bool(b));
            match (has(true), has(false)) {
                (false, true) => "`true` is not handled".into(),
                (true, false) => "`false` is not handled".into(),
                _ => "neither `true` nor `false` is handled".into(),
            }
        }
        STy::Opt(inner) => {
            let somes = some_payloads(pats);
            if !pats.iter().any(|p| is_none_pattern(p)) {
                "`None` is not handled".into()
            } else if somes.is_empty() {
                "`Some(…)` is not handled".into()
            } else {
                format!("inside `Some(…)`, {}", missing_cases(inner, &somes, ir))
            }
        }
        STy::Sem(s) => match ir.representation_of(*s).and_then(STy::of) {
            Some(rep) => missing_cases(&rep, pats, ir),
            None => "the value's kind is not decided yet".into(),
        },
        _ => "add a final `_ => …` arm for every other value".into(),
    }
}

/// A structural key: two arms with equal keys match exactly the same
/// values, so the second is unreachable.
fn pattern_key(p: &SurfacePattern) -> String {
    match &p.kind {
        PatternKind::Wildcard => "_".into(),
        PatternKind::Ident(n) if n == "None" => "None".into(),
        PatternKind::Ident(_) => "_".into(),
        PatternKind::Bool(b) => b.to_string(),
        PatternKind::Number { negative, literal } => format!(
            "{}{}",
            if *negative { "-" } else { "" },
            literal
                .decimal()
                .map(|d| d.normalized().to_string())
                .unwrap_or_else(|| literal.as_str().to_owned())
        ),
        PatternKind::Constructor { name, fields } => format!(
            "{name}({})",
            fields.iter().map(pattern_key).collect::<Vec<_>>().join(",")
        ),
    }
}

/// The first name a pattern binds, if any.
fn first_binder(p: &SurfacePattern) -> Option<String> {
    match &p.kind {
        PatternKind::Ident(n) if !is_reserved_name(n) => Some(n.clone()),
        PatternKind::Constructor { fields, .. } => fields.iter().find_map(first_binder),
        _ => None,
    }
}

fn describe_pattern(k: &PatternKind) -> String {
    match k {
        PatternKind::Wildcard => "_".into(),
        PatternKind::Ident(n) => n.clone(),
        PatternKind::Bool(b) => b.to_string(),
        PatternKind::Number { negative, literal } => {
            format!("{}{}", if *negative { "-" } else { "" }, literal.as_str())
        }
        PatternKind::Constructor { name, fields } => format!(
            "{name}({})",
            fields
                .iter()
                .map(|f| describe_pattern(&f.kind))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Convenience for tests and tools: a `DeclId`-keyed diagnostic filter.
pub fn diagnostics_for(diags: &[Diagnostic], id: DeclId) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| d.entity == Entity::Mapping { id })
        .collect()
}
