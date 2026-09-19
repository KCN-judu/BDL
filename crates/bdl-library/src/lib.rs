//! The BDL Standard Library: reusable authoring fragments built entirely
//! from ordinary BDL structures.
//!
//! A library is a set of [`LibraryItem`]s.  An item is *not* a project
//! object: it carries a [`Fragment`] — a few coordinated concepts and
//! relationships with good defaults — and [`plan`] turns it into the
//! ordinary `CreateConcept` / `CreateMapping` edits that produce them,
//! applied by the daemon in one transaction (one revision, one undo step;
//! nothing half-applied).  The compiler allocates fresh identities; from
//! then on every object is an independent copy of the defaults — renamed,
//! rebound, deleted like any other.  Two instantiations are two sets of
//! objects; the project never records where an object came from, and a
//! later library version cannot change it.
//!
//! Two categories today ([`ItemCategory`]):
//!
//! * **Concept** — one concept (`Temperature`, `AmbientLight`, …), the
//!   [`ConceptTemplate`] of the original Concept Library, kept as a view.
//! * **Source** — an environment-provided value entering the behavior
//!   model: a semantic concept and an unresolved relationship without
//!   inputs, `mapping TempSensor : () -> RoomTemp`.  Nothing about it is a
//!   sensor primitive or an I/O operation: the relationship is ordinary,
//!   normally left unresolved, and so a simulation input and, later, a
//!   deployment realization point.  Source ≠ sensor: `External Value` is a
//!   host or network value.
//!
//! Data-driven: `library/std/concepts.toml`, embedded at build time and
//! loadable from disk like any future team, project or package library
//! ([`Library::from_toml`]).  Display names and descriptions in the file
//! are the canonical English; localized presentation lives in Studio's
//! catalogs, keyed by the item id, never here.  Representations reference
//! the shared quantity vocabulary (`bdl_model::quantity`) and units the
//! shared unit table (`bdl_elab::units`); the library has no dimension
//! table of its own.
//!
//! What this crate is not: a device catalogue, a kernel type, a resolution
//! mechanism (search is authoring convenience), or a package manager.

#![forbid(unsafe_code)]

use bdl_model::surface::{Design, Representation};
use bdl_model::{quantity, EditOp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// The schema of the TOML files this crate reads: 2 adds `[[source]]`
/// items; 1 (concept templates only) is still read.
pub const SCHEMA_VERSION: u32 = 2;
/// The oldest schema still read.
pub const OLDEST_SCHEMA_VERSION: u32 = 1;

/// The embedded Standard Concept Library.
pub const STANDARD_LIBRARY_TOML: &str = include_str!("../../../library/std/concepts.toml");

/// Where a template expects to be used.  Discovery only: it groups the
/// quick-insert menu and nothing else.  A concept has no role in the
/// kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleHint {
    Input,
    Output,
    Either,
}

/// The representation a template suggests, as authored in the file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum RepresentationSpec {
    Named(NamedRepresentation),
    Quantity { quantity: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedRepresentation {
    Boolean,
    Count,
    /// Decide later: the concept is created without a representation.
    Open,
}

/// One template, as loaded.  Every field but the identity and the name is
/// a *default* the designer may change after instantiation.  A key the
/// schema does not have (a schema-1 `source` or `i18n`) refuses the file
/// rather than loading a different item than the author meant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConceptTemplate {
    /// Stable library identity (`std.environment.temperature`).  Never a
    /// `SemanticId`.
    pub id: String,
    /// Shown in menus and panels (`Ambient Light`).
    pub display_name: String,
    /// The concept's name on creation (`AmbientLight`), made unique in the
    /// project if taken.
    pub default_name: String,
    #[serde(default)]
    pub description: String,
    /// Grouping for browsing (`environment`, `motion`, …).
    pub category: String,
    #[serde(default = "either")]
    pub role_hint: RoleHint,
    pub representation: RepresentationSpec,
    /// A unit symbol from the shared unit table, for display and as the
    /// suggested unit; the dimension is the quantity's.
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    /// A generic presentation hint (`temperature`, `motor`); never semantic.
    #[serde(default)]
    pub icon: String,
}

fn either() -> RoleHint {
    RoleHint::Either
}

impl ConceptTemplate {
    /// The model representation the template's spec resolves to.  `None`
    /// for `open`.
    pub fn representation(&self) -> Option<Representation> {
        match &self.representation {
            RepresentationSpec::Named(NamedRepresentation::Boolean) => {
                Some(Representation::Boolean)
            }
            RepresentationSpec::Named(NamedRepresentation::Count) => Some(Representation::Count),
            RepresentationSpec::Named(NamedRepresentation::Open) => None,
            RepresentationSpec::Quantity { quantity: q } => Some(Representation::Quantity {
                dim: quantity::lookup(q)?.dim,
            }),
        }
    }

