//! Concept libraries: shared authoring vocabulary.
//!
//! A library is a set of [`ConceptTemplate`]s — good defaults for concepts
//! designers author again and again (`Temperature`, `AmbientLight`,
//! `MotorSpeed`).  A template is *not* a concept: [`instantiate`] turns one
//! into an ordinary `CreateConcept` edit, the compiler allocates a fresh
//! `SemanticId`, and from then on the concept is an independent copy of
//! the defaults — renamed, rebound, deleted like any other.  Two
//! instantiations are two concepts; the project never records where a
//! concept came from, and a later library version cannot change it.
//!
//! Data-driven: the Standard Concept Library is `library/std/concepts.toml`,
//! embedded at build time and loadable from disk like any future team,
//! project or package library ([`Library::from_toml`]).  Representations
//! reference the shared quantity vocabulary (`bdl_model::quantity`) and
//! units the shared unit table (`bdl_elab::units`); the library has no
//! dimension table of its own.
//!
//! What this crate is not: a device catalogue (a `BH1750` *provides* an
//! `AmbientLight : Illuminance`; it is not one), a kernel type, or a
//! resolution mechanism — search is authoring convenience.

#![forbid(unsafe_code)]

use bdl_model::surface::{Design, Representation};
use bdl_model::{quantity, EditOp};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The schema of the TOML files this crate reads.
pub const SCHEMA_VERSION: u32 = 1;

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
/// a *default* the designer may change after instantiation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// A *Source* template: instantiating it also creates one relationship
    /// `<source.default_name> : () -> <the concept>` with no definition —
    /// an ordinary unit-domain declaration the environment provides
    /// (ADR-0032).  Nothing else marks it: the Source role is derived from
    /// that shape, so a third-party library gets exactly the same result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceSpec>,
    /// Display name and description in other locales (`zh-Hans`, `ja`),
    /// for the Library panel; the generated identifiers never change with
    /// the locale.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub i18n: BTreeMap<String, TemplateText>,
}

/// The relationship a Source template creates beside its concept.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpec {
    /// The relationship's name on creation (`TempSensor`), made unique in
    /// the design if taken.
    pub default_name: String,
}

/// One locale's rendering of a template's presentation text.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateText {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
}

/// What instantiating a template does: the `CreateConcept` edit, and for a
/// Source template the name of the `() -> concept` relationship to create
/// once the concept's identity is known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instantiation {
    pub concept: EditOp,
    pub source_name: Option<String>,
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
    templates: Vec<ConceptTemplate>,
}

#[derive(Clone, Debug, Deserialize)]
struct LibraryFile {
    library: LibraryInfo,
    #[serde(default, rename = "template")]
    templates: Vec<ConceptTemplate>,
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
    /// Parse and validate a library file.  Every template must reference a
    /// known quantity and a known unit of that quantity's dimension, have a
    /// non-empty identifier-shaped default name, and a unique id under the
    /// library's prefix.
    pub fn from_toml(text: &str) -> Result<Library, LibraryError> {
        let file: LibraryFile =
            toml::from_str(text).map_err(|e| LibraryError::Parse(e.to_string()))?;
        if file.library.schema_version != SCHEMA_VERSION {
            return Err(LibraryError::Schema {
                found: file.library.schema_version,
            });
        }
        let mut seen = BTreeSet::new();
        for t in &file.templates {
            let problem = |p: &str| LibraryError::Template {
                id: t.id.clone(),
                problem: p.to_owned(),
            };
            if !seen.insert(t.id.clone()) {
                return Err(LibraryError::DuplicateId { id: t.id.clone() });
            }
            if !t.id.starts_with(&format!("{}.", file.library.id)) {
                return Err(problem(&format!(
                    "id must start with `{}.`",
                    file.library.id
                )));
            }
            if !is_identifier(&t.default_name) {
                return Err(problem("default_name must be an identifier"));
            }
            if let Some(src) = &t.source {
                if !is_identifier(&src.default_name) {
                    return Err(problem("source.default_name must be an identifier"));
                }
                if src.default_name == t.default_name {
                    return Err(problem(
                        "source.default_name must differ from the concept's default_name",
                    ));
                }
            }
            if t.display_name.trim().is_empty() || t.category.trim().is_empty() {
                return Err(problem("display_name and category are required"));
            }
            let dim = match &t.representation {
                RepresentationSpec::Quantity { quantity: q } => match quantity::lookup(q) {
                    Some(def) => Some(def.dim),
                    None => return Err(problem(&format!("unknown quantity `{q}`"))),
                },
                RepresentationSpec::Named(_) => None,
            };
            if !t.unit.is_empty() {
                let Some(dim) = dim else {
                    return Err(problem("a unit needs a quantity representation"));
                };
                match bdl_elab::units::lookup(&t.unit) {
                    Some(u) if u.dim == dim => {}
                    Some(_) => {
                        return Err(problem(&format!(
                            "unit `{}` does not measure the template's quantity",
                            t.unit
                        )))
                    }
                    None => return Err(problem(&format!("unknown unit `{}`", t.unit))),
                }
            }
        }
        Ok(Library {
            info: file.library,
            templates: file.templates,
        })
    }

    /// The embedded Standard Concept Library.
    pub fn standard() -> Library {
        Library::from_toml(STANDARD_LIBRARY_TOML).unwrap_or_else(|e| {
            unreachable!("the embedded standard library is validated by tests: {e}")
        })
    }

    pub fn templates(&self) -> &[ConceptTemplate] {
        &self.templates
    }

