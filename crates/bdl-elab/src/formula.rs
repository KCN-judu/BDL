//! Formula → Core `Expr`.
//!
//! The elaborator types the surface expression at the *representation*
//! level (a value is a quantity of some dimension, a truth value, or a count)
//! because the kernel's primitives are dimension-indexed: to emit `add` it
//! must know *which* `add`.  The checker (`bdl-check`) then re-derives the
//! type of the produced Core term under the declaration's grant; it is the
//! authority, this pass is the translator.

use crate::names::{InputEnv, Lookup};
use crate::units;
use bdl_check::pretty;
use bdl_check::ExprPath;
use bdl_diagnostics::{Diagnostic, Entity, Span};
use bdl_ir::{DesignIr, Expr, Prim, Scalar, Ty};
use bdl_model::surface::{Design, MappingBlock};
use bdl_model::{DeclId, Dim, SemanticId};
use bdl_syntax::{parse, BinaryOp, ExprKind, SurfaceExpr, UnaryOp};
use std::collections::BTreeMap;

/// A successfully elaborated realization.
#[derive(Clone, Debug, PartialEq)]
pub struct Realized {
    pub expr: Expr,
    /// Source span of each Core sub-term the surface produced, by path.
    pub spans: BTreeMap<ExprPath, Span>,
}

/// Representation-level type of a surface sub-expression.
#[derive(Clone, Debug, PartialEq)]
enum STy {
    Q(Dim),
    Bool,
    Nat,
    /// Already reported; silences cascades.
    Error,
}

impl STy {
    fn of(ty: &Ty) -> Option<STy> {
        match ty {
            Ty::Q { dim } => Some(STy::Q(*dim)),
            Ty::Bool => Some(STy::Bool),
            Ty::Nat => Some(STy::Nat),
            _ => None,
        }
    }
    fn describe(&self) -> String {
        match self {
            STy::Q(d) => pretty::describe_dim(*d),
            STy::Bool => "true or false".into(),
            STy::Nat => "a count".into(),
            STy::Error => "an erroneous value".into(),
        }
    }
}

struct Elab<'a> {
    design: &'a Design,
    mapping: &'a MappingBlock,
    env: InputEnv,
    /// Representation type of each input, `None` when unbound.
    input_tys: Vec<Option<STy>>,
    diags: Vec<Diagnostic>,
    spans: BTreeMap<ExprPath, Span>,
}

