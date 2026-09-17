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
//! added for the textual syntax (`docs/TEXTUAL_SYNTAX.md` §11):
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
use bdl_ir::{DesignIr, Expr, Prim, Scalar, Ty};
use bdl_model::surface::{Design, MappingBlock};
use bdl_model::{ClockId, DeclId, Dim, SemanticId};
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
    /// The payload of a bare `None` before context fixes it.
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
            STy::Unknown | STy::Error => return None,
        })
    }

    /// Fully determined: no `Unknown`, no `Error`.
    fn is_known(&self) -> bool {
        match self {
            STy::Opt(t) => t.is_known(),
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
            STy::Unknown => "a value of a kind not yet known".into(),
            STy::Error => "an erroneous value".into(),
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
        parts.push("constructors: Some(…), None".into());
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
        let id = match self.inputs.resolve(self.design, name) {
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
                Expr::prim(Prim::Eq { dim: Dim::ZERO }),
                [self.materialize(a), self.lit(Dim::ZERO, *v)],
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
                    out.failed = true;
                    let d = self
                        .error(
                            "formula.unsupported",
                            p.span,
                            "A count cannot be matched against a number yet.",
                        )
                        .explain("The kernel has no equality on counts (DI-12); bind the count with a name and use it as a value.");
                    self.push(d);
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
        // app (app (prim op) l) r — l at [0,1], r at [1]
        path.push(0);
        path.push(1);
        let (le, lt) = self.expr(l, path, None);
        let (le, lt) = self.observe(le, lt, path, l.span);
        path.pop();
        path.pop();
        path.push(1);
        let (re, rt) = self.expr(r, path, None);
        let (re, rt) = self.observe(re, rt, path, r.span);
        path.pop();
        if lt.is_error() || rt.is_error() {
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
                        let (a, b) = (self.describe(&lt), self.describe(&rt));
                        let d = self
                            .error(
                                "type.operand_kind",
                                span,
                                format!("`{}` compares {a} with {b}.", op.symbol()),
                            )
                            .explain("Both sides of a comparison must be the same kind of value; an optional value is taken apart with `match`.");
                        self.push(d);
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
