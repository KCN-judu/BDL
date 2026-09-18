//! The formatter: canonical spacing and indentation over the lossless
//! tree, and nothing else.  Items keep their order and their blank-line
//! grouping (runs of blank lines collapse to one), comments stay where
//! they were written, line breaks inside a formula are the author's and
//! the relative indentation of a multi-line body is preserved — only the
//! spacing within a line and the indentation level are decided here.
//!
//! Source with syntax errors is not formatted: the tree's shape is not
//! what the author means yet, so `None` is returned and the text stays.

use crate::kind::SyntaxKind as K;
use crate::syntax::{SyntaxNode, SyntaxToken};
use rowan::NodeOrToken;

const INDENT: usize = 2;

/// Format a module.  `None` when the source does not parse cleanly.
pub fn format_module(src: &str) -> Option<String> {
    let parse = crate::parser::parse_module(src);
    if !parse.errors().is_empty() {
        return None;
    }
    Some(Formatter::new(parse.syntax_node()).run())
}

/// Whether the token opens or closes a structural body, whose lines are
/// indented one level deeper than the item that owns it.
fn structural_brace(t: &SyntaxToken) -> bool {
    matches!(t.kind(), K::LBrace | K::RBrace)
        && t.parent().is_some_and(|p| {
            matches!(
                p.kind(),
                K::ComponentBody | K::EnumDecl | K::DeviceBody | K::InstanceBody
            )
        })
}

const ANCHOR_NODES: &[K] = &[
    K::ConceptDecl,
    K::MappingDecl,
    K::MappingDef,
    K::EnumDecl,
    K::EnumVariant,
    K::ClockDecl,
    K::OutputDecl,
    K::DriveDecl,
    K::DeviceDecl,
    K::PinFix,
    K::ComponentDecl,
    K::UseDecl,
    K::ParamClockDecl,
    K::PortDecl,
    K::InstanceDecl,
    K::InstanceArg,
    K::BindDecl,
    K::ExportDecl,
];

fn first_code_token(node: &SyntaxNode) -> Option<SyntaxToken> {
    node.descendants_with_tokens()
        .filter_map(|e| match e {
            NodeOrToken::Token(t) => Some(t),
            NodeOrToken::Node(_) => None,
        })
        .find(|t| !t.kind().is_trivia())
}

/// A token that starts a line at the item level of its scope: the first
/// token of an item (or of a definition, a variant, a pin, an argument
/// written on its own line), or a structural closing brace.
fn is_anchor(t: &SyntaxToken) -> bool {
    if t.kind() == K::RBrace && structural_brace(t) {
        return true;
    }
    t.parent_ancestors()
        .any(|n| ANCHOR_NODES.contains(&n.kind()) && first_code_token(&n).as_ref() == Some(t))
}

struct Formatter {
    tokens: Vec<SyntaxToken>,
    out: String,
}

impl Formatter {
    fn new(root: SyntaxNode) -> Formatter {
        let tokens = root
            .descendants_with_tokens()
            .filter_map(|e| match e {
                NodeOrToken::Token(t) => Some(t),
                NodeOrToken::Node(_) => None,
            })
            .collect();
        Formatter {
            tokens,
            out: String::new(),
        }
    }

