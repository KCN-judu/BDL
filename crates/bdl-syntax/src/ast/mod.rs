//! Typed views over the lossless tree.  A wrapper owns nothing but its
//! [`SyntaxNode`]; every accessor walks the tree on demand, so the CST is
//! the single source of truth for syntax structure.

mod nodes;
mod support;

pub use nodes::*;

use crate::kind::SyntaxKind;
use crate::syntax::{SyntaxNode, SyntaxToken};

/// A node wrapper.  `cast` is the only way to obtain one.
pub trait AstNode: Sized {
    const KIND: SyntaxKind;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == Self::KIND
    }

    fn cast(node: SyntaxNode) -> Option<Self>;

    fn syntax(&self) -> &SyntaxNode;

    /// Byte span of the node (leading/trailing trivia excluded by
    /// construction: trivia sits between nodes, except attached comments,
    /// see `docs/TEXTUAL_SYNTAX.md` §2.4).
    fn span(&self) -> bdl_diagnostics::Span {
        crate::syntax::span_of(self.syntax().text_range())
    }

    /// The source text this node covers.
    fn text(&self) -> String {
        self.syntax().text().to_string()
    }
}

/// A token wrapper, for the few tokens the typed API hands out directly.
pub trait AstToken: Sized {
    fn cast(token: SyntaxToken) -> Option<Self>;
    fn syntax(&self) -> &SyntaxToken;
    fn text(&self) -> &str {
        self.syntax().text()
    }
    fn span(&self) -> bdl_diagnostics::Span {
        crate::syntax::span_of(self.syntax().text_range())
    }
}
