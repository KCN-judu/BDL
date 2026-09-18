//! The node wrappers, one per `SyntaxKind` node, plus the enums that group
//! them (`Item`, `Type`, `Expr`, `Pattern`).

use super::support::{ast_enum, ast_node, child, children, first_token_of, nth_child, token};
use super::{AstNode, AstToken};
use crate::kind::SyntaxKind;
use crate::syntax::{SyntaxNode, SyntaxToken};

// ---- roots -----------------------------------------------------------------

ast_node!(
    /// A whole source file: `Item*`.
    Module,
    Module
);
ast_node!(
    /// The expression-only entry point (canvas formula field).
    Formula,
    Formula
);

impl Module {
    pub fn items(&self) -> impl Iterator<Item = Item> {
        children(&self.0)
    }
    pub fn concepts(&self) -> impl Iterator<Item = ConceptDecl> {
        children(&self.0)
    }
    pub fn mappings(&self) -> impl Iterator<Item = MappingDecl> {
        children(&self.0)
    }
    pub fn enums(&self) -> impl Iterator<Item = EnumDecl> {
        children(&self.0)
    }
}

impl Formula {
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
}

// ---- names -----------------------------------------------------------------

ast_node!(
    /// A definition-site identifier.
    Name,
    Name
);
ast_node!(
    /// A reference-site identifier.
    NameRef,
    NameRef
);

impl Name {
    pub fn ident(&self) -> Option<SyntaxToken> {
        token(&self.0, SyntaxKind::Ident)
            .or_else(|| self.0.first_token().filter(|t| t.kind().is_keyword()))
    }
    pub fn as_str(&self) -> String {
        self.ident()
            .map(|t| t.text().to_owned())
            .unwrap_or_default()
    }
}

impl NameRef {
    pub fn ident(&self) -> Option<SyntaxToken> {
        token(&self.0, SyntaxKind::Ident)
            .or_else(|| self.0.first_token().filter(|t| t.kind().is_keyword()))
    }
    pub fn as_str(&self) -> String {
        self.ident()
            .map(|t| t.text().to_owned())
            .unwrap_or_default()
    }
}

// ---- items -----------------------------------------------------------------

ast_enum!(
    /// A top-level declaration (§4.1, §14).
    Item {
        Concept(ConceptDecl),
        Mapping(MappingDecl),
        Enum(EnumDecl),
        Clock(ClockDecl),
        Output(OutputDecl),
        Drive(DriveDecl),
        Device(DeviceDecl),
        Component(ComponentDecl),
        Instance(InstanceDecl),
        Bind(BindDecl),
        Export(ExportDecl),
    }
);

ast_enum!(
    /// An item inside a `component { … }` body (§14.2).
    ComponentItem {
        Concept(ConceptDecl),
        Mapping(MappingDecl),
        Enum(EnumDecl),
        Clock(ClockDecl),
        Output(OutputDecl),
        Drive(DriveDecl),
        Device(DeviceDecl),
        Use(UseDecl),
        ParamClock(ParamClockDecl),
        Port(PortDecl),
    }
);

