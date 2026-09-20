//! Logos lexer.  Every byte of the source belongs to exactly one token
//! (trivia and error tokens included), so the token texts concatenate back
//! to the source.  Never panics: unknown characters become
//! [`SyntaxKind::Error`] tokens covering one whole UTF-8 character.
//!
//! Numbers are *spelled*, not valued: a [`SyntaxKind::Number`] token is its
//! text, and nothing here converts it (`docs/spec/textual-syntax.md` §2.3).

use crate::kind::SyntaxKind;
use crate::syntax::{SyntaxError, SyntaxErrorCode};
use bdl_diagnostics::Span;
use logos::Logos;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub span: Span,
}

impl Token {
    pub fn text<'a>(&self, src: &'a str) -> &'a str {
        // Logos spans are char-aligned; a defensive `get` keeps this total.
        src.get(self.span.start as usize..self.span.end as usize)
            .unwrap_or("")
    }
}

/// The lexer's own token classes; keywords are split off from `Ident`
/// afterwards so that the ident regex is the single authority on what an
/// identifier looks like.
#[derive(Logos, Debug, PartialEq)]
enum Raw {
    #[regex(r"[ \t\r\n\f]+")]
    Whitespace,
    #[token("//", line_comment)]
    LineComment,
    /// `true` when terminated by `*/`.
    #[token("/*", block_comment)]
    BlockComment(bool),
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,
    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?|\.[0-9]+([eE][+-]?[0-9]+)?")]
    Number,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Ne,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(";")]
    Semi,
    #[token("=")]
    Eq,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("!")]
    Bang,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("@")]
    At,
    #[token(".")]
    Dot,
    /// `..` — a closed range between two values (`lo .. hi`).
    #[token("..")]
    DotDot,
    /// `?` — a slot: an expression not yet written (a Composer hole).
    #[token("?")]
    Question,
    /// `??` — the value when present, a default when absent.
    #[token("??")]
    QuestionQuestion,
    /// `^` — a unit power (`s^2`), inside a unit expression only.
    #[token("^")]
    Caret,
}

fn line_comment(lex: &mut logos::Lexer<Raw>) {
    let rest = lex.remainder();
    lex.bump(rest.find('\n').unwrap_or(rest.len()));
}

fn block_comment(lex: &mut logos::Lexer<Raw>) -> bool {
    let rest = lex.remainder();
    match rest.find("*/") {
        Some(i) => {
            lex.bump(i + 2);
            true
        }
        None => {
            lex.bump(rest.len());
            false
        }
    }
}