    fn run(mut self) -> String {
        let n = self.tokens.len();
        // Pre-pass: for every token that starts a line, its original
        // indentation and whether the line is at item level.  Continuation
        // lines between two anchors keep their indentation relative to the
        // shallowest of them.
        let mut line_start = vec![false; n];
        let mut orig_indent = vec![0usize; n];
        let mut anchor = vec![false; n];
        let mut at_start = true;
        for (i, t) in self.tokens.iter().enumerate() {
            match t.kind() {
                K::Whitespace => {
                    if t.text().contains('\n') {
                        at_start = true;
                        orig_indent[i] = t
                            .text()
                            .rsplit('\n')
                            .next()
                            .map(|s| s.chars().count())
                            .unwrap_or(0);
                    }
                    continue;
                }
                _ => {
                    line_start[i] = at_start;
                    if at_start {
                        orig_indent[i] = if i > 0 && self.tokens[i - 1].kind() == K::Whitespace {
                            orig_indent[i - 1]
                        } else {
                            0
                        };
                    }
                    at_start = false;
                }
            }
        }
        // Anchor class: a code token by itself; a comment by the next code
        // token on a later line (a comment before an item is at item level).
        for i in 0..n {
            if !line_start[i] {
                continue;
            }
            let t = &self.tokens[i];
            anchor[i] = if t.kind().is_trivia() {
                self.next_code(i).is_none_or(|j| is_anchor(&self.tokens[j]))
            } else {
                is_anchor(t)
            };
        }
        // Relative indentation of continuation lines: per run, the
        // shallowest line becomes one level in.
        // A closing brace of a formula block written at the item's own
        // level stays there: it is not counted in the base and may sit
        // one level out.
        let mut extra = vec![0i64; n];
        let mut i = 0;
        while i < n {
            if line_start[i] && !anchor[i] {
                let mut j = i;
                let mut base = usize::MAX;
                while j < n && !(line_start[j] && anchor[j]) {
                    if line_start[j] && self.tokens[j].kind() != K::RBrace {
                        base = base.min(orig_indent[j]);
                    }
                    j += 1;
                }
                if base == usize::MAX {
                    base = 0;
                }
                for k in i..j {
                    if line_start[k] {
                        extra[k] = orig_indent[k] as i64 - base as i64;
                    }
                }
                i = j;
            } else {
                i += 1;
            }
        }

        let mut depth: usize = 0;
        let mut pending_newlines = 0usize;
        let mut pending_space = false;
        let mut prev: Option<SyntaxToken> = None;
        let mut prefix_minus = false;
        let mut first = true;
        for i in 0..n {
            let t = self.tokens[i].clone();
            match t.kind() {
                K::Whitespace => {
                    let nl = t.text().matches('\n').count();
                    if nl > 0 {
                        pending_newlines = pending_newlines.max(nl.min(2));
                        pending_space = false;
                    } else {
                        pending_space = true;
                    }
                    continue;
                }
                K::LineComment | K::BlockComment => {
                    if first {
                        // A comment at the very top stays at the top.
                    } else if pending_newlines > 0 {
                        let closing = self.next_code(i).is_some_and(|j| {
                            self.tokens[j].kind() == K::RBrace && structural_brace(&self.tokens[j])
                        });
                        // A comment before a closing brace sits with the
                        // body it closes.
                        let level = if anchor[i] && !closing {
                            depth
                        } else if anchor[i] {
                            depth.max(1)
                        } else {
                            depth + 1
                        };
                        self.newlines(pending_newlines, Self::indent(level, extra[i]));
                    } else {
                        self.out.push_str("  ");
                    }
                    self.out.push_str(t.text().trim_end());
                    pending_newlines = 0;
                    pending_space = false;
                    first = false;
                    continue;
                }
                _ => {}
            }
            let closing = t.kind() == K::RBrace && structural_brace(&t);
            if closing {
                depth = depth.saturating_sub(1);
            }
            if first {
                // Nothing before the first token.
            } else if pending_newlines > 0 {
                let level = if anchor[i] { depth } else { depth + 1 };
                self.newlines(pending_newlines, Self::indent(level, extra[i]));
            } else if let Some(p) = &prev {
                if Self::space_between(p, &t, prefix_minus)
                    || (pending_space && Self::space_kept(p, &t))
                {
                    self.out.push(' ');
                }
            }
            self.out.push_str(t.text());
            if t.kind() == K::LBrace && structural_brace(&t) {
                depth += 1;
            }
            prefix_minus = t.kind() == K::Minus
                && (pending_newlines > 0 || prev.as_ref().is_none_or(Self::before_prefix));
            pending_newlines = 0;
            pending_space = false;
            first = false;
            prev = Some(t);
        }
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        self.out.push('\n');
        self.out
    }

    fn next_code(&self, from: usize) -> Option<usize> {
        (from + 1..self.tokens.len()).find(|&j| !self.tokens[j].kind().is_trivia())
    }

    fn indent(level: usize, extra: i64) -> usize {
        ((level * INDENT) as i64 + extra).max(0) as usize
    }

