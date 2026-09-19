//! Syntax and semantic tokens: the one classifier of BDL text, on the
//! Language Server Protocol's semantic-token model
//! (`docs/architecture/syntax-highlighting.md`).
//!
//! ```text
//!   source text ──lexer/parser──▶ lossless CST ──▶ lexical tokens   (always)
//!                                                       │
//!   analysis snapshot ──▶ projection anchors, index ──▶ semantic tokens  (when analysis ran)
//!                                                       │
//!                                          merge: semantic enriches lexical ──▶ sorted, non-overlapping
//!                                                       │
//!                     ┌─────────────────────────────────┴──────────────────┐
//!               structured spans (bdld → Studio)              LSP `data` array ([`encode`])
//! ```
//!
//! Two layers, one vocabulary.  The **lexical** layer reads the tree
//! alone — keywords, numbers, units, operators, comments, slots, binder
//! locals — and is available for any text, however broken, because the
//! parser is total.  The **semantic** layer reads the snapshot's
//! projection anchors and index: an identifier that names an entity gets
//! the entity's class (a concept is a `type`, a Rule a `function`, a
//! Value a `variable`, …) and its modifiers (declaration site, derived
//! role, open state).  Where both speak, the semantic classification
//! wins; where analysis has nothing to say, the lexical class stands.
//!
//! The vocabulary is the LSP's: standard token types wherever an editor
//! already behaves right for them, and exactly two BDL types (`unit`,
//! `slot`) plus a few modifiers where nothing standard fits.  Colours are
//! never here — the legend names classes; every client maps them to its
//! own theme.  Status is never here either: diagnostics have their own
//! ranges, and a modifier says what a symbol *is* (open, a Source), not
//! that something is wrong.
//!
//! Ranges are UTF-8 byte offsets, as everywhere in the service; the LSP
//! line/UTF-16 encoding is a conversion at the edge ([`encode`]).

pub mod encode;

use bdl_elab::names::{InputEnv, Lookup};
use bdl_ide_db::{AnalysisSnapshot, DocumentId, EntityKind, EntityRef, EntityRole, TextRange};
use bdl_model::surface::{Definition, MappingBlock};
use bdl_model::DeclId;
use bdl_syntax::ast::{self, AstNode};
use bdl_syntax::{SyntaxKind, SyntaxNode, SyntaxToken};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The legend's version: bumped when a type or modifier is added,
/// removed or renamed, so a client that caches a legend can tell.
pub const LEGEND_VERSION: u32 = 1;

/// The token types.  Standard LSP names first (the strings the LSP legend
/// carries), then BDL's two extensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenType {
    /// A timing domain: a clock groups the declarations that update
    /// together, the way a namespace groups names.
    Namespace,
    /// A concept — the nominal type of a value — at its declaration and at
    /// every reference (a signature, a body that reads an input by its
    /// concept's name).
    Type,
    /// A component: a declared kind of thing that is instantiated.
    Class,
    /// A relationship with inputs (a Rule), and an equation of the library.
    Function,
    /// A relationship without inputs (a Value or a Source), an output, a
    /// device, an instance: things that hold or provide a value.  Which,
    /// is a modifier.
    Variable,
    /// A local of a body: a rule's parameter, a binder's or lambda's
    /// variable, a `let`, a pattern binding.
    Parameter,
    /// A port of a component's interface, and an export of the system.
    Property,
    /// A constructor / enum variant.
    EnumMember,
    Keyword,
    Number,
    Operator,
    Comment,
    /// BDL: a unit written after a number (`90 deg`).
    Unit,
    /// BDL: the authoring hole `?` — an expression not yet written.
    Slot,
}