    /// The quantity's textual type name (`Illuminance`), `Bool`, `Count`,
    /// or `None` when open — what `concept X : …` says.
    pub fn type_name(&self) -> Option<&'static str> {
        match &self.representation {
            RepresentationSpec::Named(NamedRepresentation::Boolean) => Some("Bool"),
            RepresentationSpec::Named(NamedRepresentation::Count) => Some("Count"),
            RepresentationSpec::Named(NamedRepresentation::Open) => None,
            RepresentationSpec::Quantity { quantity: q } => {
                quantity::lookup(q).map(|x| x.type_name)
            }
        }
    }

    /// The unit to show: the template's, else the quantity's canonical one.
    pub fn unit_symbol(&self) -> &str {
        if !self.unit.is_empty() {
            return &self.unit;
        }
        match &self.representation {
            RepresentationSpec::Quantity { quantity: q } => {
                quantity::lookup(q).map(|x| x.unit).unwrap_or("")
            }
            _ => "",
        }
    }

    /// Whether `query` matches the display name, default name, keywords or
    /// category (case-insensitive substring).  Authoring convenience only.
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        let hay = |s: &str| s.to_lowercase().contains(&q);
        hay(&self.display_name)
            || hay(&self.default_name)
            || hay(&self.category)
            || hay(self.unit_symbol())
            || self.keywords.iter().any(|k| hay(k))
    }
}

/// What an item makes: a concept, or a source (a concept and its
/// unresolved `() -> A` relationship).  The Library UI's sections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemCategory {
    Concept,
    Source,
}

impl ItemCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            ItemCategory::Concept => "concept",
            ItemCategory::Source => "source",
        }
    }
}

/// A concept an item creates, with its defaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptSpec {
    /// Fragment-local key other objects refer to (`value`).
    pub key: String,
    pub default_name: String,
    #[serde(default)]
    pub description: String,
    pub representation: RepresentationSpec,
    #[serde(default)]
    pub unit: String,
    #[serde(default = "either")]
    pub role_hint: RoleHint,
}

/// A relationship an item creates: its inputs and output name concepts of
/// the same fragment by key; no definition — a source stays unresolved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingSpec {
    pub key: String,
    pub default_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    pub output: String,
}

/// One object of a fragment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum FragmentObject {
    Concept(ConceptSpec),
    Mapping(MappingSpec),
}

impl FragmentObject {
    pub fn key(&self) -> &str {
        match self {
            FragmentObject::Concept(c) => &c.key,
            FragmentObject::Mapping(m) => &m.key,
        }
    }
    pub fn default_name(&self) -> &str {
        match self {
            FragmentObject::Concept(c) => &c.default_name,
            FragmentObject::Mapping(m) => &m.default_name,
        }
    }
}

/// The ordinary BDL an item expands to, in creation order.  Every object
/// gets a fresh identity on instantiation; a mapping's signature refers to
/// concepts of the same fragment by key.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fragment {
    pub objects: Vec<FragmentObject>,
}

/// One reusable authoring fragment of a library.  `display_name` and
/// `description` are the canonical English; Studio localizes by `id`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryItem {
    /// Stable library identity (`std.source.temperature_sensor`).  Never a
    /// project identity.
    pub id: String,
    pub category: ItemCategory,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    /// Grouping inside the category (`environment`, `motion`, …).
    pub group: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub icon: String,
    pub fragment: Fragment,
}

impl LibraryItem {
    /// The concept template view of a Concept item; `None` for the rest.
    pub fn as_concept_template(&self) -> Option<ConceptTemplate> {
        if self.category != ItemCategory::Concept {
            return None;
        }
        let [FragmentObject::Concept(c)] = self.fragment.objects.as_slice() else {
            return None;
        };
        Some(ConceptTemplate {
            id: self.id.clone(),
            display_name: self.display_name.clone(),
            default_name: c.default_name.clone(),
            description: self.description.clone(),
            category: self.group.clone(),
            role_hint: c.role_hint,
            representation: c.representation.clone(),
            unit: c.unit.clone(),
            keywords: self.keywords.clone(),
            icon: self.icon.clone(),
        })
    }

    /// The concepts of the fragment, by key.
    fn concept(&self, key: &str) -> Option<&ConceptSpec> {
        self.fragment.objects.iter().find_map(|o| match o {
            FragmentObject::Concept(c) if c.key == key => Some(c),
            _ => None,
        })
    }

