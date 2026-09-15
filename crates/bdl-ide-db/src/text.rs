//! Text documents and byte ranges.
//!
//! The IDE core speaks UTF-8 byte offsets, as the compiler does
//! (`bdl_diagnostics::Span`).  Line/character positions in any encoding are
//! a transport concern of the LSP adapter and never enter this crate.
//!
//! A document is identified by a [`DocumentId`] allocated by the host; its
//! [`DocumentUri`] is opaque here.  Not every document is a file: generated
//! projections (`bdl-core://`, `bdl-explain://`) are documents too, and the
//! host only ever asks whether a document is editable.

use bdl_diagnostics::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A half-open byte range `start..end` into a document or formula source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    pub const fn new(start: u32, end: u32) -> TextRange {
        TextRange { start, end }
    }
    pub const fn empty_at(offset: u32) -> TextRange {
        TextRange {
            start: offset,
            end: offset,
        }
    }
    pub fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    pub fn contains(self, offset: u32) -> bool {
        self.start <= offset && offset <= self.end
    }
    /// The same range shifted right by `by` bytes (a formula span placed
    /// inside the document that holds the formula).
    pub fn offset(self, by: u32) -> TextRange {
        TextRange {
            start: self.start.saturating_add(by),
            end: self.end.saturating_add(by),
        }
    }
    /// Clamp to a document of `len` bytes so a stale span never indexes
    /// past the end.
    pub fn clamp_to(self, len: u32) -> TextRange {
        let start = self.start.min(len);
        TextRange {
            start,
            end: self.end.min(len).max(start),
        }
    }
    pub fn to_span(self) -> Span {
        Span::new(self.start, self.end)
    }
}

impl From<Span> for TextRange {
    fn from(s: Span) -> TextRange {
        TextRange::new(s.start, s.end)
    }
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A replacement of one byte range by new text, in one document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: TextRange,
    pub new_text: String,
}

impl TextEdit {
    pub fn replace(range: TextRange, new_text: impl Into<String>) -> TextEdit {
        TextEdit {
            range,
            new_text: new_text.into(),
        }
    }

    /// Apply a set of non-overlapping edits to `source`.  Edits are applied
    /// from the end so earlier offsets stay valid; overlapping edits are
    /// rejected rather than guessed at.
    pub fn apply_all(source: &str, edits: &[TextEdit]) -> Result<String, EditConflict> {
        let mut sorted: Vec<&TextEdit> = edits.iter().collect();
        sorted.sort_by_key(|e| (e.range.start, e.range.end));
        for w in sorted.windows(2) {
            if w[1].range.start < w[0].range.end {
                return Err(EditConflict {
                    first: w[0].range,
                    second: w[1].range,
                });
            }
        }
        let len = source.len() as u32;
        let mut out = source.to_owned();
        for e in sorted.into_iter().rev() {
            let r = e.range.clamp_to(len);
            if !out.is_char_boundary(r.start as usize) || !out.is_char_boundary(r.end as usize) {
                return Err(EditConflict {
                    first: r,
                    second: r,
                });
            }
            out.replace_range(r.start as usize..r.end as usize, &e.new_text);
        }
        Ok(out)
    }
}

/// Two edits of one document overlap (or one falls inside a character).
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("text edits {first} and {second} overlap")]
pub struct EditConflict {
    pub first: TextRange,
    pub second: TextRange,
}

/// Host-allocated identity of an open document.  Stable while the document
/// is known to the host; never reused within one host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(pub u32);

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "doc#{}", self.0)
    }
}

/// The client's name for a document.  Opaque to the core except for its
/// scheme, which says whether the document is authored text or a
/// generated, read-only projection.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentUri(pub String);

impl DocumentUri {
    pub fn new(uri: impl Into<String>) -> DocumentUri {
        DocumentUri(uri.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn scheme(&self) -> Option<&str> {
        self.0.split_once(':').map(|(s, _)| s)
    }
    pub fn kind(&self) -> DocumentKind {
        match self.scheme() {
            Some("bdl-core") | Some("bdl-explain") | Some("bdl-generated") => {
                DocumentKind::Generated
            }
            _ => DocumentKind::Authored,
        }
    }
}

impl fmt::Display for DocumentUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Whether a document is something the designer writes or something the
/// tool renders.  Generated documents are never editable sources of truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    /// A `.bdl` file (or any text the client edits).
    Authored,
    /// A read-only projection (`bdl-core://`, `bdl-explain://`,
    /// `bdl-generated://`).
    Generated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_apply_from_the_end_and_refuse_overlap() {
        let src = "concept Tilt : Angle";
        let edits = vec![
            TextEdit::replace(TextRange::new(8, 12), "Lean"),
            TextEdit::replace(TextRange::new(15, 20), "Scalar"),
        ];
        assert_eq!(
            TextEdit::apply_all(src, &edits).unwrap(),
            "concept Lean : Scalar"
        );
        let bad = vec![
            TextEdit::replace(TextRange::new(0, 10), "x"),
            TextEdit::replace(TextRange::new(5, 12), "y"),
        ];
        assert!(TextEdit::apply_all(src, &bad).is_err());
    }

    #[test]
    fn edits_never_split_a_character() {
        let src = "// ångström\nconcept T";
        let edit = vec![TextEdit::replace(TextRange::new(4, 5), "a")];
        assert!(TextEdit::apply_all(src, &edit).is_err());
    }

    #[test]
    fn uri_scheme_decides_document_kind() {
        assert_eq!(
            DocumentUri::new("file:///p/lamp.bdl").kind(),
            DocumentKind::Authored
        );
        assert_eq!(
            DocumentUri::new("bdl-core://lamp/dimByTilt").kind(),
            DocumentKind::Generated
        );
        assert_eq!(DocumentUri::new("untitled-1").scheme(), None);
    }
}