impl TokenType {
    /// The LSP legend string.
    pub fn name(self) -> &'static str {
        match self {
            TokenType::Namespace => "namespace",
            TokenType::Type => "type",
            TokenType::Class => "class",
            TokenType::Function => "function",
            TokenType::Variable => "variable",
            TokenType::Parameter => "parameter",
            TokenType::Property => "property",
            TokenType::EnumMember => "enumMember",
            TokenType::Keyword => "keyword",
            TokenType::Number => "number",
            TokenType::Operator => "operator",
            TokenType::Comment => "comment",
            TokenType::Unit => "unit",
            TokenType::Slot => "slot",
        }
    }

    /// Every type, in legend order.
    pub const ALL: [TokenType; 14] = [
        TokenType::Namespace,
        TokenType::Type,
        TokenType::Class,
        TokenType::Function,
        TokenType::Variable,
        TokenType::Parameter,
        TokenType::Property,
        TokenType::EnumMember,
        TokenType::Keyword,
        TokenType::Number,
        TokenType::Operator,
        TokenType::Comment,
        TokenType::Unit,
        TokenType::Slot,
    ];

    /// The index in the legend.
    pub fn index(self) -> u32 {
        TokenType::ALL
            .iter()
            .position(|t| *t == self)
            .map_or(0, |i| i as u32)
    }

    /// Whether the type is one of the LSP's predefined ones.
    pub fn is_standard(self) -> bool {
        !matches!(self, TokenType::Unit | TokenType::Slot)
    }
}

/// Token modifiers.  Standard LSP names first, then BDL's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenModifiers {
    /// The declaration site of the entity (LSP `declaration`).
    pub declaration: bool,
    /// An equation of the library (LSP `defaultLibrary`).
    pub default_library: bool,
    /// BDL: a Source — a value the design only reads, provided by its
    /// environment (ADR-0032).  On a `variable`.
    pub source: bool,
    /// BDL: a physical output.  On a `variable`.
    pub output: bool,
    /// BDL: a device binding.  On a `variable`.
    pub device: bool,
    /// BDL: an instance of a component.  On a `variable`.
    pub instance: bool,
    /// BDL: a legal open state — a relationship with no definition, a
    /// concept with no representation.  A property of the symbol, never
    /// a diagnostic.
    pub unresolved: bool,
}

impl TokenModifiers {
    /// The legend strings, in bit order.
    pub const NAMES: [&'static str; 7] = [
        "declaration",
        "defaultLibrary",
        "source",
        "output",
        "device",
        "instance",
        "unresolved",
    ];

    /// The LSP bitset over [`TokenModifiers::NAMES`].
    pub fn bits(self) -> u32 {
        let flags = [
            self.declaration,
            self.default_library,
            self.source,
            self.output,
            self.device,
            self.instance,
            self.unresolved,
        ];
        flags
            .iter()
            .enumerate()
            .fold(0, |acc, (i, on)| if *on { acc | (1 << i) } else { acc })
    }

    /// The set names, in bit order.
    pub fn names(self) -> Vec<&'static str> {
        let bits = self.bits();
        TokenModifiers::NAMES
            .iter()
            .enumerate()
            .filter(|(i, _)| bits & (1 << i) != 0)
            .map(|(_, n)| *n)
            .collect()
    }

    pub fn declaration() -> TokenModifiers {
        TokenModifiers {
            declaration: true,
            ..TokenModifiers::default()
        }
    }
}

/// The legend: what the type and modifier indices of a token stream mean.
/// Effectively protocol; versioned by [`LEGEND_VERSION`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Legend {
    pub version: u32,
    pub types: Vec<String>,
    pub modifiers: Vec<String>,
}

pub fn legend() -> Legend {
    Legend {
        version: LEGEND_VERSION,
        types: TokenType::ALL.iter().map(|t| t.name().to_owned()).collect(),
        modifiers: TokenModifiers::NAMES
            .iter()
            .map(|m| (*m).to_owned())
            .collect(),
    }
}

/// One classified range.  Ranges of a token stream are sorted, non-empty
/// and pairwise disjoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticToken {
    pub range: TextRange,
    #[serde(rename = "type")]
    pub ty: TokenType,
    pub modifiers: TokenModifiers,
}

impl SemanticToken {
    fn new(range: TextRange, ty: TokenType) -> SemanticToken {
        SemanticToken {
            range,
            ty,
            modifiers: TokenModifiers::default(),
        }
    }
    fn with(mut self, modifiers: TokenModifiers) -> SemanticToken {
        self.modifiers = modifiers;
        self
    }
}

// ---- the lexical layer -------------------------------------------------------

/// The classes the tree alone decides, for any text: keywords (including
/// the contextual binder words), literals, units, operators, comments,
/// slots, constructors, and locals (parameters, binder variables, `let`s)
/// with their uses — resolved by scope in the tree, never by spelling
/// alone.  Identifiers that could name an entity are left out: the
/// semantic layer classifies them, and an unclassified identifier is
/// plain text.
pub fn lexical_tokens(root: &SyntaxNode) -> Vec<SemanticToken> {
    let mut out = Vec::new();
    for el in root.descendants_with_tokens() {
        let Some(t) = el.into_token() else { continue };
        if let Some(tok) = lexical_token(&t) {
            out.push(tok);
        }
    }
    finish(out)
}

