//! Event-based recursive-descent parser with Pratt expressions.
//!
//! * `grammar` — module and items; `types` — types; `pattern` — patterns;
//!   `expr` — expressions (Pratt).
//! * The parser sees only non-trivia tokens; the [`event::Sink`] re-inserts
//!   trivia when it replays the events into the green tree.
//! * Total: every input yields a tree and a (possibly empty) error list.
//!   Every loop either consumes a token or stops at an anchor its caller
//!   handles, so parsing terminates on arbitrary token streams.

mod event;
mod expr;
mod grammar;
mod pattern;
mod types;

use crate::kind::SyntaxKind;
use crate::lexer::{lex, Token};
use crate::syntax::{Parse, SyntaxError, SyntaxErrorCode};
use bdl_diagnostics::Span;
use event::{Event, Sink};

/// Parse a whole source file (`Module ::= Item* EOF`).
pub fn parse_module(src: &str) -> Parse<crate::ast::Module> {
    run(src, |p| {
        let m = p.start();
        grammar::module(p);
        m.complete(p, SyntaxKind::Module);
    })
}

/// Parse a single expression that must span the whole source (the canvas
/// formula field).  Root kind is [`SyntaxKind::Formula`].
pub fn parse_formula(src: &str) -> Parse<crate::ast::Formula> {
    run(src, |p| {
        let m = p.start();
        grammar::formula(p);
        m.complete(p, SyntaxKind::Formula);
    })
}

fn run<T>(src: &str, root: impl FnOnce(&mut Parser<'_>)) -> Parse<T> {
    let (tokens, lexical_errors) = lex(src);
    let mut p = Parser::new(src, &tokens);
    root(&mut p);
    let events = p.finish();
    let (green, errors) = Sink::new(src, &tokens, lexical_errors).build(events);
    Parse::new(green, errors)
}

pub(crate) struct Parser<'s> {
    src: &'s str,
    tokens: &'s [Token],
    /// Index into `tokens` of the next unconsumed token (may be trivia;
    /// `skip_trivia` is applied lazily by the cursor helpers).
    pos: usize,
    events: Vec<Event>,
}

/// An open node; must be completed or abandoned.
pub(crate) struct Marker {
    event_index: usize,
    bomb: bool,
}

/// A finished node, which can be wrapped by a later node via `precede`.
#[derive(Clone, Copy)]
pub(crate) struct CompletedMarker {
    event_index: usize,
}

impl Marker {
    fn new(event_index: usize) -> Marker {
        Marker {
            event_index,
            bomb: true,
        }
    }

    pub fn complete(mut self, p: &mut Parser<'_>, kind: SyntaxKind) -> CompletedMarker {
        self.bomb = false;
        match &mut p.events[self.event_index] {
            Event::StartNode { kind: k, .. } => *k = kind,
            _ => unreachable!("marker always points at a StartNode"),
        }
        p.events.push(Event::FinishNode);
        CompletedMarker {
            event_index: self.event_index,
        }
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        // A leaked marker is a grammar bug; in release the tombstone is
        // simply skipped by the sink, so this only guards development.
        debug_assert!(!self.bomb, "Marker neither completed nor abandoned");
    }
}

impl CompletedMarker {
    /// Open a new node *around* this completed one.
    pub fn precede(self, p: &mut Parser<'_>) -> Marker {
        let new = p.start();
        if let Event::StartNode { forward_parent, .. } = &mut p.events[self.event_index] {
            *forward_parent = Some(new.event_index - self.event_index);
        }
        new
    }
}