    /// What the item creates, as the designer reads it (`concept RoomTemp :
    /// Temperature`, `mapping TempSensor : () -> RoomTemp`), with the
    /// default names.
    pub fn creates(&self) -> Vec<CreatedObject> {
        self.fragment
            .objects
            .iter()
            .map(|o| match o {
                FragmentObject::Concept(c) => CreatedObject {
                    kind: "concept".into(),
                    key: c.key.clone(),
                    name: c.default_name.clone(),
                    type_name: concept_type_name(&c.representation).unwrap_or("").into(),
                    signature: String::new(),
                    description: c.description.clone(),
                    representation: representation_of(&c.representation),
                    unit: c.unit.clone(),
                },
                FragmentObject::Mapping(m) => {
                    let name_of = |k: &str| {
                        self.concept(k)
                            .map(|c| c.default_name.clone())
                            .unwrap_or_else(|| k.to_string())
                    };
                    let domain = match m.inputs.as_slice() {
                        [] => "()".to_string(),
                        [a] => name_of(a),
                        many => format!(
                            "({})",
                            many.iter()
                                .map(|k| name_of(k))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    };
                    CreatedObject {
                        kind: "mapping".into(),
                        key: m.key.clone(),
                        name: m.default_name.clone(),
                        type_name: String::new(),
                        signature: format!("{domain} -> {}", name_of(&m.output)),
                        description: m.description.clone(),
                        representation: None,
                        unit: String::new(),
                    }
                }
            })
            .collect()
    }

    /// Whether `query` matches the display name, a default name, keywords,
    /// group or unit (case-insensitive substring).  Authoring convenience
    /// only; localized matching is Studio's, over its catalogs.
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        let hay = |s: &str| s.to_lowercase().contains(&q);
        hay(&self.display_name)
            || hay(&self.group)
            || hay(self.category.as_str())
            || self.keywords.iter().any(|k| hay(k))
            || self.fragment.objects.iter().any(|o| {
                hay(o.default_name())
                    || matches!(o, FragmentObject::Concept(c) if hay(&unit_symbol_of(&c.representation, &c.unit)))
            })
    }
}

/// One object an item creates, for previews: what it is, its default
/// name, and its type as written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedObject {
    pub kind: String,
    pub key: String,
    pub name: String,
    pub type_name: String,
    pub signature: String,
    pub description: String,
    pub representation: Option<Representation>,
    pub unit: String,
}

fn representation_of(spec: &RepresentationSpec) -> Option<Representation> {
    match spec {
        RepresentationSpec::Named(NamedRepresentation::Boolean) => Some(Representation::Boolean),
        RepresentationSpec::Named(NamedRepresentation::Count) => Some(Representation::Count),
        RepresentationSpec::Named(NamedRepresentation::Open) => None,
        RepresentationSpec::Quantity { quantity: q } => Some(Representation::Quantity {
            dim: quantity::lookup(q)?.dim,
        }),
    }
}

fn concept_type_name(spec: &RepresentationSpec) -> Option<&'static str> {
    match spec {
        RepresentationSpec::Named(NamedRepresentation::Boolean) => Some("Bool"),
        RepresentationSpec::Named(NamedRepresentation::Count) => Some("Count"),
        RepresentationSpec::Named(NamedRepresentation::Open) => None,
        RepresentationSpec::Quantity { quantity: q } => quantity::lookup(q).map(|x| x.type_name),
    }
}

