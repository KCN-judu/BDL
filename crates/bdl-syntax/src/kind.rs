//! One [`SyntaxKind`] for tokens and nodes, and the Rowan [`Language`]
//! binding.  The kind is the only vocabulary the tree speaks; everything
//! else (`ast`, `lower`) is derived from it.

use rowan::Language;

/// Token and node kinds of the BDL concrete syntax tree
/// (`docs/spec/textual-syntax.md` §2, §3).  Order matters only for the
/// `TRIVIA_END` / `NODE_START` boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    // ---- trivia ----------------------------------------------------------
    Whitespace,
    LineComment,
    BlockComment,

    // ---- tokens ----------------------------------------------------------
    Ident,
    Number,
    Underscore,
    /// A character or character run the lexer has no token for.
    Error,
    /// A synthetic token at the end of input; never stored in the tree.
    Eof,

    // keywords
    KwConcept,
    KwMapping,
    KwEnum,
    KwMatch,
    KwLet,
    KwIf,
    KwThen,
    KwElse,
    KwTrue,
    KwFalse,
    // project items (§14)
    KwClock,
    KwOutput,
    KwDrive,
    KwDevice,
    KwComponent,
    KwInstance,
    KwBind,
    KwExport,
    KwRequires,
    KwProvides,
    KwParam,
    KwUse,
    // future-reserved (§2.1): lexed, no grammar
    KwContext,
    KwRequire,
    // equations (§15): membership, the ordered-concept modifier
    KwIn,
    KwOrdered,

    // punctuation and operators
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Lt,
    Gt,
    Le,
    Ge,
    EqEq,
    Ne,
    Colon,
    Comma,
    Semi,
    Eq,
    Arrow,
    FatArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    AndAnd,
    OrOr,
    At,
    Dot,
    /// `?` — a slot (docs/spec/textual-syntax.md §16).
    Question,
    /// `..` — a closed range (docs/spec/textual-syntax.md §17).
    DotDot,
    /// `??` — a default for an absent value (docs/spec/textual-syntax.md §17).
    QuestionQuestion,

    // ---- nodes -----------------------------------------------------------
    Module,
    /// Root of the expression-only entry point (the canvas formula field).
    Formula,
    ConceptDecl,
    MappingDecl,
    MappingDef,
    EnumDecl,
    EnumVariant,
    ClockDecl,
    ClockTag,
    OutputDecl,
    DriveDecl,
    DeviceDecl,
    DeviceBody,
    PinFix,
    RealizationFix,
    ProviderFix,
    ComponentDecl,
    ComponentBody,
    UseDecl,
    ParamClockDecl,
    PortDecl,
    InstanceDecl,
    InstanceBody,
    InstanceArg,
    BindDecl,
    BindEnd,
    ExportDecl,
    TypeParamList,
    TypeArgList,
    TypeList,
    ParamList,
    ArgList,
    PatternList,
    Name,
    NameRef,
    UnitSuffix,
    NamedType,
    FunctionType,
    ParenType,
    /// `()`: the empty product as a type (docs/spec/textual-syntax.md §4.2).
    UnitType,
    /// `(A, B)`: a product domain, `(A, B) -> C` = `A -> B -> C`.
    TupleType,
    NameExpr,
    LiteralExpr,
    ParenExpr,
    CallExpr,
    UnaryExpr,
    BinaryExpr,
    IfExpr,
    MatchExpr,
    MatchArmList,
    MatchArm,
    BlockExpr,
    LetStmt,
    WildcardPattern,
    IdentPattern,
    LiteralPattern,
    ConstructorPattern,
    /// `[a, b, c]`
    ListExpr,
    /// `(a, b)` — a grouped value
    TupleExpr,
    /// `x => e` or `(x, y) => e` — a rule passed to an equation
    LambdaExpr,
    LambdaParams,
    /// `?` — a slot: an expression not yet written, never elaborated.
    SlotExpr,
    /// `()`: the unique value of the empty product (§4.2).
    UnitExpr,
    /// `all x in xs: body` — a binder form over a collection (§17).
    BinderExpr,
    /// `lo .. hi` — a closed range, meaningful after `in` (§17).
    RangeExpr,
    /// Skipped tokens during recovery, kept so the tree stays lossless.
    ErrorNode,
    /// Placeholder for an abandoned marker; never appears in a tree.
    Tombstone,
}

