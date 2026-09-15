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
use bdl_syntax::{BinaryOp, ExprKind, SurfaceExpr, UnaryOp};
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
            STy::Bool => "true or false".to_string(),
            STy::Nat => "a count".to_string(),
            STy::Error => "an erroneous value".to_string(),
        }
    }
}

struct Elab<'a> {
    design: &'a Design,
    ir: &'a DesignIr,
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
    let env = InputEnv::for_inputs(design, inputs);
    let input_tys = inputs
        .iter()
        .map(|c| ir.representation_of(*c).and_then(STy::of))
        .collect();
    let mut el = Elab {
        design,
        ir,
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
            ExprKind::Number { literal, unit } => {
                // The syntax keeps the exact spelling; the machine number is
                // made here, at the elaboration boundary (DI-1, ADR-0011).
                // The exact/symbolic numeric layer will take over this
                // conversion; until then `f64` is the runtime representation.
                let Some(value) = literal.to_f64() else {
                    self.diags.push(
                        Diagnostic::error(
                            "formula.number.too_large",
                            self.entity(),
                            format!(
                                "`{}` is too large a number to compute with.",
                                literal.as_str()
                            ),
                        )
                        .at(e.span),
                    );
                    return (self.lit(Dim::ZERO, 0.0), STy::Error);
                };
                match unit {
                    None => (self.lit(Dim::ZERO, value), STy::Q(Dim::ZERO)),
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
                }
            }
            ExprKind::Name(name) => self.name(name, e.span, path),
            ExprKind::Unary { op, expr } => self.unary(*op, expr, e.span, path),
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, e.span, path),
            ExprKind::If { cond, then, els } => self.if_(cond, then, els, e.span, path),
            ExprKind::Call { callee, args } => self.call(callee, args, e.span, path),
            // Parsed by the textual syntax; `match` and blocks need a kernel
            // extension mirrored in Lean first (DI-19).
            ExprKind::Match { .. } => self.unsupported("`match`", e.span),
            ExprKind::Block { .. } => self.unsupported("a block", e.span),
        }
    }

    /// The representation type of a concept's value, or the unbound
    /// diagnostic (an *open* state, not an error).
    fn rep_ty(&mut self, concept: SemanticId, span: Span, role: &str) -> STy {
        match self.ir.representation_of(concept).and_then(STy::of) {
            Some(ty) => ty,
            None => {
                let d = self.unbound(concept, span, role);
                self.diags.push(d);
                STy::Error
            }
        }
    }

    /// A surface expression that must denote a *semantic value* — an input,
    /// a relationship without inputs, or a relationship applied to such
    /// values.  Returns the Core term (typed `sem C`) and `C`.  Anything
    /// else is refused: a formula may compute over representations, but a
    /// concept value is only made by the relationship whose signature
    /// announces it (the grant).
    fn sem_value(&mut self, e: &SurfaceExpr, path: &mut ExprPath) -> Option<(Expr, SemanticId)> {
        self.record(path, e.span);
        match &e.kind {
            ExprKind::Name(name) => match self.env.resolve(self.design, name) {
                Lookup::Input(i) => {
                    let n = self.mapping.signature.inputs.len();
                    let index = (n - 1 - i) as u32;
                    Some((Expr::Var { index }, self.mapping.signature.inputs[i]))
                }
                Lookup::Mapping(id) => {
                    let m = &self.design.mappings[&id];
                    if !m.signature.inputs.is_empty() {
                        self.needs_arguments(m, e.span);
                        return None;
                    }
                    Some((Expr::decl(id), m.signature.output))
                }
                _ => {
                    // The value-position diagnostics apply (unknown, not an input…).
                    self.name(name, e.span, path);
                    None
                }
            },
            ExprKind::Call { callee, args } => self.call_sem(callee, args, e.span, path),
            _ => {
                self.diags.push(
                    Diagnostic::error(
                        "formula.call.argument",
                        self.entity(),
                        "Only a concept's value can be passed to a relationship: an input, or another relationship's value.".to_string(),
                    )
                    .at(e.span)
                    .explain("A relationship reads concept values; a computed number has no concept until the relationship that produces it makes one."),
                );
                None
            }
        }
    }

    fn needs_arguments(&mut self, m: &MappingBlock, span: Span) {
        let params: Vec<String> = m
            .signature
            .inputs
            .iter()
            .map(|c| self.concept_name(*c))
            .collect();
        self.diags.push(
            Diagnostic::error(
                "formula.mapping.needs_arguments",
                self.entity(),
                format!(
                    "{} reads {}; give it those values.",
                    m.name,
                    params.join(", ")
                ),
            )
            .at(span)
            .fix(format!("Write {}({}).", m.name, params.join(", "))),
        );
    }

    /// `f(a₁, …, aₙ)` where `f` is a relationship: `app (… (app (declRef f) a₁) …) aₙ`,
    /// typed `sem B`.  Argument i sits at path `[0]ⁿ⁻¹⁻ⁱ ++ [1]` from the
    /// chain's root.
    fn call_sem(
        &mut self,
        callee: &SurfaceExpr,
        args: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
    ) -> Option<(Expr, SemanticId)> {
        let ExprKind::Name(name) = &callee.kind else {
            self.diags.push(
                Diagnostic::error(
                    "formula.call.not_a_relationship",
                    self.entity(),
                    "Only a relationship can be applied.".to_string(),
                )
                .at(callee.span),
            );
            return None;
        };
        let id = match self.env.resolve(self.design, name) {
            Lookup::Mapping(id) => id,
            Lookup::Input(_) | Lookup::NotAnInput(..) => {
                self.diags.push(
                    Diagnostic::error(
                        "formula.call.not_a_relationship",
                        self.entity(),
                        format!("{name} is a concept, not a relationship; it cannot be applied."),
                    )
                    .at(callee.span),
                );
                return None;
            }
            _ => {
                let relationships: Vec<&str> = self
                    .design
                    .mappings
                    .values()
                    .map(|m| m.name.as_str())
                    .collect();
                self.diags.push(
                    Diagnostic::error(
                        "formula.name.unknown",
                        self.entity(),
                        format!("`{name}` is not a relationship of this design."),
                    )
                    .at(callee.span)
                    .fix(if relationships.is_empty() {
                        "This design has no other relationship yet.".to_string()
                    } else {
                        format!("Relationships: {}.", relationships.join(", "))
                    }),
                );
                return None;
            }
        };
        let m = &self.design.mappings[&id];
        let params = m.signature.inputs.clone();
        let output = m.signature.output;
        let mname = m.name.clone();
        if params.len() != args.len() {
            let names: Vec<String> = params.iter().map(|c| self.concept_name(*c)).collect();
            self.diags.push(
                Diagnostic::error(
                    "formula.call.arity",
                    self.entity(),
                    format!(
                        "{mname} reads {} value{} ({}), but {} {} given here.",
                        params.len(),
                        if params.len() == 1 { "" } else { "s" },
                        names.join(", "),
                        args.len(),
                        if args.len() == 1 { "is" } else { "are" }
                    ),
                )
                .at(span),
            );
            return None;
        }
        let n = args.len();
        let mut ok = true;
        let mut terms = Vec::with_capacity(n);
        for (i, (arg, expected)) in args.iter().zip(params.iter()).enumerate() {
            let depth = n - 1 - i;
            path.extend(std::iter::repeat_n(0, depth));
            path.push(1);
            let value = self.sem_value(arg, path);
            path.truncate(path.len() - depth - 1);
            match value {
                Some((term, concept)) => {
                    if concept != *expected {
                        let (want, got) =
                            (self.concept_name(*expected), self.concept_name(concept));
                        self.diags.push(
                            Diagnostic::error(
                                "formula.call.argument_type",
                                self.entity(),
                                format!("{mname} reads {want} here, but this is {got}."),
                            )
                            .at(arg.span)
                            .explain("Concepts are identities: a value of one concept is never a value of another, whatever its representation."),
                        );
                        ok = false;
                    }
                    terms.push(term);
                }
                None => ok = false,
            }
        }
        if !ok {
            return None;
        }
        path.extend(std::iter::repeat_n(0, n));
        self.record(path, callee.span);
        path.truncate(path.len() - n);
        Some((Expr::apps(Expr::decl(id), terms), output))
    }

    /// A call in value position: a relationship applied (its result's
    /// representation, through `rep`), or one of the two temporal forms.
    fn call(
        &mut self,
        callee: &SurfaceExpr,
        args: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
    ) -> (Expr, STy) {
        if let ExprKind::Name(name) = &callee.kind {
            match name.as_str() {
                "delay" => return self.temporal(false, args, span, path),
                "sync" => return self.temporal(true, args, span, path),
                _ => {}
            }
        }
        // rep (chain) — the chain at [0]
        path.push(0);
        let value = self.call_sem(callee, args, span, path);
        path.pop();
        match value {
            Some((term, concept)) => {
                let ty = self.rep_ty(concept, span, "applies a relationship that produces");
                (Expr::rep(term), ty)
            }
            None => (self.lit(Dim::ZERO, 0.0), STy::Error),
        }
    }

    /// `delay(init, e)` and `sync(domain, init, e)`: the kernel's own memory
    /// forms, reachable only from a relationship without inputs (memory
    /// belongs to a relationship, not to a formula argument).
    fn temporal(
        &mut self,
        sync: bool,
        args: &[SurfaceExpr],
        span: Span,
        path: &mut ExprPath,
    ) -> (Expr, STy) {
        let word = if sync { "sync" } else { "delay" };
        let expected = if sync { 3 } else { 2 };
        if args.len() != expected {
            self.diags.push(
                Diagnostic::error(
                    "formula.temporal.arity",
                    self.entity(),
                    if sync {
                        "`sync` takes a timing domain, an initial value and the value to observe: sync(domain, init, value).".to_string()
                    } else {
                        "`delay` takes an initial value and the value to remember: delay(init, value).".to_string()
                    },
                )
                .at(span),
            );
            return (self.lit(Dim::ZERO, 0.0), STy::Error);
        }
        if !self.mapping.signature.inputs.is_empty() {
            self.diags.push(
                Diagnostic::error(
                    "formula.temporal.under_inputs",
                    self.entity(),
                    format!("`{word}` can only be used in a relationship without inputs."),
                )
                .at(span)
                .explain("Memory belongs to a relationship as a whole — a value per activation — not to a formula over its inputs. Read the inputs through relationships without inputs and remember there."),
            );
            return (self.lit(Dim::ZERO, 0.0), STy::Error);
        }
        let src = if sync {
            let domain = &args[0];
            let ExprKind::Name(dname) = &domain.kind else {
                self.diags.push(
                    Diagnostic::error(
                        "formula.sync.unknown_domain",
                        self.entity(),
                        "The first argument of `sync` must name a timing domain.".to_string(),
                    )
                    .at(domain.span),
                );
                return (self.lit(Dim::ZERO, 0.0), STy::Error);
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
                    self.diags.push(
                        Diagnostic::error(
                            "formula.sync.unknown_domain",
                            self.entity(),
                            format!("`{dname}` is not a timing domain of this design."),
                        )
                        .at(domain.span)
                        .fix(if known.is_empty() {
                            "Create a timing domain first.".to_string()
                        } else {
                            format!("Timing domains: {}.", known.join(", "))
                        }),
                    );
                    return (self.lit(Dim::ZERO, 0.0), STy::Error);
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
        path.push(0);
        let (ie, it) = self.expr(init, path);
        path.pop();
        path.push(1);
        let (ve, vt) = self.expr(value, path);
        path.pop();
        if it == STy::Error || vt == STy::Error {
            return (ie, STy::Error);
        }
        if it != vt {
            self.diags.push(
                Diagnostic::error(
                    "type.temporal_mismatch",
                    self.entity(),
                    format!(
                        "The initial value is {} but the remembered value is {}.",
                        it.describe(),
                        vt.describe()
                    ),
                )
                .at(span),
            );
            return (ie, STy::Error);
        }
        let term = match src {
            None => Expr::Delay {
                init: Box::new(ie),
                e: Box::new(ve),
            },
            Some(src) => Expr::Sync {
                src,
                init: Box::new(ie),
                e: Box::new(ve),
            },
        };
        (term, vt)
    }

    fn unsupported(&mut self, what: &str, span: Span) -> (Expr, STy) {
        self.diags.push(
            Diagnostic::error(
                "formula.unsupported",
                self.entity(),
                format!("{what} cannot be used in a formula yet."),
            )
            .at(span)
            .explain("Formulas combine the mapping's inputs and other relationships' values with arithmetic, comparisons, `&&`, `||`, `!`, `if`, `delay` and `sync`."),
        );
        (self.lit(Dim::ZERO, 0.0), STy::Error)
    }

    fn name(&mut self, name: &str, span: Span, path: &mut ExprPath) -> (Expr, STy) {
        let n = self.mapping.signature.inputs.len();
        match self.env.resolve(self.design, name) {
            Lookup::Mapping(id) => {
                let m = &self.design.mappings[&id];
                if !m.signature.inputs.is_empty() {
                    self.needs_arguments(m, span);
                    return (self.lit(Dim::ZERO, 0.0), STy::Error);
                }
                // rep (declRef m) — the reference at [0]
                let output = m.signature.output;
                path.push(0);
                self.record(path, span);
                path.pop();
                let ty = self.rep_ty(
                    output,
                    span,
                    "reads the value of a relationship that produces",
                );
                (Expr::rep(Expr::decl(id)), ty)
            }
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
                let relationships: Vec<&str> = self
                    .design
                    .mappings
                    .values()
                    .filter(|m| m.id != self.mapping.id)
                    .map(|m| m.name.as_str())
                    .collect();
                let mut fix = if available.is_empty() {
                    "This mapping reads nothing; connect a concept to it first.".to_string()
                } else {
                    format!("Available: {}.", available.join(", "))
                };
                if !relationships.is_empty() {
                    fix.push_str(&format!(" Relationships: {}.", relationships.join(", ")));
                }
                self.diags.push(
                    Diagnostic::error(
                        "formula.name.unknown",
                        self.entity(),
                        format!("`{name}` is not something this mapping reads."),
                    )
                    .at(span)
                    .fix(fix),
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