ast_node!(
    /// `concept Name (: Type)?`
    ConceptDecl,
    ConceptDecl
);
ast_node!(
    /// `mapping Name : Type MappingDef?`
    MappingDecl,
    MappingDecl
);
ast_node!(
    /// `name(params) = expr`
    MappingDef,
    MappingDef
);
ast_node!(
    /// `enum Name<T…> { variants }`
    EnumDecl,
    EnumDecl
);
ast_node!(EnumVariant, EnumVariant);
ast_node!(TypeParamList, TypeParamList);
ast_node!(ParamList, ParamList);
ast_node!(
    /// `clock Name`
    ClockDecl,
    ClockDecl
);
ast_node!(
    /// `@ NameRef`
    ClockTag,
    ClockTag
);
ast_node!(
    /// `output Name : Type ClockTag? optional?`
    OutputDecl,
    OutputDecl
);
ast_node!(
    /// `drive NameRef = NameRef`
    DriveDecl,
    DriveDecl
);
ast_node!(
    /// `device Name : kind (for NameRef)? DeviceBody?`
    DeviceDecl,
    DeviceDecl
);
ast_node!(DeviceBody, DeviceBody);
ast_node!(
    /// `pin Number = Ident`
    PinFix,
    PinFix
);
ast_node!(
    /// `component Name { ComponentItem* }`
    ComponentDecl,
    ComponentDecl
);
ast_node!(ComponentBody, ComponentBody);
ast_node!(
    /// `use concept NameRef` / `use output NameRef`
    UseDecl,
    UseDecl
);
ast_node!(
    /// `param clock Name`
    ParamClockDecl,
    ParamClockDecl
);
ast_node!(
    /// `(requires | provides | param) Name : Type ClockTag? MappingDef?`
    PortDecl,
    PortDecl
);
ast_node!(
    /// `instance Name : NameRef InstanceBody?`
    InstanceDecl,
    InstanceDecl
);
ast_node!(InstanceBody, InstanceBody);
ast_node!(
    /// `NameRef = Expr`
    InstanceArg,
    InstanceArg
);
ast_node!(
    /// `bind BindEnd = BindEnd (init Expr)?`
    BindDecl,
    BindDecl
);
ast_node!(
    /// `NameRef (. NameRef)?`
    BindEnd,
    BindEnd
);
ast_node!(
    /// `export BindEnd as Name`
    ExportDecl,
    ExportDecl
);

impl ConceptDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    /// `ordered concept …`: the designer declared the concept ordered.
    pub fn is_ordered(&self) -> bool {
        token(&self.0, SyntaxKind::KwOrdered).is_some()
    }
    /// The representation type after `:`; `None` for an open concept.
    pub fn representation(&self) -> Option<Type> {
        child(&self.0)
    }
}

impl MappingDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    /// The type after `:`.
    pub fn signature(&self) -> Option<Type> {
        child(&self.0)
    }
    /// The `@domain` tag, if written.
    pub fn clock_tag(&self) -> Option<ClockTag> {
        child(&self.0)
    }
    pub fn definition(&self) -> Option<MappingDef> {
        child(&self.0)
    }
}

impl ClockTag {
    pub fn name(&self) -> Option<NameRef> {
        child(&self.0)
    }
}

impl ClockDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
}

impl OutputDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    pub fn accepts(&self) -> Option<Type> {
        child(&self.0)
    }
    pub fn clock_tag(&self) -> Option<ClockTag> {
        child(&self.0)
    }
    /// `true` when the item ends with the contextual word `optional`.
    pub fn is_optional(&self) -> bool {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .any(|t| t.kind() == SyntaxKind::Ident && t.text() == "optional")
    }
}

impl DriveDecl {
    /// The output, then the relationship.
    pub fn output(&self) -> Option<NameRef> {
        nth_child(&self.0, 0)
    }
    pub fn driver(&self) -> Option<NameRef> {
        nth_child(&self.0, 1)
    }
}

impl DeviceDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    /// The kind after `:`, then the output after `for`.
    pub fn kind(&self) -> Option<NameRef> {
        nth_child(&self.0, 0)
    }
    pub fn output(&self) -> Option<NameRef> {
        nth_child(&self.0, 1)
    }
    pub fn pins(&self) -> impl Iterator<Item = PinFix> {
        child::<DeviceBody>(&self.0)
            .into_iter()
            .flat_map(|b| children::<PinFix>(b.syntax()))
    }
}

impl PinFix {
    pub fn index(&self) -> Option<SyntaxToken> {
        token(&self.0, SyntaxKind::Number)
    }
    pub fn pin(&self) -> Option<NameRef> {
        child(&self.0)
    }
}

impl ComponentDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    pub fn body(&self) -> Option<ComponentBody> {
        child(&self.0)
    }
    pub fn items(&self) -> impl Iterator<Item = ComponentItem> {
        self.body()
            .into_iter()
            .flat_map(|b| children::<ComponentItem>(b.syntax()))
    }
}

