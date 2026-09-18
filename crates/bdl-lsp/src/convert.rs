//! Rendering `bdl-ide` results as LSP types.  Presentation only: nothing
//! here decides what a thing means, which diagnostics exist, or what a
//! rename touches — it maps structures to structures and byte ranges to
//! positions through one [`LineIndex`] per document.

use crate::position::LineIndex;
use bdl_ide::{
    ActionKind, Applicability, CompletionKind, EntityKind, EntityStatus, SemanticAction,
    SemanticCompletion, SemanticEditPlan, SemanticHover, SemanticOperation, SemanticSeverity,
    SemanticSymbol, SemanticToken, TextDiagnostic, TokenKind,
};
use bdl_ide_db::{DocumentId, TextRange};
use lsp_types::{
    CodeAction, CodeActionKind, CompletionItem, CompletionItemKind, CompletionTextEdit, Diagnostic,
    DiagnosticRelatedInformation, DiagnosticSeverity, DiagnosticTag, DocumentSymbol, Hover,
    HoverContents, Location, MarkupContent, MarkupKind, NumberOrString, SemanticTokenModifier,
    SemanticTokenType, SemanticTokensLegend, SymbolKind, TextEdit, Uri, WorkspaceEdit,
};
use std::collections::HashMap;

pub const SOURCE: &str = "bdl";

pub fn severity(s: SemanticSeverity) -> DiagnosticSeverity {
    match s {
        SemanticSeverity::Error => DiagnosticSeverity::ERROR,
        SemanticSeverity::Warning => DiagnosticSeverity::WARNING,
        // Open is never an error in a text editor: information, not a
        // squiggle that says something is wrong.
        SemanticSeverity::Open => DiagnosticSeverity::INFORMATION,
    }
}

/// A placed diagnostic; `resolve` finds the uri and index of a related
/// document.
pub fn diagnostic(
    d: &TextDiagnostic,
    index: &LineIndex,
    resolve: &dyn Fn(DocumentId) -> Option<(Uri, LineIndex)>,
) -> Diagnostic {
    let related: Vec<DiagnosticRelatedInformation> = d
        .related
        .iter()
        .filter_map(|r| {
            let (uri, ix) = resolve(r.document)?;
            Some(DiagnosticRelatedInformation {
                location: Location {
                    uri,
                    range: ix.range(r.range),
                },
                message: r.message.clone(),
            })
        })
        .collect();
    let message = if d.explanation.is_empty() {
        d.message.clone()
    } else {
        format!("{}\n\n{}", d.message, d.explanation)
    };
    Diagnostic {
        range: index.range(d.range),
        severity: Some(severity(d.severity)),
        code: Some(NumberOrString::String(d.code.clone())),
        code_description: None,
        source: Some(SOURCE.into()),
        message,
        related_information: if related.is_empty() {
            None
        } else {
            Some(related)
        },
        tags: if d.severity == SemanticSeverity::Open {
            // Unnecessary is the closest standard tag to "not finished";
            // clients render it faded rather than red.
            Some(vec![DiagnosticTag::UNNECESSARY])
        } else {
            None
        },
        data: None,
    }
}

pub fn hover(h: &SemanticHover) -> Hover {
    let mut md = String::new();
    if let Some(sig) = &h.signature {
        md.push_str(&format!("```bdl\n{sig}\n```\n"));
    } else {
        md.push_str(&format!("**{}**\n", h.title));
    }
    let status = h.status.label();
    if !status.is_empty() {
        md.push_str(&format!("\n*{status}*\n"));
    }
    if let Some(r) = &h.representation {
        md.push_str(&format!("\n{r}\n"));
    }
    if !h.details.is_empty() {
        md.push('\n');
        for d in &h.details {
            md.push_str(&format!("- **{}**: {}\n", d.label, d.value));
        }
    }
    if let Some(t) = &h.semantic_type {
        md.push_str(&format!("\n`{t}`\n"));
    }
    if let Some(e) = &h.explanation {
        md.push_str(&format!("\n{e}\n"));
    }
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md,
        }),
        range: None,
    }
}

