//! Expressions (`docs/spec/textual-syntax.md` §4.3, §5): a Pratt loop over the
//! binary operators, prefix unary, postfix call, and the primary forms
//! (`if`, `match`, blocks, literals with units, names, parentheses).

use super::{grammar, pattern, CompletedMarker, Parser};
use crate::kind::SyntaxKind::{self, *};
use crate::syntax::SyntaxErrorCode;
use bdl_diagnostics::Span;

const ITEM_START: &[SyntaxKind] = &[KwConcept, KwMapping, KwEnum];

/// Binding powers (left, right) and whether the operator is a comparison
/// (§5: comparisons are non-associative).
fn infix(kind: SyntaxKind) -> Option<(u8, u8, bool)> {
    Some(match kind {
        OrOr => (1, 2, false),
        AndAnd => (3, 4, false),
        EqEq | Ne => (5, 6, true),
        Lt | Le | Gt | Ge => (7, 8, true),
        Plus | Minus => (9, 10, false),
        Star | Slash => (11, 12, false),
        _ => return None,
    })
}

const UNARY_BP: u8 = 13;

pub(super) fn can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        Ident | Number | KwTrue | KwFalse | LParen | LBrace | KwIf | KwMatch | Bang | Minus
    ) || kind.is_future_reserved()
}

/// Parse an expression; `None` when nothing expression-like starts here
/// (no error is reported: the caller says what it expected).
pub(super) fn expr(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    expr_bp(p, 0).map(|(cm, _)| cm)
}

/// Message for a missing expression, specialised by what is there instead.
pub(super) fn no_expression_message(p: &Parser<'_>) -> String {
    match p.current() {
        RParen => "this `)` closes nothing".into(),
        RBrace => "this `}` closes nothing".into(),
        Eof => "the source ends where a value was expected".into(),
        _ => "expected a value here".into(),
    }
}

/// After a complete expression, something that cannot continue it.
/// Adds the unit hint when the offender is a name (`x deg`, `f(x) mm`).
pub(super) fn unexpected_after_expr(p: &mut Parser<'_>, expected: &str) {
    let message = format!("expected an operator or {expected}");
    if p.at(Ident) {
        let word = p.current_text().to_owned();
        p.error(SyntaxErrorCode::Unexpected, message);
        p.hint(format!(
            "If `{word}` is meant as a unit: a unit can only follow a number, as in `90 {word}`. \
             Write the quantity as a multiplication, e.g. `x * (1 {word})`."
        ));
    } else {
        p.error(SyntaxErrorCode::Unexpected, message);
    }
}

/// Returns the completed node and, when it is an unparenthesised
/// comparison, the span of its operator (for the chaining diagnostic).
fn expr_bp(p: &mut Parser<'_>, min_bp: u8) -> Option<(CompletedMarker, Option<Span>)> {
    let mut lhs = unary(p)?;
    let mut lhs_cmp: Option<Span> = None;
    loop {
        if p.at(Error) {
            // Already reported by the lexer; keep going as if it were absent.
            p.bump_as_error();
            continue;
        }
        let Some((lbp, rbp, is_cmp)) = infix(p.current()) else {
            break;
        };
        if lbp < min_bp {
            break;
        }
        let op_span = p.current_span();
        if is_cmp && lhs_cmp.is_some() {
            chained_comparison(p, op_span);
        }
        let m = lhs.precede(p);
        let op = p.current();
        p.bump();
        match expr_bp(p, rbp) {
            None => {
                p.error_expecting(
                    format!(
                        "expected an expression after `{}`",
                        op.fixed_text().unwrap_or("")
                    )
                    .as_str(),
                    &[],
                );
                lhs = m.complete(p, BinaryExpr);
                lhs_cmp = None;
                break;
            }
            Some((_, rhs_cmp)) => {
                if is_cmp {
                    if let Some(inner) = rhs_cmp {
                        chained_comparison(p, inner);
                    }
                }
                lhs = m.complete(p, BinaryExpr);
                lhs_cmp = if is_cmp { Some(op_span) } else { None };
            }
        }
    }
    Some((lhs, lhs_cmp))
}

fn chained_comparison(p: &mut Parser<'_>, at: Span) {
    let e = crate::syntax::SyntaxError::new(
        SyntaxErrorCode::ChainedComparison,
        at,
        "comparisons cannot be chained",
        "a second comparison",
    )
    .with_hint(
        "Compare in two steps joined with `&&`, as in `a < b && b < c`; \
         use parentheses if the result of a comparison really is what you compare.",
    );
    p.push_error(e);
}