impl UseDecl {
    /// `true` for `use concept`, `false` for `use output`.
    pub fn is_concept(&self) -> bool {
        token(&self.0, SyntaxKind::KwConcept).is_some()
    }
    pub fn name(&self) -> Option<NameRef> {
        child(&self.0)
    }
}

impl ParamClockDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
}

/// Which port a `PortDecl` declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortWord {
    Requires,
    Provides,
    Param,
}

impl PortDecl {
    /// The keyword the declaration starts with — looked up by kind, because
    /// a comment directly above the port is attached inside the node and
    /// is then its first token.
    pub fn word(&self) -> Option<PortWord> {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find_map(|t| match t.kind() {
                SyntaxKind::KwRequires => Some(PortWord::Requires),
                SyntaxKind::KwProvides => Some(PortWord::Provides),
                SyntaxKind::KwParam => Some(PortWord::Param),
                _ => None,
            })
    }
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    pub fn signature(&self) -> Option<Type> {
        child(&self.0)
    }
    pub fn clock_tag(&self) -> Option<ClockTag> {
        child(&self.0)
    }
    pub fn definition(&self) -> Option<MappingDef> {
        child(&self.0)
    }
}

impl InstanceDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    pub fn component(&self) -> Option<NameRef> {
        child(&self.0)
    }
    pub fn args(&self) -> impl Iterator<Item = InstanceArg> {
        child::<InstanceBody>(&self.0)
            .into_iter()
            .flat_map(|b| children::<InstanceArg>(b.syntax()))
    }
}

