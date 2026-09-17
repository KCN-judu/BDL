//! Structured diagnostics.
//!
//! A compiler used by an editor keeps working on incomplete input, so every
//! pass *returns* diagnostics instead of failing.  A diagnostic names the
//! entity it belongs to, where in the source it applies, and says what is
//! wrong in the designer's vocabulary; the kernel vocabulary goes into
//! `technical`, for the explanation view.
//!
//! Ordering is deterministic: [`sort_diagnostics`] orders by entity, span,
//! then code, so analysis output never depends on traversal order.

#![forbid(unsafe_code)]

use bdl_model::{DeclId, SemanticId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Byte range in a formula source (`start..end`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Span {
        Span { start, end }
    }
    pub fn to(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    pub fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    /// Something open, not wrong: an unresolved mapping, an unbound concept.
    Info,
}

/// What a diagnostic is about.  Ordered so project-level notes come first,
/// then concepts, then mappings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entity {
    Project,
    Concept { id: SemanticId },
    Mapping { id: DeclId },
}

/// A stable machine-readable code such as `formula.parse.unexpected_token`.
/// The set is closed per pass and documented in `docs/architecture/compiler-pipeline.md`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Code(pub String);

impl Code {
    pub fn new(code: &'static str) -> Code {
        Code(code.to_owned())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for Code {
    fn from(c: &'static str) -> Code {
        Code::new(c)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: Code,
    pub severity: Severity,
    pub entity: Entity,
    /// Where in the entity's formula source, if it has one.
    pub span: Option<Span>,
    /// One sentence, product language.
    pub message: String,
    /// Why it matters / what the rule is, product language.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub explanation: String,
    /// Kernel-level detail for the explanation view (types, rules).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub technical: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<String>,
}

impl Diagnostic {
    pub fn new(
        code: impl Into<Code>,
        severity: Severity,
        entity: Entity,
        message: impl Into<String>,
    ) -> Self {
        Diagnostic {
            code: code.into(),
            severity,
            entity,
            span: None,
            message: message.into(),
            explanation: String::new(),
            technical: String::new(),
            fixes: Vec::new(),
        }
    }
    pub fn error(code: impl Into<Code>, entity: Entity, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Error, entity, message)
    }
    pub fn warning(code: impl Into<Code>, entity: Entity, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Warning, entity, message)
    }
    pub fn info(code: impl Into<Code>, entity: Entity, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Info, entity, message)
    }
    #[must_use]
    pub fn at(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    #[must_use]
    pub fn explain(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }
    #[must_use]
    pub fn technical(mut self, technical: impl Into<String>) -> Self {
        self.technical = technical.into();
        self
    }
    #[must_use]
    pub fn fix(mut self, fix: impl Into<String>) -> Self {
        self.fixes.push(fix.into());
        self
    }
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

fn order(a: &Diagnostic, b: &Diagnostic) -> Ordering {
    a.entity
        .cmp(&b.entity)
        .then_with(|| a.span.cmp(&b.span))
        .then_with(|| a.code.cmp(&b.code))
        .then_with(|| a.message.cmp(&b.message))
}

/// The documented stable order: entity, span, code, message.
pub fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(order);
}

pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(Diagnostic::is_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_is_entity_then_span_then_code() {
        let m = Entity::Mapping {
            id: DeclId::from_raw(1),
        };
        let c = Entity::Concept {
            id: SemanticId::from_raw(9),
        };
        let mut v = vec![
            Diagnostic::error("z", m, "late").at(Span::new(5, 6)),
            Diagnostic::error("a", m, "early").at(Span::new(0, 1)),
            Diagnostic::info("k", c, "concept"),
            Diagnostic::error("b", m, "same span, later code").at(Span::new(0, 1)),
        ];
        sort_diagnostics(&mut v);
        let codes: Vec<&str> = v.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["k", "a", "b", "z"]);
    }
}