/// `UnaryExpr ::= ("!" | "-") UnaryExpr | PostfixExpr`
fn unary(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    if p.at(Bang) || p.at(Minus) {
        let m = p.start();
        let op = p.current();
        p.bump();
        if expr_bp(p, UNARY_BP).is_none() {
            p.error_expecting(
                &format!("expected a value after `{}`", op.fixed_text().unwrap_or("")),
                &[],
            );
        }
        return Some(m.complete(p, UnaryExpr));
    }
    postfix(p)
}

/// `PostfixExpr ::= PrimaryExpr CallSuffix*`
fn postfix(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    let mut lhs = primary(p)?;
    while p.at(LParen) {
        let m = lhs.precede(p);
        arg_list(p);
        lhs = m.complete(p, CallExpr);
    }
    Some(lhs)
}

/// `ArgList ::= "(" (Expr ("," Expr)* ","?)? ")"`
fn arg_list(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // (
    loop {
        if p.at(RParen) || p.at_eof() {
            break;
        }
        if expr(p).is_none() {
            if p.at(Comma) {
                p.error_and_bump(SyntaxErrorCode::Expected, "expected an argument before `,`");
                continue;
            }
            if p.at(Error) {
                p.bump_as_error();
                continue;
            }
            p.error_expecting("expected `)` to close this call", &[RParen, Comma]);
            m.complete(p, ArgList);
            return;
        }
        if p.at(RParen) {
            break;
        }
        if !p.eat(Comma) {
            if can_start(p.current()) {
                unexpected_after_expr(p, "`,` between arguments");
                continue;
            }
            p.error_expecting("expected `)` to close this call", &[RParen, Comma]);
            m.complete(p, ArgList);
            return;
        }
    }
    p.expect(RParen, "expected `)` to close this call");
    m.complete(p, ArgList);
}

/// `PrimaryExpr ::= NameExpr | LiteralExpr | ParenExpr | IfExpr | MatchExpr | BlockExpr`
fn primary(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    loop {
        return Some(match p.current() {
            Error => {
                // Reported by the lexer; skip it and try again.
                p.bump_as_error();
                continue;
            }
            Ident => {
                let m = p.start();
                grammar::name_ref(p);
                m.complete(p, NameExpr)
            }
            k if k.is_future_reserved() => {
                grammar::reserved_word(p);
                let m = p.start();
                grammar::name_ref(p);
                m.complete(p, NameExpr)
            }
            Number => {
                let m = p.start();
                p.bump();
                if p.at(Ident) && !matches!(p.nth(1), FatArrow | LParen) {
                    // `90 deg`: any name right after a number is its unit;
                    // whether it *is* a unit is decided by elaboration.  A
                    // name followed by `=>` or `(` cannot be a unit (nothing
                    // may follow a unit but an operator), so a missing comma
                    // between match arms does not swallow the next pattern.
                    let u = p.start();
                    p.bump();
                    u.complete(p, UnitSuffix);
                }
                m.complete(p, LiteralExpr)
            }
            KwTrue | KwFalse => {
                let m = p.start();
                p.bump();
                m.complete(p, LiteralExpr)
            }
            LParen => paren_expr(p),
            KwIf => if_expr(p),
            KwMatch => match_expr(p),
            LBrace => block_expr(p),
            _ => return None,
        });
    }
}

fn paren_expr(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.bump(); // (
    if expr(p).is_none() {
        let msg = no_expression_message(p);
        p.error(SyntaxErrorCode::Expected, msg);
    }
    p.expect(RParen, "expected `)` to close the parenthesis");
    m.complete(p, ParenExpr)
}

/// `IfExpr ::= "if" Expr "then" Expr "else" Expr`
fn if_expr(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.bump(); // if
    if expr(p).is_none() {
        p.error_expecting("expected a condition after `if`", &[]);
    }
    p.expect(KwThen, "expected `then` after the condition");
    if expr(p).is_none() {
        p.error_expecting("expected an expression after `then`", &[]);
    }
    if p.expect(
        KwElse,
        "expected `else`: an `if` must say what happens otherwise",
    ) {
        if expr(p).is_none() {
            p.error_expecting("expected an expression after `else`", &[]);
        }
    } else {
        p.hint("Every `if` is a value, so both outcomes must be written: `if c then a else b`.");
    }
    m.complete(p, IfExpr)
}