fn unit_symbol_of(spec: &RepresentationSpec, unit: &str) -> String {
    if !unit.is_empty() {
        return unit.to_string();
    }
    match spec {
        RepresentationSpec::Quantity { quantity: q } => quantity::lookup(q)
            .map(|x| x.unit)
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryInfo {
    /// Library identity (`std`); template ids are prefixed by it.
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    /// The library's own version (`0.1`); informational — an instantiated
    /// concept never depends on it.
    pub version: String,
}

/// A loaded, validated library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Library {
    pub info: LibraryInfo,
    items: Vec<LibraryItem>,
    /// The Concept items as templates, in library order (a view kept for
    /// the concept-template API).
    templates: Vec<ConceptTemplate>,
}

/// `[[source]]` as authored: a value (the concept) and the relationship
/// that supplies it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEntry {
    id: String,
    display_name: String,
    #[serde(default)]
    description: String,
    category: String,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    icon: String,
    value: SourceValue,
    relationship: SourceRelationship,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceValue {
    default_name: String,
    #[serde(default)]
    description: String,
    representation: RepresentationSpec,
    #[serde(default)]
    unit: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRelationship {
    default_name: String,
    #[serde(default)]
    description: String,
}

#[derive(Clone, Debug, Deserialize)]
struct LibraryFile {
    library: LibraryInfo,
    #[serde(default, rename = "template")]
    templates: Vec<ConceptTemplate>,
    #[serde(default, rename = "source")]
    sources: Vec<SourceEntry>,
}

fn item_of_template(t: &ConceptTemplate) -> LibraryItem {
    LibraryItem {
        id: t.id.clone(),
        category: ItemCategory::Concept,
        display_name: t.display_name.clone(),
        description: t.description.clone(),
        group: t.category.clone(),
        keywords: t.keywords.clone(),
        icon: t.icon.clone(),
        fragment: Fragment {
            objects: vec![FragmentObject::Concept(ConceptSpec {
                key: "concept".into(),
                default_name: t.default_name.clone(),
                description: t.description.clone(),
                representation: t.representation.clone(),
                unit: t.unit.clone(),
                role_hint: t.role_hint,
            })],
        },
    }
}

fn item_of_source(e: &SourceEntry) -> LibraryItem {
    LibraryItem {
        id: e.id.clone(),
        category: ItemCategory::Source,
        display_name: e.display_name.clone(),
        description: e.description.clone(),
        group: e.category.clone(),
        keywords: e.keywords.clone(),
        icon: e.icon.clone(),
        fragment: Fragment {
            objects: vec![
                FragmentObject::Concept(ConceptSpec {
                    key: "value".into(),
                    default_name: e.value.default_name.clone(),
                    description: e.value.description.clone(),
                    representation: e.value.representation.clone(),
                    unit: e.value.unit.clone(),
                    role_hint: RoleHint::Input,
                }),
                FragmentObject::Mapping(MappingSpec {
                    key: "source".into(),
                    default_name: e.relationship.default_name.clone(),
                    description: e.relationship.description.clone(),
                    inputs: Vec::new(),
                    output: "value".into(),
                }),
            ],
        },
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LibraryError {
    #[error("library file: {0}")]
    Parse(String),
    #[error("unsupported library schema {found} (this build reads {SCHEMA_VERSION})")]
    Schema { found: u32 },
    #[error("template `{id}`: {problem}")]
    Template { id: String, problem: String },
    #[error("template id `{id}` appears twice")]
    DuplicateId { id: String },
}

impl Library {
    /// Parse and validate a library file.  Every concept must reference a
    /// known quantity and a known unit of that quantity's dimension and
    /// have an identifier-shaped default name; every mapping must name
    /// concepts of its own fragment; ids are unique under the library's
    /// prefix.
    pub fn from_toml(text: &str) -> Result<Library, LibraryError> {
        let file: LibraryFile =
            toml::from_str(text).map_err(|e| LibraryError::Parse(e.to_string()))?;
        if !(OLDEST_SCHEMA_VERSION..=SCHEMA_VERSION).contains(&file.library.schema_version) {
            return Err(LibraryError::Schema {
                found: file.library.schema_version,
            });
        }
        let mut items: Vec<LibraryItem> = file.templates.iter().map(item_of_template).collect();
        items.extend(file.sources.iter().map(item_of_source));
        let mut seen = BTreeSet::new();
        for item in &items {
            let problem = |p: &str| LibraryError::Template {
                id: item.id.clone(),
                problem: p.to_owned(),
            };
            if !seen.insert(item.id.clone()) {
                return Err(LibraryError::DuplicateId {
                    id: item.id.clone(),
                });
            }
            if !item.id.starts_with(&format!("{}.", file.library.id)) {
                return Err(problem(&format!(
                    "id must start with `{}.`",
                    file.library.id
                )));
            }
            if item.display_name.trim().is_empty() || item.group.trim().is_empty() {
                return Err(problem("display_name and category are required"));
            }
            if item.fragment.objects.is_empty() {
                return Err(problem("an item creates at least one object"));
            }
            let mut keys = BTreeSet::new();
            for o in &item.fragment.objects {
                if !keys.insert(o.key().to_string()) {
                    return Err(problem(&format!("object key `{}` appears twice", o.key())));
                }
                if !is_identifier(o.default_name()) {
                    return Err(problem(&format!(
                        "default name `{}` must be an identifier",
                        o.default_name()
                    )));
                }
                match o {
                    FragmentObject::Concept(c) => {
                        let dim = match &c.representation {
                            RepresentationSpec::Quantity { quantity: q } => {
                                match quantity::lookup(q) {
                                    Some(def) => Some(def.dim),
                                    None => {
                                        return Err(problem(&format!("unknown quantity `{q}`")))
                                    }
                                }
                            }
                            RepresentationSpec::Named(_) => None,
                        };
                        if !c.unit.is_empty() {
                            let Some(dim) = dim else {
                                return Err(problem("a unit needs a quantity representation"));
                            };
                            match bdl_elab::units::lookup(&c.unit) {
                                Some(u) if u.dim == dim => {}
                                Some(_) => {
                                    return Err(problem(&format!(
                                        "unit `{}` does not measure the concept's quantity",
                                        c.unit
                                    )))
                                }
                                None => return Err(problem(&format!("unknown unit `{}`", c.unit))),
                            }
                        }
                    }
                    FragmentObject::Mapping(m) => {
                        for k in m.inputs.iter().chain(std::iter::once(&m.output)) {
                            if item.concept(k).is_none() {
                                return Err(problem(&format!(
                                    "mapping `{}` names `{k}`, which is no concept of the item",
                                    m.default_name
                                )));
                            }
                        }
                    }
                }
            }
        }
        let templates = items
            .iter()
            .filter_map(|i| i.as_concept_template())
            .collect();
        Ok(Library {
            info: file.library,
            items,
            templates,
        })
    }

    /// The embedded Standard Concept Library.
    pub fn standard() -> Library {
        Library::from_toml(STANDARD_LIBRARY_TOML).unwrap_or_else(|e| {
            unreachable!("the embedded standard library is validated by tests: {e}")
        })
    }

    /// Every item, in library order.
    pub fn items(&self) -> &[LibraryItem] {
        &self.items
    }

    pub fn item(&self, id: &str) -> Option<&LibraryItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// The Concept items as templates (the concept-template view).
    pub fn templates(&self) -> &[ConceptTemplate] {
        &self.templates
    }

    pub fn get(&self, id: &str) -> Option<&ConceptTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Concept groups in first-seen order.
    pub fn categories(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for t in &self.templates {
            if !out.contains(&t.category.as_str()) {
                out.push(&t.category);
            }
        }
        out
    }

    pub fn search<'a>(&'a self, query: &'a str) -> impl Iterator<Item = &'a ConceptTemplate> + 'a {
        self.templates.iter().filter(move |t| t.matches(query))
    }

    pub fn search_items<'a>(
        &'a self,
        query: &'a str,
    ) -> impl Iterator<Item = &'a LibraryItem> + 'a {
        self.items.iter().filter(move |i| i.matches(query))
    }
}

/// Several libraries, searched together; ids stay unique because each
/// library prefixes its own.  Standard first, then team, project or
/// package libraries as they arrive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LibrarySet {
    libraries: Vec<Library>,
}

impl LibrarySet {
    /// Just the Standard Concept Library.
    pub fn standard() -> LibrarySet {
        LibrarySet {
            libraries: vec![Library::standard()],
        }
    }

    pub fn with(mut self, library: Library) -> LibrarySet {
        self.libraries.retain(|l| l.info.id != library.info.id);
        self.libraries.push(library);
        self
    }

    pub fn libraries(&self) -> &[Library] {
        &self.libraries
    }

    pub fn get(&self, id: &str) -> Option<&ConceptTemplate> {
        self.libraries.iter().find_map(|l| l.get(id))
    }

    pub fn item(&self, id: &str) -> Option<&LibraryItem> {
        self.libraries.iter().find_map(|l| l.item(id))
    }

    pub fn items(&self) -> impl Iterator<Item = &LibraryItem> {
        self.libraries.iter().flat_map(|l| l.items.iter())
    }

    pub fn templates(&self) -> impl Iterator<Item = &ConceptTemplate> {
        self.libraries.iter().flat_map(|l| l.templates.iter())
    }

    pub fn search<'a>(&'a self, query: &'a str) -> impl Iterator<Item = &'a ConceptTemplate> + 'a {
        self.templates().filter(move |t| t.matches(query))
    }
}

