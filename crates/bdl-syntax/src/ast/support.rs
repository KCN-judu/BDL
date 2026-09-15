//! Helpers shared by the node wrappers.

use super::{AstNode, AstToken};
use crate::kind::SyntaxKind;
use crate::syntax::{SyntaxNode, SyntaxToken};

pub(super) fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
    parent.children().find_map(N::cast)
}

pub(super) fn nth_child<N: AstNode>(parent: &SyntaxNode, n: usize) -> Option<N> {
    parent.children().filter_map(N::cast).nth(n)
}

pub(super) fn children<N: AstNode>(parent: &SyntaxNode) -> impl Iterator<Item = N> {
    parent.children().filter_map(N::cast)
}

pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .find(|t| t.kind() == kind)
}

pub(super) fn first_token_of<T: AstToken>(parent: &SyntaxNode) -> Option<T> {
    parent
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .find_map(T::cast)
}

/// Define a struct wrapping one node kind.
macro_rules! ast_node {
    ($(#[$doc:meta])* $name:ident, $kind:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            const KIND: SyntaxKind = SyntaxKind::$kind;
            fn cast(node: SyntaxNode) -> Option<Self> {
                if node.kind() == SyntaxKind::$kind {
                    Some($name(node))
                } else {
                    None
                }
            }
            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

/// Define an enum over several node wrappers.
macro_rules! ast_enum {
    ($(#[$doc:meta])* $name:ident { $($variant:ident($ty:ident)),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant($ty)),+
        }

        impl AstNode for $name {
            const KIND: SyntaxKind = SyntaxKind::Tombstone;
            fn can_cast(kind: SyntaxKind) -> bool {
                $(<$ty as AstNode>::can_cast(kind))||+
            }
            fn cast(node: SyntaxNode) -> Option<Self> {
                $(if let Some(n) = $ty::cast(node.clone()) {
                    return Some($name::$variant(n));
                })+
                None
            }
            fn syntax(&self) -> &SyntaxNode {
                match self {
                    $($name::$variant(n) => n.syntax()),+
                }
            }
        }

        $(impl From<$ty> for $name {
            fn from(n: $ty) -> $name {
                $name::$variant(n)
            }
        })+
    };
}

pub(super) use {ast_enum, ast_node};
