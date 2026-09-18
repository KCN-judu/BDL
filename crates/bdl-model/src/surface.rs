//! The surface model: what a designer authors.
//!
//! This is the *canonical* project content.  It is deliberately a surface
//! representation — mapping blocks with signatures over concepts and an
//! optional definition — not the kernel: elaboration (a later crate) turns
//! it into the Design IR and the Reactive Core IR.  The model may be
//! incomplete at any time; incompleteness is a legal state, not an error.
//!
//! Display names live here because designers need them, but nothing refers
//! to anything by name: all references are by stable id.

use crate::dim::Dim;
use crate::ids::{ClockId, DeclId, DeviceId, IdAllocator, OutputId, Revision, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An immutable view of the project at one revision.  Edits produce a new
/// snapshot (see [`crate::edit`]); analyses take snapshots by value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub revision: Revision,
    pub design: Design,
}

impl ProjectSnapshot {
    pub fn new(design: Design) -> Self {
        ProjectSnapshot {
            revision: Revision::INITIAL,
            design,
        }
    }
}

/// The persisted semantic content of a project.  Ordered maps keep every
/// traversal deterministic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Design {
    pub name: String,
    pub concepts: BTreeMap<SemanticId, Concept>,
    pub mappings: BTreeMap<DeclId, MappingBlock>,
    /// Timing domains, by nominal identity ("interaction", "ambient").  A
    /// domain says which values are updated *together*; never a rate.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub clocks: BTreeMap<ClockId, ClockDomain>,
    /// Physical sinks: nominal resources the product finally drives.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub outputs: BTreeMap<OutputId, PhysicalOutput>,
    /// Deployment layer: what kind of hardware realises a sink or feeds the
    /// product.  Never consulted by semantic analysis.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub devices: BTreeMap<DeviceId, DeviceBinding>,
    pub ids: IdAllocator,
}

impl Design {
    /// Move the allocator past every identity the design holds, so that a
    /// design assembled from copied entities issues fresh ones only.
    pub fn reserve_ids(&mut self) {
        self.ids = self.ids.covering(
            self.concepts.keys().map(|k| k.raw()).max(),
            self.mappings.keys().map(|k| k.raw()).max(),
            self.clocks.keys().map(|k| k.raw()).max(),
            self.outputs.keys().map(|k| k.raw()).max(),
            self.devices.keys().map(|k| k.raw()).max(),
        );
    }

    pub fn empty(name: impl Into<String>) -> Self {
        Design {
            name: name.into(),
            concepts: BTreeMap::new(),
            mappings: BTreeMap::new(),
            clocks: BTreeMap::new(),
            outputs: BTreeMap::new(),
            devices: BTreeMap::new(),
            ids: IdAllocator::default(),
        }
    }

    pub fn concept(&self, id: SemanticId) -> Option<&Concept> {
        self.concepts.get(&id)
    }

    pub fn mapping(&self, id: DeclId) -> Option<&MappingBlock> {
        self.mappings.get(&id)
    }

    /// Mappings that drive `output` (at most one in a well-formed design).
    pub fn drivers_of(&self, output: OutputId) -> impl Iterator<Item = &MappingBlock> {
        self.mappings
            .values()
            .filter(move |m| m.drives == Some(output))
    }

    /// Mappings whose signature mentions `concept` (as input or output).
    pub fn mappings_using(&self, concept: SemanticId) -> impl Iterator<Item = &MappingBlock> {
        self.mappings
            .values()
            .filter(move |m| m.signature.mentions(concept))
    }
}

/// A timing domain (the kernel's `ClockId`): identity, not rate.  Rates are
/// deployment/validation data and never enter the design.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockDomain {
    pub id: ClockId,
    pub name: String,
}

/// A physical sink (the kernel's `OutputSpec` plus surface facts).  "The
/// light" is a resource; "the desired brightness" is a value: the two are
/// different sorts.  A sink accepts one concept and lives in one timing
/// domain; a driver must produce exactly that concept in exactly that domain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalOutput {
    pub id: OutputId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// The concept this sink accepts (`OutputSpec.accepts = sem concept`).
    pub accepts: SemanticId,
    /// The domain the sink updates in; `None` while still open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<ClockId>,
    /// Whether an executable design must drive this sink.
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// A kind of hardware, described by what it needs from a board.  The
/// allocation solver never sees these names, only the requirements they
/// generate (`bdl-hardware::devices`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    /// One PWM line (a dimmable light, a servo signal).
    PwmChannel,
    /// One digital output line (a relay, a switched load).
    DigitalOutput,
    /// An H-bridge motor channel: one PWM line and one direction line.
    HBridgeChannel,
    /// A sensor on the I2C bus: SDA and SCL on the same peripheral unit.
    I2cSensor,
    /// A quadrature encoder: two interrupt-capable lines.
    QuadratureEncoder,
    /// A serial link: TX and RX on the same UART unit.
    Uart,
}

/// Deployment-layer binding of hardware to the design: which device kind
/// realises a sink (or feeds the product, for sensors), and any pins the
/// designer fixed by hand.  Fixed pins are named board-relatively and
/// resolved against the target at deployment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceBinding {
    pub id: DeviceId,
    pub name: String,
    pub kind: DeviceKind,
    /// The sink this device realises; `None` for sensors and other devices
    /// that feed the design.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputId>,
    /// Manual pin choices, by the device's local requirement index.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fixed_pins: BTreeMap<u16, String>,
}