pub fn completion_item(c: &SemanticCompletion, index: &LineIndex) -> CompletionItem {
    let kind = match c.kind {
        CompletionKind::Input => CompletionItemKind::VARIABLE,
        CompletionKind::Unit => CompletionItemKind::UNIT,
        CompletionKind::Keyword => CompletionItemKind::KEYWORD,
        CompletionKind::Concept | CompletionKind::Representation => CompletionItemKind::CLASS,
        CompletionKind::Mapping => CompletionItemKind::FUNCTION,
        // A library template writes a plain declaration; the item is a
        // class with the library named beside it, not a snippet.
        CompletionKind::Template => CompletionItemKind::CLASS,
        CompletionKind::Equation => CompletionItemKind::FUNCTION,
        // a binder's or rule's local: a variable of the formula itself
        CompletionKind::Local => CompletionItemKind::VARIABLE,
    };
    CompletionItem {
        label: c.label.clone(),
        label_details: c
            .template
            .as_ref()
            .map(|id| lsp_types::CompletionItemLabelDetails {
                detail: None,
                description: id.split('.').next().map(str::to_owned),
            }),
        kind: Some(kind),
        detail: c.resulting_type.clone(),
        documentation: c
            .documentation
            .clone()
            .map(lsp_types::Documentation::String),
        // Relevance is a byte; invert so higher relevance sorts first.
        sort_text: Some(format!("{:03}-{}", 255 - c.relevance, c.label)),
        filter_text: Some(c.label.clone()),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: index.range(c.replace),
            new_text: c.insert.clone(),
        })),
        ..CompletionItem::default()
    }
}

pub fn symbol_kind(k: EntityKind) -> SymbolKind {
    match k {
        EntityKind::Concept => SymbolKind::CLASS,
        EntityKind::Mapping => SymbolKind::FUNCTION,
        EntityKind::Output => SymbolKind::EVENT,
        EntityKind::Clock => SymbolKind::NAMESPACE,
        EntityKind::Device => SymbolKind::OBJECT,
        EntityKind::Requirement => SymbolKind::FIELD,
        EntityKind::Project => SymbolKind::MODULE,
        EntityKind::Component => SymbolKind::STRUCT,
        EntityKind::Port => SymbolKind::INTERFACE,
        EntityKind::Instance => SymbolKind::VARIABLE,
        EntityKind::Binding => SymbolKind::OPERATOR,
        EntityKind::Export => SymbolKind::PROPERTY,
    }
}

/// A symbol with its declaration and name ranges in `document`.
pub fn document_symbol(
    s: &SemanticSymbol,
    document: DocumentId,
    index: &LineIndex,
) -> Option<DocumentSymbol> {
    let range_for = |role: bdl_ide::EntityRole| {
        s.anchors
            .iter()
            .find(|a| a.document() == Some(document) && a.role == role)
            .and_then(|a| a.text_range())
    };
    let decl = range_for(bdl_ide::EntityRole::Declaration)?;
    let name = range_for(bdl_ide::EntityRole::Name).unwrap_or(decl);
    let detail = if s.status == EntityStatus::Plain || s.status.label().is_empty() {
        s.detail.clone()
    } else {
        format!("{} — {}", s.detail, s.status.label())
    };
    #[allow(deprecated)]
    Some(DocumentSymbol {
        name: s.name.clone(),
        detail: Some(detail),
        kind: symbol_kind(s.kind),
        tags: None,
        deprecated: None,
        range: index.range(decl),
        selection_range: index.range(name),
        children: None,
    })
}

// ---- edits -----------------------------------------------------------------

/// The text side of a plan as a `WorkspaceEdit`.  Model operations have
/// no textual form of their own; when the entity is projected in an open
/// document the plan already carries the text edits that keep the
/// projection in step, and the model follows when the document is
/// committed.  Model-only operations (a committed formula rewrite with no
/// document open) are reported back so the client can see what the edit
/// would not cover.
pub fn workspace_edit(
    plan: &SemanticEditPlan,
    resolve: &dyn Fn(DocumentId) -> Option<(Uri, LineIndex)>,
) -> (WorkspaceEdit, usize) {
    // `Uri` is not a valid map key for clippy (interior mutability in
    // fluent-uri); collect per document id, then build the map once.
    let mut per_document: Vec<(Uri, Vec<TextEdit>)> = Vec::new();
    let mut model_only = 0;
    for op in &plan.operations {
        match op {
            SemanticOperation::Text { document, edits } => {
                let Some((uri, index)) = resolve(*document) else {
                    continue;
                };
                let lsp_edits: Vec<TextEdit> = edits
                    .iter()
                    .map(|e| TextEdit {
                        range: index.range(e.range),
                        new_text: e.new_text.clone(),
                    })
                    .collect();
                match per_document.iter_mut().find(|(u, _)| *u == uri) {
                    Some((_, v)) => v.extend(lsp_edits),
                    None => per_document.push((uri, lsp_edits)),
                }
            }
            SemanticOperation::Model { .. } | SemanticOperation::DraftText { .. } => {
                model_only += 1;
            }
        }
    }
    #[allow(clippy::mutable_key_type)]
    let changes: HashMap<Uri, Vec<TextEdit>> = per_document.into_iter().collect();
    (
        WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        },
        model_only,
    )
}

