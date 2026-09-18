//! Lowering: typed syntax AST → the *surface tree*, a plain, id-free data
//! structure of what was written (names, spans, exact literals).  This is
//! what `bdl-elab` consumes for formulas today and what a textual project
//! loader will turn into `bdl-model` edits.  Nothing semantic happens here:
//! no name resolution, no ids, no types (`docs/spec/textual-syntax.md` §11).
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
    /// `[a, b, c]` — a collection literal.
    List(Vec<SurfaceExpr>),
    /// `(a, b)` — a grouped value; three or more parts nest to the right.
    Tuple(Vec<SurfaceExpr>),
    /// `x => e` — a rule given to an equation; never a value on its own.
    Lambda {
        params: Vec<Ident>,
        body: Box<SurfaceExpr>,
    },
    /// `?` — a slot: an expression not yet written.  The Formula
    /// Composer's hole; elaboration refuses it (`formula.slot.empty`) and
    /// nothing downstream of the surface ever sees one.
    Hole,
    /// `()`: the unique value of the empty product — the argument of a
    /// relationship without inputs (`f(())`, for which `f` is the sugar).
    Unit,
    /// `all x in xs: body` — the binder family: sugar for the equation
    /// `all(xs, x => body)` (likewise `any`, `map`, `filter`); the binder is
    /// the rule's parameter, scoped to the body.
    Binder {
        form: BinderForm,
        param: Ident,
        collection: Box<SurfaceExpr>,
        body: Box<SurfaceExpr>,
    },
    /// `lo .. hi` — a closed range; `x in lo .. hi` is `inRange(x, lo, hi)`.
    Range {
        lo: Box<SurfaceExpr>,
        hi: Box<SurfaceExpr>,
    },
}

/// The four binder words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinderForm {
    All,
    Any,
    Map,
    Filter,
}

impl BinderForm {
    pub fn word(self) -> &'static str {
        match self {
            BinderForm::All => "all",
            BinderForm::Any => "any",
            BinderForm::Map => "map",
            BinderForm::Filter => "filter",
        }
    }
    pub fn from_word(w: &str) -> Option<BinderForm> {
        Some(match w {
            "all" => BinderForm::All,
            "any" => BinderForm::Any,
            "map" => BinderForm::Map,
            "filter" => BinderForm::Filter,
            _ => return None,
        })
    }
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
            ExprKind::Name(_)
            | ExprKind::Number { .. }
            | ExprKind::Bool(_)
            | ExprKind::Hole
            | ExprKind::Unit => {}
            ExprKind::Binder {
                collection, body, ..
            } => {
                collection.walk(f);
                body.walk(f);
            }
            ExprKind::Range { lo, hi } => {
                lo.walk(f);
                hi.walk(f);
            }
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
            ExprKind::List(items) | ExprKind::Tuple(items) => items.iter().for_each(|i| i.walk(f)),
            ExprKind::Lambda { body, .. } => body.walk(f),
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
    /// `()`: the empty product, the domain of a relationship without
    /// inputs (`docs/spec/textual-syntax.md` §4.2).
    Unit,
    /// `(A, B)`: a product, spelled as a relationship's domain.
    Tuple(Vec<SurfaceType>),
}

impl SurfaceType {
    /// The inputs and output a signature spells, as the one canonical
    /// type `domain(inputs) -> B`: `A -> B -> C` and `(A, B) -> C` are
    /// `([A, B], C)`; `() -> B` and a bare `B` are `([], B)` — `mapping f :
    /// B` is shorthand for `mapping f : () -> B`.  A `()` after an input,
    /// or a product after one, stays an input for the caller to refuse.
    pub fn uncurry(&self) -> (Vec<&SurfaceType>, &SurfaceType) {
        let mut inputs: Vec<&SurfaceType> = Vec::new();
        let mut t = self;
        while let TypeKind::Function { domain, codomain } = &t.kind {
            match &domain.kind {
                TypeKind::Unit if inputs.is_empty() => {}
                TypeKind::Tuple(parts) if inputs.is_empty() => inputs.extend(parts.iter()),
                _ => inputs.push(domain.as_ref()),
            }
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
    Clock(ClockItem),
    Output(OutputItem),
    Drive(DriveItem),
    Device(DeviceItem),
    Component(ComponentItem),
    Instance(InstanceItem),
    Bind(BindItem),
    Export(ExportItem),
}

impl SurfaceItem {
    pub fn span(&self) -> Span {
        match self {
            SurfaceItem::Concept(i) => i.span,
            SurfaceItem::Mapping(i) => i.span,
            SurfaceItem::Enum(i) => i.span,
            SurfaceItem::Clock(i) => i.span,
            SurfaceItem::Output(i) => i.span,
            SurfaceItem::Drive(i) => i.span,
            SurfaceItem::Device(i) => i.span,
            SurfaceItem::Component(i) => i.span,
            SurfaceItem::Instance(i) => i.span,
            SurfaceItem::Bind(i) => i.span,
            SurfaceItem::Export(i) => i.span,
        }
    }
}

/// `clock Name`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockItem {
    pub name: Ident,
    pub span: Span,
}

/// `output Name : Type @clock? optional?` — a physical output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputItem {
    pub name: Ident,
    pub accepts: SurfaceType,
    pub clock: Option<Ident>,
    pub required: bool,
    pub span: Span,
}

