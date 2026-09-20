//! The Formula Composer's view of a formula: a structured projection of
//! the draft's surface tree with the elaborator's types, the expected
//! type of every position (local dimension inference, FV Phase 10), what
//! fits a slot, and the text edits a structured action makes.
//!
//! ```text
//!   draft text ──bdl-syntax──▶ SurfaceExpr ──bdl-elab (trace)──▶ types per span
//!                                   │                                 │
//!                                   └────────── FormulaNode tree ◀────┘
//!                                                     │
//!                          solve (expected dims, top-down) · slot candidates · compose edits
//! ```
//!
//! The formula text is the only source: the projection is a *view* over
//! the syntax and the elaboration of the effective definition (a draft
//! overlay, else the committed formula), and a structured edit is a byte-
//! range edit of that text.  A slot is the surface's `?` — an expression
//! not yet written, refused by elaboration with `formula.slot.empty` and
//! never a kernel term.  Node identity is authoring identity: the tree
//! path, stable within one draft generation, never persisted.
//!
//! Local inference (`solve`, `Composer.lean`): `a + b`, `a − b` give both
//! sides the result's dimension; `a × b` gives the unknown side
//! `result − known`; `a ÷ b` gives the numerator `result + denominator`
//! and the denominator `numerator − result`.  A product of two unknowns
//! is reported as *insufficient information*, never searched
//! (`solve_sound`, `solve_complete` are the formal warrant; the tests here
//! are the production evidence).

use crate::completion::describe_representation;
use crate::diagnostics::{lift_for_mapping, SemanticDiagnostic};
use crate::QueryError;
use bdl_check::pretty;
use bdl_elab::units;
use bdl_elab::{names::InputEnv, FormulaTrace, TypeTrace};
use bdl_equations::{self as equations, Cap, PTy};
use bdl_ide_db::{
    AnalysisSnapshot, EntityRef, OverlayGeneration, OverlayKey, SnapshotStamp, TextEdit, TextRange,
};
use bdl_ir::{DesignIr, Ty};
use bdl_model::surface::{Definition, Design, MappingBlock, Representation};
use bdl_model::{DeclId, Dim, SemanticId};
use bdl_syntax::{BinaryOp, ExprKind, SurfaceExpr, UnaryOp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---- the projection ----------------------------------------------------------

/// A type in the designer's words, with what the Composer needs of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeView {
    /// "an angle", "a Brightness", "true or false", …
    pub description: String,
    pub kind: TypeKindView,
    /// The dimension, for a quantity or a concept represented by one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dim: Option<Dim>,
    /// The concept, when the value is a concept value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concept: Option<SemanticId>,
    /// Only a value of *this* concept fits the position — a relationship's
    /// or an equation's argument bound to it — as opposed to the formula's
    /// result, where any value of the representation is observed and
    /// wrapped (ADR-0013).  Dimension equality never admits another
    /// concept here.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub nominal: bool,
    /// For a collection: what one element is (a binder's local, a `map`
    /// body's result).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<Box<TypeView>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeKindView {
    Quantity,
    Boolean,
    Count,
    Concept,
    Structured,
    /// The empty product `()`: a relationship's domain when it has no
    /// inputs.  Never a physical measurement unit.
    Unit,
    Unknown,
}

