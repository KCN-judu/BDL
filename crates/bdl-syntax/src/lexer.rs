//! Tokens with byte spans.  Never panics; unknown characters become
//! [`TokenKind::Error`] tokens the parser reports.

use bdl_diagnostics::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Number {
        text: String,
        value: f64,
    },
    If,
    Then,
    Else,
    True,
    False,
    Plus,
    Minus,
    Star,
    Slash,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    Ne,
    AndAnd,
    OrOr,
    Bang,
    LParen,
    RParen,
    /// An unrecognised character (or a lone `&`, `|`, `=`).
    Error(String),
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Ident(n) => format!("name `{n}`"),
            TokenKind::Number { text, .. } => format!("number `{text}`"),
            TokenKind::If => "`if`".into(),
            TokenKind::Then => "`then`".into(),
            TokenKind::Else => "`else`".into(),
            TokenKind::True => "`true`".into(),
            TokenKind::False => "`false`".into(),
            TokenKind::Plus => "`+`".into(),
            TokenKind::Minus => "`-`".into(),
            TokenKind::Star => "`*`".into(),
            TokenKind::Slash => "`/`".into(),
            TokenKind::Lt => "`<`".into(),
            TokenKind::Le => "`<=`".into(),
            TokenKind::Gt => "`>`".into(),
            TokenKind::Ge => "`>=`".into(),
            TokenKind::EqEq => "`==`".into(),
            TokenKind::Ne => "`!=`".into(),
            TokenKind::AndAnd => "`&&`".into(),
            TokenKind::OrOr => "`||`".into(),
            TokenKind::Bang => "`!`".into(),
            TokenKind::LParen => "`(`".into(),
            TokenKind::RParen => "`)`".into(),
            TokenKind::Error(s) => format!("`{s}`"),
            TokenKind::Eof => "end of formula".into(),
        }
    }
}

pub fn lex(src: &str) -> Vec<Token> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let at = |i: usize| bytes.get(i).copied();
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        let kind = if c.is_ascii_alphabetic() || c == b'_' {
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            match &src[start..i] {
                "if" => TokenKind::If,
                "then" => TokenKind::Then,
                "else" => TokenKind::Else,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                s => TokenKind::Ident(s.to_owned()),
            }
        } else if c.is_ascii_digit() || (c == b'.' && at(i + 1).is_some_and(|d| d.is_ascii_digit()))
        {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if at(i) == Some(b'.') && at(i + 1).is_some_and(|d| d.is_ascii_digit()) {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if matches!(at(i), Some(b'e' | b'E'))
                && (at(i + 1).is_some_and(|d| d.is_ascii_digit())
                    || (matches!(at(i + 1), Some(b'+' | b'-'))
                        && at(i + 2).is_some_and(|d| d.is_ascii_digit())))
            {
                i += 2;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let text = &src[start..i];
            match text.parse::<f64>() {
                Ok(value) if value.is_finite() => TokenKind::Number {
                    text: text.to_owned(),
                    value,
                },
                _ => TokenKind::Error(text.to_owned()),
            }
        } else {
            let two = |b: u8| at(i + 1) == Some(b);
            let (kind, len) = match c {
                b'+' => (TokenKind::Plus, 1),
                b'-' => (TokenKind::Minus, 1),
                b'*' => (TokenKind::Star, 1),
                b'/' => (TokenKind::Slash, 1),
                b'(' => (TokenKind::LParen, 1),
                b')' => (TokenKind::RParen, 1),
                b'<' if two(b'=') => (TokenKind::Le, 2),
                b'<' => (TokenKind::Lt, 1),
                b'>' if two(b'=') => (TokenKind::Ge, 2),
                b'>' => (TokenKind::Gt, 1),
                b'=' if two(b'=') => (TokenKind::EqEq, 2),
                b'!' if two(b'=') => (TokenKind::Ne, 2),
                b'!' => (TokenKind::Bang, 1),
                b'&' if two(b'&') => (TokenKind::AndAnd, 2),
                b'|' if two(b'|') => (TokenKind::OrOr, 2),
                _ => {
                    // one whole UTF-8 character
                    let ch = src[start..].chars().next().map(char::len_utf8).unwrap_or(1);
                    (TokenKind::Error(src[start..start + ch].to_owned()), ch)
                }
            };
            i += len;
            kind
        };
        out.push(Token {
            kind,
            span: Span::new(start as u32, i as u32),
        });
    }
    out.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(bytes.len() as u32, bytes.len() as u32),
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(s: &str) -> Vec<TokenKind> {
        lex(s).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn operators_and_keywords() {
        assert_eq!(
            kinds("a<=b && !c || d != e"),
            vec![
                TokenKind::Ident("a".into()),
                TokenKind::Le,
                TokenKind::Ident("b".into()),
                TokenKind::AndAnd,
                TokenKind::Bang,
                TokenKind::Ident("c".into()),
                TokenKind::OrOr,
                TokenKind::Ident("d".into()),
                TokenKind::Ne,
                TokenKind::Ident("e".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn numbers() {
        assert!(matches!(&kinds("1.5e3")[0], TokenKind::Number { value, .. } if *value == 1500.0));
        assert!(matches!(&kinds(".5")[0], TokenKind::Number { value, .. } if *value == 0.5));
        assert!(matches!(&kinds("2")[0], TokenKind::Number { value, .. } if *value == 2.0));
    }

    #[test]
    fn unknown_characters_are_error_tokens_with_utf8_spans() {
        let t = lex("a § b");
        assert!(matches!(&t[1].kind, TokenKind::Error(s) if s == "§"));
        assert_eq!(t[1].span, Span::new(2, 4));
        assert_eq!(t[2].span, Span::new(5, 6));
    }
}