impl InstanceArg {
    pub fn name(&self) -> Option<NameRef> {
        child(&self.0)
    }
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl BindEnd {
    /// `instance` of `instance.port`, or the relationship's name.
    pub fn first(&self) -> Option<NameRef> {
        nth_child(&self.0, 0)
    }
    /// `port` of `instance.port`.
    pub fn second(&self) -> Option<NameRef> {
        nth_child(&self.0, 1)
    }
}

impl BindDecl {
    pub fn destination(&self) -> Option<BindEnd> {
        nth_child(&self.0, 0)
    }
    pub fn source(&self) -> Option<BindEnd> {
        nth_child(&self.0, 1)
    }
    /// The expression after `init`, if any.
    pub fn init(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl ExportDecl {
    pub fn port(&self) -> Option<BindEnd> {
        child(&self.0)
    }
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
}

impl MappingDef {
    /// The name repeated before the parameters.
    pub fn name(&self) -> Option<NameRef> {
        child(&self.0)
    }
    pub fn param_list(&self) -> Option<ParamList> {
        child(&self.0)
    }
    pub fn params(&self) -> impl Iterator<Item = Pattern> {
        self.param_list().into_iter().flat_map(|l| l.params())
    }
    pub fn body(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl ParamList {
    pub fn params(&self) -> impl Iterator<Item = Pattern> {
        children(&self.0)
    }
}

impl EnumDecl {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    pub fn type_params(&self) -> impl Iterator<Item = Name> {
        child::<TypeParamList>(&self.0)
            .into_iter()
            .flat_map(|l| children::<Name>(l.syntax()))
    }
    pub fn variants(&self) -> impl Iterator<Item = EnumVariant> {
        children(&self.0)
    }
}

impl EnumVariant {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    /// Field types of `Variant(A, B)`; empty for a nullary variant.
    pub fn fields(&self) -> impl Iterator<Item = Type> {
        child::<TypeList>(&self.0)
            .into_iter()
            .flat_map(|l| l.types())
    }
}

// ---- types -----------------------------------------------------------------

ast_enum!(
    /// A type expression.
    Type {
        Named(NamedType),
        Function(FunctionType),
        Paren(ParenType),
    }
);

ast_node!(
    /// `Name` or `Name<Args>`
    NamedType,
    NamedType
);
ast_node!(
    /// `A -> B` (right-associative)
    FunctionType,
    FunctionType
);
ast_node!(ParenType, ParenType);
ast_node!(TypeArgList, TypeArgList);
ast_node!(TypeList, TypeList);

impl NamedType {
    pub fn name(&self) -> Option<NameRef> {
        child(&self.0)
    }
    pub fn type_args(&self) -> impl Iterator<Item = Type> {
        child::<TypeArgList>(&self.0)
            .into_iter()
            .flat_map(|l| children::<Type>(l.syntax()))
    }
    pub fn has_type_args(&self) -> bool {
        child::<TypeArgList>(&self.0).is_some()
    }
}

impl FunctionType {
    pub fn domain(&self) -> Option<Type> {
        nth_child(&self.0, 0)
    }
    pub fn codomain(&self) -> Option<Type> {
        nth_child(&self.0, 1)
    }
}

impl ParenType {
    pub fn inner(&self) -> Option<Type> {
        child(&self.0)
    }
}

impl TypeList {
    pub fn types(&self) -> impl Iterator<Item = Type> {
        children(&self.0)
    }
}

impl Type {
    /// `A -> B -> C` as `([A, B], C)`; a non-function type is `([], T)`.
    pub fn uncurry(&self) -> (Vec<Type>, Type) {
        let mut inputs = Vec::new();
        let mut current = self.clone();
        while let Type::Function(f) = &current {
            let Some(cod) = f.codomain() else { break };
            if let Some(dom) = f.domain() {
                inputs.push(dom);
            }
            current = cod;
        }
        (inputs, current)
    }
}

// ---- expressions -----------------------------------------------------------

ast_enum!(
    /// An expression.
    Expr {
        Name(NameExpr),
        Literal(LiteralExpr),
        Paren(ParenExpr),
        Call(CallExpr),
        Unary(UnaryExpr),
        Binary(BinaryExpr),
        If(IfExpr),
        Match(MatchExpr),
        Block(BlockExpr),
        List(ListExpr),
        Tuple(TupleExpr),
        Lambda(LambdaExpr),
        Slot(SlotExpr),
        Binder(BinderExpr),
        Range(RangeExpr),
    }
);
ast_node!(
    /// `?` — a slot: an expression not yet written.
    SlotExpr,
    SlotExpr
);
ast_node!(
    /// `all x in xs: body` — a binder over a collection.
    BinderExpr,
    BinderExpr
);
ast_node!(
    /// `lo .. hi` — a closed range.
    RangeExpr,
    RangeExpr
);

impl BinderExpr {
    /// The word: `all`, `any`, `map` or `filter` (an identifier token).
    pub fn word(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == SyntaxKind::Ident)
    }
    pub fn param(&self) -> Option<Name> {
        child(&self.0)
    }
    pub fn collection(&self) -> Option<Expr> {
        nth_child(&self.0, 0)
    }
    pub fn body(&self) -> Option<Expr> {
        nth_child(&self.0, 1)
    }
}

impl RangeExpr {
    pub fn lo(&self) -> Option<Expr> {
        nth_child(&self.0, 0)
    }
    pub fn hi(&self) -> Option<Expr> {
        nth_child(&self.0, 1)
    }
}
ast_node!(
    /// `[a, b, c]`
    ListExpr,
    ListExpr
);
ast_node!(
    /// `(a, b)`
    TupleExpr,
    TupleExpr
);
ast_node!(
    /// `x => e` / `(x, y) => e`
    LambdaExpr,
    LambdaExpr
);
ast_node!(LambdaParams, LambdaParams);

impl ListExpr {
    pub fn items(&self) -> impl Iterator<Item = Expr> {
        children(&self.0)
    }
}

impl TupleExpr {
    pub fn items(&self) -> impl Iterator<Item = Expr> {
        children(&self.0)
    }
}

impl LambdaExpr {
    pub fn params(&self) -> impl Iterator<Item = Name> {
        child::<LambdaParams>(&self.0)
            .into_iter()
            .flat_map(|p| children::<Name>(&p.0))
    }
    pub fn body(&self) -> Option<Expr> {
        child(&self.0)
    }
}

ast_node!(NameExpr, NameExpr);
ast_node!(
    /// A number (with optional unit) or `true`/`false`.
    LiteralExpr,
    LiteralExpr
);
ast_node!(UnitSuffix, UnitSuffix);
ast_node!(ParenExpr, ParenExpr);
ast_node!(
    /// `callee(args)`
    CallExpr,
    CallExpr
);
ast_node!(ArgList, ArgList);
ast_node!(UnaryExpr, UnaryExpr);
ast_node!(BinaryExpr, BinaryExpr);
ast_node!(IfExpr, IfExpr);
ast_node!(MatchExpr, MatchExpr);
ast_node!(MatchArmList, MatchArmList);
ast_node!(MatchArm, MatchArm);
ast_node!(BlockExpr, BlockExpr);
ast_node!(LetStmt, LetStmt);

impl NameExpr {
    pub fn name(&self) -> Option<NameRef> {
        child(&self.0)
    }

    /// The local that binds this name, when one does: the parameter of an
    /// enclosing binder (`all x in xs: … x …`, body only) or rule
    /// (`x => … x …`), a pattern of an enclosing match arm, or a `let`
    /// earlier in an enclosing block.  The nearest wins, so an inner
    /// binder shadows an outer one.  `None` for a name that reaches the
    /// design — an input, a relationship — or nothing.
    pub fn local_binding(&self) -> Option<Name> {
        let text = self.name()?.as_str();
        let here = self.0.text_range();
        let mut node = self.0.parent();
        while let Some(n) = node {
            match n.kind() {
                SyntaxKind::BinderExpr => {
                    let b = BinderExpr::cast(n.clone())?;
                    let in_body = b
                        .body()
                        .is_some_and(|body| body.syntax().text_range().contains_range(here));
                    if in_body {
                        if let Some(p) = b.param().filter(|p| p.as_str() == text) {
                            return Some(p);
                        }
                    }
                }
                SyntaxKind::LambdaExpr => {
                    let l = LambdaExpr::cast(n.clone())?;
                    if let Some(p) = l.params().find(|p| p.as_str() == text) {
                        return Some(p);
                    }
                }
                SyntaxKind::MatchArm => {
                    let a = MatchArm::cast(n.clone())?;
                    let in_body = a
                        .body()
                        .is_some_and(|body| body.syntax().text_range().contains_range(here));
                    if in_body {
                        if let Some(p) = a.pattern().and_then(|p| bound_name(p.syntax(), &text)) {
                            return Some(p);
                        }
                    }
                }
                SyntaxKind::BlockExpr => {
                    let b = BlockExpr::cast(n.clone())?;
                    // the lets before this position, nearest first
                    let earlier: Vec<LetStmt> = b
                        .lets()
                        .filter(|l| l.syntax().text_range().end() <= here.start())
                        .collect();
                    for l in earlier.into_iter().rev() {
                        if let Some(p) = l.pattern().and_then(|p| bound_name(p.syntax(), &text)) {
                            return Some(p);
                        }
                    }
                }
                _ => {}
            }
            node = n.parent();
        }
        None
    }
}

/// The `Name` of an identifier pattern spelled `text` under `pattern`.
fn bound_name(pattern: &SyntaxNode, text: &str) -> Option<Name> {
    pattern
        .descendants()
        .filter_map(IdentPattern::cast)
        .filter_map(|p| p.name())
        .find(|n| n.as_str() == text)
}

/// The spelling of a number literal; never a machine number
/// (`docs/spec/textual-syntax.md` §2.3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NumberToken(SyntaxToken);

impl AstToken for NumberToken {
    fn cast(token: SyntaxToken) -> Option<Self> {
        (token.kind() == SyntaxKind::Number).then_some(NumberToken(token))
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
}

impl NumberToken {
    pub fn literal(&self) -> crate::literal::NumberLiteral {
        crate::literal::NumberLiteral::new(self.text())
    }
}

/// What a `LiteralExpr` is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiteralKind {
    Number(NumberToken),
    Bool(bool),
}

impl LiteralExpr {
    pub fn kind(&self) -> Option<LiteralKind> {
        if let Some(n) = first_token_of::<NumberToken>(&self.0) {
            return Some(LiteralKind::Number(n));
        }
        if token(&self.0, SyntaxKind::KwTrue).is_some() {
            return Some(LiteralKind::Bool(true));
        }
        if token(&self.0, SyntaxKind::KwFalse).is_some() {
            return Some(LiteralKind::Bool(false));
        }
        None
    }
    pub fn number(&self) -> Option<NumberToken> {
        first_token_of(&self.0)
    }
    pub fn unit(&self) -> Option<UnitSuffix> {
        child(&self.0)
    }
}

impl UnitSuffix {
    pub fn ident(&self) -> Option<SyntaxToken> {
        token(&self.0, SyntaxKind::Ident)
    }
    pub fn as_str(&self) -> String {
        self.ident()
            .map(|t| t.text().to_owned())
            .unwrap_or_default()
    }
}

impl ParenExpr {
    pub fn inner(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl CallExpr {
    pub fn callee(&self) -> Option<Expr> {
        child(&self.0)
    }
    pub fn arg_list(&self) -> Option<ArgList> {
        child(&self.0)
    }
    pub fn arguments(&self) -> impl Iterator<Item = Expr> {
        self.arg_list().into_iter().flat_map(|l| l.args())
    }
}

impl ArgList {
    pub fn args(&self) -> impl Iterator<Item = Expr> {
        children(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    /// `x in xs`: membership in a collection.
    In,
    /// `x ?? d`: the value when present, `d` when absent.
    Coalesce,
}

impl BinaryOp {
    pub fn from_kind(kind: SyntaxKind) -> Option<BinaryOp> {
        Some(match kind {
            SyntaxKind::Plus => BinaryOp::Add,
            SyntaxKind::Minus => BinaryOp::Sub,
            SyntaxKind::Star => BinaryOp::Mul,
            SyntaxKind::Slash => BinaryOp::Div,
            SyntaxKind::Lt => BinaryOp::Lt,
            SyntaxKind::Le => BinaryOp::Le,
            SyntaxKind::Gt => BinaryOp::Gt,
            SyntaxKind::Ge => BinaryOp::Ge,
            SyntaxKind::EqEq => BinaryOp::Eq,
            SyntaxKind::Ne => BinaryOp::Ne,
            SyntaxKind::AndAnd => BinaryOp::And,
            SyntaxKind::OrOr => BinaryOp::Or,
            SyntaxKind::KwIn => BinaryOp::In,
            SyntaxKind::QuestionQuestion => BinaryOp::Coalesce,
            _ => return None,
        })
    }

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
            BinaryOp::In => "in",
            BinaryOp::Coalesce => "??",
        }
    }

    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::In
        )
    }
}

impl UnaryExpr {
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| matches!(t.kind(), SyntaxKind::Bang | SyntaxKind::Minus))
    }
    pub fn op(&self) -> Option<UnaryOp> {
        self.op_token().map(|t| match t.kind() {
            SyntaxKind::Bang => UnaryOp::Not,
            _ => UnaryOp::Neg,
        })
    }
    pub fn operand(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl BinaryExpr {
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| BinaryOp::from_kind(t.kind()).is_some())
    }
    pub fn op(&self) -> Option<BinaryOp> {
        self.op_token().and_then(|t| BinaryOp::from_kind(t.kind()))
    }
    pub fn lhs(&self) -> Option<Expr> {
        nth_child(&self.0, 0)
    }
    pub fn rhs(&self) -> Option<Expr> {
        nth_child(&self.0, 1)
    }
}

impl IfExpr {
    /// The first expression after `kw` and before the next keyword, so a
    /// missing branch in a malformed `if` does not shift the others.
    fn section(&self, kw: SyntaxKind) -> Option<Expr> {
        let mut inside = false;
        for el in self.0.children_with_tokens() {
            match el {
                rowan::NodeOrToken::Token(t) if t.kind() == kw => inside = true,
                rowan::NodeOrToken::Token(t)
                    if matches!(t.kind(), SyntaxKind::KwThen | SyntaxKind::KwElse) =>
                {
                    if inside {
                        return None;
                    }
                }
                rowan::NodeOrToken::Node(n) if inside => {
                    if let Some(e) = Expr::cast(n) {
                        return Some(e);
                    }
                }
                _ => {}
            }
        }
        None
    }
    pub fn condition(&self) -> Option<Expr> {
        self.section(SyntaxKind::KwIf)
    }
    pub fn then_branch(&self) -> Option<Expr> {
        self.section(SyntaxKind::KwThen)
    }
    pub fn else_branch(&self) -> Option<Expr> {
        self.section(SyntaxKind::KwElse)
    }
}

impl MatchExpr {
    pub fn scrutinee(&self) -> Option<Expr> {
        child(&self.0)
    }
    pub fn arm_list(&self) -> Option<MatchArmList> {
        child(&self.0)
    }
    pub fn arms(&self) -> impl Iterator<Item = MatchArm> {
        self.arm_list().into_iter().flat_map(|l| l.arms())
    }
}

impl MatchArmList {
    pub fn arms(&self) -> impl Iterator<Item = MatchArm> {
        children(&self.0)
    }
}

impl MatchArm {
    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }
    pub fn body(&self) -> Option<Expr> {
        child(&self.0)
    }
    pub fn has_trailing_comma(&self) -> bool {
        token(&self.0, SyntaxKind::Comma).is_some()
    }
}

impl BlockExpr {
    pub fn lets(&self) -> impl Iterator<Item = LetStmt> {
        children(&self.0)
    }
    /// The final expression, whose value is the block's.
    pub fn tail(&self) -> Option<Expr> {
        children::<Expr>(&self.0).last()
    }
}

impl LetStmt {
    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }
}