fn range_of(t: &SyntaxToken) -> TextRange {
    let r = t.text_range();
    TextRange::new(r.start().into(), r.end().into())
}

fn lexical_token(t: &SyntaxToken) -> Option<SemanticToken> {
    let range = range_of(t);
    let ty = match t.kind() {
        k if k.is_keyword() => TokenType::Keyword,
        SyntaxKind::Number => TokenType::Number,
        SyntaxKind::LineComment | SyntaxKind::BlockComment => TokenType::Comment,
        SyntaxKind::Question => TokenType::Slot,
        SyntaxKind::Ident => return identifier_token(t),
        SyntaxKind::Underscore => TokenType::Parameter,
        // Operators, not punctuation: brackets, commas, colons and
        // semicolons stay plain, as in every editor's default grammar.
        SyntaxKind::Plus
        | SyntaxKind::Minus
        | SyntaxKind::Star
        | SyntaxKind::Slash
        | SyntaxKind::Bang
        | SyntaxKind::AndAnd
        | SyntaxKind::OrOr
        | SyntaxKind::EqEq
        | SyntaxKind::Ne
        | SyntaxKind::Lt
        | SyntaxKind::Le
        | SyntaxKind::Gt
        | SyntaxKind::Ge
        | SyntaxKind::Eq
        | SyntaxKind::Arrow
        | SyntaxKind::FatArrow
        | SyntaxKind::QuestionQuestion
        | SyntaxKind::DotDot
        | SyntaxKind::At
        | SyntaxKind::Dot => TokenType::Operator,
        _ => return None,
    };
    Some(SemanticToken::new(range, ty))
}

/// An identifier the tree can classify by its position: a unit suffix, a
/// binder word, a local's declaration or use, a constructor.
fn identifier_token(t: &SyntaxToken) -> Option<SemanticToken> {
    let range = range_of(t);
    let parent = t.parent()?;
    let grand = parent.parent();
    let grand_kind = grand.as_ref().map(|g| g.kind());
    match parent.kind() {
        SyntaxKind::UnitSuffix => Some(SemanticToken::new(range, TokenType::Unit)),
        // `all x in xs: …`: the word is a construct here (and an ordinary
        // name anywhere else)
        SyntaxKind::BinderExpr => Some(SemanticToken::new(range, TokenType::Keyword)),
        // the local a binder, a lambda, a pattern or a rule declares
        SyntaxKind::Name
            if matches!(
                grand_kind,
                Some(SyntaxKind::BinderExpr | SyntaxKind::IdentPattern | SyntaxKind::LambdaParams)
            ) =>
        {
            Some(
                SemanticToken::new(range, TokenType::Parameter).with(TokenModifiers::declaration()),
            )
        }
        SyntaxKind::NameRef => {
            let expr = grand.and_then(ast::NameExpr::cast);
            if let Some(expr) = expr {
                // a use of a local: a binder's, a lambda's, a `let`'s, a
                // pattern's — or the enclosing rule's parameter
                if expr.local_binding().is_some() || rule_parameter(&expr).is_some() {
                    return Some(SemanticToken::new(range, TokenType::Parameter));
                }
            }
            if grand_kind == Some(SyntaxKind::ConstructorPattern) {
                return Some(SemanticToken::new(range, TokenType::EnumMember));
            }
            None
        }
        _ => None,
    }
}

/// The parameter of the enclosing `f(a, b) = …` a name inside its body
/// refers to, by the definition's own parameter list.
fn rule_parameter(expr: &ast::NameExpr) -> Option<ast::Name> {
    let text = expr.name()?.as_str();
    let def = expr
        .syntax()
        .ancestors()
        .find_map(ast::MappingDef::cast)
        .or_else(|| {
            expr.syntax()
                .ancestors()
                .find_map(ast::PortDecl::cast)
                .and_then(|p| p.definition())
        })?;
    def.params()
        .flat_map(|p| p.syntax().descendants().filter_map(ast::IdentPattern::cast))
        .filter_map(|p| p.name())
        .find(|n| n.as_str() == text)
}

// ---- the semantic layer ------------------------------------------------------