/// `drive output = relationship`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveItem {
    pub output: Ident,
    pub driver: Ident,
    pub span: Span,
}

/// `device Name : kind for output { pin i = name }`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceItem {
    pub name: Ident,
    pub kind: Ident,
    pub output: Option<Ident>,
    /// `(requirement index, pin name)` as written.
    pub pins: Vec<(u16, Ident)>,
    pub span: Span,
}

/// `component Name { items }`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentItem {
    pub name: Ident,
    pub items: Vec<ComponentBodyItem>,
    /// The span of the `{ … }` body, for splicing new items into it.
    pub body_span: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ComponentBodyItem {
    Concept(ConceptItem),
    Mapping(MappingItem),
    Enum(EnumItem),
    Clock(ClockItem),
    Output(OutputItem),
    Drive(DriveItem),
    Device(DeviceItem),
    Use(UseItem),
    ParamClock(ClockItem),
    Port(PortItem),
}

impl ComponentBodyItem {
    pub fn span(&self) -> Span {
        match self {
            ComponentBodyItem::Concept(i) => i.span,
            ComponentBodyItem::Mapping(i) => i.span,
            ComponentBodyItem::Enum(i) => i.span,
            ComponentBodyItem::Clock(i) => i.span,
            ComponentBodyItem::Output(i) => i.span,
            ComponentBodyItem::Drive(i) => i.span,
            ComponentBodyItem::Device(i) => i.span,
            ComponentBodyItem::Use(i) => i.span,
            ComponentBodyItem::ParamClock(i) => i.span,
            ComponentBodyItem::Port(i) => i.span,
        }
    }
}

/// `use concept Name` / `use output Name`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseItem {
    pub concept: bool,
    pub name: Ident,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortWord {
    Requires,
    Provides,
    Param,
}

/// `requires|provides|param Name : Type @clock? definition?`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PortItem {
    pub word: PortWord,
    pub name: Ident,
    pub signature: SurfaceType,
    pub clock: Option<Ident>,
    pub definition: Option<MappingDefinition>,
    pub span: Span,
}

/// `instance Name : Component { arg = value }`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstanceItem {
    pub name: Ident,
    pub component: Ident,
    pub args: Vec<InstanceArgItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstanceArgItem {
    pub name: Ident,
    pub value: SurfaceExpr,
    /// The value's source text, kept for parameter values (which the
    /// model stores as text) and for clock arguments (a bare name).
    pub text: String,
    pub span: Span,
}

/// One end of a binding: `instance.port` or a top-level relationship.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindEndItem {
    pub first: Ident,
    pub second: Option<Ident>,
    pub span: Span,
}

/// `bind destination = source (init e)?`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BindItem {
    pub destination: BindEndItem,
    pub source: BindEndItem,
    /// The initial value's source text when the binding is transported.
    pub init: Option<(SurfaceExpr, String)>,
    pub span: Span,
}

/// `export instance.port as Name`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportItem {
    pub port: BindEndItem,
    pub name: Ident,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptItem {
    pub name: Ident,
    /// `None` for an open concept (`concept Tilt`).
    pub representation: Option<SurfaceType>,
    /// `ordered concept …`.
    pub ordered: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MappingItem {
    pub name: Ident,
    /// The type as written, e.g. `Held -> Tilt -> Brightness`.
    pub signature: SurfaceType,
    /// The `@domain` tag, if written.
    pub clock: Option<Ident>,
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
    let mut cx = Lowering {
        syntax_errors: &syntax_errors,
        errors: &mut errors,
    };
    for item in parse.tree().items() {
        if let Some(i) = cx.item(&item) {
            items.push(i);
        }
    }
    errors.sort_by_key(|e| (e.span.start, e.span.end));
    (SurfaceModule { items }, errors)
}

struct Lowering<'a> {
    syntax_errors: &'a [SyntaxError],
    errors: &'a mut Vec<SyntaxError>,
}

