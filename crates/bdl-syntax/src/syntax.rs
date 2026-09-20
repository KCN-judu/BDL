//! The lossless tree (Rowan bindings), structured syntax errors, and the
//! [`Parse`] result that carries both.

use crate::kind::{BdlLanguage, SyntaxKind};
use bdl_diagnostics::Span;
use rowan::GreenNode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

pub type SyntaxNode = rowan::SyntaxNode<BdlLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<BdlLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<BdlLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<BdlLanguage>;

/// Byte span of a tree element as the diagnostics crate speaks it.
pub fn span_of(range: rowan::TextRange) -> Span {
    Span::new(range.start().into(), range.end().into())
}

/// Stable machine-readable classes of syntax error
/// (`docs/spec/textual-syntax.md` §9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyntaxErrorCode {
    /// A specific token or category was required here.
    Expected,
    /// Something was found that nothing in the grammar allows here.
    Unexpected,
    ChainedComparison,
    ReservedWord,
    InvalidCharacter,
    UnterminatedComment,
    LiteralPattern,
    Empty,
    /// A unit expression after a number is not well formed: `per` with
    /// nothing after it, a power without a whole-number exponent, a
    /// second `per` (docs/spec/textual-syntax.md §5.2).
    MalformedUnit,
}

impl SyntaxErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            SyntaxErrorCode::Expected => "syntax.expected",
            SyntaxErrorCode::Unexpected => "syntax.unexpected",
            SyntaxErrorCode::ChainedComparison => "syntax.chained_comparison",
            SyntaxErrorCode::ReservedWord => "syntax.reserved_word",
            SyntaxErrorCode::InvalidCharacter => "syntax.invalid_character",
            SyntaxErrorCode::UnterminatedComment => "syntax.unterminated_comment",
            SyntaxErrorCode::LiteralPattern => "syntax.literal_pattern",
            SyntaxErrorCode::Empty => "syntax.empty",
            SyntaxErrorCode::MalformedUnit => "syntax.malformed_unit",
        }
    }
}

/// One syntax diagnostic.  `message` is for the designer; `expected` and
/// `found` are the token-level detail for a technical line; `hint` is a
/// suggested rewrite when one is known.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxError {
    pub code: SyntaxErrorCode,
    pub span: Span,
    pub message: String,
    /// What would have been accepted here, e.g. "`)`", "an expression".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected: Vec<String>,
    /// What was there instead, e.g. "`else`", "the end of the source".
    pub found: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl SyntaxError {
    pub fn new(
        code: SyntaxErrorCode,
        span: Span,
        message: impl Into<String>,
        found: impl Into<String>,
    ) -> SyntaxError {
        SyntaxError {
            code,
            span,
            message: message.into(),
            expected: Vec::new(),
            found: found.into(),
            hint: None,
        }
    }

    #[must_use]
    pub fn expecting(mut self, expected: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.expected = expected.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// The technical line: "expected `)` or `,`, found `else`".
    pub fn technical(&self) -> String {
        if self.expected.is_empty() {
            format!("found {}", self.found)
        } else {
            format!(
                "expected {}, found {}",
                self.expected.join(" or "),
                self.found
            )
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}..{} {}: {}",
            self.span.start,
            self.span.end,
            self.code.as_str(),
            self.message
        )
    }
}

/// The outcome of parsing: always a tree, plus diagnostics.  `T` is the
/// typed root (`ast::Module` or `ast::Formula`).
#[derive(Clone)]
pub struct Parse<T> {
    green: GreenNode,
    errors: Vec<SyntaxError>,
    _root: PhantomData<fn() -> T>,
}

impl<T> Parse<T> {
    pub(crate) fn new(green: GreenNode, errors: Vec<SyntaxError>) -> Parse<T> {
        Parse {
            green,
            errors,
            _root: PhantomData,
        }
    }

    pub fn green(&self) -> &GreenNode {
        &self.green
    }

    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    /// Errors in source order (lexical and syntactic merged, stable).
    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// The tree's text, which is always exactly the parsed source.
    pub fn text(&self) -> String {
        self.syntax_node().text().to_string()
    }
}

impl<T: crate::ast::AstNode> Parse<T> {
    /// The typed root.  Total: the root node always has the root kind.
    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).unwrap_or_else(|| {
            unreachable!("the parser always produces a root of kind {:?}", T::KIND)
        })
    }

    pub fn ok(self) -> Result<T, Vec<SyntaxError>> {
        if self.errors.is_empty() {
            Ok(self.tree())
        } else {
            Err(self.errors)
        }
    }
}

impl<T> fmt::Debug for Parse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.syntax_node())?;
        for e in &self.errors {
            writeln!(f, "error {e}")?;
        }
        Ok(())
    }
}

/// Deterministic debug rendering of a tree: one line per element, kinds
/// and ranges, token texts quoted.  Used by the golden tests.
pub fn debug_tree(node: &SyntaxNode) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for event in node.preorder_with_tokens() {
        match event {
            rowan::WalkEvent::Enter(el) => {
                for _ in 0..depth {
                    out.push_str("  ");
                }
                match el {
                    rowan::NodeOrToken::Node(n) => {
                        out.push_str(&format!("{:?}@{:?}\n", n.kind(), n.text_range()));
                    }
                    rowan::NodeOrToken::Token(t) => {
                        out.push_str(&format!(
                            "{:?}@{:?} {:?}\n",
                            t.kind(),
                            t.text_range(),
                            t.text()
                        ));
                    }
                }
                depth += 1;
            }
            rowan::WalkEvent::Leave(_) => depth -= 1,
        }
    }
    out
}

/// A token's kind-plus-text pair, convenient in tests and the AST.
pub fn token_text(token: &SyntaxToken) -> (SyntaxKind, String) {
    (token.kind(), token.text().to_owned())
}
