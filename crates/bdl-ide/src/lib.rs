//! Semantic IDE queries for BDL.
//!
//! Every query here takes an immutable [`AnalysisSnapshot`] and returns a
//! BDL-owned result — no LSP types, no Studio node ids, no Markdown.  The
//! two adapters (`bdld` for Studio, `bdl-lsp` for text editors) *place*
//! and *render* these results; they never recompute meaning.
//!
//! | query | module | result |
//! |---|---|---|
//! | diagnostics | [`diagnostics`] | [`SemanticDiagnostic`]s anchored to `(entity, role)` |
//! | hover / explain | [`hover`], [`explain`] | structured sections |
//! | completion | [`completion`] | [`SemanticCompletion`]s ranked by expected type |
//! | references / navigation | [`references`] | by identity, then placed through projections |
//! | rename, actions, edit plans | [`rename`], [`actions`], [`edit_plan`] | a [`SemanticEditPlan`] any client can apply |
//! | invalidation preview | [`invalidation`] | what an edit would reopen, before it is made |
//! | symbols, tokens | [`symbols`], [`tokens`] | outline and classification for textual surfaces |
//! | draft verdict | [`draft`] | the Studio formula editor's answer |
//! | formula projection, slots, composing | [`formula`] | the Formula Composer's structured view, what a slot expects and fits, the text edit an action makes |
//!
//! Pull-style: nothing here pushes.  Cancellation is a token the caller
//! passes; staleness is the stamp every result carries.  No query panics
//! on incomplete input, a malformed overlay, a deleted entity or a stale
//! projection — it returns an empty or structured-error result.

#![forbid(unsafe_code)]

pub mod actions;
pub mod completion;
mod completion_text;
pub mod diagnostics;
pub mod draft;
pub mod edit_plan;
pub mod explain;
pub mod format;
pub mod formula;
pub mod hover;
pub mod inlay;
pub mod invalidation;
pub mod navigation;
pub mod references;
pub mod rename;
pub mod role;
pub mod symbols;
pub mod tokens;
pub mod virtual_docs;

pub use actions::{
    actions_at, actions_for, ActionChoice, ActionKind, Applicability, SemanticAction,
    SemanticActionId,
};
pub use bdl_ide_db::{
    AnalysisSnapshot, CancellationToken, Cancelled, DocumentId, EntityKind, EntityRef, EntityRole,
    IdeHost, Overlay, OverlayKey, ProjectionAnchor, SnapshotStamp, TextEdit, TextRange,
    VisualElementRef,
};
pub use completion::{
    completion, CompletionContext, CompletionKind, ExpectedType, SemanticCompletion,
};
pub use diagnostics::{
    diagnostics, project_to_document, project_to_visual, DiagnosticScope, DiagnosticSet,
    SemanticAnchor, SemanticDiagnostic, SemanticSeverity, SourceOrigin, SourceSpan, TextDiagnostic,
    TextRelated, VisualDiagnostic,
};
pub use draft::{draft_verdict, DraftVerdict};
pub use edit_plan::{Precondition, SemanticEditPlan, SemanticOperation};
pub use explain::{explain, Explanation, ExplanationSection};
pub use format::format_document;
pub use formula::{
    caret_offset, compose, formula_projection, formula_slot, navigate, render, signature, Caret,
    ComposeOp, ComposeResult, EquationCandidate, FormulaNode, FormulaProjection, FormulaRender,
    Fragment, Motion, NodeKind, ParameterHelp, ReferenceCandidate, Side, SignatureHelp, SlotInfo,
    TypeKindView, TypeView, UnitCandidate,
};
pub use hover::{hover, EntityStatus, HoverDetail, SemanticHover};
pub use inlay::{inlay_hints, InlayHint, InlayKind};
pub use invalidation::{preview_change, Fact, Invalidated, InvalidationPreview, StatusChange};
pub use navigation::{
    definition_at, definition_sites, formula_hover_at, formula_name_at, hover_at, name_at,
    reference_sites, references_at, HoverAt, HoverContent, NameAt,
};
pub use references::{definition_of, entity_at, entity_at_formula, references, ReferenceResult};
pub use rename::{plan_rename, RenameError};
pub use role::{port_backed, provider, relationship_role, Provider, RelationshipRole};
pub use symbols::{document_symbols, symbols, SemanticSymbol};
pub use tokens::{
    formula_tokens, legend, lexical_tokens, semantic_tokens, Legend, SemanticToken, TokenModifiers,
    TokenType, LEGEND_VERSION,
};
pub use virtual_docs::{virtual_document, VirtualDocument, VirtualKind};

use serde::{Deserialize, Serialize};

/// Why a query could not answer.  Never a panic.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum QueryError {
    #[error("the request was cancelled")]
    Cancelled,
    #[error("unknown entity {entity}")]
    UnknownEntity { entity: EntityRef },
    #[error("unknown document {document}")]
    UnknownDocument { document: DocumentId },
    #[error("{reason}")]
    NotApplicable { reason: String },
}

impl From<Cancelled> for QueryError {
    fn from(_: Cancelled) -> QueryError {
        QueryError::Cancelled
    }
}
