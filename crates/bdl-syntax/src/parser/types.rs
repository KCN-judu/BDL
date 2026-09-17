//! Types (`docs/spec/textual-syntax.md` §4.2).
//!
//! ```text
//! Type      ::= TypeAtom ("->" Type)?          -- right-associative
//! TypeAtom  ::= NamedType | ParenType
//! NamedType ::= NameRef TypeArgList?
//! ```

use super::{grammar, Parser};
use crate::kind::SyntaxKind::{self, *};
use crate::syntax::SyntaxErrorCode;

/// Parse a type; `false` (with an error already reported by the caller's
/// choice of message) when nothing type-like starts here.
pub(super) fn type_(p: &mut Parser<'_>) -> bool {
    let Some(lhs) = type_atom(p) else {
        return false;
    };
    if p.at(Arrow) {
        let m = lhs.precede(p);
        p.bump(); // ->
        if !type_(p) {
            p.error_expecting("expected a type after `->`", &[Ident, LParen]);
        }
        m.complete(p, FunctionType);
    }
    true
}

fn type_atom(p: &mut Parser<'_>) -> Option<super::CompletedMarker> {
    match p.current() {
        Ident => {
            let m = p.start();
            grammar::name_ref(p);
            if p.at(Lt) {
                type_list(
                    p,
                    TypeArgList,
                    Lt,
                    Gt,
                    "expected `>` to close the type arguments",
                );
            }
            Some(m.complete(p, NamedType))
        }
        LParen => {
            let m = p.start();
            p.bump();
            if !type_(p) {
                p.error_expecting("expected a type inside the parentheses", &[Ident, LParen]);
            }
            p.expect(RParen, "expected `)` to close the parenthesised type");
            Some(m.complete(p, ParenType))
        }
        k if k.is_future_reserved() => {
            grammar::reserved_word(p);
            let m = p.start();
            grammar::name_ref(p);
            Some(m.complete(p, NamedType))
        }
        _ => None,
    }
}

/// `open Type ("," Type)* ","? close` as a node of kind `list_kind`.
/// Used for `<…>` type arguments and `(…)` variant fields.
pub(super) fn type_list(
    p: &mut Parser<'_>,
    list_kind: SyntaxKind,
    open: SyntaxKind,
    close: SyntaxKind,
    close_message: &str,
) {
    let m = p.start();
    p.bump(); // open
    debug_assert!(matches!(open, Lt | LParen));
    loop {
        if p.at(close) || p.at_eof() {
            break;
        }
        if !type_(p) {
            if p.at(Comma) {
                p.error_and_bump(SyntaxErrorCode::Expected, "expected a type before `,`");
                continue;
            }
            p.error_expecting(close_message, &[close, Comma]);
            m.complete(p, list_kind);
            return;
        }
        if p.at(close) {
            break;
        }
        if !p.eat(Comma) {
            // `Ident (` is the start of a mapping definition, never a type:
            // the list was left unclosed.
            if (p.at(Ident) && p.nth(1) != LParen) || p.at(LParen) {
                p.error_expecting("expected `,` between types", &[Comma]);
                continue;
            }
            p.error_expecting(close_message, &[close, Comma]);
            m.complete(p, list_kind);
            return;
        }
    }
    p.expect(close, close_message);
    m.complete(p, list_kind);
}