/// The token an entity's name is, at a declaration or a reference.
fn entity_token(
    snapshot: &AnalysisSnapshot,
    entity: EntityRef,
    declaration: bool,
) -> Option<(TokenType, TokenModifiers)> {
    let design = &snapshot.effective().design;
    let mut m = TokenModifiers {
        declaration,
        ..TokenModifiers::default()
    };
    let ty = match entity.kind() {
        EntityKind::Concept => {
            if let EntityRef::Concept(c) = entity {
                m.unresolved = design
                    .concepts
                    .get(&c)
                    .is_some_and(|x| x.representation.is_none());
            }
            TokenType::Type
        }
        EntityKind::Mapping => {
            let EntityRef::Mapping(d) = entity else {
                return None;
            };
            let block = design.mappings.get(&d)?;
            m.unresolved = block.definition.is_none();
            match relationship_role(snapshot, d, block) {
                Role::Rule => TokenType::Function,
                Role::Value => TokenType::Variable,
                Role::Source => {
                    m.source = true;
                    TokenType::Variable
                }
            }
        }
        EntityKind::Clock => TokenType::Namespace,
        EntityKind::Output => {
            m.output = true;
            TokenType::Variable
        }
        EntityKind::Device => {
            m.device = true;
            TokenType::Variable
        }
        EntityKind::Component => TokenType::Class,
        EntityKind::Instance => {
            m.instance = true;
            TokenType::Variable
        }
        EntityKind::Port | EntityKind::Export => TokenType::Property,
        EntityKind::Project | EntityKind::Requirement | EntityKind::Binding => return None,
    };
    Some((ty, m))
}

/// The derived role of a relationship (ADR-0032): Source, Rule or Value,
/// as the committed design says when it has the relationship, else as the
/// effective (drafted) one does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Source,
    Rule,
    Value,
}

fn relationship_role(snapshot: &AnalysisSnapshot, d: DeclId, effective: &MappingBlock) -> Role {
    let block = snapshot
        .committed()
        .design
        .mappings
        .get(&d)
        .unwrap_or(effective);
    if snapshot.port_of(d).is_some() {
        // a body's port-backed declaration: the port provides it — a
        // Source of the body when it has no inputs, a Rule otherwise
        return if block.signature.is_unit_domain() {
            Role::Source
        } else {
            Role::Rule
        };
    }
    if !block.signature.is_unit_domain() {
        Role::Rule
    } else if block.definition.is_some() {
        Role::Value
    } else {
        Role::Source
    }
}

/// The tokens of a document: the semantic layer from the projection map
/// over the lexical layer from the tree.  Sorted, non-overlapping; an
/// unparsable or unbuilt document still gets its lexical tokens.
pub fn semantic_tokens(snapshot: &AnalysisSnapshot, document: DocumentId) -> Vec<SemanticToken> {
    let Some(doc) = snapshot.document(document) else {
        return Vec::new();
    };
    let parse = bdl_syntax::parse_module(&doc.source);
    let root = parse.syntax_node();
    let mut semantic: BTreeMap<TextRange, SemanticToken> = BTreeMap::new();
    for a in snapshot.projections().document_anchors(document) {
        let declaration = match a.role {
            EntityRole::Name => true,
            EntityRole::Reference => false,
            _ => continue,
        };
        let Some(range) = a.text_range() else {
            continue;
        };
        if range.is_empty() || range.end as usize > doc.source.len() {
            continue;
        }
        let Some((ty, modifiers)) = entity_token(snapshot, a.entity, declaration) else {
            continue;
        };
        // several flat entities on one authored range (a body's item, one
        // per instance) agree on the class; the first wins
        semantic.entry(range).or_insert(SemanticToken {
            range,
            ty,
            modifiers,
        });
    }
    // Uses of the library inside bodies: the callee of a call that no
    // entity claimed.
    for callee in equation_callees(&root) {
        semantic.entry(callee).or_insert(SemanticToken {
            range: callee,
            ty: TokenType::Function,
            modifiers: TokenModifiers {
                default_library: true,
                ..TokenModifiers::default()
            },
        });
    }
    // Names in bodies the projection did not anchor — a concept the
    // relationship cannot read, a relationship the loader did not resolve
    // — still name what they name: the class of the entity of that name
    // in the design (one map built per call; no per-token scan).
    let by_name = names_by_spelling(snapshot);
    for node in root.descendants() {
        let Some(name) = ast::NameExpr::cast(node) else {
            continue;
        };
        let Some(r) = name.name() else { continue };
        let range = TextRange::from(r.span());
        if semantic.contains_key(&range)
            || name.local_binding().is_some()
            || rule_parameter(&name).is_some()
        {
            continue;
        }
        let Some(entity) = by_name.get(r.as_str().as_str()) else {
            continue;
        };
        if let Some((ty, m)) = entity_token(snapshot, *entity, false) {
            semantic.insert(
                range,
                SemanticToken {
                    range,
                    ty,
                    modifiers: m,
                },
            );
        }
    }
    merge(semantic.into_values().collect(), lexical_tokens(&root))
}