use SyntaxKind::*;

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Whitespace | LineComment | BlockComment)
    }

    pub fn is_node(self) -> bool {
        self >= Module
    }

    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            KwConcept
                | KwMapping
                | KwEnum
                | KwMatch
                | KwLet
                | KwIf
                | KwThen
                | KwElse
                | KwTrue
                | KwFalse
                | KwClock
                | KwOutput
                | KwDrive
                | KwDevice
                | KwComponent
                | KwInstance
                | KwBind
                | KwExport
                | KwRequires
                | KwProvides
                | KwParam
                | KwUse
                | KwContext
                | KwRequire
                | KwIn
                | KwOrdered
        )
    }

    /// Reserved for a later version: lexed as a keyword, no production.
    pub fn is_future_reserved(self) -> bool {
        matches!(self, KwContext | KwRequire)
    }

    /// The keywords that begin a top-level item (§14.1).
    pub fn is_item_start(self) -> bool {
        matches!(
            self,
            KwConcept
                | KwOrdered
                | KwMapping
                | KwEnum
                | KwClock
                | KwOutput
                | KwDrive
                | KwDevice
                | KwComponent
                | KwInstance
                | KwBind
                | KwExport
        )
    }

    /// The keywords that begin an item inside a component body (§14.2).
    pub fn is_component_item_start(self) -> bool {
        matches!(
            self,
            KwConcept
                | KwOrdered
                | KwMapping
                | KwEnum
                | KwClock
                | KwOutput
                | KwDrive
                | KwDevice
                | KwUse
                | KwParam
                | KwRequires
                | KwProvides
        )
    }

    /// Keyword kind for an identifier spelling, if it is reserved.
    pub fn from_keyword(text: &str) -> Option<SyntaxKind> {
        Some(match text {
            "concept" => KwConcept,
            "mapping" => KwMapping,
            "enum" => KwEnum,
            "match" => KwMatch,
            "let" => KwLet,
            "if" => KwIf,
            "then" => KwThen,
            "else" => KwElse,
            "true" => KwTrue,
            "false" => KwFalse,
            "clock" => KwClock,
            "output" => KwOutput,
            "drive" => KwDrive,
            "device" => KwDevice,
            "component" => KwComponent,
            "instance" => KwInstance,
            "bind" => KwBind,
            "export" => KwExport,
            "requires" => KwRequires,
            "provides" => KwProvides,
            "param" => KwParam,
            "use" => KwUse,
            "context" => KwContext,
            "require" => KwRequire,
            "in" => KwIn,
            "ordered" => KwOrdered,
            _ => return None,
        })
    }

    /// The fixed spelling of a keyword or punctuation token.
    pub fn fixed_text(self) -> Option<&'static str> {
        Some(match self {
            KwConcept => "concept",
            KwMapping => "mapping",
            KwEnum => "enum",
            KwMatch => "match",
            KwLet => "let",
            KwIf => "if",
            KwThen => "then",
            KwElse => "else",
            KwTrue => "true",
            KwFalse => "false",
            KwClock => "clock",
            KwOutput => "output",
            KwDrive => "drive",
            KwDevice => "device",
            KwComponent => "component",
            KwInstance => "instance",
            KwBind => "bind",
            KwExport => "export",
            KwRequires => "requires",
            KwProvides => "provides",
            KwParam => "param",
            KwUse => "use",
            KwContext => "context",
            KwRequire => "require",
            KwIn => "in",
            KwOrdered => "ordered",
            Underscore => "_",
            LParen => "(",
            RParen => ")",
            LBrace => "{",
            RBrace => "}",
            LBracket => "[",
            RBracket => "]",
            Lt => "<",
            Gt => ">",
            Le => "<=",
            Ge => ">=",
            EqEq => "==",
            Ne => "!=",
            Colon => ":",
            Comma => ",",
            Semi => ";",
            Eq => "=",
            Arrow => "->",
            FatArrow => "=>",
            Plus => "+",
            Minus => "-",
            Star => "*",
            Slash => "/",
            Bang => "!",
            AndAnd => "&&",
            OrOr => "||",
            At => "@",
            Dot => ".",
            Question => "?",
            DotDot => "..",
            QuestionQuestion => "??",
            _ => return None,
        })
    }

    /// How a token kind is named in a diagnostic: "`)`", "a name", "a number".
    pub fn describe(self) -> String {
        match self {
            Ident => "a name".into(),
            Number => "a number".into(),
            Error => "an invalid character".into(),
            Eof => "the end of the source".into(),
            Whitespace => "whitespace".into(),
            LineComment | BlockComment => "a comment".into(),
            k => match k.fixed_text() {
                Some(t) => format!("`{t}`"),
                None => format!("{k:?}"),
            },
        }
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind as u16)
    }
}