impl Lowering<'_> {
    /// Structural checks are only meaningful on an item that parsed
    /// cleanly; a syntax error inside it already explains any mismatch.
    fn clean(&self, span: Span) -> bool {
        !self
            .syntax_errors
            .iter()
            .any(|e| e.span.start < span.end && span.start < e.span.end.max(e.span.start + 1))
    }

    fn item(&mut self, item: &ast::Item) -> Option<SurfaceItem> {
        Some(match item {
            ast::Item::Concept(c) => SurfaceItem::Concept(concept(c)?),
            ast::Item::Mapping(m) => {
                let clean = self.clean(m.span());
                SurfaceItem::Mapping(mapping(m, clean, self.errors)?)
            }
            ast::Item::Enum(e) => SurfaceItem::Enum(enum_(e)?),
            ast::Item::Clock(c) => SurfaceItem::Clock(clock(c)?),
            ast::Item::Output(o) => SurfaceItem::Output(output(o)?),
            ast::Item::Drive(d) => SurfaceItem::Drive(drive(d)?),
            ast::Item::Device(d) => SurfaceItem::Device(device(d, self.errors)?),
            ast::Item::Component(c) => SurfaceItem::Component(self.component(c)?),
            ast::Item::Instance(i) => SurfaceItem::Instance(instance(i)?),
            ast::Item::Bind(b) => SurfaceItem::Bind(bind(b)?),
            ast::Item::Export(e) => SurfaceItem::Export(export(e)?),
        })
    }

    fn component(&mut self, c: &ast::ComponentDecl) -> Option<ComponentItem> {
        let name = ident(&c.name()?)?;
        let body = c.body()?;
        let mut items = Vec::new();
        // An item that does not lower is left out; the rest of the body
        // and the component itself stand.
        for item in c.items() {
            if let Some(lowered) = self.component_item(&item) {
                items.push(lowered);
            }
        }
        Some(ComponentItem {
            name,
            items,
            body_span: body.span(),
            span: c.span(),
        })
    }

    fn component_item(&mut self, item: &ast::ComponentItem) -> Option<ComponentBodyItem> {
        Some(match item {
            ast::ComponentItem::Concept(x) => ComponentBodyItem::Concept(concept(x)?),
            ast::ComponentItem::Mapping(m) => {
                let clean = self.clean(m.span());
                ComponentBodyItem::Mapping(mapping(m, clean, self.errors)?)
            }
            ast::ComponentItem::Enum(e) => ComponentBodyItem::Enum(enum_(e)?),
            ast::ComponentItem::Clock(x) => ComponentBodyItem::Clock(clock(x)?),
            ast::ComponentItem::Output(o) => ComponentBodyItem::Output(output(o)?),
            ast::ComponentItem::Drive(d) => ComponentBodyItem::Drive(drive(d)?),
            ast::ComponentItem::Device(d) => ComponentBodyItem::Device(device(d, self.errors)?),
            ast::ComponentItem::Use(u) => ComponentBodyItem::Use(UseItem {
                concept: u.is_concept(),
                name: ident_ref(&u.name()?)?,
                span: u.span(),
            }),
            ast::ComponentItem::ParamClock(p) => ComponentBodyItem::ParamClock(ClockItem {
                name: ident(&p.name()?)?,
                span: p.span(),
            }),
            ast::ComponentItem::Port(p) => {
                let clean = self.clean(p.span());
                ComponentBodyItem::Port(port(p, clean, self.errors)?)
            }
        })
    }
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
        ordered: c.is_ordered(),
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
    // A definition that does not lower (a body still being typed) leaves
    // the declaration standing: the syntax error already says why.
    let definition = m
        .definition()
        .and_then(|def| definition(&def, &name, &signature, clean, errors));
    Some(MappingItem {
        name,
        signature,
        clock: m.clock_tag().and_then(|t| ident_ref(&t.name()?)),
        definition,
        span: m.span(),
    })
}