/// Concepts and relationships of the design by display name (concepts
/// first: a value read by a concept's name is that concept).
fn names_by_spelling(snapshot: &AnalysisSnapshot) -> BTreeMap<&str, EntityRef> {
    let mut out: BTreeMap<&str, EntityRef> = BTreeMap::new();
    for (e, name) in snapshot.index().entities() {
        match e.kind() {
            EntityKind::Mapping => {
                out.entry(name).or_insert(e);
            }
            EntityKind::Concept => {
                out.insert(name, e);
            }
            _ => {}
        }
    }
    out
}

/// The tokens of one formula (a mapping's definition text, as in the
/// Studio definition editor): ranges relative to the formula source.  The
/// semantic layer resolves names the elaborator's way — an input by its
/// concept's name or its parameter name, a relationship, an equation of
/// the library — over the formula's lexical tokens.
pub fn formula_tokens(snapshot: &AnalysisSnapshot, mapping: DeclId) -> Vec<SemanticToken> {
    let design = &snapshot.effective().design;
    let Some(block) = design.mappings.get(&mapping) else {
        return Vec::new();
    };
    let source = match &block.definition {
        Some(Definition::Formula { source }) | Some(Definition::ScopedFormula { source, .. }) => {
            source.as_str()
        }
        _ => return Vec::new(),
    };
    let parse = bdl_syntax::parse_formula(source);
    let root = parse.syntax_node();
    let env = match &block.definition {
        Some(Definition::ScopedFormula { scope, .. }) => {
            InputEnv::scoped(&block.signature.inputs, scope)
        }
        _ => InputEnv::for_mapping(design, block),
    };
    let mut semantic: BTreeMap<TextRange, SemanticToken> = BTreeMap::new();
    for node in root.descendants() {
        let Some(name) = ast::NameExpr::cast(node) else {
            continue;
        };
        let Some(r) = name.name() else { continue };
        if name.local_binding().is_some() {
            continue;
        }
        let range = TextRange::from(r.span());
        let text = r.as_str();
        let tok = match env.resolve(design, &text) {
            Lookup::Input(i) => {
                let by_parameter = block.parameters.get(i).is_some_and(|p| !p.is_empty())
                    && block
                        .signature
                        .inputs
                        .get(i)
                        .and_then(|c| design.concepts.get(c))
                        .is_none_or(|c| c.name != text);
                if by_parameter {
                    SemanticToken::new(range, TokenType::Parameter)
                } else {
                    let Some(c) = block.signature.inputs.get(i) else {
                        continue;
                    };
                    let Some((ty, m)) = entity_token(snapshot, EntityRef::Concept(*c), false)
                    else {
                        continue;
                    };
                    SemanticToken::new(range, ty).with(m)
                }
            }
            Lookup::Mapping(d) => {
                let Some((ty, m)) = entity_token(snapshot, EntityRef::Mapping(d), false) else {
                    continue;
                };
                SemanticToken::new(range, ty).with(m)
            }
            Lookup::NotAnInput(c, _) => {
                // a concept of the design the formula cannot read: still a
                // concept, and hover says so
                let Some((ty, m)) = entity_token(snapshot, EntityRef::Concept(c), false) else {
                    continue;
                };
                SemanticToken::new(range, ty).with(m)
            }
            Lookup::Ambiguous(_) | Lookup::Unknown => continue,
        };
        semantic.insert(range, tok);
    }
    for callee in equation_callees(&root) {
        semantic.entry(callee).or_insert(SemanticToken {
            range: callee,
            ty: TokenType::Function,
            modifiers: TokenModifiers {
                default_library: true,
                ..TokenModifiers::default()
            },
        });
    }
    merge(semantic.into_values().collect(), lexical_tokens(&root))
}

