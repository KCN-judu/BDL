//! Lowering: typed syntax AST → the *surface tree*, a plain, id-free data
//! structure of what was written (names, spans, exact literals).  This is
//! what `bdl-elab` consumes for formulas today and what a textual project
//! loader will turn into `bdl-model` edits.  Nothing semantic happens here:
//! no name resolution, no ids, no types (`docs/TEXTUAL_SYNTAX.md` §11).
//!
//! Lowering is total on error-free trees.  It refuses trees with syntax
//! errors (the errors are returned) rather than guessing at missing parts.

use crate::ast::{self, AstNode, LiteralKind, LiteralPatternKind};
pub use crate::ast::{BinaryOp, UnaryOp};
use crate::literal::NumberLiteral;
use crate::syntax::{Parse, SyntaxError, SyntaxErrorCode};
use bdl_diagnostics::Span;
use serde::{Deserialize, Serialize};

// ---- expressions -----------------------------------------------------------

/// An expression as written, with spans.  Parentheses are dropped (their
/// span is kept on the inner expression).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceExpr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    /// A reference, resolved by the elaborator.
    Name(String),
    /// A numeric literal, exactly as spelled, optionally with a unit
    /// written after it (`90 deg`).  No machine value lives here.
    Number {
        literal: NumberLiteral,
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
    Call {
        callee: Box<SurfaceExpr>,
        args: Vec<SurfaceExpr>,
    },
    Match {
        scrutinee: Box<SurfaceExpr>,
        arms: Vec<SurfaceArm>,
    },
    Block {
        lets: Vec<SurfaceLet>,
        tail: Box<SurfaceExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceArm {
    pub pattern: SurfacePattern,
    pub body: SurfaceExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceLet {
    pub pattern: SurfacePattern,
    pub value: SurfaceExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfacePattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternKind {
    Wildcard,
    /// A bare name: binding or nullary constructor (§8.1).
    Ident(String),
    Bool(bool),
    Number {
        negative: bool,
        literal: NumberLiteral,
    },
    Constructor {
        name: String,
        fields: Vec<SurfacePattern>,
    },
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
            ExprKind::Call { callee, args } => {
                callee.walk(f);
                args.iter().for_each(|a| a.walk(f));
            }
            ExprKind::Match { scrutinee, arms } => {
                scrutinee.walk(f);
                arms.iter().for_each(|a| a.body.walk(f));
            }
            ExprKind::Block { lets, tail } => {
                lets.iter().for_each(|l| l.value.walk(f));
                tail.walk(f);
            }
        }
    }
}

// ---- types and items -------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceType {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    Named {
        name: String,
        args: Vec<SurfaceType>,
    },
    Function {
        domain: Box<SurfaceType>,
        codomain: Box<SurfaceType>,
    },
}

impl SurfaceType {
    /// `A -> B -> C` as `([A, B], C)`.
    pub fn uncurry(&self) -> (Vec<&SurfaceType>, &SurfaceType) {
        let mut inputs = Vec::new();
        let mut t = self;
        while let TypeKind::Function { domain, codomain } = &t.kind {
            inputs.push(domain.as_ref());
            t = codomain;
        }
        (inputs, t)
    }
}