/// `MatchExpr ::= "match" Expr "{" MatchArm* "}"`
fn match_expr(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.bump(); // match
    if expr(p).is_none() {
        p.error_expecting("expected the value to match after `match`", &[]);
    }
    if !p.at(LBrace) {
        p.error_expecting("expected `{` and the match arms", &[LBrace]);
        return m.complete(p, MatchExpr);
    }
    let arms = p.start();
    p.bump(); // {
    loop {
        if p.at(RBrace) || p.at_eof() {
            break;
        }
        if p.at_any(ITEM_START) {
            p.error_expecting("expected `}` to close the match", &[RBrace]);
            arms.complete(p, MatchArmList);
            return m.complete(p, MatchExpr);
        }
        if p.at(FatArrow) {
            p.error_expecting("expected a pattern before `=>`", &[]);
            p.hint("A match arm is `pattern => expression,`; use `_` to match anything.");
            match_arm(p);
            continue;
        }
        if !pattern::can_start(p.current()) {
            p.error_recover(
                SyntaxErrorCode::Expected,
                "expected a pattern to start a match arm",
                &[RBrace, Comma],
            );
            if p.at(Comma) {
                p.bump_as_error();
            }
            continue;
        }
        match_arm(p);
    }
    p.expect(RBrace, "expected `}` to close the match");
    arms.complete(p, MatchArmList);
    m.complete(p, MatchExpr)
}

/// `MatchArm ::= Pattern "=>" Expr ","?`
fn match_arm(p: &mut Parser<'_>) {
    let m = p.start();
    pattern::pattern(p);
    if !p.eat(FatArrow) {
        p.error_expecting("expected `=>` after the pattern", &[FatArrow]);
    }
    if expr(p).is_none() {
        p.error_expecting("expected an expression after `=>`", &[]);
    }
    if !p.eat(Comma) && !p.at(RBrace) && !p.at_eof() && !p.at_any(ITEM_START) {
        if pattern::can_start(p.current()) {
            unexpected_after_expr(p, "`,` after this match arm");
        } else {
            unexpected_after_expr(p, "`,` or `}` after this match arm");
            p.skip_until(&[Comma, RBrace, KwConcept, KwMapping, KwEnum]);
            p.eat(Comma);
        }
    }
    m.complete(p, MatchArm);
}

/// `BlockExpr ::= "{" LetStmt* Expr? "}"`
fn block_expr(p: &mut Parser<'_>) -> CompletedMarker {
    let m = p.start();
    p.bump(); // {
    let mut has_tail = false;
    loop {
        if p.at(RBrace) || p.at_eof() {
            break;
        }
        if p.at_any(ITEM_START) {
            p.error_expecting("expected `}` to close this block", &[RBrace]);
            return m.complete(p, BlockExpr);
        }
        if p.at(KwLet) {
            if has_tail {
                p.error(
                    SyntaxErrorCode::Unexpected,
                    "a `let` cannot follow the block's final expression",
                );
                p.hint("Move the `let` above, or end the previous expression with `;` if it was meant as a `let`.");
            }
            let_stmt(p);
            continue;
        }
        if p.at(Semi) {
            p.error_and_bump(
                SyntaxErrorCode::Unexpected,
                "only a `let` ends with `;`; the block's final expression has none",
            );
            continue;
        }
        if has_tail {
            unexpected_after_expr(p, "`}` to close this block");
        }
        if expr(p).is_some() {
            has_tail = true;
        } else if p.at(Error) {
            p.bump_as_error();
        } else {
            p.error_and_bump(
                SyntaxErrorCode::Expected,
                "expected `let` or an expression in this block",
            );
        }
    }
    if !has_tail {
        p.error_expecting("this block has no final expression to be its value", &[]);
        p.hint("A block ends with the expression it stands for, after any `let`s: `{ let x = …; x + 1 }`.");
    }
    p.expect(RBrace, "expected `}` to close this block");
    m.complete(p, BlockExpr)
}

/// `LetStmt ::= "let" Pattern "=" Expr ";"`
fn let_stmt(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // let
    if !pattern::pattern(p) {
        p.error_expecting("expected a name after `let`", &[Ident]);
    }
    if !p.eat(Eq) {
        p.error_expecting("expected `=` after the name in this `let`", &[Eq]);
    }
    if expr(p).is_none() {
        p.error_expecting("expected an expression after `=`", &[]);
    }
    p.expect(Semi, "expected `;` to end this `let`");
    m.complete(p, LetStmt);
}
