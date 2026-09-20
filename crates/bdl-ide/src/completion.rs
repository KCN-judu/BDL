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
use bdl_model::surface::{Definition, Design, Representation};
use bdl_model::{DeclId, Dim, SemanticId};
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
    /// A Standard Library Concept item: accepting it writes an
    /// ordinary declaration (`AmbientLight : Illuminance`); `template`
    /// carries the library id for clients that show provenance.
    Template,
    /// An equation of the library (`min`, `any`, `clamp`): applied to
    /// values, never a value; inlined at analysis time.
    Equation,
    /// A local of the formula in scope here: a binder's or rule's
    /// parameter, a pattern's name (P11).
    Local,
}

/// What the compiler expects at the completion point, when it knows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "expected", rename_all = "snake_case")]
pub enum ExpectedType {
    Quantity {
        dim: Dim,
    },
    Boolean,
    Count,
    /// A value that may be absent, a collection or a grouped value, in
    /// the designer's words.
    Structured {
        description: String,
    },
    Unknown,
}

impl ExpectedType {
    pub(crate) fn of(r: Option<&Representation>) -> ExpectedType {
        match r {
            Some(Representation::Quantity { dim }) => ExpectedType::Quantity { dim: *dim },
            Some(Representation::Boolean) => ExpectedType::Boolean,
            Some(Representation::Count) => ExpectedType::Count,
            Some(other) => ExpectedType::Structured {
                description: describe_representation(other),
            },
            None => ExpectedType::Unknown,
        }
    }
    pub(crate) fn describe(&self) -> String {
        match self {
            ExpectedType::Quantity { dim } => pretty::describe_dim(*dim),
            ExpectedType::Boolean => "true or false".into(),
            ExpectedType::Count => "a count".into(),
            ExpectedType::Structured { description } => description.clone(),
            ExpectedType::Unknown => "unknown".into(),
        }
    }
}