impl<'s> Parser<'s> {
    fn new(src: &'s str, tokens: &'s [Token]) -> Parser<'s> {
        Parser {
            src,
            tokens,
            pos: 0,
            events: Vec::new(),
        }
    }

    fn finish(self) -> Vec<Event> {
        self.events
    }

    // ---- cursor ----------------------------------------------------------

    /// Index of the `n`-th non-trivia token at or after `pos`.
    fn nth_index(&self, n: usize) -> Option<usize> {
        self.tokens[self.pos..]
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.kind.is_trivia())
            .nth(n)
            .map(|(i, _)| self.pos + i)
    }

    pub fn nth(&self, n: usize) -> SyntaxKind {
        self.nth_index(n)
            .map(|i| self.tokens[i].kind)
            .unwrap_or(SyntaxKind::Eof)
    }

    pub fn current(&self) -> SyntaxKind {
        self.nth(0)
    }

    pub fn at(&self, kind: SyntaxKind) -> bool {
        self.current() == kind
    }

    pub fn at_any(&self, kinds: &[SyntaxKind]) -> bool {
        kinds.contains(&self.current())
    }

    pub fn at_eof(&self) -> bool {
        self.at(SyntaxKind::Eof)
    }

    /// Span of the current token (empty at the end of input).
    pub fn current_span(&self) -> Span {
        match self.nth_index(0) {
            Some(i) => self.tokens[i].span,
            None => {
                let end = self.src.len() as u32;
                Span::new(end, end)
            }
        }
    }

    pub fn current_text(&self) -> &'s str {
        match self.nth_index(0) {
            Some(i) => self.tokens[i].text(self.src),
            None => "",
        }
    }

    // ---- consuming -------------------------------------------------------

    /// Consume the current token (never `Eof`).
    pub fn bump(&mut self) {
        if let Some(i) = self.nth_index(0) {
            self.pos = i + 1;
            self.events.push(Event::Token);
        }
    }

    pub fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Consume `kind` or report `message` at the current token without
    /// consuming anything.
    pub fn expect(&mut self, kind: SyntaxKind, message: &str) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.error_expecting(message, &[kind]);
        false
    }

    // ---- nodes -----------------------------------------------------------

    pub fn start(&mut self) -> Marker {
        let index = self.events.len();
        self.events.push(Event::StartNode {
            kind: SyntaxKind::Tombstone,
            forward_parent: None,
        });
        Marker::new(index)
    }

    // ---- errors ----------------------------------------------------------

    fn found(&self) -> String {
        match self.current() {
            SyntaxKind::Eof => "the end of the source".into(),
            SyntaxKind::Ident => format!("the name `{}`", self.current_text()),
            SyntaxKind::Number => format!("the number `{}`", self.current_text()),
            SyntaxKind::Error => format!("`{}`", self.current_text()),
            k => k.describe(),
        }
    }

    pub fn push_error(&mut self, error: SyntaxError) {
        self.events.push(Event::Error(error));
    }

    /// Report at the current token; nothing is consumed.
    pub fn error(&mut self, code: SyntaxErrorCode, message: impl Into<String>) -> &mut Self {
        let e = SyntaxError::new(code, self.current_span(), message, self.found());
        self.push_error(e);
        self
    }

    pub fn error_expecting(&mut self, message: &str, expected: &[SyntaxKind]) {
        let e = SyntaxError::new(
            SyntaxErrorCode::Expected,
            self.current_span(),
            message,
            self.found(),
        )
        .expecting(expected.iter().map(|k| k.describe()));
        self.push_error(e);
    }

    /// Attach a hint to the most recently pushed error.
    pub fn hint(&mut self, hint: impl Into<String>) {
        if let Some(Event::Error(e)) = self.events.last_mut() {
            e.hint = Some(hint.into());
        }
    }

    /// Report and consume the current token inside an `ErrorNode`.
    pub fn error_and_bump(&mut self, code: SyntaxErrorCode, message: impl Into<String>) {
        self.error(code, message);
        self.bump_as_error();
    }

    /// Wrap the current token in an `ErrorNode` (no diagnostic).
    pub fn bump_as_error(&mut self) {
        if self.at_eof() {
            return;
        }
        let m = self.start();
        self.bump();
        m.complete(self, SyntaxKind::ErrorNode);
    }

    /// Report; then consume the current token unless it is an anchor the
    /// caller can resynchronise on.
    pub fn error_recover(
        &mut self,
        code: SyntaxErrorCode,
        message: impl Into<String>,
        anchors: &[SyntaxKind],
    ) {
        self.error(code, message);
        if !self.at_any(anchors) && !self.at_eof() {
            self.bump_as_error();
        }
    }

    /// Consume tokens into one `ErrorNode` until an anchor or the end.
    /// Always consumes at least one token when not at an anchor.
    pub fn skip_until(&mut self, anchors: &[SyntaxKind]) {
        if self.at_eof() || self.at_any(anchors) {
            return;
        }
        let m = self.start();
        while !self.at_eof() && !self.at_any(anchors) {
            self.bump();
        }
        m.complete(self, SyntaxKind::ErrorNode);
    }
}

#[cfg(test)]
mod tests;