    pub fn get(&self, id: &str) -> Option<&ConceptTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Categories in first-seen order.
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

    pub fn templates(&self) -> impl Iterator<Item = &ConceptTemplate> {
        self.libraries.iter().flat_map(|l| l.templates.iter())
    }

    pub fn search<'a>(&'a self, query: &'a str) -> impl Iterator<Item = &'a ConceptTemplate> + 'a {
        self.templates().filter(move |t| t.matches(query))
    }
}

/// The one instantiation operation: the `CreateConcept` edit a template
/// stands for, with the template's defaults and a name that is free in
/// `design` (`Temperature`, then `Temperature2`, …).  `name` overrides the
/// default when given.  Applying the edit allocates a fresh `SemanticId`;
/// nothing about the template is recorded.
pub fn instantiate(design: &Design, template: &ConceptTemplate, name: Option<&str>) -> EditOp {
    instantiate_with(design, template, name, None).concept
}

/// [`instantiate`] plus, for a Source template, the relationship to create
/// after the concept: `source_name` overrides its default.  The caller
/// creates the concept, reads the `SemanticId` off the outcome, and creates
/// `CreateMapping { name, inputs: [], output }` — one commit, two ordinary
/// edits, no definition: the environment provides the value.
pub fn instantiate_with(
    design: &Design,
    template: &ConceptTemplate,
    name: Option<&str>,
    source_name: Option<&str>,
) -> Instantiation {
    let wanted = name
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or(&template.default_name);
    let source_name = template.source.as_ref().map(|s| {
        let wanted = source_name
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .unwrap_or(&s.default_name);
        free_mapping_name(design, wanted)
    });
    Instantiation {
        concept: EditOp::CreateConcept {
            name: free_name(design, wanted),
            description: template.description.clone(),
            representation: template.representation(),
        },
        source_name,
    }
}

/// `wanted` if no relationship has it, else the first free `wanted2`, ….
pub fn free_mapping_name(design: &Design, wanted: &str) -> String {
    let taken = |n: &str| design.mappings.values().any(|m| m.name == n);
    if !taken(wanted) {
        return wanted.to_owned();
    }
    (2u32..)
        .map(|i| format!("{wanted}{i}"))
        .find(|n| !taken(n))
        .unwrap_or_else(|| wanted.to_owned())
}

/// `wanted` if no concept has it, else the first `wanted2`, `wanted3`, …
/// that is free.
pub fn free_name(design: &Design, wanted: &str) -> String {
    let taken = |n: &str| design.concepts.values().any(|c| c.name == n);
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
            Some(t) => write!(f, "concept {} : {}", self.default_name, t)?,
            None => write!(f, "concept {}", self.default_name)?,
        }
        if let Some(s) = &self.source {
            // the preferred spelling: the empty product written out
            write!(
                f,
                "\nmapping {} : () -> {}",
                s.default_name, self.default_name
            )?;
        }
        Ok(())
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
                "audio",
                "sources"
            ]
        );
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

    #[test]
    fn source_templates_create_a_concept_and_an_explicit_unit_domain_relationship() {
        let lib = Library::standard();
        let sources: Vec<&ConceptTemplate> = lib
            .templates()
            .iter()
            .filter(|t| t.category == "sources")
            .collect();
        assert_eq!(sources.len(), 7, "the seven standard Sources");
        let design = Design::empty("lamp");
        for t in &sources {
            let src = t
                .source
                .as_ref()
                .expect("a Source template names its relationship");
            assert!(is_identifier(&src.default_name), "{}", t.id);
            for locale in ["zh-Hans", "ja"] {
                let text = t
                    .i18n
                    .get(locale)
                    .unwrap_or_else(|| panic!("{} lacks {locale}", t.id));
                assert!(!text.display_name.is_empty() && !text.description.is_empty());
            }
            // the printed form is the preferred spelling, never the shorthand
            let shown = t.to_string();
            assert!(
                shown.contains(&format!(
                    "mapping {} : () -> {}",
                    src.default_name, t.default_name
                )),
                "{shown}"
            );
            assert!(!shown.contains(&format!(": {}\n", t.default_name)));
            // instantiation: an ordinary CreateConcept, then the relationship's name
            let i = instantiate_with(&design, t, None, None);
            assert!(matches!(i.concept, EditOp::CreateConcept { .. }));
            assert_eq!(i.source_name.as_deref(), Some(src.default_name.as_str()));
            // both names are the designer's to choose
            let i = instantiate_with(&design, t, Some("Room"), Some("Thermo"));
            assert!(matches!(&i.concept, EditOp::CreateConcept { name, .. } if name == "Room"));
            assert_eq!(i.source_name.as_deref(), Some("Thermo"));
        }
        // a template without `source` creates only the concept
        let plain = lib.get("std.environment.temperature").unwrap();
        assert!(instantiate_with(&design, plain, None, None)
            .source_name
            .is_none());
    }

    #[test]
    fn search_matches_names_keywords_units_and_categories() {
        let lib = Library::standard();
        let ids = |q: &str| -> Vec<String> { lib.search(q).map(|t| t.id.clone()).collect() };
        assert_eq!(
            ids("lux"),
            vec!["std.environment.ambient_light", "std.source.ambient_light"]
        );
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
            STANDARD_LIBRARY_TOML.replacen("schema_version = 1", "schema_version = 2", 1);
        assert_eq!(
            Library::from_toml(&bad_schema),
            Err(LibraryError::Schema { found: 2 })
        );
        assert!(Library::from_toml("not toml at all [").is_err());
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
