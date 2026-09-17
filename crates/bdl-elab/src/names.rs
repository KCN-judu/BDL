//! Formula names.
//!
//! A mapping's signature holds `SemanticId`s; the designer types names.  In
//! v0 the name of an input *is the concept's display name* at elaboration
//! time (ADR-0013): resolution yields an index into the signature, the Core
//! term holds only de Bruijn indices, and renaming a concept simply changes
//! what the formula must say — the stored formula text is re-resolved on
//! every analysis, so a stale name surfaces as `formula.name.unknown` with
//! the current names offered as fixes.
//!
//! A name that is not an input may be another **mapping** of the design
//! (DI-17): `dimByTilt(tilt)` applies the relationship, `level` reads a
//! nullary one.  Local `let` names are resolved by the elaborator before
//! this rule applies; they are lexical and never reach here.

use bdl_model::surface::{Design, FormulaScope, MappingBlock};
use bdl_model::{DeclId, SemanticId};
use std::collections::BTreeMap;

pub struct InputEnv {
    /// (concept, display name) per signature input, in signature order.
    pub inputs: Vec<(SemanticId, String)>,
    /// A pinned scope (`Definition::ScopedFormula`): relationships and
    /// concepts are looked up here by the names the component used, never
    /// in the design at large.
    pub scope: Option<PinnedScope>,
}

pub struct PinnedScope {
    pub mappings: BTreeMap<String, DeclId>,
    pub concepts: BTreeMap<String, SemanticId>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Lookup {
    /// Index into the signature inputs.
    Input(usize),
    /// Several inputs match (case-insensitively); names of the candidates.
    Ambiguous(Vec<String>),
    /// A mapping of the design (exact name, else unique case-insensitive).
    Mapping(DeclId),
    /// A concept of the project with this name that is not an input.
    NotAnInput(SemanticId, String),
    Unknown,
}

impl InputEnv {
    pub fn for_inputs(design: &Design, inputs: &[SemanticId]) -> InputEnv {
        InputEnv::with_parameters(design, inputs, &[])
    }

    /// The environment of a mapping's own body: its textual parameter
    /// names where it has them, the concepts' names otherwise
    /// (TEXTUAL_SYNTAX §14.4).
    pub fn for_mapping(design: &Design, mapping: &MappingBlock) -> InputEnv {
        InputEnv::with_parameters(design, &mapping.signature.inputs, &mapping.parameters)
    }

    /// `parameters[i]`, when present and non-empty, is the name of input
    /// `i`; every other input is named after its concept.
    pub fn with_parameters(
        design: &Design,
        inputs: &[SemanticId],
        parameters: &[String],
    ) -> InputEnv {
        InputEnv {
            inputs: inputs
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    let param = parameters.get(i).filter(|p| !p.is_empty()).cloned();
                    (
                        *id,
                        param.unwrap_or_else(|| {
                            design
                                .concepts
                                .get(id)
                                .map(|c| c.name.clone())
                                .unwrap_or_default()
                        }),
                    )
                })
                .collect(),
            scope: None,
        }
    }

    /// The environment of a scoped formula: input names come from the
    /// scope (positionally), relationships and concepts from its tables.
    pub fn scoped(inputs: &[SemanticId], scope: &FormulaScope) -> InputEnv {
        InputEnv {
            inputs: inputs
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, scope.inputs.get(i).cloned().unwrap_or_default()))
                .collect(),
            scope: Some(PinnedScope {
                mappings: scope.mappings.clone(),
                concepts: scope.concepts.clone(),
            }),
        }
    }

    /// Exact spellings first — an input, then a relationship, then a
    /// concept the mapping does not read — and only then the same three
    /// case-insensitively, so `Tilt` (a concept) is never taken for `tilt`
    /// (a relationship) when both exist.
    pub fn resolve(&self, design: &Design, name: &str) -> Lookup {
        if let Some(i) = self.inputs.iter().position(|(_, n)| n == name) {
            return Lookup::Input(i);
        }
        if let Some(scope) = &self.scope {
            return self.resolve_pinned(scope, name);
        }
        if let Some(m) = design.mappings.values().find(|m| m.name == name) {
            return Lookup::Mapping(m.id);
        }
        if let Some(c) = design.concepts.values().find(|c| c.name == name) {
            return Lookup::NotAnInput(c.id, c.name.clone());
        }
        let loose: Vec<usize> = (0..self.inputs.len())
            .filter(|&i| self.inputs[i].1.eq_ignore_ascii_case(name))
            .collect();
        match loose.as_slice() {
            [i] => return Lookup::Input(*i),
            [] => {}
            many => {
                return Lookup::Ambiguous(many.iter().map(|&i| self.inputs[i].1.clone()).collect())
            }
        }
        let loose: Vec<&MappingBlock> = design
            .mappings
            .values()
            .filter(|m| m.name.eq_ignore_ascii_case(name))
            .collect();
        if let [m] = loose.as_slice() {
            return Lookup::Mapping(m.id);
        }
        if let Some(c) = design
            .concepts
            .values()
            .find(|c| c.name.eq_ignore_ascii_case(name))
        {
            return Lookup::NotAnInput(c.id, c.name.clone());
        }
        Lookup::Unknown
    }

    fn resolve_pinned(&self, scope: &PinnedScope, name: &str) -> Lookup {
        if let Some(id) = scope.mappings.get(name) {
            return Lookup::Mapping(*id);
        }
        if let Some((n, id)) = scope.concepts.get_key_value(name) {
            return Lookup::NotAnInput(*id, n.clone());
        }
        let loose: Vec<usize> = (0..self.inputs.len())
            .filter(|&i| self.inputs[i].1.eq_ignore_ascii_case(name))
            .collect();
        match loose.as_slice() {
            [i] => return Lookup::Input(*i),
            [] => {}
            many => {
                return Lookup::Ambiguous(many.iter().map(|&i| self.inputs[i].1.clone()).collect())
            }
        }
        let loose: Vec<&DeclId> = scope
            .mappings
            .iter()
            .filter(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, id)| id)
            .collect();
        if let [id] = loose.as_slice() {
            return Lookup::Mapping(**id);
        }
        if let Some((n, id)) = scope
            .concepts
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
        {
            return Lookup::NotAnInput(*id, n.clone());
        }
        Lookup::Unknown
    }

    pub fn names(&self) -> Vec<&str> {
        self.inputs.iter().map(|(_, n)| n.as_str()).collect()
    }
}

/// The relationships a formula may call, as `name(…)` / `name`, for fixes.
pub fn mapping_names(design: &Design) -> Vec<String> {
    design
        .mappings
        .values()
        .map(|m| {
            if m.signature.inputs.is_empty() {
                m.name.clone()
            } else {
                format!("{}(…)", m.name)
            }
        })
        .collect()
}