impl TypeView {
    fn quantity(dim: Dim) -> TypeView {
        TypeView {
            description: pretty::describe_dim(dim),
            kind: TypeKindView::Quantity,
            dim: Some(dim),
            concept: None,
            nominal: false,
            element: None,
        }
    }
    fn boolean() -> TypeView {
        TypeView {
            description: "true or false".into(),
            kind: TypeKindView::Boolean,
            dim: None,
            concept: None,
            nominal: false,
            element: None,
        }
    }
    fn count() -> TypeView {
        TypeView {
            description: "a count".into(),
            kind: TypeKindView::Count,
            dim: None,
            concept: None,
            nominal: false,
            element: None,
        }
    }
    fn of_ty(ir: &DesignIr, design: &Design, t: &Ty) -> TypeView {
        match t {
            Ty::Q { dim } => TypeView::quantity(*dim),
            Ty::Bool => TypeView::boolean(),
            Ty::Nat => TypeView::count(),
            // the empty product: a relationship's domain, never a slot's
            // expectation
            Ty::Unit => TypeView {
                description: "()".into(),
                kind: TypeKindView::Unit,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
            Ty::Sem { id } => {
                let name = design
                    .concepts
                    .get(id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| id.to_string());
                TypeView {
                    description: format!("a {name}"),
                    kind: TypeKindView::Concept,
                    dim: match ir.representation_of(*id) {
                        Some(Ty::Q { dim }) => Some(*dim),
                        _ => None,
                    },
                    concept: Some(*id),
                    // a closed concept type is a concept value: nominal
                    nominal: true,
                    // a concept over a collection: its elements are the
                    // representation's, plain (a binder's local)
                    element: match ir.representation_of(*id) {
                        Some(Ty::List { elem }) => {
                            Some(Box::new(TypeView::of_ty(ir, design, elem)))
                        }
                        _ => None,
                    },
                }
            }
            Ty::List { elem } => TypeView {
                description: format!(
                    "a collection of {}",
                    plural_of(&TypeView::of_ty(ir, design, elem))
                ),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: Some(Box::new(TypeView::of_ty(ir, design, elem))),
            },
            other => TypeView {
                description: pretty::kernel(other),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
        }
    }
    /// A plain representation (no concept) as a view.
    fn of_representation_value(r: &Representation) -> TypeView {
        match r {
            Representation::Quantity { dim } => TypeView::quantity(*dim),
            Representation::Boolean => TypeView::boolean(),
            Representation::Count => TypeView::count(),
            Representation::List { element } => TypeView {
                description: describe_representation(r),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: Some(Box::new(TypeView::of_representation_value(element))),
            },
            other => TypeView {
                description: describe_representation(other),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
        }
    }
    fn of_representation(ir: &DesignIr, design: &Design, concept: SemanticId) -> Option<TypeView> {
        let c = design.concepts.get(&concept)?;
        let rep = c.representation.as_ref()?;
        Some(match rep {
            Representation::Quantity { dim } => TypeView {
                description: format!("a {} ({})", c.name, pretty::describe_dim(*dim)),
                kind: TypeKindView::Concept,
                dim: Some(*dim),
                concept: Some(concept),
                nominal: false,
                element: None,
            },
            Representation::Boolean => TypeView {
                description: format!("a {} (true or false)", c.name),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: None,
            },
            Representation::Count => TypeView {
                description: format!("a {} (a count)", c.name),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: None,
            },
            Representation::List { element } => TypeView {
                description: format!("a {} ({})", c.name, describe_representation(rep)),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: Some(Box::new(TypeView::of_representation_value(element))),
            },
            other => TypeView {
                description: format!("a {} ({})", c.name, describe_representation(other)),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: None,
            },
        })
        .inspect(|_| {
            let _ = ir;
        })
    }
}

/// What a node is, in the Composer's vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    /// A name: an input, a relationship, a bound variable.
    Reference {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<EntityRef>,
        /// Bound by an enclosing binder (`all x in xs: …`) or rule: a
        /// local of the formula, not an entity of the design.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        local: bool,
    },
    /// `all x in xs: body` and its siblings; children: the collection,
    /// the body.  `param` is the local the body reads.
    Binder {
        form: String,
        param: String,
        /// What the local is: one element of the collection.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        param_type: Option<TypeView>,
    },
    /// `lo .. hi`; children: the two ends.  Meaningful after `in`.
    Range,
    /// A number with no unit, as spelled.
    Number {
        text: String,
    },
    /// `90 deg`, `9.81 m per s^2`: a coordinate and its unit.
    Quantity {
        coordinate: String,
        /// The unit as written (`m per s^2`).
        unit: String,
        /// The registry id when the unit is one atom (`angle.deg`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit_id: Option<String>,
        /// The canonical spelling (`m per s^2`) and the mathematical
        /// rendering (`m/s²`) when the unit resolves; absent when it does
        /// not (the node's diagnostics say why).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit_source: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit_display: Option<String>,
    },
    Bool {
        value: bool,
    },
    Unary {
        op: String,
    },
    /// `+`, `-`, `*`, `/`.
    Binary {
        op: String,
    },
    /// `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, `||`, `in`.
    Compare {
        op: String,
    },
    /// `name(args…)`: an equation of the library, or a relationship.
    Call {
        name: String,
        equation: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<EntityRef>,
    },
    /// `?` — an expression not yet written.
    Slot,
    /// `if c then a else b` — a choice; children: the condition, the
    /// outcome when it holds, the outcome otherwise.
    If,
    /// `match x { p => e, … }`; children: the subject, then one arm each.
    Match,
    /// One arm of a match: the pattern as written and the names it
    /// binds; child: the body.
    Arm {
        pattern: String,
        binds: Vec<String>,
    },
    /// `{ let p = v; …; tail }`; children: one `Let` each, then the result.
    Block,
    /// `let p = v`: the pattern as written and the names it binds; child:
    /// the value.
    Let {
        pattern: String,
        binds: Vec<String>,
    },
    /// `x => e`, `(x, y) => e` — a rule given to an equation; child: the
    /// body.
    Rule {
        params: Vec<String>,
    },
    /// `[a, b, c]`; children: the items.
    List,
    /// `(a, b)`; children: the parts.
    Tuple,
    /// `delay(init, value)`; children: the initial value, the value
    /// remembered.
    Delay,
    /// `sync(domain, init, value)`; children: the domain, the initial
    /// value, the value read.
    Sync,
    /// A form the Composer shows as text: the empty product `()`.
    Opaque {
        what: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormulaNode {
    /// The tree path (`r`, `r.0`, `r.1.0`): stable within one generation.
    pub id: String,
    /// Byte range in the source, parentheses included.
    pub range: TextRange,
    pub kind: NodeKind,
    /// The source text of the node.
    pub text: String,
    /// What the node is to its parent, in the designer's words:
    /// `condition`, `then`, `else`, `numerator`, `denominator`, `left`,
    /// `right`, `operand`, `argument 1`, `collection`, `body`, `subject`,
    /// `arm 1`, `value`, `result`, `item 1`, `part 1`, `initial`,
    /// `domain`, `from`, `to`; empty at the root.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role: String,
    /// The formula's own names in scope at this node — a binder's or
    /// rule's parameters, a pattern's names, earlier `let`s — innermost
    /// last.  What a completion at this position may name besides the
    /// design's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locals: Vec<String>,
    /// Where a further child would be inserted (`f(a, b|)`, `[a, b|]`,
    /// `(a, b|)`, a match's last arm): the byte offset before the closing
    /// delimiter.  Absent on nodes with a fixed shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub append_at: Option<u32>,
    /// What the elaborator found the node to be.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<TypeView>,
    /// What the position asks for, from the context (local inference).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<TypeView>,
    /// Why the position expects that, in the designer's words.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub because: String,
    /// The diagnostics whose primary span is this node's and no child's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<SemanticDiagnostic>,
    pub children: Vec<FormulaNode>,
}

impl FormulaNode {
    pub fn find(&self, id: &str) -> Option<&FormulaNode> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }
    fn parent_of<'a>(&'a self, id: &str) -> Option<(&'a FormulaNode, usize)> {
        for (i, c) in self.children.iter().enumerate() {
            if c.id == id {
                return Some((self, i));
            }
            if let Some(p) = c.parent_of(id) {
                return Some(p);
            }
        }
        None
    }
    /// Every slot, in source order.
    pub fn slots(&self, out: &mut Vec<String>) {
        if matches!(self.kind, NodeKind::Slot) {
            out.push(self.id.clone());
        }
        for c in &self.children {
            c.slots(out);
        }
    }
    /// Every node in source (pre-)order.
    pub fn walk<'a>(&'a self, out: &mut Vec<&'a FormulaNode>) {
        out.push(self);
        for c in &self.children {
            c.walk(out);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormulaProjection {
    pub stamp: SnapshotStamp,
    pub mapping: DeclId,
    /// The draft's generation when the projection is of a draft overlay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_generation: Option<OverlayGeneration>,
    /// The source the ranges index.
    pub source: String,
    /// False when the source did not parse: no tree, the diagnostics say
    /// where.
    pub parse_ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<FormulaNode>,
    /// What the whole formula must produce: the output concept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<TypeView>,
    /// Whether the formula elaborates (no slot, no error).
    pub complete: bool,
    /// The slots in source order — the Tab order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<String>,
    /// Diagnostics no node owns (the whole formula's, or spanless).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unplaced: Vec<SemanticDiagnostic>,
}

/// The effective definition source of `mapping`: its draft overlay, else
/// its committed formula.
fn effective_source(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
) -> Result<(String, Option<OverlayGeneration>), QueryError> {
    let design = &snapshot.effective().design;
    let Some(block) = design.mappings.get(&mapping) else {
        return Err(QueryError::UnknownEntity {
            entity: EntityRef::Mapping(mapping),
        });
    };
    let generation = snapshot
        .overlay(OverlayKey::MappingDefinition { mapping })
        .filter(|a| a.fault.is_none())
        .map(|a| a.generation);
    match &block.definition {
        Some(Definition::Formula { source }) | Some(Definition::ScopedFormula { source, .. }) => {
            Ok((source.clone(), generation))
        }
        Some(Definition::Reference { .. }) => Err(QueryError::NotApplicable {
            reason: format!("{} is defined by reference, not by a formula", block.name),
        }),
        None => Ok((String::new(), generation)),
    }
}

fn trace(snapshot: &AnalysisSnapshot, block: &MappingBlock, source: &str) -> FormulaTrace {
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let env = match &block.definition {
        Some(Definition::ScopedFormula { scope, .. }) => {
            InputEnv::scoped(&block.signature.inputs, scope)
        }
        _ => InputEnv::for_mapping(design, block),
    };
    bdl_elab::trace_formula_in(design, ir, block, source, env)
}

/// The projection of `mapping`'s effective definition.
pub fn formula_projection(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
) -> Result<FormulaProjection, QueryError> {
    let (source, draft_generation) = effective_source(snapshot, mapping)?;
    projection_of(snapshot, mapping, source, draft_generation)
}

fn projection_of(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    source: String,
    draft_generation: Option<OverlayGeneration>,
) -> Result<FormulaProjection, QueryError> {
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let block = &design.mappings[&mapping];
    let result = TypeView::of_representation(ir, design, block.signature.output);
    if source.trim().is_empty() {
        return Ok(FormulaProjection {
            stamp: snapshot.stamp(),
            mapping,
            draft_generation,
            source,
            parse_ok: true,
            root: None,
            result,
            complete: false,
            slots: Vec::new(),
            unplaced: Vec::new(),
        });
    }
    let t = trace(snapshot, block, &source);
    let diagnostics = lift_for_mapping(snapshot, mapping);
    let mut root = t.surface.as_ref().map(|surface| {
        let types: BTreeMap<(u32, u32), &TypeTrace> = t
            .types
            .iter()
            .map(|e| ((e.span.start, e.span.end), e))
            .collect();
        let mut b = Builder {
            design,
            ir,
            block,
            source: &source,
            types: &types,
            locals: Vec::new(),
        };
        let mut node = b.node(surface, "r".into());
        b.solve(&mut node, result.as_ref(), "".into());
        node
    });
    let mut unplaced = Vec::new();
    if let Some(root) = root.as_mut() {
        for d in diagnostics {
            if !place(root, &d) {
                unplaced.push(d);
            }
        }
    } else {
        unplaced = diagnostics;
    }
    let mut slots = Vec::new();
    if let Some(r) = &root {
        r.slots(&mut slots);
    }
    Ok(FormulaProjection {
        stamp: snapshot.stamp(),
        mapping,
        draft_generation,
        parse_ok: t.surface.is_some(),
        complete: t.ok && slots.is_empty(),
        root,
        result,
        slots,
        unplaced,
        source,
    })
}

/// The unit expression a literal's suffix denotes, resolved against the
/// registry (the elaborator's own resolution, repeated for the view).
pub(crate) fn resolve_surface_unit(
    u: &bdl_syntax::Unit,
) -> Result<units::UnitExpr, units::UnitError> {
    let num: Vec<(String, i64)> = u
        .numerator
        .iter()
        .map(|f| (f.symbol.clone(), f.exponent))
        .collect();
    let den: Vec<(String, i64)> = u
        .denominator
        .iter()
        .map(|f| (f.symbol.clone(), f.exponent))
        .collect();
    units::UnitExpr::build(&num, &den).map_err(|(_, e)| e)
}

/// The names a pattern binds: a bare name, and the names inside a
/// constructor's fields (a nullary constructor such as `None` binds
/// nothing; the elaborator tells the two apart — here a capitalised bare
/// name is taken as a constructor, the surface's convention).
fn pattern_names(p: &bdl_syntax::SurfacePattern) -> Vec<String> {
    use bdl_syntax::PatternKind;
    let mut out = Vec::new();
    fn go(p: &bdl_syntax::SurfacePattern, out: &mut Vec<String>) {
        match &p.kind {
            PatternKind::Ident(n) => {
                if !n.starts_with(|c: char| c.is_ascii_uppercase()) {
                    out.push(n.clone());
                }
            }
            PatternKind::Constructor { fields, .. } => fields.iter().for_each(|f| go(f, out)),
            _ => {}
        }
    }
    go(p, &mut out);
    out
}

/// The byte offset of the closing delimiter of `e` (its last character,
/// when it is `close`): where a further argument, item or arm goes.
fn closing_offset(e: &SurfaceExpr, source: &str, close: char) -> Option<u32> {
    let text = source.get(e.span.start as usize..e.span.end as usize)?;
    let trimmed = text.trim_end();
    if trimmed.ends_with(close) {
        Some(e.span.start + trimmed.len() as u32 - 1)
    } else {
        None
    }
}

/// Attach a diagnostic to the innermost node whose range contains its
/// primary span; false when no node does.
fn place(node: &mut FormulaNode, d: &SemanticDiagnostic) -> bool {
    let Some(span) = d.primary.source.as_ref().map(|s| s.range) else {
        return false;
    };
    if span.start < node.range.start || span.end > node.range.end {
        return false;
    }
    for c in node.children.iter_mut() {
        if place(c, d) {
            return true;
        }
    }
    node.diagnostics.push(d.clone());
    true
}

struct Builder<'a> {
    design: &'a Design,
    ir: &'a DesignIr,
    block: &'a MappingBlock,
    source: &'a str,
    types: &'a BTreeMap<(u32, u32), &'a TypeTrace>,
    /// The binder locals in scope while building, innermost last.
    locals: Vec<String>,
}

fn binary_symbol(op: BinaryOp) -> (&'static str, bool) {
    match op {
        BinaryOp::Add => ("+", true),
        BinaryOp::Sub => ("-", true),
        BinaryOp::Mul => ("*", true),
        BinaryOp::Div => ("/", true),
        BinaryOp::Lt => ("<", false),
        BinaryOp::Le => ("<=", false),
        BinaryOp::Gt => (">", false),
        BinaryOp::Ge => (">=", false),
        BinaryOp::Eq => ("==", false),
        BinaryOp::Ne => ("!=", false),
        BinaryOp::And => ("&&", false),
        BinaryOp::Or => ("||", false),
        BinaryOp::In => ("in", false),
        BinaryOp::Coalesce => ("??", false),
    }
}

impl Builder<'_> {
    fn text(&self, span: bdl_diagnostics::Span) -> String {
        self.source
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
            .to_owned()
    }

    fn actual(&self, e: &SurfaceExpr) -> Option<TypeView> {
        let t = self.types.get(&(e.span.start, e.span.end))?;
        Some(match &t.ty {
            Some(ty) => TypeView::of_ty(self.ir, self.design, ty),
            None => TypeView {
                description: t.description.clone(),
                kind: TypeKindView::Unknown,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
        })
    }

    fn reference_entity(&self, name: &str) -> Option<EntityRef> {
        let scope = match &self.block.definition {
            Some(Definition::ScopedFormula { scope, .. }) => Some(scope),
            _ => None,
        };
        for (i, c) in self.block.signature.inputs.iter().enumerate() {
            let bound = scope
                .and_then(|s| s.inputs.get(i).cloned())
                .or_else(|| {
                    self.block
                        .parameters
                        .get(i)
                        .filter(|p| !p.is_empty())
                        .cloned()
                })
                .or_else(|| self.design.concepts.get(c).map(|c| c.name.clone()));
            if bound.as_deref() == Some(name) {
                return Some(EntityRef::Concept(*c));
            }
        }
        match scope {
            Some(s) => s.mappings.get(name).map(|id| EntityRef::Mapping(*id)),
            None => self
                .design
                .mappings
                .values()
                .find(|m| m.name == name)
                .map(|m| EntityRef::Mapping(m.id)),
        }
    }

    fn node(&mut self, e: &SurfaceExpr, id: String) -> FormulaNode {
        let locals_here = self.locals.clone();
        let child = |b: &mut Self, c: &SurfaceExpr, i: usize, role: &str| {
            let mut n = b.node(c, format!("{id}.{i}"));
            n.role = role.to_owned();
            n
        };
        let mut append_at = None;
        let (kind, children) = match &e.kind {
            ExprKind::Hole => (NodeKind::Slot, Vec::new()),
            ExprKind::Name(name) => {
                let local = self.locals.iter().any(|l| l == name);
                (
                    NodeKind::Reference {
                        name: name.clone(),
                        entity: if local {
                            None
                        } else {
                            self.reference_entity(name)
                        },
                        local,
                    },
                    Vec::new(),
                )
            }
            ExprKind::Binder {
                form,
                param,
                collection,
                body,
            } => {
                let coll = child(self, collection, 0, "collection");
                let param_type = coll
                    .actual
                    .as_ref()
                    .and_then(|t| t.element.as_deref().cloned());
                self.locals.push(param.name.clone());
                let body_node = child(self, body, 1, "body");
                self.locals.pop();
                (
                    NodeKind::Binder {
                        form: form.word().into(),
                        param: param.name.clone(),
                        param_type,
                    },
                    vec![coll, body_node],
                )
            }
            ExprKind::Range { lo, hi } => (
                NodeKind::Range,
                vec![child(self, lo, 0, "from"), child(self, hi, 1, "to")],
            ),
            ExprKind::Number { literal, unit } => (
                match unit {
                    None => NodeKind::Number {
                        text: literal.as_str().to_owned(),
                    },
                    Some(u) => {
                        let resolved = resolve_surface_unit(u).ok();
                        NodeKind::Quantity {
                            coordinate: literal.as_str().to_owned(),
                            unit: u.name.clone(),
                            unit_id: resolved
                                .as_ref()
                                .and_then(|x| x.as_atom())
                                .map(|d| d.id.to_owned()),
                            unit_source: resolved.as_ref().map(|x| x.source()),
                            unit_display: resolved.as_ref().map(|x| x.display()),
                        }
                    }
                },
                Vec::new(),
            ),
            ExprKind::Bool(v) => (NodeKind::Bool { value: *v }, Vec::new()),
            ExprKind::Unary { op, expr } => (
                NodeKind::Unary {
                    op: match op {
                        UnaryOp::Neg => "-".into(),
                        UnaryOp::Not => "!".into(),
                    },
                },
                vec![child(self, expr, 0, "operand")],
            ),
            ExprKind::Binary { op, lhs, rhs } => {
                let (sym, arithmetic) = binary_symbol(*op);
                let (lr, rr) = match op {
                    BinaryOp::Div => ("numerator", "denominator"),
                    _ => ("left", "right"),
                };
                (
                    if arithmetic {
                        NodeKind::Binary { op: sym.into() }
                    } else {
                        NodeKind::Compare { op: sym.into() }
                    },
                    vec![child(self, lhs, 0, lr), child(self, rhs, 1, rr)],
                )
            }
            ExprKind::Call { callee, args } => {
                let name = match &callee.kind {
                    ExprKind::Name(n) => n.clone(),
                    _ => self.text(callee.span),
                };
                append_at = closing_offset(e, self.source, ')');
                match name.as_str() {
                    "delay" => {
                        let roles = ["initial", "value"];
                        (
                            NodeKind::Delay,
                            args.iter()
                                .enumerate()
                                .map(|(i, a)| {
                                    child(self, a, i, roles.get(i).unwrap_or(&"argument"))
                                })
                                .collect(),
                        )
                    }
                    "sync" => {
                        let roles = ["domain", "initial", "value"];
                        (
                            NodeKind::Sync,
                            args.iter()
                                .enumerate()
                                .map(|(i, a)| {
                                    child(self, a, i, roles.get(i).unwrap_or(&"argument"))
                                })
                                .collect(),
                        )
                    }
                    _ => {
                        let entity = self.reference_entity(&name);
                        let equation = entity.is_none() && equations::lookup(&name).is_some();
                        (
                            NodeKind::Call {
                                name,
                                equation,
                                entity,
                            },
                            args.iter()
                                .enumerate()
                                .map(|(i, a)| child(self, a, i, &format!("argument {}", i + 1)))
                                .collect(),
                        )
                    }
                }
            }
            ExprKind::If { cond, then, els } => (
                NodeKind::If,
                vec![
                    child(self, cond, 0, "condition"),
                    child(self, then, 1, "then"),
                    child(self, els, 2, "else"),
                ],
            ),
            ExprKind::Match { scrutinee, arms } => {
                let mut children = vec![child(self, scrutinee, 0, "subject")];
                append_at = closing_offset(e, self.source, '}');
                for (i, arm) in arms.iter().enumerate() {
                    let binds = pattern_names(&arm.pattern);
                    let before = self.locals.len();
                    self.locals.extend(binds.iter().cloned());
                    let body = {
                        let mut n = self.node(&arm.body, format!("{id}.{}.0", i + 1));
                        n.role = "body".into();
                        n
                    };
                    self.locals.truncate(before);
                    children.push(FormulaNode {
                        id: format!("{id}.{}", i + 1),
                        range: TextRange::new(arm.span.start, arm.span.end),
                        text: self.text(arm.span),
                        role: format!("arm {}", i + 1),
                        locals: locals_here.clone(),
                        append_at: None,
                        actual: body.actual.clone(),
                        expected: None,
                        because: String::new(),
                        diagnostics: Vec::new(),
                        children: vec![body],
                        kind: NodeKind::Arm {
                            pattern: self.text(arm.pattern.span),
                            binds,
                        },
                    });
                }
                (NodeKind::Match, children)
            }
            ExprKind::Block { lets, tail } => {
                let mut children = Vec::new();
                let before = self.locals.len();
                for (i, l) in lets.iter().enumerate() {
                    let scope = self.locals.clone();
                    let value = {
                        let mut n = self.node(&l.value, format!("{id}.{i}.0"));
                        n.role = "value".into();
                        n
                    };
                    let binds = pattern_names(&l.pattern);
                    self.locals.extend(binds.iter().cloned());
                    children.push(FormulaNode {
                        id: format!("{id}.{i}"),
                        range: TextRange::new(l.span.start, l.span.end),
                        text: self.text(l.span),
                        role: format!("let {}", i + 1),
                        locals: scope,
                        append_at: None,
                        actual: value.actual.clone(),
                        expected: None,
                        because: String::new(),
                        diagnostics: Vec::new(),
                        children: vec![value],
                        kind: NodeKind::Let {
                            pattern: self.text(l.pattern.span),
                            binds,
                        },
                    });
                }
                let n = lets.len();
                children.push(child(self, tail, n, "result"));
                self.locals.truncate(before);
                (NodeKind::Block, children)
            }
            ExprKind::List(items) => {
                append_at = closing_offset(e, self.source, ']');
                (
                    NodeKind::List,
                    items
                        .iter()
                        .enumerate()
                        .map(|(i, a)| child(self, a, i, &format!("item {}", i + 1)))
                        .collect(),
                )
            }
            ExprKind::Tuple(items) => {
                append_at = closing_offset(e, self.source, ')');
                (
                    NodeKind::Tuple,
                    items
                        .iter()
                        .enumerate()
                        .map(|(i, a)| child(self, a, i, &format!("part {}", i + 1)))
                        .collect(),
                )
            }
            // `f(())`: the explicit application to the unique argument; the
            // Composer offers `f` and never writes this form itself
            ExprKind::Unit => (
                NodeKind::Opaque {
                    what: "the empty product ()".into(),
                },
                Vec::new(),
            ),
            ExprKind::Lambda { params, body } => {
                let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let before = self.locals.len();
                self.locals.extend(names.iter().cloned());
                let body_node = child(self, body, 0, "body");
                self.locals.truncate(before);
                (NodeKind::Rule { params: names }, vec![body_node])
            }
        };
        FormulaNode {
            id,
            range: TextRange::new(e.span.start, e.span.end),
            text: self.text(e.span),
            role: String::new(),
            locals: locals_here,
            append_at,
            actual: self.actual(e),
            expected: None,
            because: String::new(),
            diagnostics: Vec::new(),
            children,
            kind,
        }
    }

    /// Top-down: what each position expects, from what the parent expects
    /// and what the siblings are (FV `solve`).
    fn solve(&self, node: &mut FormulaNode, expected: Option<&TypeView>, because: String) {
        node.expected = expected.cloned();
        node.because = because;
        let r = expected.and_then(|t| t.dim);
        let dim_of = |n: &FormulaNode| n.actual.as_ref().and_then(|t| t.dim);
        match node.kind.clone() {
            NodeKind::Binary { op } => {
                let (a, b) = (dim_of(&node.children[0]), dim_of(&node.children[1]));
                let (ea, eb, why) = match (op.as_str(), r) {
                    ("+", Some(r)) | ("-", Some(r)) => (
                        Some(r),
                        Some(r),
                        format!(
                            "both sides of {} must be {}, the result",
                            if op == "+" { "a sum" } else { "a difference" },
                            pretty::describe_dim(r)
                        ),
                    ),
                    ("*", Some(r)) => match (a, b) {
                        (Some(a), _) => (
                            Some(a),
                            Some(r - a),
                            format!(
                                "{} × {} = {}",
                                pretty::describe_dim(a),
                                pretty::describe_dim(r - a),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, Some(b)) => (
                            Some(r - b),
                            Some(b),
                            format!(
                                "{} × {} = {}",
                                pretty::describe_dim(r - b),
                                pretty::describe_dim(b),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, None) => (
                            None,
                            None,
                            "a product of two unknown values: not enough is known to say what each must be".into(),
                        ),
                    },
                    ("/", Some(r)) => match (a, b) {
                        (Some(a), _) => (
                            Some(a),
                            Some(a - r),
                            format!(
                                "{} ÷ {} = {}",
                                pretty::describe_dim(a),
                                pretty::describe_dim(a - r),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, Some(b)) => (
                            Some(r + b),
                            Some(b),
                            format!(
                                "{} ÷ {} = {}",
                                pretty::describe_dim(r + b),
                                pretty::describe_dim(b),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, None) => (
                            None,
                            None,
                            "a quotient of two unknown values: not enough is known to say what each must be".into(),
                        ),
                    },
                    // no expected result: an operand still constrains the other
                    ("+", None) | ("-", None) => match (a, b) {
                        (Some(a), _) => (Some(a), Some(a), format!("both sides must be {}", pretty::describe_dim(a))),
                        (None, Some(b)) => (Some(b), Some(b), format!("both sides must be {}", pretty::describe_dim(b))),
                        _ => (None, None, String::new()),
                    },
                    _ => (None, None, String::new()),
                };
                let (l, rr) = node.children.split_at_mut(1);
                self.solve(&mut l[0], ea.map(TypeView::quantity).as_ref(), why.clone());
                self.solve(&mut rr[0], eb.map(TypeView::quantity).as_ref(), why);
            }
            NodeKind::Compare { op } => {
                let (a, b) = (dim_of(&node.children[0]), dim_of(&node.children[1]));
                let (ea, eb, why) = if matches!(op.as_str(), "&&" | "||") {
                    (
                        Some(TypeView::boolean()),
                        Some(TypeView::boolean()),
                        "both sides of a logical operator are true or false".to_string(),
                    )
                } else if op == "??" {
                    let why = match expected {
                        Some(t) => format!(
                            "`??` gives {} or, when it is absent, the default",
                            t.description
                        ),
                        None => String::new(),
                    };
                    (None, expected.cloned(), why)
                } else if op == "in" {
                    // `x in lo .. hi`: both ends must be comparable with x —
                    // the same concept when x is one, else its dimension
                    if matches!(node.children[1].kind, NodeKind::Range) {
                        let subject = node.children[0].actual.clone().map(|mut t| {
                            t.nominal = t.kind == TypeKindView::Concept;
                            t
                        });
                        let why = match &subject {
                            Some(t) => format!(
                                "both ends of the range must be comparable with {} ({})",
                                node.children[0].text.trim(),
                                t.description
                            ),
                            None => String::new(),
                        };
                        (None, subject, why)
                    } else {
                        (None, None, String::new())
                    }
                } else {
                    match (a, b) {
                        (Some(a), _) => (
                            Some(TypeView::quantity(a)),
                            Some(TypeView::quantity(a)),
                            format!("a comparison puts two {} side by side", plural(a)),
                        ),
                        (None, Some(b)) => (
                            Some(TypeView::quantity(b)),
                            Some(TypeView::quantity(b)),
                            format!("a comparison puts two {} side by side", plural(b)),
                        ),
                        _ => (
                            None,
                            None,
                            "both sides of a comparison must be the same kind of value".into(),
                        ),
                    }
                };
                let (l, rr) = node.children.split_at_mut(1);
                self.solve(&mut l[0], ea.as_ref(), why.clone());
                self.solve(&mut rr[0], eb.as_ref(), why);
            }
            NodeKind::Range => {
                // what the test expects, passed to both ends
                let why = node.because.clone();
                for c in node.children.iter_mut() {
                    self.solve(c, expected, why.clone());
                }
            }
            NodeKind::Binder { form, param, .. } => {
                let (form, param) = (form.clone(), param.clone());
                let coll_why = format!("{form} reads every element of a collection");
                let (body_expected, body_why) = match form.as_str() {
                    "map" => (
                        expected.and_then(|t| t.element.as_deref().cloned()),
                        format!("map makes a collection of what the body gives for each {param}"),
                    ),
                    _ => (
                        Some(TypeView::boolean()),
                        format!(
                            "{form} asks a question of every {param}: the body is true or false"
                        ),
                    ),
                };
                let (c, b) = node.children.split_at_mut(1);
                self.solve(&mut c[0], None, coll_why);
                self.solve(&mut b[0], body_expected.as_ref(), body_why);
            }
            NodeKind::If => {
                // the condition is a question; both outcomes give what the
                // choice gives — the position's expectation, else what the
                // other outcome already is
                let known =
                    |n: &FormulaNode| n.actual.clone().filter(|t| t.kind != TypeKindView::Unknown);
                let (then_known, else_known) = (known(&node.children[1]), known(&node.children[2]));
                let (outcome, why) = match (expected, then_known, else_known) {
                    (Some(t), _, _) => (
                        Some(t.clone()),
                        format!(
                            "both outcomes of a choice give what the choice gives: {}",
                            t.description
                        ),
                    ),
                    (None, Some(t), _) | (None, None, Some(t)) => (
                        Some(t.clone()),
                        format!(
                            "both outcomes of a choice are the same kind of value: {}",
                            t.description
                        ),
                    ),
                    (None, None, None) => (None, String::new()),
                };
                let (c, rest) = node.children.split_at_mut(1);
                self.solve(
                    &mut c[0],
                    Some(&TypeView::boolean()),
                    "a choice asks a question: the condition is true or false".into(),
                );
                for o in rest.iter_mut() {
                    self.solve(o, outcome.as_ref(), why.clone());
                }
            }
            NodeKind::Unary { op } => {
                let (e, why) = if op == "!" {
                    (
                        Some(TypeView::boolean()),
                        "negation applies to true or false".to_string(),
                    )
                } else {
                    (
                        expected.cloned(),
                        "a negated value has the kind of its result".to_string(),
                    )
                };
                self.solve(&mut node.children[0], e.as_ref(), why);
            }
            NodeKind::Call {
                name,
                equation,
                entity,
            } => {
                if equation {
                    let params = self.equation_params(&name, node, expected);
                    for (i, c) in node.children.iter_mut().enumerate() {
                        let (e, why) = params.get(i).cloned().unwrap_or((None, String::new()));
                        self.solve(c, e.as_ref(), why);
                    }
                } else if let Some(EntityRef::Mapping(id)) = entity {
                    let inputs = self
                        .design
                        .mappings
                        .get(&id)
                        .map(|m| m.signature.inputs.clone())
                        .unwrap_or_default();
                    for (i, c) in node.children.iter_mut().enumerate() {
                        // a relationship reads its input *concepts*: only a
                        // value of that concept fits (DI-17), never another
                        // concept of the same representation
                        let e = inputs.get(i).and_then(|s| {
                            TypeView::of_representation(self.ir, self.design, *s)
                                .map(|v| TypeView { nominal: true, ..v })
                        });
                        self.solve(
                            c,
                            e.as_ref(),
                            format!(
                                "{name} reads {} here",
                                e.as_ref()
                                    .map(|t| t.description.clone())
                                    .unwrap_or_else(|| "a value".into())
                            ),
                        );
                    }
                } else {
                    for c in node.children.iter_mut() {
                        self.solve(c, None, String::new());
                    }
                }
            }
            NodeKind::Match => {
                // the subject is free; every arm's body gives what the
                // match gives
                let why = match expected {
                    Some(t) => format!(
                        "every arm of a match gives what the match gives: {}",
                        t.description
                    ),
                    None => "every arm of a match gives the same kind of value".into(),
                };
                let (s, arms) = node.children.split_at_mut(1);
                self.solve(&mut s[0], None, "a match looks at its subject".into());
                for arm in arms.iter_mut() {
                    arm.expected = expected.cloned();
                    arm.because = why.clone();
                    for body in arm.children.iter_mut() {
                        self.solve(body, expected, why.clone());
                    }
                }
            }
            NodeKind::Block => {
                // the `let`s are free; the result gives what the block gives
                let n = node.children.len();
                for (i, c) in node.children.iter_mut().enumerate() {
                    if i + 1 == n {
                        self.solve(
                            c,
                            expected,
                            "the result of a block is what the block gives".into(),
                        );
                    } else {
                        for v in c.children.iter_mut() {
                            self.solve(v, None, String::new());
                        }
                    }
                }
            }
            NodeKind::Delay => {
                // both the initial and the remembered value are what the
                // delay gives
                let why = "a remembered value and its initial value are the same kind of value"
                    .to_string();
                for c in node.children.iter_mut() {
                    self.solve(c, expected, why.clone());
                }
            }
            NodeKind::Sync => {
                let why = "a value read from another domain and its initial value are the same kind of value".to_string();
                for (i, c) in node.children.iter_mut().enumerate() {
                    if i == 0 {
                        self.solve(c, None, "the timing domain read from".into());
                    } else {
                        self.solve(c, expected, why.clone());
                    }
                }
            }
            NodeKind::List => {
                let element = expected.and_then(|t| t.element.as_deref().cloned());
                for c in node.children.iter_mut() {
                    self.solve(
                        c,
                        element.as_ref(),
                        "every item of a collection is one element".into(),
                    );
                }
            }
            _ => {
                for c in node.children.iter_mut() {
                    self.solve(c, None, String::new());
                }
            }
        }
    }

    /// The expected type of each argument of an equation call, from its
    /// scheme instantiated by the result and the arguments already known
    /// (P9's matcher; nothing is guessed).
    fn equation_params(
        &self,
        name: &str,
        node: &FormulaNode,
        expected: Option<&TypeView>,
    ) -> Vec<(Option<TypeView>, String)> {
        let Some(entry) = equations::lookup(name) else {
            return Vec::new();
        };
        // the arguments already written bind the variables first — a
        // concept value binds its concept, as the elaborator's instance
        // resolution keeps it; the expected result seeds only what they
        // leave open (its concept is observed there, ADR-0013)
        let mut subst = equations::Subst::default();
        for (c, pat) in node.children.iter().zip(&entry.scheme.params) {
            let known = c.actual.as_ref().and_then(|t| match t.kind {
                TypeKindView::Concept => t.concept.map(|id| Ty::Sem { id }),
                _ => closed_ty(t),
            });
            if let Some(t) = known {
                let mut s = subst.clone();
                if equations::match_ty(pat, &t, &mut s).is_ok() {
                    subst = s;
                }
            }
        }
        if let Some(t) = expected.and_then(closed_ty) {
            let mut s = subst.clone();
            if equations::match_ty(&entry.scheme.result, &t, &mut s).is_ok() {
                subst = s;
            }
        }
        entry
            .scheme
            .params
            .iter()
            .enumerate()
            .map(|(i, pat)| {
                let p = entry.params.get(i).copied().unwrap_or("value");
                match equations::instantiate(pat, &subst) {
                    Some(t) if !matches!(t, Ty::Arr { .. }) => (
                        Some(TypeView::of_ty(self.ir, self.design, &t)),
                        format!("{} takes {p} here", entry.shape()),
                    ),
                    Some(_) => (None, format!("{} takes a rule here", entry.shape())),
                    None => (None, format!("{} takes {p} here", entry.shape())),
                }
            })
            .collect()
    }
}

fn plural_of(t: &TypeView) -> String {
    match (t.kind, t.dim) {
        (TypeKindView::Quantity, Some(d)) => plural(d),
        (TypeKindView::Concept, _) => format!(
            "{} values",
            t.description
                .trim_start_matches("a ")
                .split(" (")
                .next()
                .unwrap_or("")
        ),
        (TypeKindView::Boolean, _) => "truth values".into(),
        (TypeKindView::Count, _) => "counts".into(),
        _ => format!("values ({})", t.description),
    }
}

fn plural(d: Dim) -> String {
    let one = pretty::describe_dim(d);
    let noun = one
        .strip_prefix("an ")
        .or_else(|| one.strip_prefix("a "))
        .unwrap_or(&one);
    if noun.starts_with("dimensionless") {
        "dimensionless quantities".into()
    } else if noun.starts_with("quantity of") {
        format!("quantities {}", &noun["quantity ".len()..])
    } else {
        format!("{noun}s")
    }
}

/// The closed kernel type a view stands for: a quantity, a truth value or
/// a count; a concept as its representation (observation).
fn closed_ty(v: &TypeView) -> Option<Ty> {
    match v.kind {
        TypeKindView::Quantity | TypeKindView::Concept => v.dim.map(|dim| Ty::Q { dim }),
        TypeKindView::Boolean => Some(Ty::Bool),
        TypeKindView::Count => Some(Ty::Nat),
        _ => None,
    }
}

// ---- slots ---------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitCandidate {
    /// The registry id of an atom (`angle.deg`); empty for a composite.
    pub id: String,
    /// The canonical source spelling: `deg`, `rad per s`, `m per s^2`.
    pub symbol: String,
    /// The mathematical rendering, for display: `rad/s`, `m/s²`.
    pub display: String,
    /// "an angle"
    pub measures: String,
}

impl UnitCandidate {
    pub fn of(u: &units::UnitExpr) -> UnitCandidate {
        UnitCandidate {
            id: u.as_atom().map(|d| d.id.to_owned()).unwrap_or_default(),
            symbol: u.source(),
            display: u.display(),
            measures: pretty::describe_dim(u.dim()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceCandidate {
    pub label: String,
    /// The text that fills the slot (`tilt`, `dimByTilt(?)`).
    pub insert: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityRef>,
    /// What the reference is, in the designer's words.
    pub produces: String,
    /// Higher first: the expected concept itself, then the same kind of
    /// value, then the rest.
    pub relevance: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquationCandidate {
    pub name: String,
    /// `min(a, b)`
    pub shape: String,
    /// The text that fills the slot, with a slot per argument.
    pub insert: String,
    pub summary: String,
}

/// What a slot (or any node's position) expects and what fits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotInfo {
    pub node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<TypeView>,
    /// "Expected: an angle, because an angle ÷ an angle = a dimensionless
    /// quantity."
    pub explanation: String,
    /// The same in the kernel's terms (exponent vectors), for Explain.
    pub technical: String,
    /// True when the position is not locally determined (two unknown
    /// operands): nothing is offered by dimension.
    pub insufficient: bool,
    pub units: Vec<UnitCandidate>,
    pub references: Vec<ReferenceCandidate>,
    pub equations: Vec<EquationCandidate>,
    /// The truth values, as text to fill the slot with (`true`, `false`),
    /// when the position expects true or false.
    pub booleans: Vec<String>,
}

/// What fits at `node` of `mapping`'s projection.
pub fn formula_slot(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    node: &str,
) -> Result<SlotInfo, QueryError> {
    let projection = formula_projection(snapshot, mapping)?;
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let block = &design.mappings[&mapping];
    let (expected, because, insufficient) = match &projection.root {
        Some(root) if root.id == node => (projection.result.clone(), String::new(), false),
        Some(root) => match root.find(node) {
            Some(n) => (
                n.expected.clone(),
                n.because.clone(),
                n.expected.is_none() && n.because.contains("not enough"),
            ),
            None => {
                return Err(QueryError::NotApplicable {
                    reason: format!("no node {node} in the projection"),
                })
            }
        },
        None if node == "r" => (projection.result.clone(), String::new(), false),
        None => {
            return Err(QueryError::NotApplicable {
                reason: "the formula does not parse".into(),
            })
        }
    };
    let explanation = match (&expected, insufficient) {
        (Some(t), _) if because.is_empty() => format!("Expected: {}.", t.description),
        (Some(t), _) => format!("Expected: {}, because {}.", t.description, because),
        (None, true) => format!("Not enough is known yet: {because}."),
        (None, false) => "Any value fits here; what it produces decides the rest.".into(),
    };
    let technical = match &expected {
        Some(TypeView { dim: Some(d), .. }) => {
            format!("expected {}", pretty::kernel(&Ty::Q { dim: *d }))
        }
        Some(t) => format!("expected {}", t.description),
        None => "expected: undetermined".into(),
    };
    // a literal's own unit pop-up switches the unit while keeping the
    // quantity (18D): its candidates are the units of the literal's own
    // dimension; an empty slot's are those of the expected dimension (18C)
    let literal_dim = projection
        .root
        .as_ref()
        .and_then(|r| r.find(node))
        .and_then(|n| match &n.kind {
            NodeKind::Quantity {
                unit_source: Some(_),
                ..
            } => n.actual.as_ref().and_then(|t| t.dim),
            _ => None,
        });
    let dim = literal_dim.or(expected.as_ref().and_then(|t| t.dim));
    let units: Vec<UnitCandidate> = match dim {
        Some(d) => units::candidates_for(d)
            .iter()
            .map(UnitCandidate::of)
            .collect(),
        None => Vec::new(),
    };
    let references = reference_candidates(design, ir, block, expected.as_ref());
    let equations = equation_candidates(ir, expected.as_ref());
    let booleans = if expects_truth_value(design, expected.as_ref()) {
        vec!["true".to_owned(), "false".to_owned()]
    } else {
        Vec::new()
    };
    Ok(SlotInfo {
        node: node.to_owned(),
        expected,
        explanation,
        technical,
        insufficient,
        units,
        references,
        equations,
        booleans,
    })
}

/// Whether the position takes a truth value: true or false itself, or a
/// concept represented by one (a literal is observed with it, as a
/// quantity literal is with a quantity concept).
fn expects_truth_value(design: &Design, expected: Option<&TypeView>) -> bool {
    match expected {
        Some(TypeView {
            kind: TypeKindView::Boolean,
            ..
        }) => true,
        Some(TypeView {
            kind: TypeKindView::Concept,
            concept: Some(id),
            ..
        }) => matches!(
            design
                .concepts
                .get(id)
                .and_then(|c| c.representation.as_ref()),
            Some(Representation::Boolean)
        ),
        _ => false,
    }
}

/// Whether a value of representation `rep` (or concept `concept`) fits
/// an expected view: the expected concept itself ranks 100, its
/// representation (a concept is observed where a representation is
/// needed) 80, a quantity of another dimension 0 (unfit), anything when
/// nothing is expected 50.
fn fit(
    expected: Option<&TypeView>,
    concept: Option<SemanticId>,
    rep: Option<&Representation>,
) -> Option<u8> {
    let Some(e) = expected else {
        return Some(50);
    };
    if e.concept.is_some() && e.concept == concept {
        return Some(100);
    }
    if e.nominal && e.concept.is_some() {
        // another concept, or a plain value where a concept value is read:
        // nominally wrong however the representations compare
        return None;
    }
    match (e.kind, rep) {
        (
            TypeKindView::Quantity | TypeKindView::Concept,
            Some(Representation::Quantity { dim }),
        ) => (Some(*dim) == e.dim).then_some(80),
        (TypeKindView::Boolean, Some(Representation::Boolean)) => Some(80),
        (TypeKindView::Concept, Some(Representation::Boolean)) if e.dim.is_none() => Some(80),
        (TypeKindView::Count, Some(Representation::Count)) => Some(80),
        (TypeKindView::Unknown, _) => Some(50),
        _ => None,
    }
}

fn reference_candidates(
    design: &Design,
    ir: &DesignIr,
    block: &MappingBlock,
    expected: Option<&TypeView>,
) -> Vec<ReferenceCandidate> {
    let _ = ir;
    let scope = match &block.definition {
        Some(Definition::ScopedFormula { scope, .. }) => Some(scope),
        _ => None,
    };
    let mut out = Vec::new();
    for (i, c) in block.signature.inputs.iter().enumerate() {
        let Some(concept) = design.concepts.get(c) else {
            continue;
        };
        let name = scope
            .and_then(|s| s.inputs.get(i).cloned())
            .or_else(|| block.parameters.get(i).filter(|p| !p.is_empty()).cloned())
            .unwrap_or_else(|| concept.name.clone());
        let Some(relevance) = fit(expected, Some(*c), concept.representation.as_ref()) else {
            continue;
        };
        out.push(ReferenceCandidate {
            label: name.clone(),
            insert: name,
            entity: Some(EntityRef::Concept(*c)),
            produces: format!(
                "{} ({})",
                concept.name,
                concept
                    .representation
                    .as_ref()
                    .map(describe_representation)
                    .unwrap_or_else(|| "no value form yet".into())
            ),
            relevance: relevance + 10,
        });
    }
    let visible: Vec<(String, &MappingBlock)> = match scope {
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
        if m.id == block.id {
            continue;
        }
        let out_concept = design.concepts.get(&m.signature.output);
        let Some(relevance) = fit(
            expected,
            Some(m.signature.output),
            out_concept.and_then(|c| c.representation.as_ref()),
        ) else {
            continue;
        };
        // a relationship with the unit domain is read as a value: its
        // unique argument is nothing to fill
        let callable = !m.signature.is_unit_domain();
        let slots = vec!["?"; m.signature.inputs.len()].join(", ");
        out.push(ReferenceCandidate {
            label: if callable {
                format!("{name}(…)")
            } else {
                name.clone()
            },
            insert: if callable {
                format!("{name}({slots})")
            } else {
                name
            },
            entity: Some(EntityRef::Mapping(m.id)),
            produces: out_concept
                .map(|c| {
                    format!(
                        "{} ({})",
                        c.name,
                        c.representation
                            .as_ref()
                            .map(describe_representation)
                            .unwrap_or_else(|| "no value form yet".into())
                    )
                })
                .unwrap_or_default(),
            relevance: relevance.saturating_sub(if callable { 10 } else { 5 }),
        });
    }
    out.sort_by(|a, b| {
        b.relevance
            .cmp(&a.relevance)
            .then_with(|| a.label.cmp(&b.label))
    });
    out
}

/// The equations whose result can be the expected value and whose
/// capabilities the instance satisfies (P9's scheme matcher): with an
/// angle expected, `min`, `max`, `clamp`, `sum`, … but not `any`.
fn equation_candidates(ir: &DesignIr, expected: Option<&TypeView>) -> Vec<EquationCandidate> {
    let Some(want) = expected.and_then(closed_ty) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in equations::entries() {
        let mut subst = equations::Subst::default();
        if equations::match_ty(&e.scheme.result, &want, &mut subst).is_err() {
            continue;
        }
        // a capability on a variable the result binds must hold there
        let satisfiable = e.scheme.caps.iter().all(|(v, cap)| match subst.tys.get(v) {
            Some(t) => match cap {
                Cap::Ord => ir.is_ordered(t),
                Cap::Eq | Cap::Data => !matches!(t, Ty::Arr { .. }),
            },
            None => true,
        });
        if !satisfiable {
            continue;
        }
        // the equation must take at least one value (a rule-only shape
        // is not a value to fill)
        if e.scheme.params.iter().all(|p| matches!(p, PTy::Arr(..))) {
            continue;
        }
        let args: Vec<&str> = e
            .scheme
            .params
            .iter()
            .map(|p| match p {
                PTy::Arr(a, _) => match a.as_ref() {
                    PTy::Prod(..) => "(x, y) => ?",
                    _ => "x => ?",
                },
                _ => "?",
            })
            .collect();
        out.push(EquationCandidate {
            name: e.name.to_owned(),
            shape: e.shape(),
            insert: format!("{}({})", e.name, args.join(", ")),
            summary: e.summary.to_owned(),
        });
    }
    out
}

// ---- composing edits ----------------------------------------------------------

/// A structured action on the formula, answered with the text it makes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ComposeOp {
    /// Put `text` where the node is (`?` → `90 deg`, `tilt`, `min(?, ?)`).
    Fill { node: String, text: String },
    /// `node` becomes `node op ?` (or `? op node` with `before`); `!` is
    /// the prefix form, `!node`, and takes no slot.
    Operator {
        node: String,
        op: String,
        #[serde(default)]
        before: bool,
    },
    /// `node` becomes `name(node, ?, …)` with `arity` arguments.
    Call {
        node: String,
        name: String,
        arity: u32,
    },
    /// A literal's unit — a registry id (`angle.deg`) or a canonical
    /// spelling (`rad per s`): with `preserve_value` the coordinate is
    /// converted so the physical quantity is unchanged (`180 deg` →
    /// `3.14… rad`); without it the coordinate stays (`180 deg` →
    /// `180 rad`, a different quantity).
    SetUnit {
        node: String,
        unit_id: String,
        preserve_value: bool,
    },
    /// Keyboard input at a structural caret: `text` typed before or after
    /// `node` (or into it, when it is a slot), interpreted by the grammar
    /// — an operator becomes `node op ?`, `(` parenthesises, a word fills
    /// a slot with its canonical form (`clamp` → `clamp(?, ?, ?)`, `if` →
    /// `if ? then ? else ?`), a unit word after a number becomes the
    /// literal's unit, `per` / `*` / `^` after a quantity extend its
    /// unit.  Refused, with the reason, when the text cannot stand at the
    /// caret.
    Insert {
        node: String,
        side: Side,
        text: String,
    },
    /// A literal's coordinate: the same unit, another quantity.
    SetCoordinate { node: String, text: String },
    /// The node becomes a slot again; a slot that is an operand of `+ - *
    /// /` removes the operator with it.
    Remove { node: String },
    /// `node` becomes `form item in node: ?` with a fresh local name
    /// (`all`, `any`, `map` or `filter`).
    Binder { node: String, form: String },
    /// `node` becomes `node in ? .. ?`.
    Range { node: String },
    /// A choice: a slot becomes `if ? then ? else ?`; any other node
    /// becomes one outcome of it, `if ? then node else ?`.
    Choose { node: String },
    /// A reference naming an equation or a rule becomes a call with one
    /// slot per argument (`clamp` → `clamp(?, ?, ?)`); the arity is the
    /// library's or the rule's signature's — `(` typed after a name.
    /// Refused on a value or a literal.
    Apply { node: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposeResult {
    pub source: String,
    pub edits: Vec<TextEdit>,
    /// The node to select afterwards (the first new slot, else the
    /// edited node), by its path in the *new* source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<String>,
}

/// Whether removing an empty operand of `parent` removes the operator
/// with it: arithmetic, a comparison, `&&`, `||` — every two-sided
/// operator whose other side then stands alone (not `in`, whose range
/// is not a value on its own).
fn removes_operator(parent: &FormulaNode) -> bool {
    match &parent.kind {
        NodeKind::Binary { .. } => true,
        NodeKind::Compare { op } => op != "in",
        _ => false,
    }
}

/// Precedence of a node as an operand: what needs parentheses under what.
/// The ladder is the parser's (`bdl-syntax::parser::expr::infix`), one
/// step per level.
fn precedence(kind: &NodeKind) -> u8 {
    match kind {
        // a binder extends as far right as its body, a choice as far as
        // its `else` does, a rule as far as its body: an operand only in
        // parentheses
        NodeKind::Binder { .. } | NodeKind::If | NodeKind::Rule { .. } => 0,
        NodeKind::Compare { op } | NodeKind::Binary { op } => op_precedence(op),
        NodeKind::Range => op_precedence(".."),
        NodeKind::Unary { .. } => UNARY,
        _ => ATOM,
    }
}

const UNARY: u8 = 9;
const ATOM: u8 = 10;
/// What a comparison's operand must bind at: comparisons do not chain
/// (`a < b == c` is refused), so one under another is parenthesised on
/// either side, and a range (`..`) stands unparenthesised after `in`.
const COMPARISON_OPERAND: u8 = 5;

fn op_precedence(op: &str) -> u8 {
    match op {
        "||" => 1,
        "&&" => 2,
        "==" | "!=" => 3,
        "<" | "<=" | ">" | ">=" | "in" => 4,
        ".." => 5,
        "??" => 6,
        "+" | "-" => 7,
        "*" | "/" => 8,
        _ => 4,
    }
}

/// `<`, `<=`, `>`, `>=`, `==`, `!=`, `in`: the operators that do not chain.
fn is_comparison(op: &str) -> bool {
    matches!(op, "<" | "<=" | ">" | ">=" | "==" | "!=" | "in")
}

/// Whether one pair of parentheses encloses the whole text (`(a + b)`,
/// not `(a) + b`).
fn wholly_parenthesised(text: &str) -> bool {
    let text = text.trim();
    if !text.starts_with('(') || !text.ends_with(')') {
        return false;
    }
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return i == text.len() - 1;
                }
            }
            _ => {}
        }
    }
    false
}

/// The precedence of a node as an operand: its own, unless it ends in a
/// form that extends as far right as it can (a choice, a binder) — then
/// anything written after it would be swallowed, and it binds like that
/// form: `-if c then a else b` before `+ ?` needs parentheses as much as
/// the choice alone does.
fn node_precedence(node: &FormulaNode) -> u8 {
    let own = precedence(&node.kind);
    let open_right = match &node.kind {
        NodeKind::Unary { .. }
        | NodeKind::Binary { .. }
        | NodeKind::Compare { .. }
        | NodeKind::Range => node
            .children
            .last()
            .is_some_and(|c| node_precedence(c) == 0),
        _ => false,
    };
    if open_right {
        0
    } else {
        own
    }
}

/// The node's text, parenthesised when it would bind weaker than `under`.
fn operand(node: &FormulaNode, under: u8) -> String {
    let text = node.text.trim();
    if node_precedence(node) < under && !wholly_parenthesised(text) {
        format!("({text})")
    } else {
        text.to_owned()
    }
}

/// The precedence a piece of text would have as an operand, for
/// parenthesising it (`node_precedence` over its syntax).
fn precedence_of_text(text: &str) -> u8 {
    fn of(e: &SurfaceExpr) -> u8 {
        let own = match &e.kind {
            ExprKind::Binary { op, .. } => op_precedence(binary_symbol(*op).0),
            ExprKind::Unary { .. } => UNARY,
            ExprKind::If { .. } | ExprKind::Binder { .. } | ExprKind::Lambda { .. } => 0,
            ExprKind::Range { .. } => op_precedence(".."),
            _ => ATOM,
        };
        let open_right = match &e.kind {
            ExprKind::Binary { rhs: last, .. }
            | ExprKind::Unary { expr: last, .. }
            | ExprKind::Range { hi: last, .. } => of(last) == 0,
            _ => false,
        };
        if open_right {
            0
        } else {
            own
        }
    }
    match bdl_syntax::formula(text) {
        Ok(e) => of(&e),
        Err(_) => ATOM,
    }
}

/// Apply a structured action to `source`; the projection it acts on is
/// rebuilt here so the action and the text never disagree.
pub fn compose(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    source: &str,
    op: &ComposeOp,
) -> Result<ComposeResult, QueryError> {
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let Some(block) = design.mappings.get(&mapping) else {
        return Err(QueryError::UnknownEntity {
            entity: EntityRef::Mapping(mapping),
        });
    };
    // An empty formula is one slot: every action starts from `?`.
    let empty = source.trim().is_empty();
    let base = if empty {
        "?".to_owned()
    } else {
        source.to_owned()
    };
    let t = trace(snapshot, block, &base);
    let Some(surface) = &t.surface else {
        return Err(QueryError::NotApplicable {
            reason: "the formula does not parse; edit it as text".into(),
        });
    };
    let types: BTreeMap<(u32, u32), &TypeTrace> = t
        .types
        .iter()
        .map(|e| ((e.span.start, e.span.end), e))
        .collect();
    let mut b = Builder {
        design,
        ir,
        block,
        source: &base,
        types: &types,
        locals: Vec::new(),
    };
    let root = b.node(surface, "r".into());
    let node_id = match op {
        ComposeOp::Fill { node, .. }
        | ComposeOp::Operator { node, .. }
        | ComposeOp::Call { node, .. }
        | ComposeOp::SetUnit { node, .. }
        | ComposeOp::SetCoordinate { node, .. }
        | ComposeOp::Remove { node }
        | ComposeOp::Binder { node, .. }
        | ComposeOp::Range { node }
        | ComposeOp::Insert { node, .. }
        | ComposeOp::Choose { node }
        | ComposeOp::Apply { node } => node.as_str(),
    };
    let Some(node) = root.find(node_id) else {
        return Err(QueryError::NotApplicable {
            reason: format!("no node {node_id} in the formula"),
        });
    };
    let parent = root.parent_of(node_id);
    // What the position of `node` demands of whatever stands there: the
    // precedence its parent binds with (a right operand of `-` or `/` may
    // not bind equally: `a - (b - c)`), everything under a unary minus, and
    // nothing at the root or as a call's argument.
    let context = |parent: Option<(&FormulaNode, usize)>| -> u8 {
        parent
            .map(|(p, i)| match &p.kind {
                NodeKind::Compare { op } if is_comparison(op) => COMPARISON_OPERAND,
                NodeKind::Binary { op } | NodeKind::Compare { op } => {
                    op_precedence(op) + if i == 1 { 1 } else { 0 }
                }
                // a range's ends: arithmetic and `??` bind tighter than
                // `..`, a comparison or a logical operator does not
                NodeKind::Range => op_precedence("??"),
                NodeKind::Unary { .. } => UNARY,
                // `if`, `then` and `else` delimit a choice's parts
                NodeKind::If => 0,
                // a binder's collection stops at the colon: a comparison
                // or another binder there needs parentheses (a range does
                // not); its body extends to the end and needs none
                NodeKind::Binder { .. } if i == 0 => COMPARISON_OPERAND,
                _ => 0,
            })
            .unwrap_or(0)
    };
    let under = context(parent);
    let grouped = |text: String, prec: u8| -> String {
        if prec < under && !wholly_parenthesised(&text) {
            format!("({text})")
        } else {
            text
        }
    };
    let (range, new_text): (TextRange, String) = match op {
        ComposeOp::Fill { text, .. } => {
            let text = text.trim();
            (
                node.range,
                grouped(text.to_owned(), precedence_of_text(text)),
            )
        }
        ComposeOp::Operator { op, .. } if op == "!" => {
            // negation binds tighter than any operator: its operand is
            // parenthesised unless it is atomic or a negation itself
            let operand = operand(node, UNARY);
            (node.range, grouped(format!("!{operand}"), UNARY))
        }
        ComposeOp::Operator { op, before, .. } => {
            let p = op_precedence(op);
            // as the left operand the node may bind equally (`a - b - ?` is
            // `(a - b) - ?`); as the right one it may not (`? / (a / b)`);
            // under a comparison it may be no comparison at all
            let under = if is_comparison(op) {
                COMPARISON_OPERAND
            } else if *before {
                p + 1
            } else {
                p
            };
            let operand = operand(node, under);
            let text = if *before {
                format!("? {op} {operand}")
            } else {
                format!("{operand} {op} ?")
            };
            (node.range, grouped(text, p))
        }
        ComposeOp::Binder { form, .. } => {
            if bdl_syntax::BinderForm::from_word(form).is_none() {
                return Err(QueryError::NotApplicable {
                    reason: format!("`{form}` is not a binder form"),
                });
            }
            let local = fresh_local(design, block, &root, &node.text);
            // the collection is parsed up to the colon: a comparison there
            // needs parentheses
            let coll = operand(node, COMPARISON_OPERAND);
            (
                node.range,
                grouped(format!("{form} {local} in {coll}: ?"), 0),
            )
        }
        ComposeOp::Range { .. } => {
            let subject = operand(node, COMPARISON_OPERAND);
            (
                node.range,
                grouped(format!("{subject} in ? .. ?"), op_precedence("in")),
            )
        }
        ComposeOp::Choose { .. } => {
            // a choice's parts are delimited by its words: nothing inside
            // needs parentheses; the choice itself extends to the end
            let outcome = if matches!(node.kind, NodeKind::Slot) {
                "?".to_owned()
            } else {
                node.text.trim().to_owned()
            };
            (
                node.range,
                grouped(format!("if ? then {outcome} else ?"), 0),
            )
        }
        ComposeOp::Apply { .. } => {
            let NodeKind::Reference { name, entity, .. } = &node.kind else {
                return Err(QueryError::NotApplicable {
                    reason: "only a name can be applied".into(),
                });
            };
            let arity = match entity {
                Some(EntityRef::Mapping(id)) => {
                    let n = design
                        .mappings
                        .get(id)
                        .map(|m| m.signature.inputs.len())
                        .unwrap_or(0);
                    if n == 0 {
                        // a relationship without inputs is a value, written
                        // by its name (ADR-0029): nothing to apply it to
                        return Err(QueryError::NotApplicable {
                            reason: format!("`{name}` is a value, not a rule or an equation"),
                        });
                    }
                    n
                }
                Some(_) => {
                    return Err(QueryError::NotApplicable {
                        reason: format!("`{name}` is a value, not a rule or an equation"),
                    })
                }
                None => match equations::lookup(name) {
                    Some(e) => e.scheme.params.len(),
                    None => {
                        return Err(QueryError::NotApplicable {
                            reason: format!("`{name}` is not an equation or a rule"),
                        })
                    }
                },
            };
            let args = vec!["?"; arity];
            (node.range, format!("{name}({})", args.join(", ")))
        }
        ComposeOp::Call { name, arity, .. } => {
            let mut args = vec![node.text.trim().to_owned()];
            args.extend(std::iter::repeat_n(
                "?".to_owned(),
                (*arity as usize).saturating_sub(1),
            ));
            (node.range, format!("{name}({})", args.join(", ")))
        }
        ComposeOp::SetUnit {
            unit_id,
            preserve_value,
            ..
        } => {
            let to = resolve_unit_ref(unit_id).ok_or_else(|| QueryError::NotApplicable {
                reason: format!("no unit {unit_id}"),
            })?;
            let (coordinate, from): (String, Option<units::UnitExpr>) = match &node.kind {
                NodeKind::Quantity {
                    coordinate,
                    unit_source,
                    ..
                } => (
                    coordinate.clone(),
                    unit_source
                        .as_deref()
                        .and_then(|s| units::UnitExpr::parse_canonical(s).ok()),
                ),
                NodeKind::Number { text } => (text.clone(), None),
                _ => {
                    return Err(QueryError::NotApplicable {
                        reason: "only a number can take a unit".into(),
                    })
                }
            };
            let coordinate = match (from, *preserve_value) {
                (Some(from), true) if from != to => {
                    let x: f64 = coordinate.parse().map_err(|_| QueryError::NotApplicable {
                        reason: format!("`{coordinate}` is not a number"),
                    })?;
                    if from.dim() != to.dim() {
                        return Err(QueryError::NotApplicable {
                            reason: format!(
                                "{} measures {}, {} measures {}: the value cannot be kept",
                                from.source(),
                                pretty::describe_dim(from.dim()),
                                to.source(),
                                pretty::describe_dim(to.dim())
                            ),
                        });
                    }
                    format_coordinate(to.from_canonical(from.to_canonical(x)))
                }
                _ => coordinate,
            };
            (node.range, format!("{coordinate} {}", to.source()))
        }
        ComposeOp::Insert { side, text, .. } => {
            insertion(node, parent, *side, text, &root, design, block)?
        }
        ComposeOp::SetCoordinate { text, .. } => {
            let text = text.trim();
            if text.parse::<f64>().is_err() {
                return Err(QueryError::NotApplicable {
                    reason: format!("`{text}` is not a number"),
                });
            }
            match &node.kind {
                NodeKind::Quantity { unit, .. } => (node.range, format!("{text} {unit}")),
                NodeKind::Number { .. } | NodeKind::Slot => (node.range, text.to_owned()),
                _ => {
                    return Err(QueryError::NotApplicable {
                        reason: "only a number has a coordinate".into(),
                    })
                }
            }
        }
        ComposeOp::Remove { .. } => match parent {
            Some((p, i)) if matches!(node.kind, NodeKind::Slot) && removes_operator(p) => {
                // removing an empty operand removes the operator: the
                // other operand stands alone, grouped as *its* new
                // position demands
                let other = &p.children[1 - i];
                (p.range, operand(other, context(root.parent_of(&p.id))))
            }
            Some((p, _))
                if matches!(node.kind, NodeKind::Slot)
                    && matches!(p.kind, NodeKind::Unary { .. }) =>
            {
                // an empty negation is nothing: the slot stands alone
                (p.range, "?".to_owned())
            }
            _ => (node.range, "?".to_owned()),
        },
    };
    let edit = TextEdit::replace(range, new_text.clone());
    let mut out = base.clone();
    out.replace_range(range.start as usize..range.end as usize, &new_text);
    // the node to select next: what the action made — the new slot of an
    // operator or call, the first slot inside a filled text, the edited
    // node otherwise (never a slot that was already there elsewhere)
    let select = bdl_syntax::formula(&out).ok().map(|e| {
        let mut b = Builder {
            design,
            ir,
            block,
            source: &out,
            types: &BTreeMap::new(),
            locals: Vec::new(),
        };
        let root = b.node(&e, "r".into());
        let first_slot_in = |id: &str| -> Option<String> {
            let n = root.find(id)?;
            let mut slots = Vec::new();
            n.slots(&mut slots);
            slots.into_iter().next()
        };
        match op {
            ComposeOp::Operator { op, .. } if op == "!" => Some(node_id.to_owned()),
            ComposeOp::Operator { before, .. } => {
                Some(format!("{node_id}.{}", if *before { 0 } else { 1 }))
            }
            ComposeOp::Call { arity, .. } if *arity > 1 => Some(format!("{node_id}.1")),
            ComposeOp::Apply { .. } => Some(format!("{node_id}.0")),
            ComposeOp::Binder { .. } => Some(format!("{node_id}.1")),
            ComposeOp::Range { .. } => Some(format!("{node_id}.1.0")),
            ComposeOp::Choose { .. } => Some(format!("{node_id}.0")),
            ComposeOp::Fill { .. } | ComposeOp::Insert { .. } => {
                first_slot_in(node_id).or_else(|| {
                    // no slot in what was written: the next slot after it, the
                    // Tab order, else the node itself
                    let mut slots = Vec::new();
                    root.slots(&mut slots);
                    slots
                        .into_iter()
                        .find(|s| root.find(s).is_some_and(|n| n.range.start >= range.end))
                        .or_else(|| Some(node_id.to_owned()))
                })
            }
            ComposeOp::Remove { .. } => match parent {
                Some((p, _))
                    if matches!(node.kind, NodeKind::Slot)
                        && (removes_operator(p) || matches!(p.kind, NodeKind::Unary { .. })) =>
                {
                    Some(p.id.clone())
                }
                _ => Some(node_id.to_owned()),
            },
            _ => Some(node_id.to_owned()),
        }
        .filter(|id| root.find(id).is_some())
        .unwrap_or_else(|| node_id.to_owned())
    });
    Ok(ComposeResult {
        source: out,
        edits: if empty {
            vec![TextEdit::replace(
                TextRange::new(0, source.len() as u32),
                new_text,
            )]
        } else {
            vec![edit]
        },
        select,
    })
}

// ---- structural carets: sides, navigation, insertion ---------------------------

/// Which side of a node a structural caret sits on.  A caret is a `(node,
/// side)` pair — authoring identity, stable within one projection
/// generation, never persisted: the draft text stays the authored state.
/// `Before` a slot and `After` it are the slot itself (typing into it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Before,
    After,
}

/// A structural caret position and the byte offset it stands at.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Caret {
    pub node: String,
    pub side: Side,
    pub offset: u32,
}

/// A structural motion over the projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Motion {
    /// The previous / next caret in source order (`Before` a node, its
    /// children's carets, `After` it).
    Left,
    Right,
    /// The parent's caret on the same side; the root stays.
    Up,
    /// The first child's `Before` (from `Before`), the last child's
    /// `After` (from `After`); a leaf stays.
    Down,
    /// `After` the parent.
    Exit,
    /// The next / previous slot in source order, wrapping.
    NextSlot,
    PreviousSlot,
}

/// The caret sequence of a tree in source order: `Before` each node, its
/// children's carets, `After` it.
fn caret_sequence(root: &FormulaNode) -> Vec<Caret> {
    fn go(n: &FormulaNode, out: &mut Vec<Caret>) {
        out.push(Caret {
            node: n.id.clone(),
            side: Side::Before,
            offset: n.range.start,
        });
        for c in &n.children {
            go(c, out);
        }
        out.push(Caret {
            node: n.id.clone(),
            side: Side::After,
            offset: n.range.end,
        });
    }
    let mut out = Vec::new();
    go(root, &mut out);
    out
}

/// Where a caret sits, as a byte offset into the projection's source.
pub fn caret_offset(root: &FormulaNode, node: &str, side: Side) -> Option<u32> {
    let n = root.find(node)?;
    Some(match side {
        Side::Before => n.range.start,
        Side::After => n.range.end,
    })
}

/// Move a structural caret.  Pure over the projection: the same tree and
/// the same motion give the same answer, and nothing is kept between
/// calls.  `None` when the caret names no node of the tree.
pub fn navigate(root: &FormulaNode, node: &str, side: Side, motion: Motion) -> Option<Caret> {
    let seq = caret_sequence(root);
    let here = seq.iter().position(|c| c.node == node && c.side == side)?;
    let at = |i: usize| seq.get(i).cloned();
    match motion {
        Motion::Left => at(here.saturating_sub(1)),
        Motion::Right => at((here + 1).min(seq.len() - 1)),
        Motion::Up => {
            let parent = root.parent_of(node).map(|(p, _)| p.id.clone());
            match parent {
                Some(p) => seq.iter().find(|c| c.node == p && c.side == side).cloned(),
                None => at(here),
            }
        }
        Motion::Down => {
            let n = root.find(node)?;
            let child = match side {
                Side::Before => n.children.first(),
                Side::After => n.children.last(),
            };
            match child {
                Some(c) => seq
                    .iter()
                    .find(|x| x.node == c.id && x.side == side)
                    .cloned(),
                None => at(here),
            }
        }
        Motion::Exit => {
            let parent = root.parent_of(node).map(|(p, _)| p.id.clone());
            match parent {
                Some(p) => seq
                    .iter()
                    .find(|c| c.node == p && c.side == Side::After)
                    .cloned(),
                None => at(here),
            }
        }
        Motion::NextSlot | Motion::PreviousSlot => {
            let mut slots = Vec::new();
            root.slots(&mut slots);
            if slots.is_empty() {
                return at(here);
            }
            let offset = seq[here].offset;
            let mut ordered: Vec<(u32, String)> = slots
                .into_iter()
                .filter_map(|s| root.find(&s).map(|n| (n.range.start, s)))
                .collect();
            ordered.sort();
            let pick = match motion {
                Motion::NextSlot => ordered
                    .iter()
                    .find(|(o, s)| *o > offset || (*o == offset && s != node))
                    .or(ordered.first()),
                _ => ordered
                    .iter()
                    .rev()
                    .find(|(o, s)| *o < offset || (*o == offset && s != node))
                    .or(ordered.last()),
            };
            pick.map(|(o, s)| Caret {
                node: s.clone(),
                side: Side::Before,
                offset: *o,
            })
        }
    }
}

/// The two-sided operators the keyboard may insert at a caret.
const OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "<", "<=", ">", ">=", "==", "!=", "&&", "||", "??", "in",
];

/// A unit by registry id or by canonical spelling.
fn resolve_unit_ref(text: &str) -> Option<units::UnitExpr> {
    if let Some(def) = units::by_id(text) {
        return units::UnitExpr::atom(def).ok();
    }
    units::UnitExpr::parse_canonical(text).ok()
}

/// The canonical text a word stands for when it fills a slot: an
/// equation with one slot per argument, a relationship with inputs as a
/// call, `if` as a choice, a binder word as its form, else the word.
fn canonical_fill(design: &Design, block: &MappingBlock, root: &FormulaNode, word: &str) -> String {
    match word {
        "if" => return "if ? then ? else ?".into(),
        "all" | "any" | "map" | "filter" => {
            let local = fresh_local(design, block, root, "");
            return format!("{word} {local} in ?: ?");
        }
        "delay" => return "delay(?, ?)".into(),
        "sync" => return "sync(?, ?, ?)".into(),
        _ => {}
    }
    if let Some(m) = design.mappings.values().find(|m| m.name == word) {
        if !m.signature.is_unit_domain() {
            let slots = vec!["?"; m.signature.inputs.len()].join(", ");
            return format!("{word}({slots})");
        }
        return word.to_owned();
    }
    if let Some(e) = equations::lookup(word) {
        let args: Vec<&str> = e
            .scheme
            .params
            .iter()
            .map(|p| match p {
                PTy::Arr(a, _) => match a.as_ref() {
                    PTy::Prod(..) => "(x, y) => ?",
                    _ => "x => ?",
                },
                _ => "?",
            })
            .collect();
        return format!("{word}({})", args.join(", "));
    }
    word.to_owned()
}

/// Interpret keyboard `text` at the caret `(node, side)`: the byte range
/// to replace and the text to put there.
#[allow(clippy::too_many_arguments)]
fn insertion(
    node: &FormulaNode,
    parent: Option<(&FormulaNode, usize)>,
    side: Side,
    text: &str,
    root: &FormulaNode,
    design: &Design,
    block: &MappingBlock,
) -> Result<(TextRange, String), QueryError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(QueryError::NotApplicable {
            reason: "nothing to insert".into(),
        });
    }
    let is_slot = matches!(node.kind, NodeKind::Slot);
    let is_number = matches!(
        node.kind,
        NodeKind::Number { .. } | NodeKind::Quantity { .. }
    );
    let _ = parent;
    // an operator after (or before) a value: the value becomes one operand
    if OPERATORS.contains(&text) && !is_slot {
        let p = op_precedence(text);
        let before = side == Side::Before;
        let under = if is_comparison(text) {
            COMPARISON_OPERAND
        } else if before {
            p + 1
        } else {
            p
        };
        let operand = operand(node, under);
        let made = if before {
            format!("? {text} {operand}")
        } else {
            format!("{operand} {text} ?")
        };
        return Ok((node.range, made));
    }
    if text == "!" && (is_slot || side == Side::Before) {
        return Ok((node.range, format!("!{}", operand(node, UNARY))));
    }
    if text == "(" {
        // a slot becomes a grouped slot; a value is grouped
        let inner = if is_slot {
            "?".to_owned()
        } else {
            node.text.trim().to_owned()
        };
        return Ok((node.range, format!("({inner})")));
    }
    // a unit word, `per`, `*` or `^` after a number extends its unit: the
    // literal's text grows; a resulting incomplete unit (`10 deg per`) is a
    // draft state the parser names, and completion offers the units
    if is_number && side == Side::After {
        let base = node.text.trim();
        let extended = match text {
            "per" | "*" | "^" => {
                if matches!(node.kind, NodeKind::Number { .. }) {
                    return Err(QueryError::NotApplicable {
                        reason: "a plain number has no unit to extend; write a unit first".into(),
                    });
                }
                if text == "^" {
                    format!("{base}^")
                } else {
                    format!("{base} {text} ")
                }
            }
            w if units::lookup(w).is_some() => format!("{base} {w}"),
            w if w.chars().all(|c| c.is_ascii_digit() || c == '-') && base.ends_with('^') => {
                format!("{base}{w}")
            }
            _ => {
                return Err(QueryError::NotApplicable {
                    reason: format!(
                        "`{text}` cannot follow a number here; a unit, `per`, `*` or `^` can"
                    ),
                })
            }
        };
        return Ok((node.range, extended));
    }
    // a word or number into a slot: its canonical form
    if is_slot {
        let filled = if text.parse::<f64>().is_ok() || text == "true" || text == "false" {
            text.to_owned()
        } else if text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            canonical_fill(design, block, root, text)
        } else {
            return Err(QueryError::NotApplicable {
                reason: format!(
                    "`{text}` is not something a slot takes; type a name, a number or a form"
                ),
            });
        };
        return Ok((node.range, filled));
    }
    Err(QueryError::NotApplicable {
        reason: format!(
            "`{text}` cannot stand {} `{}`; insert an operator first",
            match side {
                Side::Before => "before",
                Side::After => "after",
            },
            node.text.trim()
        ),
    })
}