/// A representation in the designer's words: "an angle", "a collection of
/// temperatures", "a grouped value (a temperature and a dimensionless
/// quantity)".
pub fn describe_representation(r: &Representation) -> String {
    match r {
        Representation::Quantity { dim } => pretty::describe_dim(*dim),
        Representation::Boolean => "true or false".into(),
        Representation::Count => "a count".into(),
        Representation::Optional { inner } => {
            format!("an optional value ({})", describe_representation(inner))
        }
        Representation::List { element } => {
            format!(
                "a collection of values ({})",
                describe_representation(element)
            )
        }
        Representation::Pair { first, second } => format!(
            "a grouped value ({} and {})",
            describe_representation(first),
            describe_representation(second)
        ),
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
    /// The library template this completion instantiates, for
    /// [`CompletionKind::Template`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

const KEYWORDS: &[&str] = &["if", "then", "else", "true", "false", "in"];

/// The hover and completion text of an equation: its designer-facing
/// summary, its shape and what its values must be able to do.
pub fn equation_documentation(e: &bdl_equations::Entry) -> String {
    use bdl_equations::Cap;
    let needs: Vec<&str> = e
        .scheme
        .caps
        .iter()
        .map(|(_, c)| match c {
            Cap::Ord => "values of an ordered kind: a quantity, or a concept declared ordered",
            Cap::Eq | Cap::Data => {
                "values that can be compared for equality (any value that is not a rule)"
            }
        })
        .collect();
    let mut s = format!("{} — {}", e.shape(), e.summary);
    if !needs.is_empty() {
        s.push_str(&format!(" Needs {}.", needs.join("; ")));
    }
    s
}

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
                Some(Definition::Formula { source })
                | Some(Definition::ScopedFormula { source, .. }) => source.as_str(),
                // A reference has no formula text to complete in.
                Some(Definition::Reference { .. }) | None => "",
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
            .and_then(|c| c.representation.as_ref()),
    );
    let matches = |label: &str| {
        prefix.is_empty()
            || label
                .to_ascii_lowercase()
                .starts_with(&prefix.to_ascii_lowercase())
    };
    let mut out = Vec::new();
    let unit_position = after_number(source, replace);

    // A formula pinned to a component scope names its inputs and the
    // relationships it may call by the component's spellings, not by
    // whatever the flat design calls them (or another instance's copies).
    let scope = match &block.definition {
        Some(Definition::ScopedFormula { scope, .. }) => Some(scope),
        _ => None,
    };
    for (i, c) in block.signature.inputs.iter().enumerate() {
        let Some(concept) = design.concepts.get(c) else {
            continue;
        };
        // The name the body uses for this input: its textual parameter
        // name when it has one, the concept's name otherwise.
        let name = scope
            .and_then(|s| s.inputs.get(i).cloned())
            .or_else(|| block.parameters.get(i).filter(|p| !p.is_empty()).cloned())
            .unwrap_or_else(|| concept.name.clone());
        if !matches(&name) {
            continue;
        }
        let ty = ExpectedType::of(concept.representation.as_ref());
        let relevance = if unit_position {
            10
        } else {
            rank(&expected, &ty)
        };
        out.push(SemanticCompletion {
            label: name.clone(),
            kind: CompletionKind::Input,
            entity: Some(EntityRef::Concept(*c)),
            resulting_type: Some(ty.describe()),
            replace,
            insert: name,
            relevance,
            template: None,
            documentation: Some(format!(
                "input of `{}` ({}){}",
                block.name,
                concept.name,
                if concept.description.is_empty() {
                    String::new()
                } else {
                    format!(": {}", concept.description)
                }
            )),
        });
    }

    // Other relationships (DI-17): a value when its domain is `()`, a call
    // to complete when it has inputs.  The mapping's own name is not offered
    // (an instantaneous self-reference is a cycle; memory uses `delay`).
    if !unit_position {
        let visible: Vec<(String, &bdl_model::surface::MappingBlock)> = match scope {
            Some(s) => s
                .mappings
                .iter()
                .filter_map(|(name, id)| design.mappings.get(id).map(|m| (name.clone(), m)))
                .collect(),
            None => design
                .mappings
                .values()
                .map(|m| (m.name.clone(), m))
                .collect(),
        };
        for (name, m) in visible {
            if m.id == mapping || !matches(&name) {
                continue;
            }
            let params: Vec<String> = m
                .signature
                .inputs
                .iter()
                .map(|c| {
                    design
                        .concepts
                        .get(c)
                        .map(|c| c.name.clone())
                        .unwrap_or_default()
                })
                .collect();
            let ty = ExpectedType::of(
                design
                    .concepts
                    .get(&m.signature.output)
                    .and_then(|c| c.representation.as_ref()),
            );
            let callable = !m.signature.is_unit_domain();
            out.push(SemanticCompletion {
                label: if callable {
                    format!("{}({})", name, params.join(", "))
                } else {
                    name.clone()
                },
                kind: CompletionKind::Mapping,
                entity: Some(EntityRef::Mapping(m.id)),
                resulting_type: Some(ty.describe()),
                replace,
                insert: if callable {
                    format!("{name}(")
                } else {
                    name.clone()
                },
                relevance: rank(&expected, &ty).saturating_sub(5),
                documentation: Some(if callable {
                    format!("relationship: reads {}", params.join(", "))
                } else {
                    "relationship without inputs: its current value".to_string()
                }),
                template: None,
            });
        }
        for (name, doc) in [
            ("delay", "delay(init, value): the value at the previous activation; init at the first"),
            ("sync", "sync(domain, init, value): the value at the source domain's last activation strictly before now"),
        ] {
            if !matches(name) || !block.signature.is_unit_domain() {
                continue;
            }
            out.push(SemanticCompletion {
                label: format!("{name}(…)"),
                kind: CompletionKind::Keyword,
                entity: None,
                resulting_type: None,
                replace,
                insert: format!("{name}("),
                relevance: 15,
                documentation: Some(doc.to_string()),
                template: None,
            });
        }
    }

    // The locals in scope at the point: a binder's element inside its
    // body, a rule's parameters, a pattern's names.  They shadow the
    // design, so they rank above its names.
    if !unit_position {
        for local in locals_in_scope(source, offset) {
            if !matches(&local) {
                continue;
            }
            out.push(SemanticCompletion {
                label: local.clone(),
                kind: CompletionKind::Local,
                entity: None,
                resulting_type: None,
                replace,
                insert: local,
                relevance: 70,
                documentation: Some("local of this formula: one element, or a rule's input".into()),
                template: None,
            });
        }
    }

    // The natural forms, when there is a collection to read: `all reading
    // in readings: …` — with the one collection in scope filled in when
    // there is exactly one, and a readable local name.
    if !unit_position {
        let collections: Vec<String> = visible_collections(design, block, scope);
        if !collections.is_empty() {
            let coll = if collections.len() == 1 {
                collections[0].clone()
            } else {
                String::new()
            };
            let taken: Vec<String> = design.mappings.values().map(|m| m.name.clone()).collect();
            let local = crate::formula::fresh_local_name(&coll, &taken);
            for (word, doc, ty) in [
                (
                    "all",
                    "whether every element satisfies the condition",
                    Some(ExpectedType::Boolean),
                ),
                (
                    "any",
                    "whether some element satisfies the condition",
                    Some(ExpectedType::Boolean),
                ),
                ("map", "the collection with every element transformed", None),
                ("filter", "the elements that satisfy the condition", None),
            ] {
                if !matches(word) {
                    continue;
                }
                let relevance = match &ty {
                    Some(t) => 30 + rank(&expected, t) / 3,
                    None => 30,
                };
                out.push(SemanticCompletion {
                    label: format!(
                        "{word} {local} in {}: …",
                        if coll.is_empty() { "collection" } else { &coll }
                    ),
                    kind: CompletionKind::Keyword,
                    entity: None,
                    resulting_type: ty.map(|t| t.describe()),
                    replace,
                    insert: format!(
                        "{word} {local} in {coll}{}",
                        if coll.is_empty() { "" } else { ": " }
                    ),
                    relevance,
                    documentation: Some(format!(
                        "{doc}: `{word} {local} in {}: …` reads each {local} of the collection",
                        if coll.is_empty() { "xs" } else { &coll }
                    )),
                    template: None,
                });
            }
        }
    }

    // The equation library, in the designer's words: `min(a, b)` — the
    // smaller of two values of the same ordered kind.  A relationship of
    // the design with the same name is offered above and wins.  Not after
    // a number, where only a unit can follow.
    for e in bdl_equations::entries() {
        if unit_position || !matches(e.name) || design.mappings.values().any(|m| m.name == e.name) {
            continue;
        }
        out.push(SemanticCompletion {
            label: e.shape(),
            kind: CompletionKind::Equation,
            entity: None,
            resulting_type: None,
            replace,
            insert: format!("{}(", e.name),
            relevance: 12,
            documentation: Some(equation_documentation(e)),
            template: None,
        });
    }

    for u in bdl_elab::units::UNITS {
        if !matches(u.symbol) {
            continue;
        }
        let ty = ExpectedType::Quantity { dim: u.dim };
        let relevance = if unit_position {
            60 + rank(&expected, &ty) / 2
        } else {
            10 + rank(&expected, &ty) / 4
        };
        out.push(SemanticCompletion {
            label: u.symbol.to_owned(),
            kind: CompletionKind::Unit,
            entity: None,
            resulting_type: Some(pretty::describe_dim(u.dim)),
            replace,
            insert: u.symbol.to_owned(),
            relevance,
            template: None,
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
                template: None,
            });
        }
    }
    out
}