/// A semantic property: `Tilt`, `Brightness`, `Held`.  Identity is the
/// `SemanticId`; the name is mutable documentation.  The representation is a
/// write-once binding (the kernel's `Θ s = some R`); a concept without one is
/// a legal, still-open declaration of intent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Concept {
    pub id: SemanticId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representation: Option<Representation>,
    /// The designer declared this concept *ordered*: its values may be
    /// compared with `<`, `min`, `max`, `clamp`, `inRange` — through the
    /// quantity that represents them.  Never inferred from the
    /// representation (a `Mode` encoded as a number has no order); the
    /// formal `OrdDecl` (Phase 9c).  Meaningful only with a quantity
    /// representation; equality needs no declaration.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ordered: bool,
}

/// What a concept is represented by.  Must be a semantic-free *data* type
/// (kernel `ConceptEnv.WF`); the enum makes that unrepresentable otherwise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Representation {
    /// A physical quantity of dimension `dim` (`q dim`).  Dimensionless
    /// levels such as `Brightness ∈ [0,1]` use `Dim::ZERO`.
    Quantity { dim: Dim },
    /// A truth value (`bool`).
    Boolean,
    /// A count (`nat`).
    Count,
    /// A value that may be absent (`opt R`).
    Optional { inner: Box<Representation> },
    /// A finite collection of values, in order (`list R`, Phase 9a).
    List { element: Box<Representation> },
    /// A grouped value: two parts side by side (`R₁ × R₂`, Phase 9b).
    Pair {
        first: Box<Representation>,
        second: Box<Representation>,
    },
}

impl Representation {
    pub fn quantity(dim: Dim) -> Representation {
        Representation::Quantity { dim }
    }
    pub fn optional(inner: Representation) -> Representation {
        Representation::Optional {
            inner: Box::new(inner),
        }
    }
    pub fn list(element: Representation) -> Representation {
        Representation::List {
            element: Box::new(element),
        }
    }
    pub fn pair(first: Representation, second: Representation) -> Representation {
        Representation::Pair {
            first: Box::new(first),
            second: Box::new(second),
        }
    }
    /// The quantity dimension when the representation is a plain quantity.
    pub fn as_quantity(&self) -> Option<Dim> {
        match self {
            Representation::Quantity { dim } => Some(*dim),
            _ => None,
        }
    }
}

/// A mapping block: a declaration with a signature over concepts and an
/// optional definition.  `definition == None` is the paper's "hole" — an
/// unresolved declaration, distinguished by nothing else.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingBlock {
    pub id: DeclId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub signature: Signature,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<Definition>,
    /// The domain this relationship updates in; `None` for a pure mapping
    /// that may serve any domain (the kernel's `Κ d = none`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<ClockId>,
    /// The drive edge (`β d`): the sink this declaration is the final
    /// driver of.  Write-once as a refinement; retargeting is an edit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drives: Option<OutputId>,
    /// The names a formula body uses for the inputs, positionally — the
    /// textual surface's `f(t, held) = …` (TEXTUAL_SYNTAX §14.4).  Empty
    /// for a relationship authored in Studio: its body names the concepts
    /// themselves (ADR-0013).  A missing or empty entry falls back to the
    /// concept's name for that position.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<String>,
}

impl MappingBlock {
    pub fn is_unresolved(&self) -> bool {
        self.definition.is_none()
    }
}

/// `(A₁, …, Aₙ) -> B` over concepts.  A nullary signature `() -> B` is a
/// value declaration of type `B`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub inputs: Vec<SemanticId>,
    pub output: SemanticId,
}

impl Signature {
    pub fn mentions(&self, concept: SemanticId) -> bool {
        self.output == concept || self.inputs.contains(&concept)
    }
}

/// A definition attached to a mapping block.  All forms elaborate to the same
/// kernel realization; the surface keeps the authoring form so it can be
/// re-edited in the way it was written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Definition {
    /// A formula in the surface expression language over the signature's
    /// input names.  Parsed and checked by the compiler, never by the editor.
    Formula { source: String },
    /// A formula whose free names are pinned to identities: what a
    /// component body's formula becomes when the body is instantiated into
    /// a system (docs/architecture/behavior-systems.md §8).  `source` is the
    /// designer's text, unchanged; `scope` says which input position and
    /// which relationship each name meant *in the component*, so neither a
    /// display name in the flattened design nor another instance's
    /// homonym can capture it.  Never authored by hand.
    ScopedFormula { source: String, scope: FormulaScope },
    /// A realization by identity: this declaration *is* `target` — the
    /// kernel's `declRef target`, or `sync src init (declRef target)` when
    /// transported from `src`'s domain.  Produced by system flattening for
    /// a port binding (docs/architecture/behavior-systems.md §8); never
    /// authored by hand and never resolved through a name.  `init` is a
    /// closed formula (no inputs, no relationships in scope).
    Reference {
        target: DeclId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport: Option<Transport>,
    },
}

impl Definition {
    /// The formula text, for the text-oriented tools; a reference has none.
    pub fn formula_source(&self) -> Option<&str> {
        match self {
            Definition::Formula { source } | Definition::ScopedFormula { source, .. } => {
                Some(source)
            }
            Definition::Reference { .. } => None,
        }
    }
}

/// The names a scoped formula may use and what each one is.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FormulaScope {
    /// The input names in signature order (the concepts' display names as
    /// the component saw them).
    pub inputs: Vec<String>,
    /// Relationships the formula may reference or apply, by the names the
    /// component used, resolved to their flattened identities.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mappings: BTreeMap<String, DeclId>,
    /// Concepts the formula may name (for the "not an input" diagnostic).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub concepts: BTreeMap<String, SemanticId>,
}

/// Transport of a referenced value across timing domains: the kernel's
/// `sync src init e`, strictly before (docs/spec/runtime-semantics.md).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transport {
    /// The source's domain.
    pub source: ClockId,
    /// The value before the first activation of the source: a closed
    /// formula in the destination's units.
    pub init: String,
}