// ---- signature help --------------------------------------------------------------

/// One parameter of a call, for signature help.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterHelp {
    pub name: String,
    /// What the parameter takes, in the designer's words, when known
    /// (the equation's scheme instantiated by the arguments written; a
    /// relationship's input concept).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub expected: String,
}

/// What a call at or around a caret takes: the callee, its parameters,
/// its result and which argument the caret is in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureHelp {
    /// The call node.
    pub node: String,
    pub name: String,
    /// `clamp(x, lo, hi)`.
    pub shape: String,
    pub parameters: Vec<ParameterHelp>,
    /// What the call gives, in the designer's words.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub result: String,
    /// The argument the caret is in, when it is in one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<u32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
}

/// The signature help for the innermost call enclosing `node` (or `node`
/// itself when it is a call); `None` when no call encloses it.
pub fn signature(
    projection: &FormulaProjection,
    design: &Design,
    node: &str,
) -> Option<SignatureHelp> {
    let root = projection.root.as_ref()?;
    // walk up from the node to the nearest call, remembering which child
    // we came from
    let mut current = node.to_owned();
    let mut active: Option<u32> = None;
    loop {
        let n = root.find(&current)?;
        if let NodeKind::Call {
            name,
            equation,
            entity,
        } = &n.kind
        {
            let (params, result, summary): (Vec<ParameterHelp>, String, String) = if *equation {
                let e = equations::lookup(name)?;
                let params = e
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| ParameterHelp {
                        name: (*p).to_owned(),
                        expected: n
                            .children
                            .get(i)
                            .and_then(|c| c.expected.as_ref())
                            .map(|t| t.description.clone())
                            .unwrap_or_default(),
                    })
                    .collect();
                (
                    params,
                    n.actual
                        .as_ref()
                        .map(|t| t.description.clone())
                        .unwrap_or_default(),
                    e.summary.to_owned(),
                )
            } else if let Some(EntityRef::Mapping(id)) = entity {
                let m = design.mappings.get(id)?;
                let params = m
                    .signature
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(i, c)| ParameterHelp {
                        name: m
                            .parameters
                            .get(i)
                            .filter(|p| !p.is_empty())
                            .cloned()
                            .or_else(|| design.concepts.get(c).map(|c| c.name.clone()))
                            .unwrap_or_default(),
                        expected: design
                            .concepts
                            .get(c)
                            .map(|c| c.name.clone())
                            .unwrap_or_default(),
                    })
                    .collect();
                (
                    params,
                    design
                        .concepts
                        .get(&m.signature.output)
                        .map(|c| c.name.clone())
                        .unwrap_or_default(),
                    m.description.clone(),
                )
            } else {
                (Vec::new(), String::new(), String::new())
            };
            let shape = format!(
                "{name}({})",
                params
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Some(SignatureHelp {
                node: n.id.clone(),
                name: name.clone(),
                shape,
                parameters: params,
                result,
                active,
                summary,
            });
        }
        let (p, i) = root.parent_of(&current)?;
        active = Some(i as u32);
        current = p.id.clone();
    }
}