/// The callee ranges of calls to the equation library (`clamp(…)`,
/// `any(xs, …)`).  A relationship of the design with the same name is an
/// entity anchor and takes precedence in the merge.
fn equation_callees(root: &SyntaxNode) -> Vec<TextRange> {
    let mut out = Vec::new();
    for node in root.descendants() {
        let Some(call) = ast::CallExpr::cast(node) else {
            continue;
        };
        let Some(ast::Expr::Name(callee)) = call.callee() else {
            continue;
        };
        let Some(r) = callee.name() else { continue };
        if callee.local_binding().is_some() || rule_parameter(&callee).is_some() {
            continue;
        }
        if bdl_equations::lookup(&r.as_str()).is_some() {
            out.push(TextRange::from(r.span()));
        }
    }
    out
}

// ---- merge ---------------------------------------------------------------------

/// Sort, drop empty ranges, and keep the first of any two that overlap.
fn finish(mut tokens: Vec<SemanticToken>) -> Vec<SemanticToken> {
    tokens.retain(|t| !t.range.is_empty());
    tokens.sort_by_key(|t| (t.range.start, t.range.end));
    let mut out: Vec<SemanticToken> = Vec::with_capacity(tokens.len());
    for t in tokens {
        if out.last().is_some_and(|p| t.range.start < p.range.end) {
            continue;
        }
        out.push(t);
    }
    out
}

/// The merge policy: a semantic token replaces every lexical token it
/// overlaps; lexical tokens fill the rest.  Linear in the two sorted
/// streams.
fn merge(semantic: Vec<SemanticToken>, lexical: Vec<SemanticToken>) -> Vec<SemanticToken> {
    let semantic = finish(semantic);
    let mut out = Vec::with_capacity(semantic.len() + lexical.len());
    let mut s = semantic.iter().peekable();
    for l in lexical {
        // advance past semantic tokens that end before this lexical one
        while s.peek().is_some_and(|x| x.range.end <= l.range.start) {
            if let Some(x) = s.next() {
                out.push(*x);
            }
        }
        match s.peek() {
            Some(x) if x.range.start < l.range.end => {
                // overlap: the semantic token wins; the lexical one is
                // dropped (an identifier is one token in both layers)
            }
            _ => out.push(l),
        }
    }
    out.extend(s.copied());
    finish(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legend_is_stable_and_indices_round_trip() {
        let l = legend();
        assert_eq!(l.version, LEGEND_VERSION);
        assert_eq!(l.types.len(), TokenType::ALL.len());
        for (i, t) in TokenType::ALL.iter().enumerate() {
            assert_eq!(t.index() as usize, i);
            assert_eq!(l.types[i], t.name());
        }
        let m = TokenModifiers {
            declaration: true,
            source: true,
            ..TokenModifiers::default()
        };
        assert_eq!(m.bits(), 0b101);
        assert_eq!(m.names(), vec!["declaration", "source"]);
        assert!(TokenType::Type.is_standard());
        assert!(!TokenType::Unit.is_standard());
        assert!(!TokenType::Slot.is_standard());
    }

    #[test]
    fn merge_keeps_semantic_over_lexical_and_stays_disjoint() {
        let lex = vec![
            SemanticToken::new(TextRange::new(0, 3), TokenType::Keyword),
            SemanticToken::new(TextRange::new(4, 8), TokenType::Number),
            SemanticToken::new(TextRange::new(9, 12), TokenType::Unit),
        ];
        let sem = vec![
            SemanticToken::new(TextRange::new(4, 8), TokenType::Type),
            SemanticToken::new(TextRange::new(14, 16), TokenType::Function),
            SemanticToken::new(TextRange::new(15, 17), TokenType::Variable),
        ];
        let out = merge(sem, lex);
        let kinds: Vec<(u32, u32, TokenType)> = out
            .iter()
            .map(|t| (t.range.start, t.range.end, t.ty))
            .collect();
        assert_eq!(
            kinds,
            vec![
                (0, 3, TokenType::Keyword),
                (4, 8, TokenType::Type),
                (9, 12, TokenType::Unit),
                (14, 16, TokenType::Function),
            ]
        );
        for w in out.windows(2) {
            assert!(w[0].range.end <= w[1].range.start);
        }
    }
}