pub fn code_action(
    a: &SemanticAction,
    diagnostics: Vec<Diagnostic>,
    resolve: &dyn Fn(DocumentId) -> Option<(Uri, LineIndex)>,
) -> CodeAction {
    let kind = match a.kind {
        ActionKind::QuickFix => CodeActionKind::QUICKFIX,
        ActionKind::Refactor => CodeActionKind::REFACTOR,
    };
    let (title, edit, disabled) = match (&a.applicability, &a.plan) {
        (Applicability::Ready, Some(plan)) => {
            let (edit, _) = workspace_edit(plan, resolve);
            (a.title.clone(), Some(edit), None)
        }
        (Applicability::NeedsChoice { options }, _) => (
            format!("{} ({} choices)", a.title, options.len()),
            None,
            Some(lsp_types::CodeActionDisabled {
                reason:
                    "this action needs a choice; make it in BDL Studio or through bdl/previewEdit"
                        .into(),
            }),
        ),
        (Applicability::Blocked { reason }, _) => (
            a.title.clone(),
            None,
            Some(lsp_types::CodeActionDisabled {
                reason: reason.clone(),
            }),
        ),
        (Applicability::Ready, None) => (a.title.clone(), None, None),
    };
    CodeAction {
        title,
        kind: Some(kind),
        diagnostics: if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        },
        edit,
        command: None,
        is_preferred: None,
        disabled,
        data: Some(serde_json::json!({ "id": a.id.0, "explanation": a.explanation })),
    }
}

// ---- semantic tokens ---------------------------------------------------------

pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::TYPE,      // concept
    SemanticTokenType::FUNCTION,  // mapping
    SemanticTokenType::EVENT,     // output
    SemanticTokenType::NAMESPACE, // clock
    SemanticTokenType::CLASS,     // device
    SemanticTokenType::new("unit"),
    SemanticTokenType::KEYWORD,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::ENUM_MEMBER, // constructor
    SemanticTokenType::NUMBER,
    SemanticTokenType::COMMENT,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::MACRO, // equation of the library
];

pub const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::new("unresolved"),
];

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: TOKEN_MODIFIERS.to_vec(),
    }
}

fn token_type(k: TokenKind) -> u32 {
    match k {
        TokenKind::Concept => 0,
        TokenKind::Mapping => 1,
        TokenKind::Output => 2,
        TokenKind::Clock => 3,
        TokenKind::Device => 4,
        TokenKind::Unit => 5,
        TokenKind::Keyword => 6,
        TokenKind::Parameter => 7,
        TokenKind::Constructor => 8,
        TokenKind::Number => 9,
        TokenKind::Comment => 10,
        TokenKind::Operator => 11,
        TokenKind::Equation => 12,
    }
}

/// Delta-encode tokens (LSP `SemanticTokens.data`).  Multi-line tokens
/// (block comments) are split per line because the encoding is per line.
pub fn semantic_tokens(
    tokens: &[SemanticToken],
    index: &LineIndex,
    text: &str,
) -> Vec<lsp_types::SemanticToken> {
    let mut out = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    for t in tokens {
        let start = t.range.start.min(text.len() as u32);
        let end = t.range.end.min(text.len() as u32).max(start);
        let mut piece_start = start;
        loop {
            let s_pos = index.position(piece_start);
            let line_end = text[piece_start as usize..end as usize]
                .find('\n')
                .map(|i| piece_start + i as u32)
                .unwrap_or(end);
            let e_pos = index.position(line_end);
            let length = e_pos.character.saturating_sub(s_pos.character);
            if length > 0 {
                let delta_line = s_pos.line - prev_line;
                let delta_start = if delta_line == 0 {
                    s_pos.character - prev_start
                } else {
                    s_pos.character
                };
                let mut modifiers = 0u32;
                if t.modifiers.declaration {
                    modifiers |= 1;
                }
                if t.modifiers.unresolved {
                    modifiers |= 2;
                }
                out.push(lsp_types::SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type: token_type(t.kind),
                    token_modifiers_bitset: modifiers,
                });
                prev_line = s_pos.line;
                prev_start = s_pos.character;
            }
            if line_end >= end {
                break;
            }
            piece_start = line_end + 1;
        }
    }
    out
}

pub fn text_range_of(index: &LineIndex, r: lsp_types::Range) -> TextRange {
    index.text_range(r)
}
