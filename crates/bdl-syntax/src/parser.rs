//! Hand-written Pratt parser.  Deterministic, total (never panics), reports
//! the first error with an exact span, and requires the whole source to be
//! consumed.

use crate::ast::{BinaryOp, ExprKind, SurfaceExpr, UnaryOp, Unit};
use crate::lexer::{lex, Token, TokenKind};
use bdl_diagnostics::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    /// Designer-readable, e.g. "expected a value here".
    pub message: String,
    /// What the parser found, for the technical line.
    pub found: String,
}

pub fn parse(src: &str) -> Result<SurfaceExpr, ParseError> {
    let tokens = lex(src);
    let mut p = Parser { tokens, pos: 0 };
    if p.peek().kind == TokenKind::Eof {
        return Err(ParseError {
            span: p.peek().span,
            message: "the formula is empty".into(),
            found: "nothing".into(),
        });
    }
    let expr = p.expr(0)?;
    match &p.peek().kind {
        TokenKind::Eof => Ok(expr),
        TokenKind::RParen => Err(p.error("this `)` closes nothing")),
        TokenKind::Error(_) => Err(p.error("this character cannot appear in a formula")),
        _ => Err(p.error("expected an operator or the end of the formula here")),
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

/// Binding powers (left, right); higher binds tighter.
fn infix_power(op: &TokenKind) -> Option<(u8, u8, BinaryOp)> {
    Some(match op {
        TokenKind::OrOr => (1, 2, BinaryOp::Or),
        TokenKind::AndAnd => (3, 4, BinaryOp::And),
        TokenKind::EqEq => (5, 6, BinaryOp::Eq),
        TokenKind::Ne => (5, 6, BinaryOp::Ne),
        TokenKind::Lt => (7, 8, BinaryOp::Lt),
        TokenKind::Le => (7, 8, BinaryOp::Le),
        TokenKind::Gt => (7, 8, BinaryOp::Gt),
        TokenKind::Ge => (7, 8, BinaryOp::Ge),
        TokenKind::Plus => (9, 10, BinaryOp::Add),
        TokenKind::Minus => (9, 10, BinaryOp::Sub),
        TokenKind::Star => (11, 12, BinaryOp::Mul),
        TokenKind::Slash => (11, 12, BinaryOp::Div),
        _ => return None,
    })
}

const UNARY_POWER: u8 = 13;

impl Parser {
    fn peek(&self) -> &Token {
        // The token stream always ends with Eof and `pos` never passes it.
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn bump(&mut self) -> Token {
        let t = self.peek().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn error(&self, message: &str) -> ParseError {
        let t = self.peek();
        ParseError {
            span: t.span,
            message: message.into(),
            found: t.kind.describe(),
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParseError> {
        if self.peek().kind == kind {
            Ok(self.bump())
        } else {
            Err(self.error(message))
        }
    }

    fn expr(&mut self, min_power: u8) -> Result<SurfaceExpr, ParseError> {
        let mut lhs = self.prefix()?;
        loop {
            let Some((lp, rp, op)) = infix_power(&self.peek().kind) else {
                break;
            };
            if lp < min_power {
                break;
            }
            self.bump();
            let rhs = self.expr(rp)?;
            let span = lhs.span.to(rhs.span);
            lhs = SurfaceExpr {
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    fn prefix(&mut self) -> Result<SurfaceExpr, ParseError> {
        let t = self.peek().clone();
        match t.kind {
            TokenKind::Ident(name) => {
                self.bump();
                Ok(SurfaceExpr {
                    kind: ExprKind::Name(name),
                    span: t.span,
                })
            }
            TokenKind::Number { text, value } => {
                self.bump();
                // `90 deg`: a name right after a number is its unit.
                let (unit, span) = match &self.peek().kind {
                    TokenKind::Ident(u) => {
                        let u = u.clone();
                        let ut = self.bump();
                        (
                            Some(Unit {
                                name: u,
                                span: ut.span,
                            }),
                            t.span.to(ut.span),
                        )
                    }
                    _ => (None, t.span),
                };
                Ok(SurfaceExpr {
                    kind: ExprKind::Number { text, value, unit },
                    span,
                })
            }
            TokenKind::True => {
                self.bump();
                Ok(SurfaceExpr {
                    kind: ExprKind::Bool(true),
                    span: t.span,
                })
            }
            TokenKind::False => {
                self.bump();
                Ok(SurfaceExpr {
                    kind: ExprKind::Bool(false),
                    span: t.span,
                })
            }
            TokenKind::Bang | TokenKind::Minus => {
                self.bump();
                let op = if t.kind == TokenKind::Bang {
                    UnaryOp::Not
                } else {
                    UnaryOp::Neg
                };
                let expr = self.expr(UNARY_POWER)?;
                let span = t.span.to(expr.span);
                Ok(SurfaceExpr {
                    kind: ExprKind::Unary {
                        op,
                        expr: Box::new(expr),
                    },
                    span,
                })
            }
            TokenKind::LParen => {
                self.bump();
                let inner = self.expr(0)?;
                let close =
                    self.expect(TokenKind::RParen, "expected `)` to close the parenthesis")?;
                Ok(SurfaceExpr {
                    kind: inner.kind,
                    span: t.span.to(close.span),
                })
            }
            TokenKind::If => {
                self.bump();
                let cond = self.expr(0)?;
                self.expect(TokenKind::Then, "expected `then` after the condition")?;
                let then = self.expr(0)?;
                self.expect(
                    TokenKind::Else,
                    "expected `else`: an `if` must say what happens otherwise",
                )?;
                let els = self.expr(0)?;
                let span = t.span.to(els.span);
                Ok(SurfaceExpr {
                    kind: ExprKind::If {
                        cond: Box::new(cond),
                        then: Box::new(then),
                        els: Box::new(els),
                    },
                    span,
                })
            }
            TokenKind::Eof => Err(self.error("the formula ends where a value was expected")),
            TokenKind::RParen => Err(self.error("this `)` closes nothing")),
            TokenKind::Error(_) => Err(self.error("this character cannot appear in a formula")),
            _ => Err(self.error("expected a value here")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(e: &SurfaceExpr) -> String {
        match &e.kind {
            ExprKind::Name(n) => n.clone(),
            ExprKind::Number { text, unit, .. } => match unit {
                Some(u) => format!("{text}{}", u.name),
                None => text.clone(),
            },
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Unary { op, expr } => {
                format!(
                    "({}{})",
                    if *op == UnaryOp::Not { "!" } else { "-" },
                    show(expr)
                )
            }
            ExprKind::Binary { op, lhs, rhs } => {
                format!("({} {} {})", show(lhs), op.symbol(), show(rhs))
            }
            ExprKind::If { cond, then, els } => {
                format!("(if {} then {} else {})", show(cond), show(then), show(els))
            }
        }
    }

    #[test]
    fn precedence() {
        assert_eq!(show(&parse("1 + 2 * 3").unwrap()), "(1 + (2 * 3))");
        assert_eq!(show(&parse("(1 + 2) * 3").unwrap()), "((1 + 2) * 3)");
        assert_eq!(show(&parse("!x").unwrap()), "(!x)");
        assert_eq!(show(&parse("x < y").unwrap()), "(x < y)");
        assert_eq!(
            show(&parse("if x then y else z").unwrap()),
            "(if x then y else z)"
        );
        assert_eq!(
            show(&parse("a || b && c == d < e + f * -g").unwrap()),
            "(a || (b && (c == (d < (e + (f * (-g)))))))"
        );
        assert_eq!(show(&parse("1 - 2 - 3").unwrap()), "((1 - 2) - 3)");
        assert_eq!(show(&parse("90 deg / 2").unwrap()), "(90deg / 2)");
        assert_eq!(
            show(&parse("if a < 1 then b else if c then d else e").unwrap()),
            "(if (a < 1) then b else (if c then d else e))"
        );
    }

    #[test]
    fn spans_cover_the_source() {
        let e = parse("tilt / 90 deg").unwrap();
        assert_eq!(e.span, Span::new(0, 13));
        let ExprKind::Binary { rhs, .. } = &e.kind else {
            panic!()
        };
        assert_eq!(rhs.span, Span::new(7, 13));
        let e = parse("(a + b)").unwrap();
        assert_eq!(e.span, Span::new(0, 7));
    }

    #[test]
    fn malformed() {
        let e = parse("1 +").unwrap_err();
        assert_eq!(e.span, Span::new(3, 3));
        assert!(e.message.contains("ends where a value"));
        let e = parse("(1 + 2").unwrap_err();
        assert!(e.message.contains("`)`"));
        let e = parse("1 2").unwrap_err();
        assert_eq!(e.span, Span::new(2, 3));
        let e = parse("if a then b").unwrap_err();
        assert!(e.message.contains("`else`"));
        let e = parse("a ) b").unwrap_err();
        assert!(e.message.contains("closes nothing"));
        let e = parse("").unwrap_err();
        assert!(e.message.contains("empty"));
        let e = parse("a # b").unwrap_err();
        assert!(e.message.contains("cannot appear"));
        assert_eq!(e.span, Span::new(2, 3));
        let e = parse("a & b").unwrap_err();
        assert_eq!(e.found, "`&`");
    }

    proptest::proptest! {
        #[test]
        fn never_panics(s in "\\PC{0,64}") {
            let _ = parse(&s);
        }

        #[test]
        fn well_formed_arithmetic_round_trips(a in 0u32..1000, b in 0u32..1000, c in 0u32..1000) {
            let e = parse(&format!("{a} + {b} * {c}")).unwrap();
            proptest::prop_assert_eq!(show(&e), format!("({a} + ({b} * {c}))"));
        }
    }
}