// ---- patterns --------------------------------------------------------------

ast_enum!(
    /// A pattern (`docs/spec/textual-syntax.md` §4.4).
    Pattern {
        Wildcard(WildcardPattern),
        Ident(IdentPattern),
        Literal(LiteralPattern),
        Constructor(ConstructorPattern),
    }
);

ast_node!(WildcardPattern, WildcardPattern);
ast_node!(
    /// A bare name: a binding or a nullary constructor, decided by name
    /// resolution (§8.1).
    IdentPattern,
    IdentPattern
);
ast_node!(LiteralPattern, LiteralPattern);
ast_node!(
    /// `Name(patterns)`
    ConstructorPattern,
    ConstructorPattern
);
ast_node!(PatternList, PatternList);

impl IdentPattern {
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
}

/// What a `LiteralPattern` matches.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiteralPatternKind {
    Bool(bool),
    /// `negative` when written with a leading `-`.
    Number {
        negative: bool,
        number: NumberToken,
    },
}

impl LiteralPattern {
    pub fn kind(&self) -> Option<LiteralPatternKind> {
        if token(&self.0, SyntaxKind::KwTrue).is_some() {
            return Some(LiteralPatternKind::Bool(true));
        }
        if token(&self.0, SyntaxKind::KwFalse).is_some() {
            return Some(LiteralPatternKind::Bool(false));
        }
        let number = first_token_of::<NumberToken>(&self.0)?;
        Some(LiteralPatternKind::Number {
            negative: token(&self.0, SyntaxKind::Minus).is_some(),
            number,
        })
    }
}

impl ConstructorPattern {
    pub fn name(&self) -> Option<NameRef> {
        child(&self.0)
    }
    pub fn fields(&self) -> impl Iterator<Item = Pattern> {
        child::<PatternList>(&self.0)
            .into_iter()
            .flat_map(|l| children::<Pattern>(l.syntax()))
    }
}

/// Every descendant node of type `N`, in source order.
pub fn descendants<N: AstNode>(node: &SyntaxNode) -> impl Iterator<Item = N> {
    node.descendants().filter_map(N::cast)
}