/// Tokenise `src` completely.  The result never contains `Eof`; the parser
/// synthesises it.  Lexical diagnostics (invalid characters, unterminated
/// comments) are returned alongside so the parser can merge them.
pub fn lex(src: &str) -> (Vec<Token>, Vec<SyntaxError>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut lexer = Raw::lexer(src);
    while let Some(item) = lexer.next() {
        let range = lexer.span();
        let span = Span::new(range.start as u32, range.end as u32);
        let text = &src[range.clone()];
        let kind = match item {
            Ok(Raw::Whitespace) => SyntaxKind::Whitespace,
            Ok(Raw::LineComment) => SyntaxKind::LineComment,
            Ok(Raw::BlockComment(terminated)) => {
                if !terminated {
                    errors.push(SyntaxError::new(
                        SyntaxErrorCode::UnterminatedComment,
                        span,
                        "this `/*` comment is never closed",
                        "the end of the source",
                    ));
                }
                SyntaxKind::BlockComment
            }
            Ok(Raw::Ident) => match text {
                "_" => SyntaxKind::Underscore,
                t => SyntaxKind::from_keyword(t).unwrap_or(SyntaxKind::Ident),
            },
            Ok(Raw::Number) => SyntaxKind::Number,
            Ok(Raw::LParen) => SyntaxKind::LParen,
            Ok(Raw::RParen) => SyntaxKind::RParen,
            Ok(Raw::LBrace) => SyntaxKind::LBrace,
            Ok(Raw::RBrace) => SyntaxKind::RBrace,
            Ok(Raw::LBracket) => SyntaxKind::LBracket,
            Ok(Raw::RBracket) => SyntaxKind::RBracket,
            Ok(Raw::Lt) => SyntaxKind::Lt,
            Ok(Raw::Gt) => SyntaxKind::Gt,
            Ok(Raw::Le) => SyntaxKind::Le,
            Ok(Raw::Ge) => SyntaxKind::Ge,
            Ok(Raw::EqEq) => SyntaxKind::EqEq,
            Ok(Raw::Ne) => SyntaxKind::Ne,
            Ok(Raw::Colon) => SyntaxKind::Colon,
            Ok(Raw::Comma) => SyntaxKind::Comma,
            Ok(Raw::Semi) => SyntaxKind::Semi,
            Ok(Raw::Eq) => SyntaxKind::Eq,
            Ok(Raw::Arrow) => SyntaxKind::Arrow,
            Ok(Raw::FatArrow) => SyntaxKind::FatArrow,
            Ok(Raw::Plus) => SyntaxKind::Plus,
            Ok(Raw::Minus) => SyntaxKind::Minus,
            Ok(Raw::Star) => SyntaxKind::Star,
            Ok(Raw::Slash) => SyntaxKind::Slash,
            Ok(Raw::Bang) => SyntaxKind::Bang,
            Ok(Raw::AndAnd) => SyntaxKind::AndAnd,
            Ok(Raw::OrOr) => SyntaxKind::OrOr,
            Ok(Raw::At) => SyntaxKind::At,
            Ok(Raw::Dot) => SyntaxKind::Dot,
            Ok(Raw::DotDot) => SyntaxKind::DotDot,
            Ok(Raw::Question) => SyntaxKind::Question,
            Ok(Raw::QuestionQuestion) => SyntaxKind::QuestionQuestion,
            Ok(Raw::Caret) => SyntaxKind::Caret,
            Err(()) => {
                errors.push(SyntaxError::new(
                    SyntaxErrorCode::InvalidCharacter,
                    span,
                    format!("`{text}` cannot appear in BDL source"),
                    format!("`{text}`"),
                ));
                SyntaxKind::Error
            }
        };
        tokens.push(Token { kind, span });
    }
    (tokens, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use SyntaxKind::*;

    fn kinds(s: &str) -> Vec<SyntaxKind> {
        lex(s).0.into_iter().map(|t| t.kind).collect()
    }

    fn texts(s: &str) -> Vec<String> {
        lex(s).0.iter().map(|t| t.text(s).to_owned()).collect()
    }

    #[test]
    fn longest_match_for_compound_operators() {
        assert_eq!(
            kinds("a->b=>c==d!=e<=f>=g&&h||i"),
            vec![
                Ident, Arrow, Ident, FatArrow, Ident, EqEq, Ident, Ne, Ident, Le, Ident, Ge, Ident,
                AndAnd, Ident, OrOr, Ident
            ]
        );
        assert_eq!(kinds("a- >b"), vec![Ident, Minus, Whitespace, Gt, Ident]);
        assert_eq!(kinds("=>="), vec![FatArrow, Eq]);
        assert_eq!(kinds(">>"), vec![Gt, Gt]);
    }

    #[test]
    fn keywords_underscore_and_identifiers() {
        assert_eq!(
            kinds("concept mapping enum match let if then else true false"),
            vec![
                KwConcept, Whitespace, KwMapping, Whitespace, KwEnum, Whitespace, KwMatch,
                Whitespace, KwLet, Whitespace, KwIf, Whitespace, KwThen, Whitespace, KwElse,
                Whitespace, KwTrue, Whitespace, KwFalse
            ]
        );
        assert_eq!(
            kinds("_ _x iffy context"),
            vec![Underscore, Whitespace, Ident, Whitespace, Ident, Whitespace, KwContext]
        );
        assert_eq!(
            kinds("clock output drive device component instance bind export requires provides param use"),
            vec![
                KwClock, Whitespace, KwOutput, Whitespace, KwDrive, Whitespace, KwDevice,
                Whitespace, KwComponent, Whitespace, KwInstance, Whitespace, KwBind, Whitespace,
                KwExport, Whitespace, KwRequires, Whitespace, KwProvides, Whitespace, KwParam,
                Whitespace, KwUse
            ]
        );
        assert_eq!(
            kinds("a.b @c .5"),
            vec![Ident, Dot, Ident, Whitespace, At, Ident, Whitespace, Number]
        );
    }

    #[test]
    fn numbers_are_spelled_not_valued() {
        assert_eq!(
            texts("1.5e3 .5 2 0.1 1e999"),
            vec!["1.5e3", " ", ".5", " ", "2", " ", "0.1", " ", "1e999"]
        );
        assert_eq!(kinds("1."), vec![Number, Dot]);
        assert_eq!(kinds("1e"), vec![Number, Ident]);
        assert_eq!(kinds("90deg"), vec![Number, Ident]);
    }

    #[test]
    fn comments_are_tokens() {
        assert_eq!(
            kinds("a // c\nb"),
            vec![Ident, Whitespace, LineComment, Whitespace, Ident]
        );
        assert_eq!(texts("/* a **/ x"), vec!["/* a **/", " ", "x"]);
        let (t, e) = lex("/* open");
        assert_eq!(t[0].kind, BlockComment);
        assert_eq!(e[0].code, SyntaxErrorCode::UnterminatedComment);
    }

    #[test]
    fn unknown_characters_are_error_tokens_with_utf8_spans() {
        let (t, e) = lex("a § b");
        assert_eq!(t[2].kind, Error);
        assert_eq!(t[2].span, Span::new(2, 4));
        assert_eq!(t[4].span, Span::new(5, 6));
        assert_eq!(e[0].message, "`§` cannot appear in BDL source");
        assert_eq!(
            kinds("a & b"),
            vec![Ident, Whitespace, Error, Whitespace, Ident]
        );
        assert_eq!(texts("x😀y"), vec!["x", "😀", "y"]);
    }

    #[test]
    fn tokens_cover_the_source() {
        for s in [
            "",
            "a",
            "a § b\n/* x */ 1.5 deg -> => \t",
            "😀😀",
            "/* open",
        ] {
            let (t, _) = lex(s);
            let joined: String = t.iter().map(|t| t.text(s)).collect();
            assert_eq!(joined, s);
        }
    }
}