/// The Rowan language marker for BDL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BdlLanguage {}

impl Language for BdlLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        // SAFETY-free decoding: an out-of-range discriminant (impossible for
        // trees this crate builds) maps to `Tombstone` rather than UB.
        ALL_KINDS
            .get(raw.0 as usize)
            .copied()
            .unwrap_or(SyntaxKind::Tombstone)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        kind.into()
    }
}

/// Every kind in discriminant order; the inverse of `as u16`.
const ALL_KINDS: &[SyntaxKind] = &[
    Whitespace,
    LineComment,
    BlockComment,
    Ident,
    Number,
    Underscore,
    Error,
    Eof,
    KwConcept,
    KwMapping,
    KwEnum,
    KwMatch,
    KwLet,
    KwIf,
    KwThen,
    KwElse,
    KwTrue,
    KwFalse,
    KwClock,
    KwOutput,
    KwDrive,
    KwDevice,
    KwComponent,
    KwInstance,
    KwBind,
    KwExport,
    KwRequires,
    KwProvides,
    KwParam,
    KwUse,
    KwContext,
    KwRequire,
    KwIn,
    KwOrdered,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Lt,
    Gt,
    Le,
    Ge,
    EqEq,
    Ne,
    Colon,
    Comma,
    Semi,
    Eq,
    Arrow,
    FatArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    AndAnd,
    OrOr,
    At,
    Dot,
    Question,
    DotDot,
    QuestionQuestion,
    Module,
    Formula,
    ConceptDecl,
    MappingDecl,
    MappingDef,
    EnumDecl,
    EnumVariant,
    ClockDecl,
    ClockTag,
    OutputDecl,
    DriveDecl,
    DeviceDecl,
    DeviceBody,
    PinFix,
    RealizationFix,
    ProviderFix,
    ComponentDecl,
    ComponentBody,
    UseDecl,
    ParamClockDecl,
    PortDecl,
    InstanceDecl,
    InstanceBody,
    InstanceArg,
    BindDecl,
    BindEnd,
    ExportDecl,
    TypeParamList,
    TypeArgList,
    TypeList,
    ParamList,
    ArgList,
    PatternList,
    Name,
    NameRef,
    UnitSuffix,
    NamedType,
    FunctionType,
    ParenType,
    UnitType,
    TupleType,
    NameExpr,
    LiteralExpr,
    ParenExpr,
    CallExpr,
    UnaryExpr,
    BinaryExpr,
    IfExpr,
    MatchExpr,
    MatchArmList,
    MatchArm,
    BlockExpr,
    LetStmt,
    WildcardPattern,
    IdentPattern,
    LiteralPattern,
    ConstructorPattern,
    ListExpr,
    TupleExpr,
    LambdaExpr,
    LambdaParams,
    SlotExpr,
    UnitExpr,
    BinderExpr,
    RangeExpr,
    ErrorNode,
    Tombstone,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_round_trip_covers_every_kind() {
        for (i, k) in ALL_KINDS.iter().enumerate() {
            assert_eq!(*k as u16 as usize, i, "{k:?} out of order in ALL_KINDS");
            assert_eq!(BdlLanguage::kind_from_raw(rowan::SyntaxKind(i as u16)), *k);
        }
        assert_eq!(ALL_KINDS.len(), Tombstone as usize + 1);
    }

    #[test]
    fn keywords_round_trip() {
        for k in ALL_KINDS.iter().filter(|k| k.is_keyword()) {
            let text = k.fixed_text().expect("keyword has text");
            assert_eq!(SyntaxKind::from_keyword(text), Some(*k));
        }
        assert_eq!(SyntaxKind::from_keyword("tilt"), None);
    }
}