/// The one instantiation operation of a concept template: the
/// `CreateConcept` edit it stands for, with the template's defaults and a
/// name that is free in `design` (`Temperature`, then `Temperature2`, …).
/// `name` overrides the default when given.  Applying the edit allocates a
/// fresh `SemanticId`; nothing about the template is recorded.
pub fn instantiate(design: &Design, template: &ConceptTemplate, name: Option<&str>) -> EditOp {
    let wanted = name
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or(&template.default_name);
    EditOp::CreateConcept {
        name: free_name(design, wanted),
        description: template.description.clone(),
        representation: template.representation(),
    }
}

/// One step of an item's instantiation, in order.  A concept is a plain
/// edit; a mapping names the concepts it reads and produces by fragment
/// key, resolved to the identities the earlier steps allocated by whoever
/// applies the plan (the daemon, in one transaction).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannedStep {
    Concept {
        key: String,
        op: EditOp,
    },
    Mapping {
        key: String,
        name: String,
        description: String,
        inputs: Vec<String>,
        output: String,
    },
}

impl PlannedStep {
    pub fn key(&self) -> &str {
        match self {
            PlannedStep::Concept { key, .. } | PlannedStep::Mapping { key, .. } => key,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            PlannedStep::Concept {
                op: EditOp::CreateConcept { name, .. },
                ..
            } => name,
            PlannedStep::Mapping { name, .. } => name,
            PlannedStep::Concept { .. } => "",
        }
    }
}

