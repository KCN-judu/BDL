//! Events and the sink that replays them into a Rowan green tree.
//!
//! The parser never touches `GreenNodeBuilder`; it records what it decided
//! (`StartNode`, `Token`, `FinishNode`, `Error`).  `precede` — wrapping an
//! already-parsed left operand into a `BinaryExpr` or `CallExpr` — is a
//! `forward_parent` link on the earlier `StartNode`, resolved by the sink.
//! The sink also re-inserts trivia, which the parser skipped, so the tree
//! is lossless.

use crate::kind::SyntaxKind;
use crate::lexer::Token;
use crate::syntax::SyntaxError;
use rowan::{GreenNode, GreenNodeBuilder};

#[derive(Debug)]
pub enum Event {
    StartNode {
        kind: SyntaxKind,
        /// Relative index of a later `StartNode` that must open *before*
        /// this one (the node that `precede`d us).
        forward_parent: Option<usize>,
    },
    /// Consume the next non-trivia token (and any trivia before it).
    Token,
    FinishNode,
    Error(SyntaxError),
}

/// Kinds whose leading comments (no blank line between) are attached inside
/// the node rather than left in the parent (`docs/TEXTUAL_SYNTAX.md` §2.4).
fn attaches_leading_comments(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ConceptDecl
            | SyntaxKind::MappingDecl
            | SyntaxKind::EnumDecl
            | SyntaxKind::EnumVariant
            | SyntaxKind::MatchArm
            | SyntaxKind::LetStmt
    )
}

pub struct Sink<'s> {
    src: &'s str,
    tokens: &'s [Token],
    /// Next token (of any kind) not yet emitted.
    pos: usize,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<SyntaxError>,
    /// Open nodes; the root is depth 1.  Trivia may be emitted at depth ≥ 1.
    depth: usize,
}

impl<'s> Sink<'s> {
    pub fn new(src: &'s str, tokens: &'s [Token], lexical_errors: Vec<SyntaxError>) -> Sink<'s> {
        Sink {
            src,
            tokens,
            pos: 0,
            builder: GreenNodeBuilder::new(),
            errors: lexical_errors,
            depth: 0,
        }
    }

    pub fn build(mut self, mut events: Vec<Event>) -> (GreenNode, Vec<SyntaxError>) {
        let mut forward_parents: Vec<SyntaxKind> = Vec::new();
        for i in 0..events.len() {
            match std::mem::replace(
                &mut events[i],
                Event::StartNode {
                    kind: SyntaxKind::Tombstone,
                    forward_parent: None,
                },
            ) {
                Event::StartNode {
                    kind,
                    forward_parent,
                } => {
                    if kind == SyntaxKind::Tombstone && forward_parent.is_none() {
                        continue;
                    }
                    // Walk the forward-parent chain, collecting kinds from the
                    // innermost (this node) to the outermost, then open them
                    // outermost first.
                    forward_parents.push(kind);
                    let mut idx = i;
                    let mut fp = forward_parent;
                    while let Some(rel) = fp {
                        idx += rel;
                        fp = match std::mem::replace(
                            &mut events[idx],
                            Event::StartNode {
                                kind: SyntaxKind::Tombstone,
                                forward_parent: None,
                            },
                        ) {
                            Event::StartNode {
                                kind,
                                forward_parent,
                            } => {
                                forward_parents.push(kind);
                                forward_parent
                            }
                            // The parser only links StartNode events.
                            _ => None,
                        };
                    }
                    for kind in forward_parents.drain(..).rev() {
                        if kind != SyntaxKind::Tombstone {
                            self.start_node(kind);
                        }
                    }
                }
                Event::Token => self.token(),
                Event::FinishNode => {
                    if self.depth == 1 {
                        // Trailing trivia belongs to the root.
                        self.eat_trivia(usize::MAX);
                    }
                    self.depth = self.depth.saturating_sub(1);
                    self.builder.finish_node();
                }
                Event::Error(e) => self.errors.push(e),
            }
        }
        self.errors.sort_by_key(|e| (e.span.start, e.span.end));
        (self.builder.finish(), self.errors)
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        if self.depth == 0 {
            // The root takes everything, leading trivia included.
            self.depth = 1;
            self.builder.start_node(kind.into());
            return;
        }
        self.depth += 1;
        // Pending trivia goes to the parent, except comments directly
        // attached to an item, which go inside it.
        let pending_end = self.pos + self.pending_trivia_len();
        let attach_from = if attaches_leading_comments(kind) {
            self.attached_start(pending_end)
        } else {
            pending_end
        };
        self.eat_trivia(attach_from);
        self.builder.start_node(kind.into());
    }

    fn token(&mut self) {
        self.eat_trivia(usize::MAX);
        if let Some(t) = self.tokens.get(self.pos) {
            self.builder.token(t.kind.into(), t.text(self.src));
            self.pos += 1;
        }
    }

    fn pending_trivia_len(&self) -> usize {
        self.tokens[self.pos..]
            .iter()
            .take_while(|t| t.kind.is_trivia())
            .count()
    }

    /// Emit trivia tokens from `pos` up to (exclusive) `until` or the next
    /// non-trivia token.
    fn eat_trivia(&mut self, until: usize) {
        while self.pos < until {
            match self.tokens.get(self.pos) {
                Some(t) if t.kind.is_trivia() => {
                    self.builder.token(t.kind.into(), t.text(self.src));
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }

    /// Index of the first pending trivia token that should go *inside* the
    /// node about to start: the comments after the last blank line, if any.
    fn attached_start(&self, pending_end: usize) -> usize {
        let pending = &self.tokens[self.pos..pending_end];
        let mut start = pending.len();
        let mut saw_comment = false;
        for (i, t) in pending.iter().enumerate().rev() {
            match t.kind {
                SyntaxKind::Whitespace => {
                    if t.text(self.src).matches('\n').count() >= 2 {
                        break;
                    }
                }
                SyntaxKind::LineComment | SyntaxKind::BlockComment => {
                    saw_comment = true;
                    start = i;
                }
                _ => break,
            }
        }
        if saw_comment {
            self.pos + start
        } else {
            pending_end
        }
    }
}
