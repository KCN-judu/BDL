//! Patterns (`docs/TEXTUAL_SYNTAX.md` §4.4).
//!
//! ```text
//! Pattern            ::= "_" | Name | LiteralPattern | ConstructorPattern
//! LiteralPattern     ::= "true" | "false" | "-"? Number
//! ConstructorPattern ::= NameRef "(" PatternList? ")"
//! ```
//! A bare identifier is always an `IdentPattern`; name resolution decides
//! whether it is a binding or a nullary constructor (§8.1).

use super::{grammar, Parser};
use crate::kind::SyntaxKind::{self, *};
use crate::syntax::SyntaxErrorCode;

pub(super) fn can_start(kind: SyntaxKind) -> bool {
    matches!(kind, Underscore | Ident | KwTrue | KwFalse | Number | Minus)
        || kind.is_future_reserved()
}

/// Parse a pattern; `false` when nothing pattern-like starts here (no
/// error is reported, the caller knows what it expected).
pub(super) fn pattern(p: &mut Parser<'_>) -> bool {
    match p.current() {
        Underscore => {
            let m = p.start();
            p.bump();
            m.complete(p, WildcardPattern);
        }
        Ident => {
            if p.nth(1) == LParen {
                let m = p.start();
                grammar::name_ref(p);
                pattern_list(p);
                m.complete(p, ConstructorPattern);
            } else {
                let m = p.start();
                grammar::name(p, "expected a name");
                m.complete(p, IdentPattern);
            }
        }
        k if k.is_future_reserved() => {
            let m = p.start();
            grammar::name(p, "expected a name");
            m.complete(p, IdentPattern);
        }
        KwTrue | KwFalse => {
            let m = p.start();
            p.bump();
            m.complete(p, LiteralPattern);
        }
        Number | Minus => {
            let m = p.start();
            if p.eat(Minus) && !p.at(Number) {
                p.error_expecting(
                    "expected a whole number after `-` in this pattern",
                    &[Number],
                );
                m.complete(p, LiteralPattern);
                return true;
            }
            let text = p.current_text();
            if text.contains(['.', 'e', 'E']) {
                let text = text.to_owned();
                p.error(
                    SyntaxErrorCode::LiteralPattern,
                    format!("only whole numbers can be matched, not `{text}`"),
                );
                p.hint("Match on a whole number, or compare with `<` in an `if` instead.");
            }
            p.bump();
            m.complete(p, LiteralPattern);
        }
        _ => return false,
    }
    true
}

/// `PatternList ::= "(" (Pattern ("," Pattern)* ","?)? ")"`
fn pattern_list(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // (
    loop {
        if p.at(RParen) || p.at_eof() {
            break;
        }
        if !pattern(p) {
            if p.at(Comma) {
                p.error_and_bump(SyntaxErrorCode::Expected, "expected a pattern before `,`");
                continue;
            }
            p.error_expecting(
                "expected `)` to close the constructor's patterns",
                &[RParen, Comma],
            );
            m.complete(p, PatternList);
            return;
        }
        if p.at(RParen) {
            break;
        }
        if !p.eat(Comma) {
            if can_start(p.current()) {
                p.error_expecting("expected `,` between patterns", &[Comma]);
                continue;
            }
            p.error_expecting(
                "expected `)` to close the constructor's patterns",
                &[RParen, Comma],
            );
            m.complete(p, PatternList);
            return;
        }
    }
    p.expect(RParen, "expected `)` to close the constructor's patterns");
    m.complete(p, PatternList);
}
