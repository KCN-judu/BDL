//! Formatting a document: the syntax crate decides the canonical layout
//! (`bdl_syntax::format`); the service only turns it into one edit and
//! refuses to touch source that does not parse.

use bdl_ide_db::{AnalysisSnapshot, DocumentId, TextEdit, TextRange};

/// The edits that format `document`: one whole-document replacement, or
/// none when it is already canonical or does not parse cleanly.
pub fn format_document(snapshot: &AnalysisSnapshot, document: DocumentId) -> Vec<TextEdit> {
    let Some(doc) = snapshot.document(document) else {
        return Vec::new();
    };
    let Some(formatted) = bdl_syntax::format::format_module(&doc.source) else {
        return Vec::new();
    };
    if formatted == doc.source {
        return Vec::new();
    }
    vec![TextEdit::replace(
        TextRange::new(0, doc.source.len() as u32),
        formatted,
    )]
}