// ---- the read-only render --------------------------------------------------------

/// One piece of a rendered formula, in source order: the node it belongs
/// to (a leaf's own text, or the words and marks between a node's
/// children) and what it is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fragment {
    pub text: String,
    /// `reference`, `local`, `number`, `unit`, `bool`, `slot`,
    /// `operator`, `keyword`, `punctuation`, `pattern`, `name`.
    pub kind: String,
    /// The node the fragment belongs to (`r.1` for the `+` of `a + b`).
    pub node: String,
    pub range: TextRange,
}

/// The display projection of a formula: the same tree the editor uses,
/// flattened to fragments a canvas node can lay out — nothing parsed a
/// second time.  Composite units carry their mathematical rendering
/// beside the source text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormulaRender {
    pub fragments: Vec<Fragment>,
    /// The formatted source on one line, for a compact summary.
    pub compact: String,
    /// The result type, in the designer's words.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub result: String,
    pub error_count: u32,
    pub warning_count: u32,
    /// The references the formula makes, by name.
    pub references: Vec<String>,
}

/// Flatten a projection into fragments.
pub fn render(projection: &FormulaProjection) -> FormulaRender {
    let source = projection.source.as_str();
    let mut fragments = Vec::new();
    let mut references = Vec::new();
    fn kind_of_gap(text: &str) -> &'static str {
        if text.chars().all(|c| c.is_ascii_alphabetic() || c == '_') {
            "keyword"
        } else if text
            .chars()
            .all(|c| matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':'))
        {
            "punctuation"
        } else {
            "operator"
        }
    }
    fn gap(source: &str, from: u32, to: u32, node: &str, out: &mut Vec<Fragment>) {
        if to <= from {
            return;
        }
        let Some(text) = source.get(from as usize..to as usize) else {
            return;
        };
        // each maximal run of one class — a word, an operator, a single
        // delimiter — is one fragment; whitespace separates
        let class = |c: char| -> u8 {
            if c.is_whitespace() {
                0
            } else if c.is_ascii_alphanumeric() || c == '_' {
                1
            } else if matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':') {
                3
            } else {
                2
            }
        };
        let mut start = from as usize;
        let mut current = 0u8;
        let flush = |start: usize, end: usize, out: &mut Vec<Fragment>| {
            if end > start {
                let t = &source[start..end];
                out.push(Fragment {
                    text: t.to_owned(),
                    kind: kind_of_gap(t).into(),
                    node: node.to_owned(),
                    range: TextRange::new(start as u32, end as u32),
                });
            }
        };
        for (i, c) in text.char_indices() {
            let abs = from as usize + i;
            let k = class(c);
            if k == 0 {
                flush(start, abs, out);
                start = abs + c.len_utf8();
                current = 0;
                continue;
            }
            // a delimiter is always its own fragment; a class change ends a run
            if k != current || k == 3 {
                flush(start, abs, out);
                start = abs;
                current = k;
            }
        }
        flush(start, to as usize, out);
    }
    fn go(n: &FormulaNode, source: &str, out: &mut Vec<Fragment>, refs: &mut Vec<String>) {
        let leaf_kind = match &n.kind {
            NodeKind::Reference { local: true, .. } => Some("local"),
            NodeKind::Reference { name, .. } => {
                if !refs.contains(name) {
                    refs.push(name.clone());
                }
                Some("reference")
            }
            NodeKind::Number { .. } => Some("number"),
            NodeKind::Bool { .. } => Some("bool"),
            NodeKind::Slot => Some("slot"),
            NodeKind::Opaque { .. } => Some("punctuation"),
            _ => None,
        };
        if let Some(k) = leaf_kind {
            out.push(Fragment {
                text: n.text.trim().to_owned(),
                kind: k.into(),
                node: n.id.clone(),
                range: n.range,
            });
            return;
        }
        if let NodeKind::Quantity {
            coordinate,
            unit,
            unit_display,
            ..
        } = &n.kind
        {
            let coord_end = n.range.start + coordinate.len() as u32;
            out.push(Fragment {
                text: coordinate.clone(),
                kind: "number".into(),
                node: n.id.clone(),
                range: TextRange::new(n.range.start, coord_end),
            });
            out.push(Fragment {
                text: unit_display.clone().unwrap_or_else(|| unit.clone()),
                kind: "unit".into(),
                node: n.id.clone(),
                range: TextRange::new(coord_end, n.range.end),
            });
            return;
        }
        // an inner node: the text between its children is its own —
        // operators, keywords, delimiters, a pattern before `=>`; a call's
        // head word is the reference it makes
        let head_word = match &n.kind {
            NodeKind::Call { name, equation, .. } => {
                if !*equation && !refs.contains(name) {
                    refs.push(name.clone());
                }
                Some(name.clone())
            }
            NodeKind::Delay => Some("delay".to_owned()),
            NodeKind::Sync => Some("sync".to_owned()),
            _ => None,
        };
        let first = out.len();
        let mut cursor = n.range.start;
        for c in &n.children {
            let skip_pattern = matches!(n.kind, NodeKind::Arm { .. } | NodeKind::Let { .. });
            if skip_pattern && cursor == n.range.start {
                // the pattern (and `let`) before the child
                let head = source
                    .get(cursor as usize..c.range.start as usize)
                    .unwrap_or("");
                let head_trim = head.trim_end();
                if let NodeKind::Arm { pattern, .. } | NodeKind::Let { pattern, .. } = &n.kind {
                    let pat_start = head_trim
                        .rfind(pattern.as_str())
                        .map(|i| cursor as usize + i);
                    if let Some(ps) = pat_start {
                        gap(source, cursor, ps as u32, &n.id, out);
                        out.push(Fragment {
                            text: pattern.clone(),
                            kind: "pattern".into(),
                            node: n.id.clone(),
                            range: TextRange::new(ps as u32, (ps + pattern.len()) as u32),
                        });
                        gap(
                            source,
                            (ps + pattern.len()) as u32,
                            c.range.start,
                            &n.id,
                            out,
                        );
                        go(c, source, out, refs);
                        cursor = c.range.end;
                        continue;
                    }
                }
            }
            gap(source, cursor, c.range.start, &n.id, out);
            go(c, source, out, refs);
            cursor = c.range.end;
        }
        gap(source, cursor, n.range.end, &n.id, out);
        if let Some(word) = head_word {
            if let Some(f) = out[first..].iter_mut().find(|f| f.text == word) {
                f.kind = if matches!(n.kind, NodeKind::Call { equation: true, .. }) {
                    "equation".into()
                } else if matches!(n.kind, NodeKind::Call { .. }) {
                    "reference".into()
                } else {
                    "keyword".into()
                };
            }
        }
    }
    if let Some(root) = &projection.root {
        go(root, source, &mut fragments, &mut references);
    }
    let (mut errors, mut warnings) = (0u32, 0u32);
    let mut count = |ds: &[SemanticDiagnostic]| {
        for d in ds {
            match d.severity {
                crate::diagnostics::SemanticSeverity::Error => errors += 1,
                crate::diagnostics::SemanticSeverity::Warning => warnings += 1,
                _ => {}
            }
        }
    };
    if let Some(root) = &projection.root {
        let mut all = Vec::new();
        root.walk(&mut all);
        for n in all {
            count(&n.diagnostics);
        }
    }
    count(&projection.unplaced);
    FormulaRender {
        compact: source.split_whitespace().collect::<Vec<_>>().join(" "),
        result: projection
            .result
            .as_ref()
            .map(|t| t.description.clone())
            .unwrap_or_default(),
        error_count: errors,
        warning_count: warnings,
        references,
        fragments,
    }
}