/// The steps that instantiate `item` in `design`: every object with a
/// name free in the design and among the steps before it (`RoomTemp`,
/// else `RoomTemp2`, …; `names` overrides a default by fragment key).
/// Nothing is applied here and nothing about the item is recorded: after
/// the steps run, the project holds ordinary objects.
pub fn plan(
    design: &Design,
    item: &LibraryItem,
    names: &std::collections::BTreeMap<String, String>,
) -> Vec<PlannedStep> {
    let mut taken: Vec<String> = Vec::new();
    let mut steps = Vec::new();
    for o in &item.fragment.objects {
        let wanted = names
            .get(o.key())
            .map(|n| n.trim())
            .filter(|n| !n.is_empty())
            .unwrap_or(o.default_name());
        let name = free_name_among(design, wanted, &taken);
        taken.push(name.clone());
        steps.push(match o {
            FragmentObject::Concept(c) => PlannedStep::Concept {
                key: c.key.clone(),
                op: EditOp::CreateConcept {
                    name,
                    description: c.description.clone(),
                    representation: representation_of(&c.representation),
                },
            },
            FragmentObject::Mapping(m) => PlannedStep::Mapping {
                key: m.key.clone(),
                name,
                description: m.description.clone(),
                inputs: m.inputs.clone(),
                output: m.output.clone(),
            },
        });
    }
    steps
}

/// `wanted` if no concept or relationship has it, else the first
/// `wanted2`, `wanted3`, … that is free.  One namespace for both kinds, so
/// a formula never meets a concept and a relationship of one name.
pub fn free_name(design: &Design, wanted: &str) -> String {
    free_name_among(design, wanted, &[])
}

