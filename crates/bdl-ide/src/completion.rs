//! Completion, independent of any editor's item shape.
//!
//! A completion names a semantic candidate (an input concept, a unit, a
//! keyword, a concept in signature position), the type it would leave
//! the expression with, and a byte-range edit.  Ranking is type-directed
//! where the compiler already knows the answer: inside a formula the
//! expected result is the representation of the mapping's output concept,
//! so inputs and units of that dimension rank above the rest.  When the
//! compiler cannot say (an unbound concept), everything ranks alike — no
//! heroics, but the [`ExpectedType`] slot is where finer inference will
//! plug in.

use bdl_check::pretty;
use bdl_ide_db::{AnalysisSnapshot, DocumentId, EntityKind, EntityRef, EntityRole, TextRange};
use bdl_model::surface::{Definition, Representation};
use bdl_model::{DeclId, Dim};
use serde::{Deserialize, Serialize};

/// Where completion is asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionContext {
    /// Inside a mapping's formula (the Studio definition field); the
    /// source is the snapshot's effective definition, `offset` is
    /// body-relative.
    Formula { mapping: DeclId, offset: u32 },
    /// At a byte offset in a text document.
    Document { document: DocumentId, offset: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionKind {
    /// A signature input, by its display name.
    Input,
    Unit,
    Keyword,
    /// A concept in type position.
    Concept,
    /// A representation name in a `concept … :` position.
    Representation,
    Mapping,
}

/// What the compiler expects at the completion point, when it knows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "expected", rename_all = "snake_case")]
pub enum ExpectedType {
    Quantity { dim: Dim },
    Boolean,
    Count,
    Unknown,
}

impl ExpectedType {
    fn of(r: Option<Representation>) -> ExpectedType {
        match r {
            Some(Representation::Quantity { dim }) => ExpectedType::Quantity { dim },
            Some(Representation::Boolean) => ExpectedType::Boolean,
            Some(Representation::Count) => ExpectedType::Count,
            None => ExpectedType::Unknown,
        }
    }
    fn describe(&self) -> String {
        match self {
            ExpectedType::Quantity { dim } => pretty::describe_dim(*dim),
            ExpectedType::Boolean => "true or false".into(),
            ExpectedType::Count => "a count".into(),
            ExpectedType::Unknown => "unknown".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticCompletion {
    pub label: String,
    pub kind: CompletionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityRef>,
    /// The type the completed expression has, product language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resulting_type: Option<String>,
    /// Replace this range (of the formula or the document) with `insert`.
    pub replace: TextRange,
    pub insert: String,
    /// Higher is better; ties keep label order.
    pub relevance: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

const KEYWORDS: &[&str] = &["if", "then", "else", "true", "false"];

/// Candidates at the context, best first.  Empty when nothing sensible
/// applies; never an error.
pub fn completion(snapshot: &AnalysisSnapshot, ctx: &CompletionContext) -> Vec<SemanticCompletion> {
    let mut out = match ctx {
        CompletionContext::Formula { mapping, offset } => {
            let design = &snapshot.effective().design;
            let Some(block) = design.mappings.get(mapping) else {
                return Vec::new();
            };
            let source = match &block.definition {
                Some(Definition::Formula { source }) => source.as_str(),
                None => "",
            };
            formula_completions(snapshot, *mapping, source, *offset)
        }
        CompletionContext::Document { document, offset } => {
            document_completions(snapshot, *document, *offset)
        }
    };
    out.sort_by(|a, b| {
        b.relevance
            .cmp(&a.relevance)
            .then_with(|| a.label.cmp(&b.label))
    });
    out
}

/// The identifier prefix ending at `offset`, and its range.
fn prefix_at(source: &str, offset: u32) -> (String, TextRange) {
    let end = (offset as usize).min(source.len());
    let end = (0..=end)
        .rev()
        .find(|i| source.is_char_boundary(*i))
        .unwrap_or(0);
    let bytes = source.as_bytes();
    let mut start = end;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    (
        source[start..end].to_owned(),
        TextRange::new(start as u32, end as u32),
    )
}

/// Whether the prefix follows a number (`90 d|`): a unit is expected.
fn after_number(source: &str, range: TextRange) -> bool {
    let before = source[..range.start as usize].trim_end();
    before
        .chars()
        .last()
        .is_some_and(|c| c.is_ascii_digit() || c == '.')
}

fn formula_completions(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    source: &str,
    offset: u32,
) -> Vec<SemanticCompletion> {
    let design = &snapshot.effective().design;
    let Some(block) = design.mappings.get(&mapping) else {
        return Vec::new();
    };
    let (prefix, replace) = prefix_at(source, offset);
    let expected = ExpectedType::of(
        design
            .concepts
            .get(&block.signature.output)
            .and_then(|c| c.representation),
    );
    let matches = |label: &str| {
        prefix.is_empty()
            || label
                .to_ascii_lowercase()
                .starts_with(&prefix.to_ascii_lowercase())
    };
    let mut out = Vec::new();
    let unit_position = after_number(source, replace);

    for c in &block.signature.inputs {
        let Some(concept) = design.concepts.get(c) else {
            continue;
        };
        if !matches(&concept.name) {
            continue;
        }
        let ty = ExpectedType::of(concept.representation);
        let relevance = if unit_position {
            10
        } else {
            rank(&expected, &ty)
        };
        out.push(SemanticCompletion {
            label: concept.name.clone(),
            kind: CompletionKind::Input,
            entity: Some(EntityRef::Concept(*c)),
            resulting_type: Some(ty.describe()),
            replace,
            insert: concept.name.clone(),
            relevance,
            documentation: Some(format!(
                "input of `{}`{}",
                block.name,
                if concept.description.is_empty() {
                    String::new()
                } else {
                    format!(": {}", concept.description)
                }
            )),
        });
    }

    for u in bdl_elab::units::UNITS {
        if !matches(u.name) {
            continue;
        }
        let ty = ExpectedType::Quantity { dim: u.dim };
        let relevance = if unit_position {
            60 + rank(&expected, &ty) / 2
        } else {
            10 + rank(&expected, &ty) / 4
        };
        out.push(SemanticCompletion {
            label: u.name.to_owned(),
            kind: CompletionKind::Unit,
            entity: None,
            resulting_type: Some(pretty::describe_dim(u.dim)),
            replace,
            insert: u.name.to_owned(),
            relevance,
            documentation: Some(format!("unit of {}", pretty::describe_dim(u.dim))),
        });
    }

    if !unit_position {
        for k in KEYWORDS {
            if !matches(k) {
                continue;
            }
            let ty = if *k == "true" || *k == "false" {
                Some(ExpectedType::Boolean)
            } else {
                None
            };
            let relevance = match &ty {
                Some(t) => 20 + rank(&expected, t) / 4,
                None => 20,
            };
            out.push(SemanticCompletion {
                label: (*k).to_owned(),
                kind: CompletionKind::Keyword,
                entity: None,
                resulting_type: ty.map(|t| t.describe()),
                replace,
                insert: (*k).to_owned(),
                relevance,
                documentation: None,
            });
        }
    }
    out
}

/// 0..=100: how well a candidate's type fits the expected one.
fn rank(expected: &ExpectedType, candidate: &ExpectedType) -> u8 {
    match (expected, candidate) {
        (ExpectedType::Unknown, _) | (_, ExpectedType::Unknown) => 50,
        (a, b) if a == b => 100,
        (ExpectedType::Quantity { .. }, ExpectedType::Quantity { .. }) => 70,
        _ => 30,
    }
}

fn document_completions(
    snapshot: &AnalysisSnapshot,
    document: DocumentId,
    offset: u32,
) -> Vec<SemanticCompletion> {
    let Some(doc) = snapshot.document(document) else {
        return Vec::new();
    };
    let source = doc.source.as_str();
    let (prefix, replace) = prefix_at(source, offset);
    // Inside a formula body: delegate with a body-relative offset.
    if let Some(a) = snapshot.projections().anchor_at(document, offset) {
        if a.role == EntityRole::Definition {
            if let (Some(m), Some(r)) = (a.entity.as_mapping(), a.text_range()) {
                let body = &source[r.start as usize..r.end as usize];
                let mut items = formula_completions(snapshot, m, body, offset - r.start);
                for i in &mut items {
                    i.replace = i.replace.offset(r.start);
                }
                return items;
            }
        }
    }
    let matches = |label: &str| {
        prefix.is_empty()
            || label
                .to_ascii_lowercase()
                .starts_with(&prefix.to_ascii_lowercase())
    };
    let before = source[..replace.start as usize].trim_end();
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = &source[line_start..replace.start as usize];
    let mut out = Vec::new();

    // `concept X : |` → representation names; `mapping f : |`, after `->`
    // → concept names.
    let in_type_position = line.contains(':') || line.trim_end().ends_with("->");
    if in_type_position {
        let is_concept_decl = line.trim_start().starts_with("concept");
        if is_concept_decl {
            for name in [
                "Scalar",
                "Bool",
                "Count",
                "Angle",
                "Length",
                "Time",
                "Mass",
                "Current",
                "Temperature",
                "Amount",
                "Luminous",
            ] {
                if matches(name) {
                    out.push(SemanticCompletion {
                        label: name.into(),
                        kind: CompletionKind::Representation,
                        entity: None,
                        resulting_type: bdl_ide_db::textual::representation_named(name)
                            .map(|r| ExpectedType::of(Some(r)).describe()),
                        replace,
                        insert: name.into(),
                        relevance: 50,
                        documentation: None,
                    });
                }
            }
        } else {
            for (e, name) in snapshot.index().entities_of_kind(EntityKind::Concept) {
                if matches(name) {
                    let rep = e
                        .as_concept()
                        .and_then(|c| snapshot.effective().design.concepts.get(&c))
                        .and_then(|c| c.representation);
                    out.push(SemanticCompletion {
                        label: name.to_owned(),
                        kind: CompletionKind::Concept,
                        entity: Some(e),
                        resulting_type: Some(ExpectedType::of(rep).describe()),
                        replace,
                        insert: name.to_owned(),
                        relevance: 50,
                        documentation: None,
                    });
                }
            }
        }
        return out;
    }
    // Item start.
    if line.trim().is_empty() {
        for k in ["concept", "mapping", "enum"] {
            if matches(k) {
                out.push(SemanticCompletion {
                    label: k.into(),
                    kind: CompletionKind::Keyword,
                    entity: None,
                    resulting_type: None,
                    replace,
                    insert: format!("{k} "),
                    relevance: 50,
                    documentation: None,
                });
            }
        }
    }
    out
}
