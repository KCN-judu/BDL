//! Semantic tokens for a text document: classification of ranges, for
//! highlighting.  Presentation metadata over the textual projection —
//! nothing here decides meaning, it reads what the projection map and the
//! lossless tree already know.  Status is never encoded here (colour is
//! not a channel for it); it travels as a modifier a client may show by
//! other means.

use bdl_ide_db::{AnalysisSnapshot, DocumentId, EntityKind, EntityRole, TextRange};
use bdl_syntax::ast::AstNode;
use bdl_syntax::SyntaxKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    Concept,
    Mapping,
    Output,
    Clock,
    Device,
    Unit,
    Keyword,
    Parameter,
    Constructor,
    Number,
    Comment,
    Operator,
    /// A name of the equation library applied in a formula (`min`, `any`).
    Equation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenModifiers {
    /// A declaration site rather than a use.
    pub declaration: bool,
    /// The entity is in a legal open state (unresolved, unbound).
    pub unresolved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticToken {
    pub range: TextRange,
    pub kind: TokenKind,
    pub modifiers: TokenModifiers,
}

/// The tokens of a document, in source order, non-overlapping.
pub fn semantic_tokens(snapshot: &AnalysisSnapshot, document: DocumentId) -> Vec<SemanticToken> {
    let Some(doc) = snapshot.document(document) else {
        return Vec::new();
    };
    let design = &snapshot.effective().design;
    let parse = bdl_syntax::parse_module(&doc.source);
    let root = parse.syntax_node();
    let mut out: Vec<SemanticToken> = Vec::new();

    // Entity names from the projection map: the authoritative
    // classification of identifiers.
    for a in snapshot.projections().document_anchors(document) {
        let Some(range) = a.text_range() else {
            continue;
        };
        let kind = match (a.entity.kind(), a.role) {
            (EntityKind::Concept, EntityRole::Name | EntityRole::Reference) => TokenKind::Concept,
            (EntityKind::Mapping, EntityRole::Name | EntityRole::Reference) => TokenKind::Mapping,
            (EntityKind::Output, EntityRole::Name | EntityRole::Reference) => TokenKind::Output,
            (EntityKind::Clock, EntityRole::Name | EntityRole::Reference) => TokenKind::Clock,
            (EntityKind::Device, EntityRole::Name | EntityRole::Reference) => TokenKind::Device,
            _ => continue,
        };
        let unresolved = match a.entity {
            bdl_ide_db::EntityRef::Concept(c) => design
                .concepts
                .get(&c)
                .is_some_and(|x| x.representation.is_none()),
            bdl_ide_db::EntityRef::Mapping(m) => design
                .mappings
                .get(&m)
                .is_some_and(|x| x.definition.is_none()),
            _ => false,
        };
        push(
            &mut out,
            SemanticToken {
                range,
                kind,
                modifiers: TokenModifiers {
                    declaration: a.role == EntityRole::Name,
                    unresolved,
                },
            },
        );
    }

    // Lexical classes from the tree.
    for el in root.descendants_with_tokens() {
        let Some(t) = el.into_token() else { continue };
        let kind = match t.kind() {
            k if k.is_keyword() => TokenKind::Keyword,
            SyntaxKind::Number => TokenKind::Number,
            SyntaxKind::LineComment | SyntaxKind::BlockComment => TokenKind::Comment,
            SyntaxKind::Ident => {
                // A unit suffix is an identifier under `UnitSuffix`; a
                // parameter is an identifier under `IdentPattern` in a
                // parameter list; a constructor under `ConstructorPattern`.
                match t.parent().map(|p| p.kind()) {
                    Some(SyntaxKind::UnitSuffix) => TokenKind::Unit,
                    // `all x in xs: …`: the word is a construct here (and
                    // an ordinary name anywhere else); the local it
                    // declares is a parameter
                    Some(SyntaxKind::BinderExpr) => TokenKind::Keyword,
                    Some(SyntaxKind::Name)
                        if t.parent()
                            .and_then(|p| p.parent())
                            .is_some_and(|g| g.kind() == SyntaxKind::BinderExpr) =>
                    {
                        push(
                            &mut out,
                            SemanticToken {
                                range: TextRange::new(
                                    t.text_range().start().into(),
                                    t.text_range().end().into(),
                                ),
                                kind: TokenKind::Parameter,
                                modifiers: TokenModifiers {
                                    declaration: true,
                                    unresolved: false,
                                },
                            },
                        );
                        continue;
                    }
                    // a use of a local — of a binder, a rule, a pattern
                    Some(SyntaxKind::NameRef)
                        if t.parent()
                            .and_then(|p| p.parent())
                            .and_then(bdl_syntax::ast::NameExpr::cast)
                            .is_some_and(|n| n.local_binding().is_some()) =>
                    {
                        TokenKind::Parameter
                    }
                    Some(SyntaxKind::Name)
                        if t.parent().and_then(|p| p.parent()).is_some_and(|g| {
                            matches!(
                                g.kind(),
                                SyntaxKind::IdentPattern | SyntaxKind::LambdaParams
                            )
                        }) =>
                    {
                        TokenKind::Parameter
                    }
                    // `min(a, b)`, `any(xs, …)`: the callee is an equation of
                    // the library when no relationship of the design shadows it
                    // (an entity anchor was pushed first and wins).
                    Some(SyntaxKind::NameRef)
                        if t.parent()
                            .and_then(|p| p.parent())
                            .and_then(|n| n.parent())
                            .is_some_and(|c| c.kind() == SyntaxKind::CallExpr)
                            && bdl_equations::lookup(t.text()).is_some() =>
                    {
                        TokenKind::Equation
                    }
                    Some(SyntaxKind::NameRef)
                        if t.parent()
                            .and_then(|p| p.parent())
                            .is_some_and(|g| g.kind() == SyntaxKind::ConstructorPattern) =>
                    {
                        TokenKind::Constructor
                    }
                    _ => continue,
                }
            }
            k if k.fixed_text().is_some() => TokenKind::Operator,
            _ => continue,
        };
        let r = t.text_range();
        push(
            &mut out,
            SemanticToken {
                range: TextRange::new(r.start().into(), r.end().into()),
                kind,
                modifiers: TokenModifiers::default(),
            },
        );
    }
    out.sort_by_key(|t| (t.range.start, t.range.end));
    out
}

/// Keep the first classification of a range; entity names win over
/// lexical classes because they are pushed first.
fn push(out: &mut Vec<SemanticToken>, t: SemanticToken) {
    if out
        .iter()
        .any(|x| x.range.start < t.range.end && t.range.start < x.range.end)
    {
        return;
    }
    out.push(t);
}