fn free_name_among(design: &Design, wanted: &str, also_taken: &[String]) -> String {
    let taken = |n: &str| {
        design.concepts.values().any(|c| c.name == n)
            || design.mappings.values().any(|m| m.name == n)
            || also_taken.iter().any(|t| t == n)
    };
    if !taken(wanted) {
        return wanted.to_owned();
    }
    (2u32..)
        .map(|i| format!("{wanted}{i}"))
        .find(|n| !taken(n))
        .unwrap_or_else(|| wanted.to_owned())
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl fmt::Display for ConceptTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.type_name() {
            Some(t) => write!(f, "concept {} : {}", self.default_name, t),
            None => write!(f, "concept {}", self.default_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::edit::apply_edit;
    use bdl_model::surface::ProjectSnapshot;
    use bdl_model::Dim;

    #[test]
    fn the_standard_library_loads_and_is_well_formed() {
        let lib = Library::standard();
        assert_eq!(lib.info.id, "std");
        assert_eq!(lib.info.schema_version, SCHEMA_VERSION);
        assert!(
            (30..=50).contains(&lib.templates().len()),
            "deliberately small: {} templates",
            lib.templates().len()
        );
        assert_eq!(
            lib.categories(),
            vec![
                "environment",
                "human",
                "motion",
                "mechanical",
                "electrical",
                "visual",
                "actuation",
                "audio"
            ]
        );
        // the Sources category: eight items, each a concept and an
        // unresolved relationship without inputs
        let sources: Vec<&LibraryItem> = lib
            .items()
            .iter()
            .filter(|i| i.category == ItemCategory::Source)
            .collect();
        assert_eq!(
            sources.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
            vec![
                "std.source.temperature",
                "std.source.tilt",
                "std.source.distance",
                "std.source.ambient_light",
                "std.source.button",
                "std.source.encoder",
                "std.source.analog",
                "std.source.external",
            ]
        );
        for i in &sources {
            assert_eq!(i.fragment.objects.len(), 2, "{}", i.id);
            let created = i.creates();
            assert_eq!(created[0].kind, "concept");
            assert_eq!(created[1].kind, "mapping");
            assert!(
                created[1].signature.starts_with("() -> "),
                "{}: {}",
                i.id,
                created[1].signature
            );
            assert!(!i.description.is_empty());
        }
        assert_eq!(lib.items().len(), lib.templates().len() + 8);
        for t in lib.templates() {
            assert!(
                t.representation().is_some()
                    || matches!(
                        t.representation,
                        RepresentationSpec::Named(NamedRepresentation::Open)
                    )
            );
            assert!(!t.description.is_empty(), "{} has no description", t.id);
        }
        let light = lib
            .get("std.environment.ambient_light")
            .expect("ambient light");
        assert_eq!(light.type_name(), Some("Illuminance"));
        assert_eq!(light.unit_symbol(), "lx");
        assert_eq!(light.to_string(), "concept AmbientLight : Illuminance");
        // Brightness is a level, not a photometric quantity.
        let b = lib.get("std.output.brightness").expect("brightness");
        assert_eq!(
            b.representation(),
            Some(Representation::Quantity { dim: Dim::ZERO })
        );
        assert_eq!(b.type_name(), Some("Scalar"));
    }

    /// A Source item plans a concept and a relationship `() -> Value`;
    /// names are free in the design and among the steps; the mapping names
    /// its output by fragment key.  `Analog Input` and `External Value`
    /// leave the value form open.
    #[test]
    fn a_source_item_plans_a_concept_and_an_unresolved_unit_domain_relationship() {
        let lib = Library::standard();
        let item = lib.item("std.source.temperature").expect("item");
        let s = ProjectSnapshot::new(Design::empty("lamp"));
        let steps = plan(&s.design, item, &Default::default());
        assert_eq!(steps.len(), 2);
        assert!(
            matches!(&steps[0], PlannedStep::Concept { key, op: EditOp::CreateConcept { name, representation: Some(Representation::Quantity { dim }), .. } }
            if key == "value" && name == "RoomTemp" && *dim == Dim::TEMPERATURE)
        );
        assert!(
            matches!(&steps[1], PlannedStep::Mapping { key, name, inputs, output, .. }
            if key == "source" && name == "TempSensor" && inputs.is_empty() && output == "value")
        );
        // a taken name, on either kind, moves to the next free one
        let a = apply_edit(
            &s,
            &EditOp::CreateConcept {
                name: "RoomTemp".into(),
                description: String::new(),
                representation: None,
            },
        )
        .expect("c");
        let b = apply_edit(
            &a.snapshot,
            &EditOp::CreateMapping {
                name: "TempSensor".into(),
                description: String::new(),
                signature: bdl_model::surface::Signature {
                    inputs: vec![],
                    output: a.outcome.created_concept.expect("id"),
                },
            },
        )
        .expect("m");
        let steps = plan(&b.snapshot.design, item, &Default::default());
        assert_eq!(steps[0].name(), "RoomTemp2");
        assert_eq!(steps[1].name(), "TempSensor2");
        // a chosen name wins; a name a concept holds is not given to a mapping
        let mut names = std::collections::BTreeMap::new();
        names.insert("value".to_string(), "OvenTemp".to_string());
        names.insert("source".to_string(), "RoomTemp".to_string());
        let steps = plan(&b.snapshot.design, item, &names);
        assert_eq!(steps[0].name(), "OvenTemp");
        assert_eq!(steps[1].name(), "RoomTemp2");
        for id in ["std.source.analog", "std.source.external"] {
            let i = lib.item(id).expect(id);
            let steps = plan(&s.design, i, &Default::default());
            assert!(matches!(
                &steps[0],
                PlannedStep::Concept {
                    op: EditOp::CreateConcept {
                        representation: None,
                        ..
                    },
                    ..
                }
            ));
        }
        // the concept view of the library is unchanged by the sources
        assert!(lib.get("std.source.temperature").is_none());
        assert!(lib.get("std.environment.temperature").is_some());
        assert_eq!(
            lib.item("std.environment.temperature")
                .and_then(|i| i.as_concept_template())
                .map(|t| t.default_name),
            Some("Temperature".into())
        );
    }

    #[test]
    fn search_matches_names_keywords_units_and_categories() {
        let lib = Library::standard();
        let ids = |q: &str| -> Vec<String> { lib.search(q).map(|t| t.id.clone()).collect() };
        assert_eq!(ids("lux"), vec!["std.environment.ambient_light"]);
        assert!(ids("tilt").contains(&"std.motion.tilt".to_owned()));
        let motor = ids("motor");
        assert!(motor.contains(&"std.actuator.motor_speed".to_owned()));
        assert!(motor.contains(&"std.actuator.motor_angle".to_owned()));
        assert!(ids("environment").len() >= 5);
        assert_eq!(ids("").len(), lib.templates().len());
        assert!(ids("zzzz").is_empty());
    }

    #[test]
    fn two_instantiations_are_two_concepts_with_independent_defaults() {
        let lib = Library::standard();
        let t = lib.get("std.environment.temperature").expect("temperature");
        let s = ProjectSnapshot::new(Design::empty("lamp"));
        let a = apply_edit(&s, &instantiate(&s.design, t, None)).expect("first");
        let id_a = a.outcome.created_concept.expect("id");
        let b = apply_edit(&a.snapshot, &instantiate(&a.snapshot.design, t, None)).expect("second");
        let id_b = b.outcome.created_concept.expect("id");
        assert_ne!(id_a, id_b);
        let d = &b.snapshot.design;
        assert_eq!(d.concepts[&id_a].name, "Temperature");
        assert_eq!(
            d.concepts[&id_b].name, "Temperature2",
            "a free name, never a refusal"
        );
        // Rename both; the defaults stay with each independently.
        let mut s = b.snapshot;
        for (id, name) in [(id_a, "RoomTemperature"), (id_b, "MotorTemperature")] {
            s = apply_edit(
                &s,
                &EditOp::RenameConcept {
                    id,
                    name: name.into(),
                },
            )
            .expect("rename")
            .snapshot;
        }
        let kelvin = Some(Representation::Quantity {
            dim: Dim::TEMPERATURE,
        });
        assert_eq!(s.design.concepts[&id_a].name, "RoomTemperature");
        assert_eq!(s.design.concepts[&id_b].name, "MotorTemperature");
        assert_eq!(s.design.concepts[&id_a].representation, kelvin);
        assert_eq!(s.design.concepts[&id_b].representation, kelvin);
        // Rebinding one does not touch the other.
        let s = apply_edit(
            &s,
            &EditOp::SetConceptRepresentation {
                id: id_a,
                representation: Some(Representation::Quantity { dim: Dim::ZERO }),
            },
        )
        .expect("rebind")
        .snapshot;
        assert_eq!(s.design.concepts[&id_b].representation, kelvin);
        // An explicit name wins over the default.
        let op = instantiate(&s.design, t, Some("OvenTemperature"));
        assert!(matches!(&op, EditOp::CreateConcept { name, .. } if name == "OvenTemperature"));
    }

    #[test]
    fn a_project_persists_without_the_library_and_a_template_change_does_not_reach_it() {
        let lib = Library::standard();
        let t = lib
            .get("std.environment.ambient_light")
            .expect("ambient light");
        let s = ProjectSnapshot::new(Design::empty("lamp"));
        let a = apply_edit(&s, &instantiate(&s.design, t, None)).expect("create");
        let id = a.outcome.created_concept.expect("id");
        let dir = tempfile::tempdir().expect("tempdir");
        let created = bdl_model::persist::init_project(dir.path(), "lamp", "test").expect("init");
        bdl_model::persist::save_project(dir.path(), &a.snapshot, &created.layout, "test")
            .expect("save");

        // The library "moves on": AmbientLight becomes a Scalar level.
        let changed = STANDARD_LIBRARY_TOML.replace(
            "representation = { quantity = \"illuminance\" }\nunit = \"lx\"",
            "representation = { quantity = \"scalar\" }",
        );
        let v2 = Library::from_toml(&changed).expect("v2 parses");
        assert_eq!(
            v2.get("std.environment.ambient_light")
                .and_then(|t| t.type_name()),
            Some("Scalar")
        );

        // The project loads with no library at all and still says lux.
        let loaded = bdl_model::persist::load_project(dir.path()).expect("load");
        let c = &loaded.snapshot.design.concepts[&id];
        assert_eq!(c.name, "AmbientLight");
        assert_eq!(
            c.representation,
            Some(Representation::Quantity {
                dim: quantity::lookup("illuminance").expect("lux").dim
            })
        );
        assert_eq!(c.description, t.description);
    }

    #[test]
    fn invalid_libraries_are_refused_with_the_reason() {
        let bad_unit = STANDARD_LIBRARY_TOML.replacen("unit = \"K\"", "unit = \"m\"", 1);
        assert!(matches!(
            Library::from_toml(&bad_unit),
            Err(LibraryError::Template { id, .. }) if id == "std.environment.temperature"
        ));
        let bad_quantity = STANDARD_LIBRARY_TOML.replacen(
            "quantity = \"temperature\"",
            "quantity = \"warmth\"",
            1,
        );
        assert!(Library::from_toml(&bad_quantity).is_err());
        let bad_schema =
            STANDARD_LIBRARY_TOML.replacen("schema_version = 2", "schema_version = 3", 1);
        assert_eq!(
            Library::from_toml(&bad_schema),
            Err(LibraryError::Schema { found: 3 })
        );
        // a source whose relationship names no concept of its own fragment
        let bad_source = STANDARD_LIBRARY_TOML.replacen(
            "default_name = \"TempSensor\"",
            "default_name = \"Temp Sensor\"",
            1,
        );
        assert!(matches!(
            Library::from_toml(&bad_source),
            Err(LibraryError::Template { id, .. }) if id == "std.source.temperature"
        ));
        assert!(Library::from_toml("not toml at all [").is_err());
        // the first shipping form of a Source (a template with a `source`
        // field, `i18n` text) is refused, never loaded as a plain concept
        let old_source = "[library]\nid = \"old\"\nname = \"Old\"\nschema_version = 1\nversion = \"0\"\n[[template]]\nid = \"old.t\"\ndisplay_name = \"T\"\ndefault_name = \"T\"\ncategory = \"c\"\nrepresentation = \"open\"\nsource = { default_name = \"S\" }\n";
        assert!(Library::from_toml(old_source).is_err());
        let old_i18n = old_source.replace(
            "source = { default_name = \"S\" }",
            "i18n = { ja = { display_name = \"T\" } }",
        );
        assert!(Library::from_toml(&old_i18n).is_err());
        let set = LibrarySet::standard().with(
            Library::from_toml(
                "[library]\nid = \"team\"\nname = \"Team\"\nschema_version = 1\nversion = \"0\"\n[[template]]\nid = \"team.x\"\ndisplay_name = \"X\"\ndefault_name = \"X\"\ncategory = \"team\"\nrepresentation = \"open\"\n",
            )
            .expect("team library"),
        );
        assert_eq!(set.libraries().len(), 2);
        assert!(set.get("team.x").is_some());
        assert!(set.get("std.motion.tilt").is_some());
        assert_eq!(set.get("team.x").and_then(|t| t.representation()), None);
    }
}