/// Elaborate `mapping`'s formula `source` into `λ x₁ … xₙ. mk B (…)`.
///
/// `Err` carries the diagnostics that stopped elaboration (parse errors,
/// unknown names, unbound representations, dimension and kind errors); a
/// realization is only produced when every sub-expression typed.
pub fn elaborate_formula(
    design: &Design,
    ir: &DesignIr,
    mapping: &MappingBlock,
    source: &str,
) -> Result<Realized, Vec<Diagnostic>> {
    let entity = Entity::Mapping { id: mapping.id };
    let surface = match parse(source) {
        Ok(e) => e,
        Err(e) => {
            return Err(vec![Diagnostic::error(
                "formula.parse.unexpected_token",
                entity,
                e.message,
            )
            .at(e.span)
            .technical(format!("found {}", e.found))])
        }
    };

    let inputs = &mapping.signature.inputs;
    let env = InputEnv::for_inputs(design, inputs);
    let input_tys = inputs
        .iter()
        .map(|c| ir.representation_of(*c).and_then(STy::of))
        .collect();
    let mut el = Elab {
        design,
        mapping,
        env,
        input_tys,
        diags: Vec::new(),
        spans: BTreeMap::new(),
    };

    // Path of the body: under n lambdas, then under `mk`.
    let n = inputs.len();
    let mut path: ExprPath = vec![0; n + 1];
    let (body, body_ty) = el.expr(&surface, &mut path);

    // The result is constructed as the output concept.
    let out = mapping.signature.output;
    let out_rep = ir.representation_of(out).and_then(STy::of);
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
                        expected.describe(),
                        found.describe()
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

    if el.diags.iter().any(Diagnostic::is_error)
        || out_rep.is_none()
        || el.input_tys.iter().any(Option::is_none)
    {
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
    Ok(Realized {
        expr,
        spans: el.spans,
    })
}

impl Elab<'_> {
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

    fn record(&mut self, path: &ExprPath, span: Span) {
        self.spans.insert(path.clone(), span);
    }

    fn lit(&self, dim: Dim, value: f64) -> Expr {
        Expr::prim(Prim::Lit {
            dim,
            value: Scalar(value),
        })
    }

    /// Elaborate at `path`; returns the Core term and its representation type.
    fn expr(&mut self, e: &SurfaceExpr, path: &mut ExprPath) -> (Expr, STy) {
        self.record(path, e.span);
        match &e.kind {
            ExprKind::Bool(b) => (Expr::BoolLit { value: *b }, STy::Bool),
            ExprKind::Number { value, unit, .. } => match unit {
                None => (self.lit(Dim::ZERO, *value), STy::Q(Dim::ZERO)),
                Some(u) => match units::lookup(&u.name) {
                    Some(def) => (self.lit(def.dim, value * def.factor), STy::Q(def.dim)),
                    None => {
                        self.diags.push(
                            Diagnostic::error(
                                "formula.unit.unknown",
                                self.entity(),
                                format!("`{}` is not a unit.", u.name),
                            )
                            .at(u.span)
                            .fix(format!("Units available: {}.", units::names().join(", "))),
                        );
                        (self.lit(Dim::ZERO, 0.0), STy::Error)
                    }
                },
            },
            ExprKind::Name(name) => self.name(name, e.span),
            ExprKind::Unary { op, expr } => self.unary(*op, expr, e.span, path),
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, e.span, path),
            ExprKind::If { cond, then, els } => self.if_(cond, then, els, e.span, path),
        }
    }

    fn name(&mut self, name: &str, span: Span) -> (Expr, STy) {
        let n = self.mapping.signature.inputs.len();
        match self.env.resolve(self.design, name) {
            Lookup::Input(i) => {
                // input i is bound by the (n-1-i)-th innermost lambda
                let index = (n - 1 - i) as u32;
                match self.input_tys[i].clone() {
                    Some(ty) => (Expr::rep(Expr::Var { index }), ty),
                    None => {
                        let d = self.unbound(self.mapping.signature.inputs[i], span, "reads");
                        self.diags.push(d);
                        (Expr::rep(Expr::Var { index }), STy::Error)
                    }
                }
            }
            Lookup::Ambiguous(candidates) => {
                self.diags.push(
                    Diagnostic::error(
                        "formula.name.ambiguous",
                        self.entity(),
                        format!("`{name}` could mean any of {}.", candidates.join(", ")),
                    )
                    .at(span)
                    .fix("Write the name exactly as the concept is called, including its capitalisation."),
                );
                (self.lit(Dim::ZERO, 0.0), STy::Error)
            }
            Lookup::NotAnInput(_, concept_name) => {
                let mapping = self.mapping.name.clone();
                self.diags.push(
                    Diagnostic::error(
                        "formula.name.not_an_input",
                        self.entity(),
                        format!("{mapping} does not read {concept_name}."),
                    )
                    .at(span)
                    .explain("A formula can only use the concepts its mapping reads.")
                    .fix(format!("Connect {concept_name} to {mapping} as an input.")),
                );
                (self.lit(Dim::ZERO, 0.0), STy::Error)
            }
            Lookup::Unknown => {
                let available = self.env.names();
                self.diags.push(
                    Diagnostic::error(
                        "formula.name.unknown",
                        self.entity(),
                        format!("`{name}` is not something this mapping reads."),
                    )
                    .at(span)
                    .fix(if available.is_empty() {
                        "This mapping reads nothing; connect a concept to it first.".to_string()
                    } else {
                        format!("Available: {}.", available.join(", "))
                    }),
                );
                (self.lit(Dim::ZERO, 0.0), STy::Error)
            }
        }
    }

    fn unary(
        &mut self,
        op: UnaryOp,
        inner: &SurfaceExpr,
        span: Span,
        path: &mut ExprPath,
    ) -> (Expr, STy) {
        match op {
            UnaryOp::Not => {
                // app (prim not) x  — x at [1]
                path.push(1);
                let (x, tx) = self.expr(inner, path);
                path.pop();
                match tx {
                    STy::Bool | STy::Error => (
                        Expr::app(Expr::prim(Prim::Not), x),
                        if tx == STy::Error {
                            STy::Error
                        } else {
                            STy::Bool
                        },
                    ),
                    other => {
                        self.kind_error(
                            span,
                            "`!` needs a true-or-false value",
                            &other,
                            inner.span,
                        );
                        (x, STy::Error)
                    }
                }
            }
            UnaryOp::Neg => {
                // app (app (prim sub) 0) x — x at [1]
                path.push(1);
                let (x, tx) = self.expr(inner, path);
                path.pop();
                match tx {
                    STy::Q(d) => (
                        Expr::apps(Expr::prim(Prim::Sub { dim: d }), [self.lit(d, 0.0), x]),
                        STy::Q(d),
                    ),
                    STy::Error => (x, STy::Error),
                    other => {
                        self.kind_error(span, "`-` needs a quantity", &other, inner.span);
                        (x, STy::Error)
                    }
                }
            }
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
        // app (app (prim op) l) r — l at [0,1], r at [1]
        path.push(0);
        path.push(1);
        let (le, lt) = self.expr(l, path);
        path.pop();
        path.pop();
        path.push(1);
        let (re, rt) = self.expr(r, path);
        path.pop();
        if lt == STy::Error || rt == STy::Error {
            return (Expr::apps(Expr::prim(Prim::And), [le, re]), STy::Error);
        }
        use BinaryOp::*;
        match op {
            Add | Sub | Mul | Div | Lt | Le | Gt | Ge => {
                let (STy::Q(dl), STy::Q(dr)) = (&lt, &rt) else {
                    let bad = if !matches!(lt, STy::Q(_)) {
                        (&lt, l.span)
                    } else {
                        (&rt, r.span)
                    };
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
                            Lt => (
                                Expr::apps(Expr::prim(Prim::Lt { dim: dl }), [le, re]),
                                STy::Bool,
                            ),
                            // a > b  ≡  b < a
                            Gt => (
                                Expr::apps(Expr::prim(Prim::Lt { dim: dl }), [re, le]),
                                STy::Bool,
                            ),
                            // a <= b ≡ !(b < a);  a >= b ≡ !(a < b)
                            Le => (
                                Expr::app(
                                    Expr::prim(Prim::Not),
                                    Expr::apps(Expr::prim(Prim::Lt { dim: dl }), [re, le]),
                                ),
                                STy::Bool,
                            ),
                            Ge => (
                                Expr::app(
                                    Expr::prim(Prim::Not),
                                    Expr::apps(Expr::prim(Prim::Lt { dim: dl }), [le, re]),
                                ),
                                STy::Bool,
                            ),
                            _ => unreachable!("arithmetic/comparison operators only"),
                        }
                    }
                }
            }
            Eq | Ne => {
                let eq = match (&lt, &rt) {
                    (STy::Q(dl), STy::Q(dr)) => {
                        if dl != dr {
                            self.dimension_mismatch(op, span, *dl, *dr);
                            return (le, STy::Error);
                        }
                        Expr::apps(Expr::prim(Prim::Eq { dim: *dl }), [le, re])
                    }
                    (STy::Bool, STy::Bool) => {
                        // (a && b) || (!a && !b); the kernel has no boolean equality
                        let both = Expr::apps(Expr::prim(Prim::And), [le.clone(), re.clone()]);
                        let neither = Expr::apps(
                            Expr::prim(Prim::And),
                            [
                                Expr::app(Expr::prim(Prim::Not), le),
                                Expr::app(Expr::prim(Prim::Not), re),
                            ],
                        );
                        Expr::apps(Expr::prim(Prim::Or), [both, neither])
                    }
                    _ => {
                        self.diags.push(
                            Diagnostic::error(
                                "type.operand_kind",
                                self.entity(),
                                format!(
                                    "`{}` compares {} with {}.",
                                    op.symbol(),
                                    lt.describe(),
                                    rt.describe()
                                ),
                            )
                            .at(span)
                            .explain("Both sides of a comparison must be the same kind of value."),
                        );
                        return (le, STy::Error);
                    }
                };
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
        }
    }

    fn if_(
        &mut self,
        c: &SurfaceExpr,
        t: &SurfaceExpr,
        f: &SurfaceExpr,
        span: Span,
        path: &mut ExprPath,
    ) -> (Expr, STy) {
        // app (app (app (prim ite) c) t) f — c at [0,0,1], t at [0,1], f at [1]
        path.extend([0, 0, 1]);
        let (ce, ct) = self.expr(c, path);
        path.truncate(path.len() - 3);
        path.extend([0, 1]);
        let (te, tt) = self.expr(t, path);
        path.truncate(path.len() - 2);
        path.push(1);
        let (fe, ft) = self.expr(f, path);
        path.pop();
        if ct == STy::Error || tt == STy::Error || ft == STy::Error {
            return (ce, STy::Error);
        }
        if ct != STy::Bool {
            self.kind_error(
                span,
                "the condition of `if` must be true or false",
                &ct,
                c.span,
            );
            return (ce, STy::Error);
        }
        if tt != ft {
            self.diags.push(
                Diagnostic::error(
                    "type.branch_mismatch",
                    self.entity(),
                    format!("The two branches of this `if` produce different kinds of value: {} and {}.", tt.describe(), ft.describe()),
                )
                .at(span)
                .explain("An `if` is one value that depends on a condition; both branches must be the same kind."),
            );
            return (ce, STy::Error);
        }
        let ty = match &tt {
            STy::Q(d) => Ty::q(*d),
            STy::Bool => Ty::Bool,
            STy::Nat => Ty::Nat,
            STy::Error => unreachable!("errors returned above"),
        };
        (Expr::apps(Expr::prim(Prim::Ite { ty }), [ce, te, fe]), tt)
    }

    fn kind_error(&mut self, span: Span, need: &str, found: &STy, at: Span) {
        self.diags.push(
            Diagnostic::error(
                "type.operand_kind",
                self.entity(),
                format!("{need}, but this is {}.", found.describe()),
            )
            .at(at)
            .technical(format!("in expression at {}..{}", span.start, span.end)),
        );
    }

    fn dimension_mismatch(&mut self, op: BinaryOp, span: Span, l: Dim, r: Dim) {
        let verb = match op {
            BinaryOp::Add => "adds",
            BinaryOp::Sub => "subtracts",
            _ => "compares",
        };
        self.diags.push(
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

/// Convenience for tests and tools: a `DeclId`-keyed diagnostic filter.
pub fn diagnostics_for(diags: &[Diagnostic], id: DeclId) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| d.entity == Entity::Mapping { id })
        .collect()
}