/// The definition part shared by mappings and ports: name check, arity
/// check, params and body.
fn definition(
    def: &ast::MappingDef,
    name: &Ident,
    signature: &SurfaceType,
    clean: bool,
    errors: &mut Vec<SyntaxError>,
) -> Option<MappingDefinition> {
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

fn clock(c: &ast::ClockDecl) -> Option<ClockItem> {
    Some(ClockItem {
        name: ident(&c.name()?)?,
        span: c.span(),
    })
}

fn output(o: &ast::OutputDecl) -> Option<OutputItem> {
    Some(OutputItem {
        name: ident(&o.name()?)?,
        accepts: type_(&o.accepts()?)?,
        clock: o.clock_tag().and_then(|t| ident_ref(&t.name()?)),
        required: !o.is_optional(),
        span: o.span(),
    })
}

fn drive(d: &ast::DriveDecl) -> Option<DriveItem> {
    Some(DriveItem {
        output: ident_ref(&d.output()?)?,
        driver: ident_ref(&d.driver()?)?,
        span: d.span(),
    })
}

fn device(d: &ast::DeviceDecl, errors: &mut Vec<SyntaxError>) -> Option<DeviceItem> {
    let name = ident(&d.name()?)?;
    let kind = ident_ref(&d.kind()?)?;
    let output = match d.output() {
        Some(o) => Some(ident_ref(&o)?),
        None => None,
    };
    let mut pins = Vec::new();
    for pin in d.pins() {
        let index_tok = pin.index()?;
        let index: u16 = match index_tok.text().parse() {
            Ok(i) => i,
            Err(_) => {
                errors.push(SyntaxError::new(
                    SyntaxErrorCode::Unexpected,
                    crate::syntax::span_of(index_tok.text_range()),
                    "a pin index is a small whole number",
                    format!("`{}`", index_tok.text()),
                ));
                return None;
            }
        };
        pins.push((index, ident_ref(&pin.pin()?)?));
    }
    Some(DeviceItem {
        name,
        kind,
        output,
        pins,
        span: d.span(),
    })
}

fn port(p: &ast::PortDecl, clean: bool, errors: &mut Vec<SyntaxError>) -> Option<PortItem> {
    let word = match p.word()? {
        ast::PortWord::Requires => PortWord::Requires,
        ast::PortWord::Provides => PortWord::Provides,
        ast::PortWord::Param => PortWord::Param,
    };
    let name = ident(&p.name()?)?;
    let signature = type_(&p.signature()?)?;
    let def = p
        .definition()
        .and_then(|d| definition(&d, &name, &signature, clean, errors));
    Some(PortItem {
        word,
        name,
        signature,
        clock: p.clock_tag().and_then(|t| ident_ref(&t.name()?)),
        definition: def,
        span: p.span(),
    })
}

fn instance(i: &ast::InstanceDecl) -> Option<InstanceItem> {
    let mut args = Vec::new();
    for a in i.args() {
        let value_node = a.value()?;
        args.push(InstanceArgItem {
            name: ident_ref(&a.name()?)?,
            text: value_node.text().trim().to_owned(),
            value: expr(&value_node)?,
            span: a.span(),
        });
    }
    Some(InstanceItem {
        name: ident(&i.name()?)?,
        component: ident_ref(&i.component()?)?,
        args,
        span: i.span(),
    })
}

fn bind_end(e: &ast::BindEnd) -> Option<BindEndItem> {
    Some(BindEndItem {
        first: ident_ref(&e.first()?)?,
        second: match e.second() {
            Some(s) => Some(ident_ref(&s)?),
            None => None,
        },
        span: e.span(),
    })
}

fn bind(b: &ast::BindDecl) -> Option<BindItem> {
    let init = match b.init() {
        Some(e) => Some((expr(&e)?, e.text().trim().to_owned())),
        None => None,
    };
    Some(BindItem {
        destination: bind_end(&b.destination()?)?,
        source: bind_end(&b.source()?)?,
        init,
        span: b.span(),
    })
}

fn export(e: &ast::ExportDecl) -> Option<ExportItem> {
    Some(ExportItem {
        port: bind_end(&e.port()?)?,
        name: ident(&e.name()?)?,
        span: e.span(),
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
        ast::Type::Unit(_) => SurfaceType {
            kind: TypeKind::Unit,
            span,
        },
        ast::Type::Tuple(t) => SurfaceType {
            kind: TypeKind::Tuple(t.parts().map(|p| type_(&p)).collect::<Option<Vec<_>>>()?),
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
        ast::Expr::Slot(_) => ExprKind::Hole,
        ast::Expr::Unit(_) => ExprKind::Unit,
        ast::Expr::Binder(b) => ExprKind::Binder {
            form: BinderForm::from_word(b.word()?.text())?,
            param: ident(&b.param()?)?,
            collection: Box::new(expr(&b.collection()?)?),
            body: Box::new(expr(&b.body()?)?),
        },
        ast::Expr::Range(r) => ExprKind::Range {
            lo: Box::new(expr(&r.lo()?)?),
            hi: Box::new(expr(&r.hi()?)?),
        },
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
        ast::Expr::List(l) => {
            let items: Option<Vec<SurfaceExpr>> = l.items().map(|i| expr(&i)).collect();
            ExprKind::List(items?)
        }
        ast::Expr::Tuple(t) => {
            let items: Option<Vec<SurfaceExpr>> = t.items().map(|i| expr(&i)).collect();
            ExprKind::Tuple(items?)
        }
        ast::Expr::Lambda(l) => {
            let params: Option<Vec<Ident>> = l.params().map(|n| ident(&n)).collect();
            ExprKind::Lambda {
                params: params?,
                body: Box::new(expr(&l.body()?)?),
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