    fn newlines(&mut self, count: usize, indent: usize) {
        while self.out.ends_with(' ') {
            self.out.pop();
        }
        for _ in 0..count {
            self.out.push('\n');
        }
        for _ in 0..indent {
            self.out.push(' ');
        }
    }

    /// A token after which `-` is a sign, not a subtraction.
    fn before_prefix(p: &SyntaxToken) -> bool {
        matches!(
            p.kind(),
            K::LParen
                | K::LBrace
                | K::Comma
                | K::Semi
                | K::Eq
                | K::Arrow
                | K::FatArrow
                | K::Plus
                | K::Minus
                | K::Star
                | K::Slash
                | K::Lt
                | K::Gt
                | K::Le
                | K::Ge
                | K::EqEq
                | K::Ne
                | K::AndAnd
                | K::OrOr
                | K::KwThen
                | K::KwElse
                | K::KwIf
                | K::KwMatch
                | K::KwIn
                | K::LBracket
        )
    }

    fn in_type_list(t: &SyntaxToken) -> bool {
        t.parent()
            .is_some_and(|p| matches!(p.kind(), K::TypeArgList | K::TypeParamList))
    }

    /// Whether canonical spacing puts a space between two adjacent tokens
    /// on one line.
    fn space_between(p: &SyntaxToken, t: &SyntaxToken, prefix_minus: bool) -> bool {
        if prefix_minus {
            return false;
        }
        if matches!(
            t.kind(),
            K::RParen | K::RBracket | K::Comma | K::Dot | K::Semi
        ) {
            return false;
        }
        if matches!(p.kind(), K::LParen | K::LBracket | K::Dot | K::At | K::Bang) {
            return false;
        }
        // `all x in xs: body` — the colon closes the head
        if t.kind() == K::Colon && t.parent().is_some_and(|n| n.kind() == K::BinderExpr) {
            return false;
        }
        if t.kind() == K::LParen && p.kind() == K::Ident {
            return false;
        }
        if matches!(t.kind(), K::Lt | K::Gt) && Self::in_type_list(t) {
            return false;
        }
        if matches!(p.kind(), K::Lt) && Self::in_type_list(p) {
            return false;
        }
        if p.kind() == K::Gt && Self::in_type_list(p) && matches!(t.kind(), K::Gt | K::LParen) {
            return false;
        }
        true
    }

    /// Where canonical spacing is silent, the author's space is kept:
    /// nowhere today, so the rule set above is total.
    fn space_kept(_p: &SyntaxToken, _t: &SyntaxToken) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::format_module;

    #[test]
    fn corpus_is_a_fixed_point() {
        for name in ["lamp", "lamp_mode", "heater", "system"] {
            let src = std::fs::read_to_string(format!(
                "{}/test_data/valid/{name}.bdl",
                env!("CARGO_MANIFEST_DIR")
            ))
            .expect("corpus");
            let formatted = format_module(&src).expect("parses");
            let again = format_module(&formatted).expect("parses");
            assert_eq!(formatted, again, "{name}: not idempotent");
        }
    }

    #[test]
    fn normalises_spacing_and_indentation_and_keeps_comments() {
        let src = "concept   Tilt:Angle // the tilt\n\n\n\nmapping f:Tilt->Tilt\nf( t )=t/(90 deg)\ncomponent C{\nuse concept Tilt\n      requires x : Tilt @c\n   x() =\n       match x {\n         1 => -1,\n         _ => x\n       }\n// last\n}\n";
        let want = "concept Tilt : Angle  // the tilt\n\nmapping f : Tilt -> Tilt\nf(t) = t / (90 deg)\ncomponent C {\n  use concept Tilt\n  requires x : Tilt @c\n  x() =\n    match x {\n      1 => -1,\n      _ => x\n    }\n  // last\n}\n";
        assert_eq!(format_module(src).expect("parses"), want);
        assert_eq!(format_module(want).expect("parses"), want);
    }

    #[test]
    fn broken_source_is_not_formatted() {
        assert!(format_module("mapping f : A ->\n").is_none());
    }
}