/// A name at a definition site.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceModule {
    pub items: Vec<SurfaceItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SurfaceItem {
    Concept(ConceptItem),
    Mapping(MappingItem),
    Enum(EnumItem),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptItem {
    pub name: Ident,
    /// `None` for an open concept (`concept Tilt`).
    pub representation: Option<SurfaceType>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MappingItem {
    pub name: Ident,
    /// The type as written, e.g. `Held -> Tilt -> Brightness`.
    pub signature: SurfaceType,
    pub definition: Option<MappingDefinition>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MappingDefinition {
    /// The name repeated before the parameters (checked against the
    /// declaration's name, §8.2).
    pub name: Ident,
    pub params: Vec<SurfacePattern>,
    pub body: SurfaceExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumItem {
    pub name: Ident,
    pub type_params: Vec<Ident>,
    pub variants: Vec<VariantItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantItem {
    pub name: Ident,
    pub fields: Vec<SurfaceType>,
    pub span: Span,
}

impl SurfaceModule {
    pub fn concepts(&self) -> impl Iterator<Item = &ConceptItem> {
        self.items.iter().filter_map(|i| match i {
            SurfaceItem::Concept(c) => Some(c),
            _ => None,
        })
    }
    pub fn mappings(&self) -> impl Iterator<Item = &MappingItem> {
        self.items.iter().filter_map(|i| match i {
            SurfaceItem::Mapping(m) => Some(m),
            _ => None,
        })
    }
    pub fn enums(&self) -> impl Iterator<Item = &EnumItem> {
        self.items.iter().filter_map(|i| match i {
            SurfaceItem::Enum(e) => Some(e),
            _ => None,
        })
    }
}

// ---- entry points ----------------------------------------------------------

/// Lower a parsed formula.  `Err` carries the syntax errors when there are
/// any; an error-free tree always lowers.
pub fn lower_formula(parse: &Parse<ast::Formula>) -> Result<SurfaceExpr, Vec<SyntaxError>> {
    if !parse.is_ok() {
        return Err(parse.errors().to_vec());
    }
    let root = parse.tree();
    root.expr()
        .and_then(|e| expr(&e))
        .ok_or_else(|| vec![incomplete(root.span())])
}

/// Lower a parsed module.  Beyond syntax errors, lowering reports the
/// structural mismatches it can see without any semantic context: a
/// definition whose name differs from its declaration (§8.2) and a
/// parameter count that disagrees with the signature's arity.  Those are
/// returned as `syntax.*`-coded errors alongside the (possibly partial)
/// module so a loader can still show what parsed.
pub fn lower_module(parse: &Parse<ast::Module>) -> (SurfaceModule, Vec<SyntaxError>) {
    let syntax_errors = parse.errors().to_vec();
    let mut errors = syntax_errors.clone();
    let mut items = Vec::new();
    for item in parse.tree().items() {
        match item {
            ast::Item::Concept(c) => {
                if let Some(c) = concept(&c) {
                    items.push(SurfaceItem::Concept(c));
                }
            }
            ast::Item::Mapping(m) => {
                // Structural checks are only meaningful on a mapping that
                // parsed cleanly; a syntax error inside it already explains
                // any mismatch.
                let span = m.span();
                let clean = !syntax_errors.iter().any(|e| {
                    e.span.start < span.end && span.start < e.span.end.max(e.span.start + 1)
                });
                if let Some(m) = mapping(&m, clean, &mut errors) {
                    items.push(SurfaceItem::Mapping(m));
                }
            }
            ast::Item::Enum(e) => {
                if let Some(e) = enum_(&e) {
                    items.push(SurfaceItem::Enum(e));
                }
            }
        }
    }
    errors.sort_by_key(|e| (e.span.start, e.span.end));
    (SurfaceModule { items }, errors)
}

fn incomplete(span: Span) -> SyntaxError {
    SyntaxError::new(
        SyntaxErrorCode::Expected,
        span,
        "this part of the source is incomplete",
        "nothing",
    )
}

fn ident(name: &ast::Name) -> Option<Ident> {
    let tok = name.ident()?;
    Some(Ident {
        name: tok.text().to_owned(),
        span: crate::syntax::span_of(tok.text_range()),
    })
}

fn ident_ref(name: &ast::NameRef) -> Option<Ident> {
    let tok = name.ident()?;
    Some(Ident {
        name: tok.text().to_owned(),
        span: crate::syntax::span_of(tok.text_range()),
    })
}

fn concept(c: &ast::ConceptDecl) -> Option<ConceptItem> {
    let name = ident(&c.name()?)?;
    let representation = match c.representation() {
        Some(t) => Some(type_(&t)?),
        None => None,
    };
    Some(ConceptItem {
        name,
        representation,
        span: c.span(),
    })
}

fn mapping(
    m: &ast::MappingDecl,
    clean: bool,
    errors: &mut Vec<SyntaxError>,
) -> Option<MappingItem> {
    let name = ident(&m.name()?)?;
    let signature = type_(&m.signature()?)?;
    let definition = match m.definition() {
        Some(def) => {
            let def_name = ident_ref(&def.name()?)?;
            let params: Option<Vec<SurfacePattern>> = def.params().map(|p| pattern(&p)).collect();
            let params = params?;
            let body = expr(&def.body()?)?;
            if clean && def_name.name != name.name {
                errors.push(
                    SyntaxError::new(
                        SyntaxErrorCode::Unexpected,
                        def_name.span,
                        format!(
                            "this definition is named `{}`, but the mapping declared above it is `{}`",
                            def_name.name, name.name
                        ),
                        format!("`{}`", def_name.name),
                    )
                    .with_hint(format!(
                        "A definition repeats the name of its signature: `{}({}) = …`.",
                        name.name,
                        params
                            .iter()
                            .map(|p| p.kind.describe())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                );
            }
            let arity = signature.uncurry().0.len();
            if clean && params.len() != arity {
                errors.push(
                    SyntaxError::new(
                        SyntaxErrorCode::Unexpected,
                        def.param_list().map(|l| l.span()).unwrap_or(def.span()),
                        format!(
                            "`{}` reads {arity} input{} by its signature, but its definition names {}",
                            name.name,
                            if arity == 1 { "" } else { "s" },
                            params.len()
                        ),
                        format!("{} parameter(s)", params.len()),
                    )
                    .with_hint("Each `->` in the signature except the last introduces one input."),
                );
            }
            Some(MappingDefinition {
                name: def_name,
                params,
                body,
                span: def.span(),
            })
        }
        None => None,
    };
    Some(MappingItem {
        name,
        signature,
        definition,
        span: m.span(),
    })
}

fn enum_(e: &ast::EnumDecl) -> Option<EnumItem> {
    let name = ident(&e.name()?)?;
    let type_params: Option<Vec<Ident>> = e.type_params().map(|n| ident(&n)).collect();
    let variants: Option<Vec<VariantItem>> = e
        .variants()
        .map(|v| {
            let fields: Option<Vec<SurfaceType>> = v.fields().map(|t| type_(&t)).collect();
            Some(VariantItem {
                name: ident(&v.name()?)?,
                fields: fields?,
                span: v.span(),
            })
        })
        .collect();
    Some(EnumItem {
        name,
        type_params: type_params?,
        variants: variants?,
        span: e.span(),
    })
}

fn type_(t: &ast::Type) -> Option<SurfaceType> {
    let span = t.span();
    Some(match t {
        ast::Type::Named(n) => {
            let args: Option<Vec<SurfaceType>> = n.type_args().map(|a| type_(&a)).collect();
            SurfaceType {
                kind: TypeKind::Named {
                    name: n.name()?.as_str(),
                    args: args?,
                },
                span,
            }
        }
        ast::Type::Function(f) => SurfaceType {
            kind: TypeKind::Function {
                domain: Box::new(type_(&f.domain()?)?),
                codomain: Box::new(type_(&f.codomain()?)?),
            },
            span,
        },
        ast::Type::Paren(p) => SurfaceType {
            kind: type_(&p.inner()?)?.kind,
            span,
        },
    })
}

fn pattern(p: &ast::Pattern) -> Option<SurfacePattern> {
    let span = p.span();
    let kind = match p {
        ast::Pattern::Wildcard(_) => PatternKind::Wildcard,
        ast::Pattern::Ident(i) => PatternKind::Ident(i.name()?.as_str()),
        ast::Pattern::Literal(l) => match l.kind()? {
            LiteralPatternKind::Bool(b) => PatternKind::Bool(b),
            LiteralPatternKind::Number { negative, number } => PatternKind::Number {
                negative,
                literal: number.literal(),
            },
        },
        ast::Pattern::Constructor(c) => {
            let fields: Option<Vec<SurfacePattern>> = c.fields().map(|f| pattern(&f)).collect();
            PatternKind::Constructor {
                name: c.name()?.as_str(),
                fields: fields?,
            }
        }
    };
    Some(SurfacePattern { kind, span })
}

impl PatternKind {
    fn describe(&self) -> String {
        match self {
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
                    .map(|f| f.kind.describe())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

fn expr(e: &ast::Expr) -> Option<SurfaceExpr> {
    let span = e.span();
    let kind = match e {
        ast::Expr::Name(n) => ExprKind::Name(n.name()?.as_str()),
        ast::Expr::Literal(l) => match l.kind()? {
            LiteralKind::Bool(b) => ExprKind::Bool(b),
            LiteralKind::Number(n) => ExprKind::Number {
                literal: n.literal(),
                unit: l.unit().and_then(|u| {
                    let tok = u.ident()?;
                    Some(Unit {
                        name: tok.text().to_owned(),
                        span: crate::syntax::span_of(tok.text_range()),
                    })
                }),
            },
        },
        ast::Expr::Paren(p) => expr(&p.inner()?)?.kind,
        ast::Expr::Call(c) => {
            let args: Option<Vec<SurfaceExpr>> = c.arguments().map(|a| expr(&a)).collect();
            ExprKind::Call {
                callee: Box::new(expr(&c.callee()?)?),
                args: args?,
            }
        }
        ast::Expr::Unary(u) => ExprKind::Unary {
            op: u.op()?,
            expr: Box::new(expr(&u.operand()?)?),
        },
        ast::Expr::Binary(b) => ExprKind::Binary {
            op: b.op()?,
            lhs: Box::new(expr(&b.lhs()?)?),
            rhs: Box::new(expr(&b.rhs()?)?),
        },
        ast::Expr::If(i) => ExprKind::If {
            cond: Box::new(expr(&i.condition()?)?),
            then: Box::new(expr(&i.then_branch()?)?),
            els: Box::new(expr(&i.else_branch()?)?),
        },
        ast::Expr::Match(m) => {
            let arms: Option<Vec<SurfaceArm>> = m
                .arms()
                .map(|a| {
                    Some(SurfaceArm {
                        pattern: pattern(&a.pattern()?)?,
                        body: expr(&a.body()?)?,
                        span: a.span(),
                    })
                })
                .collect();
            ExprKind::Match {
                scrutinee: Box::new(expr(&m.scrutinee()?)?),
                arms: arms?,
            }
        }
        ast::Expr::Block(b) => {
            let lets: Option<Vec<SurfaceLet>> = b
                .lets()
                .map(|l| {
                    Some(SurfaceLet {
                        pattern: pattern(&l.pattern()?)?,
                        value: expr(&l.value()?)?,
                        span: l.span(),
                    })
                })
                .collect();
            ExprKind::Block {
                lets: lets?,
                tail: Box::new(expr(&b.tail()?)?),
            }
        }
    };
    Some(SurfaceExpr { kind, span })
}
