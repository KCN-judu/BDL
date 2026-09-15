//! Formula input names.
//!
//! A mapping's signature holds `SemanticId`s; the designer types names.  In
//! v0 the name of an input *is the concept's display name* at elaboration
//! time (ADR-0013): resolution yields an index into the signature, the Core
//! term holds only de Bruijn indices, and renaming a concept simply changes
//! what the formula must say — the stored formula text is re-resolved on
//! every analysis, so a stale name surfaces as `formula.name.unknown` with
//! the current names offered as fixes.

use bdl_model::surface::Design;
use bdl_model::SemanticId;

pub struct InputEnv {
    /// (concept, display name) per signature input, in signature order.
    pub inputs: Vec<(SemanticId, String)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Lookup {
    /// Index into the signature inputs.
    Input(usize),
    /// Several inputs match (case-insensitively); names of the candidates.
    Ambiguous(Vec<String>),
    /// A concept of the project with this name that is not an input.
    NotAnInput(SemanticId, String),
    Unknown,
}

impl InputEnv {
    pub fn for_inputs(design: &Design, inputs: &[SemanticId]) -> InputEnv {
        InputEnv {
            inputs: inputs
                .iter()
                .map(|id| {
                    (
                        *id,
                        design
                            .concepts
                            .get(id)
                            .map(|c| c.name.clone())
                            .unwrap_or_default(),
                    )
                })
                .collect(),
        }
    }

    pub fn resolve(&self, design: &Design, name: &str) -> Lookup {
        if let Some(i) = self.inputs.iter().position(|(_, n)| n == name) {
            return Lookup::Input(i);
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
        if let Some(c) = design
            .concepts
            .values()
            .find(|c| c.name == name || c.name.eq_ignore_ascii_case(name))
        {
            return Lookup::NotAnInput(c.id, c.name.clone());
        }
        Lookup::Unknown
    }

    pub fn names(&self) -> Vec<&str> {
        self.inputs.iter().map(|(_, n)| n.as_str()).collect()
    }
}