/// A readable fresh name for a binder's local: the collection's name
/// without its plural `s` when that is a free identifier (`readings` →
/// `reading`), else `item`, `item2`, … — never a name in scope (an input,
/// a relationship, a concept, an enclosing local, a word of the language).
fn fresh_local(
    design: &Design,
    block: &MappingBlock,
    root: &FormulaNode,
    collection: &str,
) -> String {
    let mut taken: Vec<String> = Vec::new();
    taken.extend(design.mappings.values().map(|m| m.name.clone()));
    taken.extend(design.concepts.values().map(|c| c.name.clone()));
    taken.extend(block.parameters.iter().cloned());
    fn locals(n: &FormulaNode, out: &mut Vec<String>) {
        if let NodeKind::Binder { param, .. } = &n.kind {
            out.push(param.clone());
        }
        n.children.iter().for_each(|c| locals(c, out));
    }
    locals(root, &mut taken);
    fresh_local_name(collection, &taken)
}

/// The fresh name itself: `readings` → `reading` when that is free, else
/// `item`, `item2`, …  `taken` lists the names of the design and the
/// formula; the words of the language and the equations are always taken.
pub(crate) fn fresh_local_name(collection: &str, taken: &[String]) -> String {
    let mut taken: std::collections::BTreeSet<String> = taken.iter().cloned().collect();
    taken.extend(
        [
            "all", "any", "map", "filter", "in", "if", "then", "else", "match", "let", "true",
            "false", "delay", "sync", "None", "Some", "ordered", "concept", "mapping",
        ]
        .map(String::from),
    );
    taken.extend(equations::names().into_iter().map(String::from));
    let free = |n: &str| !taken.contains(n) && bdl_syntax::formula(n).is_ok();
    let name = collection.trim();
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        if let Some(singular) = name.strip_suffix('s') {
            // a local reads as a value, so `Readings` gives `reading`
            let mut chars = singular.chars();
            let singular: String = match chars.next() {
                Some(c) => c.to_lowercase().chain(chars).collect(),
                None => String::new(),
            };
            if singular.len() >= 2 && !singular.ends_with('s') && free(&singular) {
                return singular;
            }
        }
    }
    if free("item") {
        return "item".into();
    }
    (2..)
        .map(|i| format!("item{i}"))
        .find(|n| free(n))
        .unwrap_or_else(|| "item".into())
}

/// A converted coordinate as source text: shortest round-trip form,
/// never exponent notation for ordinary magnitudes.
fn format_coordinate(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        let s = format!("{x}");
        if s.contains('e') {
            format!("{x:.12}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_owned()
        } else {
            s
        }
    }
}