/// The locals a name at `offset` could be: parameters of the binders and
/// rules enclosing the point (body only for a binder), pattern names of
/// enclosing match arms, `let`s earlier in enclosing blocks — innermost
/// first, each name once.
pub(crate) fn locals_in_scope(source: &str, offset: u32) -> Vec<String> {
    use bdl_syntax::ast::{self, AstNode};
    use bdl_syntax::SyntaxKind;
    let parse = bdl_syntax::parse_formula(source);
    let root = parse.syntax_node();
    let at = offset.min(source.len() as u32);
    let holds = |n: &bdl_syntax::SyntaxNode| {
        let r = n.text_range();
        u32::from(r.start()) <= at && at <= u32::from(r.end())
    };
    // the innermost node whose range holds the offset (a node ending at
    // the offset counts, so a name being typed sees its scope)
    let mut node = root.descendants().filter(holds).last();
    let mut out: Vec<String> = Vec::new();
    let mut push = |n: String| {
        if !n.is_empty() && !out.contains(&n) {
            out.push(n);
        }
    };
    while let Some(n) = node {
        match n.kind() {
            SyntaxKind::BinderExpr => {
                if let Some(b) = ast::BinderExpr::cast(n.clone()) {
                    let in_body = match b.body() {
                        Some(body) => u32::from(body.syntax().text_range().start()) <= at,
                        // no body yet: right after the colon counts
                        None => b
                            .syntax()
                            .children_with_tokens()
                            .filter_map(|el| el.into_token())
                            .any(|t| {
                                t.kind() == SyntaxKind::Colon
                                    && u32::from(t.text_range().end()) <= at
                            }),
                    };
                    if in_body {
                        if let Some(p) = b.param() {
                            push(p.as_str());
                        }
                    }
                }
            }
            SyntaxKind::LambdaExpr => {
                if let Some(l) = ast::LambdaExpr::cast(n.clone()) {
                    for p in l.params() {
                        push(p.as_str());
                    }
                }
            }
            SyntaxKind::MatchArm => {
                if let Some(a) = ast::MatchArm::cast(n.clone()) {
                    let in_body = a
                        .body()
                        .is_some_and(|body| u32::from(body.syntax().text_range().start()) <= at);
                    if in_body {
                        if let Some(p) = a.pattern() {
                            for ip in p.syntax().descendants().filter_map(ast::IdentPattern::cast) {
                                if let Some(name) = ip.name() {
                                    push(name.as_str());
                                }
                            }
                        }
                    }
                }
            }
            SyntaxKind::BlockExpr => {
                if let Some(b) = ast::BlockExpr::cast(n.clone()) {
                    let earlier: Vec<ast::LetStmt> = b
                        .lets()
                        .filter(|l| u32::from(l.syntax().text_range().end()) <= at)
                        .collect();
                    for l in earlier.into_iter().rev() {
                        if let Some(p) = l.pattern() {
                            for ip in p.syntax().descendants().filter_map(ast::IdentPattern::cast) {
                                if let Some(name) = ip.name() {
                                    push(name.as_str());
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        node = n.parent();
    }
    out
}

/// The collections a formula can read by name: inputs and relationships
/// without inputs whose representation is a list, by the spelling the
/// body uses.
fn visible_collections(
    design: &Design,
    block: &bdl_model::surface::MappingBlock,
    scope: Option<&bdl_model::surface::FormulaScope>,
) -> Vec<String> {
    let is_list = |c: &SemanticId| {
        design
            .concepts
            .get(c)
            .and_then(|c| c.representation.as_ref())
            .is_some_and(|r| matches!(r, Representation::List { .. }))
    };
    let mut out = Vec::new();
    for (i, c) in block.signature.inputs.iter().enumerate() {
        if !is_list(c) {
            continue;
        }
        let name = scope
            .and_then(|s| s.inputs.get(i).cloned())
            .or_else(|| block.parameters.get(i).filter(|p| !p.is_empty()).cloned())
            .or_else(|| design.concepts.get(c).map(|c| c.name.clone()));
        if let Some(n) = name {
            out.push(n);
        }
    }
    let visible: Vec<(String, &bdl_model::surface::MappingBlock)> = match scope {
        Some(s) => s
            .mappings
            .iter()
            .filter_map(|(name, id)| design.mappings.get(id).map(|m| (name.clone(), m)))
            .collect(),
        None => design
            .mappings
            .values()
            .map(|m| (m.name.clone(), m))
            .collect(),
    };
    for (name, m) in visible {
        if m.id != block.id && m.signature.is_unit_domain() && is_list(&m.signature.output) {
            out.push(name);
        }
    }
    out
}

/// The relationship a definition line defines and where its body text
/// starts, when the line up to `at` is `name(…) = …` and `name` is a
/// relationship in the scope the line is in (the system's, or the
/// enclosing component body's, whose first flattened copy stands for the
/// body).  A line the model has no relationship for is not a definition.
fn defining_line(
    world: &bdl_ide_db::TextWorld,
    snapshot: &AnalysisSnapshot,
    document: DocumentId,
    source: &str,
    at: u32,
) -> Option<(DeclId, u32)> {
    let line_start = source[..at as usize]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let line = &source[line_start..at as usize];
    let eq = line.find('=')?;
    let head = line[..eq].trim();
    let head = head
        .split(|c: char| c == '(' || c.is_whitespace())
        .next()?
        .trim();
    if head.is_empty() || line[..eq].contains(':') {
        return None;
    }
    if matches!(
        head,
        "bind" | "drive" | "instance" | "device" | "export" | "concept" | "mapping" | "output"
    ) {
        return None;
    }
    let body_start = (line_start + eq + 1) as u32;
    let system = &world.system;
    let mapping = match world.component_at(document, at) {
        None => system.base.mappings.values().find(|m| m.name == head)?.id,
        Some(c) => {
            let comp = system.components.get(&c)?;
            let d = comp.body.mappings.values().find(|m| m.name == head)?.id;
            let flat = bdl_ide_db::workspace::flat_entities(
                system,
                bdl_text::TextEntity::BodyMapping(c, d),
            );
            let m = flat.into_iter().find_map(|e| e.as_mapping())?;
            // the copy names the body's own spellings only through the
            // scope its last built definition pinned; without one the
            // body's names come from the text path
            let block = snapshot.effective().design.mappings.get(&m)?;
            if !matches!(block.definition, Some(Definition::ScopedFormula { .. })) {
                return None;
            }
            m
        }
    };
    snapshot
        .effective()
        .design
        .mappings
        .contains_key(&mapping)
        .then_some((mapping, body_start))
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
    // Inside a formula body that built: the formula engine, with a
    // body-relative offset (the body anchor encloses the smaller name
    // anchors, so it is looked for by role, not as the innermost).
    let body_anchor = snapshot
        .projections()
        .document_anchors(document)
        .iter()
        .filter(|a| a.role == EntityRole::Definition)
        .find(|a| a.text_range().is_some_and(|r| r.contains(offset)));
    if let Some(a) = body_anchor {
        if let (Some(m), Some(r)) = (a.entity.as_mapping(), a.text_range()) {
            let body = &source[r.start as usize..r.end as usize];
            let mut items = formula_completions(snapshot, m, body, offset - r.start);
            for i in &mut items {
                i.replace = i.replace.offset(r.start);
            }
            return items;
        }
    }
    // A text workspace decides scope from the authored system.
    if let Some(world) = snapshot.text() {
        // A definition line whose body has not built yet (`level() = |`,
        // an unclosed call): still the formula engine, for the relationship
        // the line defines, over the text after `=`.
        if let Some((m, body_start)) =
            defining_line(world, snapshot, document, source, replace.start)
        {
            let body = &source[body_start as usize..];
            let end = body
                .find('\n')
                .map(|i| i as u32)
                .unwrap_or(body.len() as u32);
            let body = &body[..end as usize];
            let mut items = formula_completions(snapshot, m, body, (offset - body_start).min(end));
            for i in &mut items {
                i.replace = i.replace.offset(body_start);
            }
            return items;
        }
        return crate::completion_text::text_completions(world, document, source, prefix, replace);
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

    // `concept Amb|` → the Standard Library's Concept items, from the same
    // data Studio's library panel reads.  Accepting one writes ordinary
    // syntax; nothing about the template stays in the source.
    let trimmed = line.trim_start();
    if !line.contains(':') && trimmed.starts_with("concept") && trimmed.len() > "concept".len() {
        let names_taken: Vec<&str> = snapshot
            .index()
            .entities_of_kind(EntityKind::Concept)
            .map(|(_, n)| n)
            .collect();
        for t in bdl_library::LibrarySet::shared().templates() {
            let by_name = matches(&t.default_name) || matches(&t.display_name);
            let by_keyword = !prefix.is_empty()
                && t.keywords.iter().any(|k| {
                    k.to_ascii_lowercase()
                        .starts_with(&prefix.to_ascii_lowercase())
                });
            if !(by_name || by_keyword) {
                continue;
            }
            let insert = match t.type_name() {
                Some(ty) => format!("{} : {}", t.default_name, ty),
                None => t.default_name.clone(),
            };
            out.push(SemanticCompletion {
                label: insert.clone(),
                kind: CompletionKind::Template,
                entity: None,
                resulting_type: Some(ExpectedType::of(t.representation().as_ref()).describe()),
                replace,
                insert,
                // A name already in the project ranks below the rest: the
                // designer probably means a second, differently named one.
                relevance: if names_taken.contains(&t.default_name.as_str()) {
                    30
                } else if by_name {
                    60
                } else {
                    40
                },
                documentation: Some(format!("{} — {}", t.display_name, t.description)),
                template: Some(t.id.clone()),
            });
        }
        return out;
    }
    // `concept X : |` → representation names; `mapping f : |`, after `->`
    // → concept names.
    let in_type_position = line.contains(':') || line.trim_end().ends_with("->");
    if in_type_position {
        let is_concept_decl = line.trim_start().starts_with("concept");
        if is_concept_decl {
            for name in bdl_ide_db::textual::representation_names() {
                if matches(name) {
                    out.push(SemanticCompletion {
                        label: name.into(),
                        kind: CompletionKind::Representation,
                        entity: None,
                        resulting_type: bdl_ide_db::textual::representation_named(name)
                            .map(|r| ExpectedType::of(Some(&r)).describe()),
                        replace,
                        insert: name.into(),
                        relevance: 50,
                        documentation: None,
                        template: None,
                    });
                }
            }
        } else {
            for (e, name) in snapshot.index().entities_of_kind(EntityKind::Concept) {
                if matches(name) {
                    let rep = e
                        .as_concept()
                        .and_then(|c| snapshot.effective().design.concepts.get(&c))
                        .and_then(|c| c.representation.clone());
                    out.push(SemanticCompletion {
                        label: name.to_owned(),
                        kind: CompletionKind::Concept,
                        entity: Some(e),
                        resulting_type: Some(ExpectedType::of(rep.as_ref()).describe()),
                        replace,
                        insert: name.to_owned(),
                        relevance: 50,
                        documentation: None,
                        template: None,
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
                    template: None,
                });
            }
        }
    }
    out
}
