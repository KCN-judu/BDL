//! The opened-project session: one canonical revision stream.
//!
//! `Session` is a plain state machine with no I/O of its own except the
//! explicit `open`/`init`/`save` operations that touch the project directory.
//! Every method that changes the project takes `&mut self` on the single
//! coordinator (see `server.rs`); analyses will receive `ProjectSnapshot`
//! values by clone and never a reference into the session.
//!
//! Revisions are strictly monotone for the life of the session — undo does
//! not rewind the revision, it produces a new revision whose design equals
//! an earlier one — so a stale analysis result can always be recognised by
//! a simple `<` comparison.
//!
//! The session also owns the project's [`IdeHost`]: the committed snapshot
//! mirrored as IDE ground state, plus the overlays Studio's definition
//! drafts live in.  Every commit re-seats the host; a draft is analysed by
//! taking an immutable snapshot of committed + overlays and asking
//! `bdl-ide`, never by a side path through the compiler.

use bdl_ide::{
    completion, draft_verdict, entity_at_formula, hover, AnalysisSnapshot, CompletionContext,
    DraftVerdict, EntityRef, IdeHost, OverlayKey, QueryError, SemanticCompletion, SemanticHover,
    TextRange,
};
use bdl_ide_db::CancelScope;
use bdl_model::edit::{EditError, EditKind, EditOp, EditOutcome};
use bdl_model::layout::Layout;
use bdl_model::persist::{self, PersistError};
use bdl_model::surface::{Design, ProjectSnapshot};
use bdl_model::{DeclId, Revision};
use bdl_reactive::Simulation;
use bdl_system::{
    analyze_system, apply_group_edit, apply_system_edit, flatten, preview_extraction,
    AppliedSystem, BehaviorGroup, BehaviorGroupId, BehaviorSystem, ComponentId, ExtractError,
    ExtractionChoices, ExtractionPreview, FlattenedSystem, GroupEditError, GroupEditOp, GroupScope,
    SystemAnalysis, SystemEditError, SystemEditOp, SystemEditOutcome, SystemSnapshot,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("no project is open")]
    NoProject,
    #[error("a project is already open ({0}); close it first")]
    AlreadyOpen(PathBuf),
    #[error("edit targets revision {expected} but the project is at {actual}")]
    StaleRevision {
        expected: Revision,
        actual: Revision,
    },
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
    #[error(transparent)]
    Edit(#[from] EditError),
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Ide(#[from] QueryError),
    #[error("this project is a flat design, not a behaviour system")]
    NotASystem,
    /// A library item's plan does not hold together (a mapping names a
    /// concept no step created).
    #[error("library item: {0}")]
    LibraryPlan(String),
    #[error(transparent)]
    SystemEdit(#[from] SystemEditError),
    #[error(transparent)]
    GroupEdit(#[from] GroupEditError),
    #[error(transparent)]
    Extraction(#[from] ExtractError),
    #[error("unknown component {0}")]
    UnknownComponent(ComponentId),
    #[error("group edit targets authoring generation {expected} but the project is at {actual}")]
    StaleGeneration { expected: u64, actual: u64 },
    #[error(transparent)]
    Text(#[from] bdl_text::TextError),
    #[error("{} changed on disk since the project was opened: {}", files.len(), files.join(", "))]
    ChangedOnDisk { files: Vec<String> },
    /// A name the source cannot spell (docs/spec/textual-syntax.md §2.5): the
    /// text is the semantic source, so it is refused on every surface.
    #[error("`{name}` cannot be a name: {reason}")]
    InvalidName { name: String, reason: String },
    #[error("`{path}` is not a source file of the project: sources are `.bdl` files under `src/`")]
    InvalidSourcePath { path: String },
}

/// Refuse an edit that would introduce a name the source cannot spell.
/// The sources with the current system written back: what the Code view
/// shows and what a save would write.
fn written_back(sys: &SystemState) -> Result<bdl_text::WriteBack, SessionError> {
    let loaded = sys.text.as_ref().ok_or(SessionError::NotASystem)?;
    Ok(bdl_text::write_back(
        &loaded.build,
        &loaded.files,
        &sys.current.system,
    ))
}

/// A source path the Code view may write: a `.bdl` file under `src/`,
/// relative, with no `..`.
fn is_source_path(path: &str) -> bool {
    path.starts_with(&format!("{}/", bdl_text::SOURCE_DIR))
        && path.ends_with(".bdl")
        && !path
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == "..")
}

fn source_diagnostic(
    files: &[bdl_text::SourceFile],
    fault: &bdl_text::TextFault,
) -> SourceDiagnostic {
    let span = fault.span();
    let path = files
        .get(fault.file())
        .map(|f| f.path.clone())
        .unwrap_or_default();
    match fault {
        bdl_text::TextFault::Syntax { error, .. } => SourceDiagnostic {
            path,
            code: "syntax".into(),
            message: error.message.clone(),
            start: span.start,
            end: span.end,
            open: false,
        },
        bdl_text::TextFault::Load(l) => SourceDiagnostic {
            path,
            code: l.code.clone(),
            message: l.message.clone(),
            start: span.start,
            end: span.end,
            open: l.open,
        },
    }
}

/// The layout service on open (ADR-0023 §7): every entity the sources
/// declare but the layout does not place gets a position, and the layout
/// is written back so the first graphical projection is the persisted one.
/// Nothing positioned moves.
fn place_on_open(
    root: &Path,
    system: &BehaviorSystem,
    layout: &Layout,
) -> Result<Layout, SessionError> {
    let placement = bdl_layout::place_missing(system, layout);
    if !placement.is_empty() {
        tracing::info!(root = %root.display(), placed = placement.placed.len(), "placed unpositioned entities");
        persist::save_layout(root, &placement.layout)?;
    }
    Ok(placement.layout)
}

/// The layout service on commit: what an edit created and did not place
/// (a Code-view edit, a template, an extraction) is placed now, so the
/// projection that answers the edit already has a position for it.  The
/// layout is saved with the project; a placement alone never makes the
/// project dirty (it is derived, and derived again the same way), so the
/// saved copy learns the same positions.
fn place_on_commit(system: &BehaviorSystem, layout: &mut Layout, saved: &mut Layout) {
    let placement = bdl_layout::place_missing(system, layout);
    if placement.is_empty() {
        return;
    }
    for placed in &placement.placed {
        let canvas = match placed.component {
            None => &mut *saved,
            Some(c) => saved.components.entry(c).or_default(),
        };
        match placed.node {
            bdl_layout::Node::Concept(id) => {
                canvas.concepts.entry(id).or_insert(placed.at);
            }
            bdl_layout::Node::Mapping(id) => {
                canvas.mappings.entry(id).or_insert(placed.at);
            }
            bdl_layout::Node::Output(id) => {
                canvas.outputs.entry(id).or_insert(placed.at);
            }
            bdl_layout::Node::Instance(id) => {
                canvas.instances.entry(id).or_insert(placed.at);
            }
        }
    }
    *layout = placement.layout;
}

fn check_names<'a>(names: impl IntoIterator<Item = &'a str>) -> Result<(), SessionError> {
    for name in names {
        if let Some(reason) = bdl_text::why_not_identifier(name) {
            return Err(SessionError::InvalidName {
                name: name.to_owned(),
                reason,
            });
        }
    }
    Ok(())
}

/// One authored step of a system project, for undo/redo: a semantic edit
/// (the whole system before it) or an authoring edit (the group table
/// before it).  One history, two kinds: undoing a group edit never
/// replays a compilation.
pub enum HistoryEntry {
    Semantic(Box<BehaviorSystem>),
    Authoring(BTreeMap<BehaviorGroupId, BehaviorGroup>),
}

/// A system edit in flight: the working copy the steps of one authored
/// transaction apply to, and their merged outcome.
pub struct Transaction {
    working: SystemSnapshot,
    outcome: Option<SystemEditOutcome>,
}

impl Transaction {
    /// Apply one system edit (with its rename expansion) to the working
    /// copy; the outcome of this edit alone is returned, and merged into
    /// the transaction's.
    pub fn apply(&mut self, op: &SystemEditOp) -> Result<SystemEditOutcome, SessionError> {
        check_names(bdl_text::names::names_in_system_edit(op))?;
        let mut this: Option<SystemEditOutcome> = None;
        for step in &crate::rename::expand_system(&self.working.system, op) {
            let applied = apply_system_edit(&self.working, step)?;
            self.working = applied.snapshot;
            this = Some(match this {
                None => applied.outcome,
                Some(acc) => merge_system_outcomes(acc, applied.outcome),
            });
        }
        let this = this.unwrap_or_default();
        self.outcome = Some(match self.outcome.take() {
            None => this.clone(),
            Some(acc) => merge_system_outcomes(acc, this.clone()),
        });
        Ok(this)
    }
}

/// What undoing or redoing did on a system project.
pub enum Stepped {
    /// A semantic step: a new revision, a re-derived flat design.
    Semantic(Box<Committed>),
    /// An authoring step: the group table moved, nothing semantic did.
    Authoring,
}

/// The authored truth of a system project, beside the derived `current`.
pub struct SystemState {
    pub current: SystemSnapshot,
    /// The flattening of `current` (its origins serve every projection).
    pub flattened: FlattenedSystem,
    /// Bumped by every group edit — authoring metadata that changes no
    /// semantic fact and therefore no revision (Phase 8b, Theorem A).
    pub authoring_generation: u64,
    /// IDE ground state per component body, for component-scoped drafts:
    /// created on first use, re-seated on every commit.
    component_ide: BTreeMap<ComponentId, IdeHost>,
    saved: BehaviorSystem,
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    /// Present for a text project (ADR-0020): the sources as last loaded
    /// or written, with their anchors and modification stamps.  Saving
    /// writes the system back as text through them.
    pub text: Option<bdl_text::LoadedWorkspace>,
    /// Text the Code view typed that does not build (ADR-0023 §5), by
    /// path, exactly as typed, with why.  The committed project is the
    /// last revision that built.
    drafts: BTreeMap<String, SourceDraft>,
    /// Sources accepted from the Code view since the last save: their
    /// text is in `text` but not on disk yet.
    dirty_sources: BTreeSet<String>,
    /// The unfinished edits as last saved (or loaded): source drafts by
    /// path and definition drafts by scope and relationship.  Dirty is
    /// "the persistent state differs from this".
    saved_source_drafts: BTreeMap<String, String>,
    saved_definition_drafts: BTreeMap<DraftKey, String>,
}

/// A definition draft's identity: the relationship, in the scope it
/// belongs to (a component body's are component-local).
pub type DraftKey = (Option<ComponentId>, DeclId);

/// A text draft the semantic project has not accepted.
#[derive(Clone, Debug)]
pub struct SourceDraft {
    pub text: String,
    pub diagnostics: Vec<SourceDiagnostic>,
}

/// Why a draft does not build: one fault on one file, in byte offsets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDiagnostic {
    pub path: String,
    /// `syntax`, or the loader's `text.<reason>`.
    pub code: String,
    pub message: String,
    pub start: u32,
    pub end: u32,
    /// Incompleteness, not an error: does not keep a draft from building.
    pub open: bool,
}

/// One source file as the Code view shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFileView {
    pub path: String,
    pub text: String,
    /// The text is a draft the semantic project has not accepted.
    pub draft: bool,
    /// Where each canvas entity's item is in `text` (none for a draft).
    pub anchors: Vec<SourceAnchor>,
}

/// The whole item declaring one entity, in byte offsets of its file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceAnchor {
    pub entity: bdl_text::TextEntity,
    pub start: u32,
    pub end: u32,
}

/// The sources of the open project: the files as loaded with every
/// committed semantic edit written back, drafts substituted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sources {
    pub revision: Revision,
    pub files: Vec<SourceFileView>,
    pub diagnostics: Vec<SourceDiagnostic>,
}

/// What a source edit did.
pub struct SourceEdit {
    /// The sources built: a new revision.
    pub accepted: bool,
    /// The derived flat design now (new when accepted).
    pub snapshot: ProjectSnapshot,
}

impl SystemState {
    fn source_draft_texts(&self) -> BTreeMap<String, String> {
        self.drafts
            .iter()
            .map(|(k, v)| (k.clone(), v.text.clone()))
            .collect()
    }

    /// After a commit: every component host sees its body at the new
    /// revision; hosts of components that no longer exist go.  Overlays
    /// (drafts) survive, as on the project host.
    fn reseat_component_hosts(&mut self) {
        let revision = self.current.revision;
        let components = &self.current.system.components;
        self.component_ide
            .retain(|id, _| components.contains_key(id));
        for (id, host) in self.component_ide.iter_mut() {
            if let Some(c) = components.get(id) {
                host.set_committed(ProjectSnapshot {
                    revision,
                    design: c.body.clone(),
                });
                host.set_port_backed(
                    c.interface
                        .ports
                        .values()
                        .map(|p| (p.decl, p.kind))
                        .collect(),
                );
            }
        }
    }
}

/// A simulation run, valid for exactly one project revision.
pub struct SimulationRun {
    pub revision: Revision,
    pub simulation: Simulation,
    /// Concept names at that revision, for rendering samples.
    pub concept_names: std::collections::BTreeMap<bdl_model::SemanticId, String>,
}

pub struct OpenProject {
    pub root: PathBuf,
    pub current: ProjectSnapshot,
    pub layout: Layout,
    /// Dropped on every commit: a run belongs to the revision it started at.
    pub simulation: Option<SimulationRun>,
    /// IDE ground state over `current`: committed snapshot + overlays.
    pub ide: IdeHost,
    /// Present for a behaviour-system project: then `current` is derived
    /// from `system.current` on every commit and never edited directly.
    pub system: Option<SystemState>,
    /// Designs before the current one, oldest first.
    undo: Vec<Design>,
    /// Designs undone, most recently undone last.
    redo: Vec<Design>,
    saved: Design,
    saved_layout: Layout,
}

impl OpenProject {
    pub fn can_undo(&self) -> bool {
        match &self.system {
            Some(s) => !s.undo.is_empty(),
            None => !self.undo.is_empty(),
        }
    }
    pub fn can_redo(&self) -> bool {
        match &self.system {
            Some(s) => !s.redo.is_empty(),
            None => !self.redo.is_empty(),
        }
    }
    /// Whether the persistent state — the system, the layout, the text of
    /// every file (typed or accepted), the definition drafts — differs from
    /// what was last saved or loaded.  One question, one answer: nothing a
    /// designer can see and edit is left out because it does not build.
    pub fn dirty(&self) -> bool {
        let content = match &self.system {
            Some(s) => {
                s.current.system != s.saved
                    || !s.dirty_sources.is_empty()
                    || s.source_draft_texts() != s.saved_source_drafts
                    || self.definition_drafts() != s.saved_definition_drafts
            }
            None => self.current.design != self.saved,
        };
        content || self.layout != self.saved_layout
    }

    /// Every definition draft held for this project, by scope and
    /// relationship: the IDE overlays of the project host and of each
    /// component body's host.
    pub fn definition_drafts(&self) -> BTreeMap<DraftKey, String> {
        let mut out = BTreeMap::new();
        for e in self.ide.overlays().iter() {
            if let bdl_ide_db::Overlay::MappingDefinitionDraft { mapping, source } = &e.overlay {
                out.insert((None, *mapping), source.clone());
            }
        }
        if let Some(s) = &self.system {
            for (component, host) in &s.component_ide {
                for e in host.overlays().iter() {
                    if let bdl_ide_db::Overlay::MappingDefinitionDraft { mapping, source } =
                        &e.overlay
                    {
                        out.insert((Some(*component), *mapping), source.clone());
                    }
                }
            }
        }
        out
    }
    pub fn is_system(&self) -> bool {
        self.system.is_some()
    }
    /// A text project: canonical source under `src/`, written on save.
    pub fn is_text(&self) -> bool {
        self.system.as_ref().is_some_and(|s| s.text.is_some())
    }
}

#[derive(Default)]
pub struct Session {
    project: Option<OpenProject>,
    compiler_version: String,
}

/// What a committed change produced, for subscribers.
#[derive(Debug)]
pub struct Committed {
    pub snapshot: ProjectSnapshot,
    pub outcome: Option<EditOutcome>,
}

/// What a committed system edit produced.
#[derive(Debug)]
pub struct CommittedSystem {
    /// The derived flat design at the new revision.
    pub snapshot: ProjectSnapshot,
    pub outcome: SystemEditOutcome,
}

impl Session {
    pub fn new(compiler_version: &str) -> Self {
        Session {
            project: None,
            compiler_version: compiler_version.to_owned(),
        }
    }

    pub fn project(&self) -> Result<&OpenProject, SessionError> {
        self.project.as_ref().ok_or(SessionError::NoProject)
    }

    fn project_mut(&mut self) -> Result<&mut OpenProject, SessionError> {
        self.project.as_mut().ok_or(SessionError::NoProject)
    }

    fn ensure_closed(&self) -> Result<(), SessionError> {
        match &self.project {
            Some(p) => Err(SessionError::AlreadyOpen(p.root.clone())),
            None => Ok(()),
        }
    }

    /// Open a project (ADR-0023): the sources and sidecars are the truth of
    /// every project; a legacy JSON project is migrated in place first.
    pub fn open(&mut self, root: &Path) -> Result<&OpenProject, SessionError> {
        self.ensure_closed()?;
        let loaded = bdl_text::load_project_with(root, &self.compiler_version)?;
        if let Some(m) = &loaded.migrated {
            tracing::info!(root = %root.display(), from = ?m.from, "migrated a legacy project");
        }
        let snapshot = SystemSnapshot::new(loaded.build.system.clone());
        let layout = place_on_open(root, &snapshot.system, &loaded.layout)?;
        self.install_system(root, snapshot, layout, Some(loaded));
        self.project()
    }

    /// Create a project: `src/main.bdl`, the sidecars, the manifest.
    pub fn init(&mut self, root: &Path, name: &str) -> Result<&OpenProject, SessionError> {
        self.ensure_closed()?;
        let loaded = bdl_text::init_project(root, name, &self.compiler_version)?;
        let snapshot = SystemSnapshot::new(loaded.build.system.clone());
        let layout = loaded.layout.clone();
        self.install_system(root, snapshot, layout, Some(loaded));
        self.project()
    }

    /// The names the protocol still carries for [`Session::init`]: there is
    /// one kind of project (ADR-0023).
    pub fn init_text(&mut self, root: &Path, name: &str) -> Result<&OpenProject, SessionError> {
        self.init(root, name)
    }

    /// Re-read a text project from disk, dropping the in-memory design:
    /// identities come back through the sidecar, layout through its file.
    ///
    /// The reloaded state is a new revision after the one it replaces —
    /// a client that ignores stale revisions (ADR-0009) must see it as
    /// what supersedes its edits, not as an old answer.
    pub fn reload_text(&mut self) -> Result<&OpenProject, SessionError> {
        let root = self.project()?.root.clone();
        if !self.project()?.is_text() {
            return Err(SessionError::NotASystem);
        }
        let after = self.project()?.current.revision.next();
        let loaded = bdl_text::load_project_with(&root, &self.compiler_version)?;
        self.project = None;
        let snapshot = SystemSnapshot {
            revision: after,
            system: loaded.build.system.clone(),
        };
        let layout = place_on_open(&root, &snapshot.system, &loaded.layout)?;
        self.install_system(&root, snapshot, layout, Some(loaded));
        self.project()
    }

    /// The source files a text project has that changed on disk since they
    /// were loaded (empty for other kinds).
    pub fn changed_on_disk(&self) -> Result<Vec<String>, SessionError> {
        let p = self.project()?;
        match p.system.as_ref().and_then(|s| s.text.as_ref()) {
            Some(loaded) => Ok(bdl_text::workspace::changed_on_disk(loaded)?),
            None => Ok(Vec::new()),
        }
    }

    pub fn init_system(&mut self, root: &Path, name: &str) -> Result<&OpenProject, SessionError> {
        self.init(root, name)
    }

    fn install_system(
        &mut self,
        root: &Path,
        system: SystemSnapshot,
        layout: Layout,
        text: Option<bdl_text::LoadedWorkspace>,
    ) {
        let flattened = flatten(&system);
        let snapshot = flattened.snapshot.clone();
        self.install(root, snapshot, layout);
        // The unfinished edits the project was saved with come back exactly
        // as they were: the typed text of every file that does not build
        // (judged again, for its reasons) and every definition draft.
        let (source_drafts, definition_drafts) = match &text {
            Some(loaded) => (
                loaded
                    .drafts
                    .sources
                    .iter()
                    .map(|(path, typed)| {
                        let mut files = loaded.files.clone();
                        if let Some(f) = files.iter_mut().find(|f| f.path == *path) {
                            f.text = typed.clone();
                        }
                        let name = system.system.base.name.clone();
                        let build = bdl_text::load_workspace(&name, &files, &loaded.build.table);
                        let diagnostics = build
                            .faults
                            .iter()
                            .map(|f| source_diagnostic(&files, f))
                            .collect();
                        (
                            path.clone(),
                            SourceDraft {
                                text: typed.clone(),
                                diagnostics,
                            },
                        )
                    })
                    .collect::<BTreeMap<_, _>>(),
                loaded
                    .drafts
                    .definitions
                    .iter()
                    .map(|d| {
                        (
                            (
                                d.component.map(ComponentId::from_raw),
                                DeclId::from_raw(d.mapping),
                            ),
                            d.source.clone(),
                        )
                    })
                    .collect::<BTreeMap<DraftKey, String>>(),
            ),
            None => (BTreeMap::new(), BTreeMap::new()),
        };
        if let Some(p) = self.project.as_mut() {
            p.system = Some(SystemState {
                saved: system.system.clone(),
                current: system,
                flattened,
                authoring_generation: 0,
                component_ide: BTreeMap::new(),
                undo: Vec::new(),
                redo: Vec::new(),
                text,
                saved_source_drafts: source_drafts
                    .iter()
                    .map(|(k, v)| (k.clone(), v.text.clone()))
                    .collect(),
                drafts: source_drafts,
                dirty_sources: BTreeSet::new(),
                saved_definition_drafts: definition_drafts.clone(),
            });
        }
        for ((scope, mapping), source) in definition_drafts {
            if let Ok(host) = self.ide_in(scope) {
                host.set_definition_draft(mapping, source);
            }
        }
    }

    fn install(&mut self, root: &Path, snapshot: ProjectSnapshot, layout: Layout) {
        self.project = Some(OpenProject {
            root: root.to_path_buf(),
            saved: snapshot.design.clone(),
            saved_layout: layout.clone(),
            ide: IdeHost::new(snapshot.clone()),
            current: snapshot,
            layout,
            simulation: None,
            system: None,
            undo: Vec::new(),
            redo: Vec::new(),
        });
    }

    pub fn close(&mut self) -> Result<(), SessionError> {
        self.project()?;
        self.project = None;
        Ok(())
    }

    /// Save; a text project refuses when its sources changed on disk since
    /// they were loaded, unless `force` overwrites them (ADR-0020 §9).
    pub fn save_with(&mut self, force: bool) -> Result<(), SessionError> {
        let version = self.compiler_version.clone();
        if !force {
            let changed = self.changed_on_disk()?;
            if !changed.is_empty() {
                return Err(SessionError::ChangedOnDisk { files: changed });
            }
        }
        let definition_drafts = self.project()?.definition_drafts();
        let p = self.project_mut()?;
        match p.system.as_mut() {
            Some(s) if s.text.is_some() => {
                // Text is the only truth written: the system goes back as
                // item-level edits of the sources — a file whose typed text
                // does not build is written as typed, its last good text
                // beside it — then the sources are re-read so anchors and
                // stamps describe what is on disk.  Text accepted from the
                // Code view is already in the files the splice started
                // from; `rewrite` takes it to disk.
                let loaded = s.text.as_ref().expect("checked");
                let drafts = bdl_text::Drafts {
                    sources: s.source_draft_texts(),
                    definitions: definition_drafts
                        .iter()
                        .map(|((scope, mapping), source)| bdl_text::DefinitionDraftFile {
                            component: scope.map(|c| c.raw()),
                            mapping: mapping.raw(),
                            source: source.clone(),
                        })
                        .collect(),
                };
                let rewrite: Vec<String> = s.dirty_sources.iter().cloned().collect();
                bdl_text::save_project_with(
                    &p.root,
                    loaded,
                    &s.current.system,
                    &p.layout,
                    &drafts,
                    &rewrite,
                    &version,
                )?;
                s.dirty_sources.clear();
                let reloaded = bdl_text::load_project_with(&p.root, &version)?;
                if reloaded.build.system != s.current.system {
                    tracing::warn!(
                        root = %p.root.display(),
                        "the written text does not read back as the saved system; keeping the in-memory design"
                    );
                }
                s.text = Some(reloaded);
                s.saved = s.current.system.clone();
                s.saved_source_drafts = drafts.sources;
                s.saved_definition_drafts = definition_drafts;
            }
            // Every open project has sources (ADR-0023); these arms are the
            // type's, not a second persistence.
            Some(_) | None => return Err(SessionError::NotASystem),
        }
        p.saved_layout = p.layout.clone();
        Ok(())
    }

    /// Apply one flat edit against `base`: the same as the system edit
    /// `Base { op }` — every project is a behaviour system whose flat
    /// design is derived (ADR-0023), so there is one edit path and one
    /// history.  The protocol's `ApplyEdit` lands here.
    pub fn apply(&mut self, base: Revision, op: &EditOp) -> Result<Committed, SessionError> {
        let c = self
            .apply_system(base, &SystemEditOp::Base { op: op.clone() })
            .map_err(|e| match e {
                // the flat model's own refusal, in its own words
                SessionError::SystemEdit(SystemEditError::Base(inner)) => SessionError::Edit(inner),
                other => other,
            })?;
        Ok(Committed {
            snapshot: c.snapshot,
            outcome: c.outcome.inner,
        })
    }

    /// Apply one system edit against `base`: the system moves, the flat
    /// design is re-derived, and every consumer of `current` sees the new
    /// revision as after any other commit.
    pub fn apply_system(
        &mut self,
        base: Revision,
        op: &SystemEditOp,
    ) -> Result<CommittedSystem, SessionError> {
        self.transaction(base, |tx| tx.apply(op).map(|_| ()))
    }

    /// Instantiate a library item: every planned step applied in one
    /// transaction — one history entry (one undo removes the whole
    /// fragment; a redo recreates it with the same identities), and nothing
    /// at all when any step is refused.  A mapping step names the concepts
    /// it reads and produces by fragment key, resolved here to the
    /// identities the earlier steps allocated; the keys reach nothing.  The
    /// result is ordinary objects: nothing about the item is recorded, and
    /// a Source is a Source because of its shape (ADR-0032).
    pub fn apply_library_item(
        &mut self,
        base: Revision,
        component: Option<ComponentId>,
        steps: &[bdl_library::PlannedStep],
    ) -> Result<CommittedSystem, SessionError> {
        let wrap = |op: EditOp| match component {
            None => SystemEditOp::Base { op },
            Some(c) => SystemEditOp::EditComponentBody { component: c, op },
        };
        self.transaction(base, |tx| {
            if let Some(c) = component {
                if !tx.working.system.components.contains_key(&c) {
                    return Err(SessionError::UnknownComponent(c));
                }
            }
            let mut created: BTreeMap<String, bdl_model::SemanticId> = BTreeMap::new();
            for step in steps {
                let op = match step {
                    bdl_library::PlannedStep::Concept { op, .. } => op.clone(),
                    bdl_library::PlannedStep::Mapping {
                        name,
                        description,
                        inputs,
                        output,
                        ..
                    } => {
                        let resolve = |key: &String| {
                            created.get(key).copied().ok_or_else(|| {
                                SessionError::LibraryPlan(format!(
                                    "`{name}` names `{key}`, which no earlier step created"
                                ))
                            })
                        };
                        let inputs = inputs.iter().map(resolve).collect::<Result<Vec<_>, _>>()?;
                        let output = resolve(output)?;
                        EditOp::CreateMapping {
                            name: name.clone(),
                            description: description.clone(),
                            signature: bdl_model::surface::Signature { inputs, output },
                            definition: None,
                            clock: None,
                        }
                    }
                };
                let outcome = tx.apply(&wrap(op))?;
                if let Some(id) = outcome.inner.as_ref().and_then(|o| o.created_concept) {
                    created.insert(step.key().to_string(), id);
                }
            }
            Ok(())
        })
    }

    /// One authored step of a system project: `body` applies any number of
    /// system edits to a working copy; they land together — one history
    /// entry, one re-derivation, one merged outcome — or not at all.
    fn transaction(
        &mut self,
        base: Revision,
        body: impl FnOnce(&mut Transaction) -> Result<(), SessionError>,
    ) -> Result<CommittedSystem, SessionError> {
        let p = self.project_mut()?;
        let Some(sys) = p.system.as_mut() else {
            return Err(SessionError::NotASystem);
        };
        if p.current.revision != base {
            return Err(SessionError::StaleRevision {
                expected: base,
                actual: p.current.revision,
            });
        }
        // The system's own revision counter follows the project's, so the
        // derived snapshot and the authored system share one number.
        let mut tx = Transaction {
            working: SystemSnapshot {
                revision: p.current.revision,
                system: sys.current.system.clone(),
            },
            outcome: None,
        };
        body(&mut tx)?;
        // One authored step is one revision, however many edits it
        // expanded to: the working copy's counter advanced per edit.
        let mut snapshot = tx.working;
        snapshot.revision = base.next();
        let applied = AppliedSystem {
            snapshot,
            outcome: tx.outcome.unwrap_or_default(),
        };
        let previous = std::mem::replace(&mut sys.current, applied.snapshot);
        sys.undo
            .push(HistoryEntry::Semantic(Box::new(previous.system)));
        sys.redo.clear();
        sys.flattened = flatten(&sys.current);
        sys.reseat_component_hosts();
        p.current = sys.flattened.snapshot.clone();
        p.simulation = None;
        p.ide.set_committed(p.current.clone());
        place_on_commit(&sys.current.system, &mut p.layout, &mut p.saved_layout);
        Ok(CommittedSystem {
            snapshot: p.current.clone(),
            outcome: applied.outcome,
        })
    }

    /// Apply one group edit: authoring metadata only.  No revision, no
    /// re-derivation, no simulation reset — the flat design is the same
    /// value before and after (FV Theorem A).  The authoring generation
    /// moves so a client can tell the views apart.
    pub fn apply_group(
        &mut self,
        base_generation: Option<u64>,
        op: &GroupEditOp,
    ) -> Result<(), SessionError> {
        let p = self.project_mut()?;
        let Some(sys) = p.system.as_mut() else {
            return Err(SessionError::NotASystem);
        };
        // Two clients never silently overwrite each other's membership: an
        // edit names the generation it saw.
        if let Some(g) = base_generation {
            if g != sys.authoring_generation {
                return Err(SessionError::StaleGeneration {
                    expected: g,
                    actual: sys.authoring_generation,
                });
            }
        }
        let (system, _outcome) = apply_group_edit(&sys.current.system, op)?;
        let previous = std::mem::replace(&mut sys.current.system, system);
        sys.undo.push(HistoryEntry::Authoring(previous.groups));
        sys.redo.clear();
        sys.authoring_generation += 1;
        Ok(())
    }

    /// What "Package as reusable component" would do to a group.  Reads only.
    pub fn preview_extraction(
        &self,
        group: BehaviorGroupId,
        choices: &ExtractionChoices,
    ) -> Result<ExtractionPreview, SessionError> {
        let p = self.project()?;
        let sys = p.system.as_ref().ok_or(SessionError::NotASystem)?;
        Ok(preview_extraction(&sys.current, group, choices)?)
    }

    /// Every group's boundary, read off the analyses the IDE hosts already
    /// hold for the revision — the flat one for the system's own groups,
    /// a body's own for a component's — no re-analysis for a group edit.
    pub fn group_boundaries(
        &mut self,
    ) -> Result<BTreeMap<BehaviorGroupId, bdl_system::GroupBoundary>, SessionError> {
        let groups: Vec<BehaviorGroup> = {
            let p = self.project()?;
            let sys = p.system.as_ref().ok_or(SessionError::NotASystem)?;
            sys.current.system.groups.values().cloned().collect()
        };
        let mut out = BTreeMap::new();
        for g in groups {
            let scope = match g.scope {
                GroupScope::SystemBase => None,
                GroupScope::Component { component } => Some(component),
            };
            let analysis = self.ide_in(scope)?.committed_analysis();
            let p = self.project()?;
            let sys = p.system.as_ref().ok_or(SessionError::NotASystem)?;
            let Some(design) = sys.current.system.design_of(g.scope) else {
                continue;
            };
            out.insert(
                g.id,
                bdl_system::group_boundary(design, &analysis, &g.members),
            );
        }
        Ok(out)
    }

    /// The system analysis of the open system project.
    pub fn system_analysis(&self) -> Result<SystemAnalysis, SessionError> {
        let p = self.project()?;
        let sys = p.system.as_ref().ok_or(SessionError::NotASystem)?;
        Ok(analyze_system(&sys.current))
    }

    /// Undo/redo of a system project: one history of semantic and
    /// authoring steps.  A semantic step moves the system back and
    /// re-derives the flat design (a new revision); an authoring step moves
    /// the group table back (a new authoring generation, no revision, no
    /// compilation).
    pub fn system_step(&mut self, undo: bool) -> Result<Stepped, SessionError> {
        let p = self.project_mut()?;
        let Some(sys) = p.system.as_mut() else {
            return Err(SessionError::NotASystem);
        };
        let entry = if undo {
            sys.undo.pop().ok_or(SessionError::NothingToUndo)?
        } else {
            sys.redo.pop().ok_or(SessionError::NothingToRedo)?
        };
        match entry {
            HistoryEntry::Authoring(groups) => {
                let previous = std::mem::replace(&mut sys.current.system.groups, groups);
                let back = HistoryEntry::Authoring(previous);
                if undo {
                    sys.redo.push(back);
                } else {
                    sys.undo.push(back);
                }
                sys.authoring_generation += 1;
                Ok(Stepped::Authoring)
            }
            HistoryEntry::Semantic(system) => {
                let next = SystemSnapshot {
                    revision: p.current.revision.next(),
                    system: *system,
                };
                let previous = std::mem::replace(&mut sys.current, next);
                let back = HistoryEntry::Semantic(Box::new(previous.system));
                if undo {
                    sys.redo.push(back);
                } else {
                    sys.undo.push(back);
                }
                sys.flattened = flatten(&sys.current);
                sys.reseat_component_hosts();
                p.current = sys.flattened.snapshot.clone();
                p.simulation = None;
                p.ide.set_committed(p.current.clone());
                place_on_commit(&sys.current.system, &mut p.layout, &mut p.saved_layout);
                Ok(Stepped::Semantic(Box::new(Committed {
                    snapshot: p.current.clone(),
                    outcome: None,
                })))
            }
        }
    }

    pub fn undo(&mut self) -> Result<Committed, SessionError> {
        if self.project()?.system.is_some() {
            return match self.system_step(true)? {
                Stepped::Semantic(c) => Ok(*c),
                Stepped::Authoring => Ok(Committed {
                    snapshot: self.project()?.current.clone(),
                    outcome: None,
                }),
            };
        }
        let p = self.project_mut()?;
        let design = p.undo.pop().ok_or(SessionError::NothingToUndo)?;
        let next = ProjectSnapshot {
            revision: p.current.revision.next(),
            design,
        };
        let previous = std::mem::replace(&mut p.current, next);
        p.redo.push(previous.design);
        p.simulation = None;
        p.ide.set_committed(p.current.clone());
        Ok(Committed {
            snapshot: p.current.clone(),
            outcome: None,
        })
    }

    pub fn redo(&mut self) -> Result<Committed, SessionError> {
        if self.project()?.system.is_some() {
            return match self.system_step(false)? {
                Stepped::Semantic(c) => Ok(*c),
                Stepped::Authoring => Ok(Committed {
                    snapshot: self.project()?.current.clone(),
                    outcome: None,
                }),
            };
        }
        let p = self.project_mut()?;
        let design = p.redo.pop().ok_or(SessionError::NothingToRedo)?;
        let next = ProjectSnapshot {
            revision: p.current.revision.next(),
            design,
        };
        let previous = std::mem::replace(&mut p.current, next);
        p.undo.push(previous.design);
        p.simulation = None;
        p.ide.set_committed(p.current.clone());
        Ok(Committed {
            snapshot: p.current.clone(),
            outcome: None,
        })
    }

    /// The sources as the Code view shows them (ADR-0023 §3): the files as
    /// loaded with every committed semantic edit written back — the same
    /// item-level splice a save performs — and any draft substituted.
    pub fn sources(&self) -> Result<Sources, SessionError> {
        let p = self.project()?;
        let Some(sys) = p.system.as_ref() else {
            return Err(SessionError::NotASystem);
        };
        let wb = written_back(sys)?;
        // Anchors are spans into the text as shown, which the splice moved:
        // read the written-back files once more for them.
        let name = sys.current.system.base.name.clone();
        let build = bdl_text::load_workspace(&name, &wb.files, &wb.table);
        let files = wb
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| match sys.drafts.get(&f.path) {
                Some(d) => SourceFileView {
                    path: f.path.clone(),
                    text: d.text.clone(),
                    draft: true,
                    anchors: Vec::new(),
                },
                None => SourceFileView {
                    path: f.path.clone(),
                    text: f.text.clone(),
                    draft: false,
                    anchors: build
                        .anchors
                        .iter()
                        .filter(|a| a.file == i && a.role == bdl_text::AnchorRole::Item)
                        // bindings and exports are not canvas objects
                        .filter(|a| {
                            !matches!(
                                a.entity,
                                bdl_text::TextEntity::Binding(_) | bdl_text::TextEntity::Export(_)
                            )
                        })
                        .map(|a| SourceAnchor {
                            entity: a.entity,
                            start: a.span.start,
                            end: a.span.end,
                        })
                        .collect(),
                },
            })
            .collect();
        Ok(Sources {
            revision: p.current.revision,
            files,
            diagnostics: sys
                .drafts
                .values()
                .flat_map(|d| d.diagnostics.iter().cloned())
                .collect(),
        })
    }

    /// A text edit from the Code view (ADR-0023 §4–§5): the whole text of
    /// one file against `base`.  When the sources build, every declaration
    /// is bound to its identity by reconciliation against the working
    /// table and the project moves to a new revision; when they do not,
    /// the committed project stays and the draft is kept with its faults.
    /// A text change that leaves the semantic project equal moves the
    /// revision but adds no history entry: undo is semantic history.
    pub fn apply_source_edit(
        &mut self,
        base: Revision,
        path: &str,
        text: &str,
    ) -> Result<SourceEdit, SessionError> {
        let p = self.project_mut()?;
        let Some(sys) = p.system.as_mut() else {
            return Err(SessionError::NotASystem);
        };
        if p.current.revision != base {
            return Err(SessionError::StaleRevision {
                expected: base,
                actual: p.current.revision,
            });
        }
        if !is_source_path(path) {
            return Err(SessionError::InvalidSourcePath {
                path: path.to_owned(),
            });
        }
        let wb = written_back(sys)?;
        let mut files = wb.files.clone();
        match files.iter_mut().find(|f| f.path == path) {
            Some(f) => f.text = text.to_owned(),
            None => files.push(bdl_text::SourceFile {
                path: path.to_owned(),
                text: text.to_owned(),
            }),
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let name = sys.current.system.base.name.clone();
        let mut build = bdl_text::load_workspace(&name, &files, &wb.table);
        let diagnostics: Vec<SourceDiagnostic> = build
            .faults
            .iter()
            .map(|f| source_diagnostic(&files, f))
            .collect();
        if diagnostics.iter().any(|d| !d.open) {
            sys.drafts.insert(
                path.to_owned(),
                SourceDraft {
                    text: text.to_owned(),
                    diagnostics,
                },
            );
            return Ok(SourceEdit {
                accepted: false,
                snapshot: p.current.clone(),
            });
        }
        // Groups are authoring metadata the text does not carry: they
        // follow the identities that survived.
        let known: std::collections::BTreeSet<DeclId> =
            build.system.base.mappings.keys().copied().collect();
        for (id, g) in &sys.current.system.groups {
            let mut g = g.clone();
            g.members
                .retain(|m| known.contains(m) || g.scope != GroupScope::SystemBase);
            build.system.groups.insert(*id, g);
        }
        bdl_system::prune_groups(&mut build.system);
        let system = build.system.clone();
        if let Some(loaded) = sys.text.as_mut() {
            loaded.files = files;
            loaded.build = build;
        }
        sys.drafts.remove(path);
        sys.dirty_sources.insert(path.to_owned());
        let changed = system != sys.current.system;
        let next = SystemSnapshot {
            revision: p.current.revision.next(),
            system,
        };
        let previous = std::mem::replace(&mut sys.current, next);
        if changed {
            sys.undo
                .push(HistoryEntry::Semantic(Box::new(previous.system)));
            sys.redo.clear();
        }
        sys.flattened = flatten(&sys.current);
        sys.reseat_component_hosts();
        p.current = sys.flattened.snapshot.clone();
        p.simulation = None;
        p.ide.set_committed(p.current.clone());
        place_on_commit(&sys.current.system, &mut p.layout, &mut p.saved_layout);
        Ok(SourceEdit {
            accepted: true,
            snapshot: p.current.clone(),
        })
    }

    /// The IDE ground state of the open project.
    pub fn ide(&mut self) -> Result<&mut IdeHost, SessionError> {
        Ok(&mut self.project_mut()?.ide)
    }

    /// The IDE host a draft lives in: the project's, or a component body's
    /// (created on first use, re-seated on every commit).  A component's
    /// body is an ordinary design, so the same service serves it — in the
    /// body's own scope, where the component's names mean what they mean
    /// to the component, not to any instance.
    fn ide_in(&mut self, scope: Option<ComponentId>) -> Result<&mut IdeHost, SessionError> {
        let p = self.project_mut()?;
        let Some(component) = scope else {
            return Ok(&mut p.ide);
        };
        let sys = p.system.as_mut().ok_or(SessionError::NotASystem)?;
        let c = sys
            .current
            .system
            .components
            .get(&component)
            .ok_or(SessionError::UnknownComponent(component))?;
        let revision = sys.current.revision;
        let body = c.body.clone();
        // the body's port-backed declarations present their port's role
        let ports: BTreeMap<DeclId, bdl_system::PortKind> = c
            .interface
            .ports
            .values()
            .map(|p| (p.decl, p.kind))
            .collect();
        let host = sys.component_ide.entry(component).or_insert_with(|| {
            IdeHost::new(ProjectSnapshot {
                revision,
                design: body,
            })
        });
        host.set_port_backed(ports);
        Ok(host)
    }

    /// Studio typed in the definition editor: `source` becomes the draft
    /// overlay of `mapping` and the compiler's verdict on the resulting
    /// world is returned.  The project, its revision and its history are
    /// untouched; the overlay stays until a commit makes it the committed
    /// definition, the mapping is deleted, or a newer draft replaces it.
    /// Served by the `AnalyzeDefinitionDraft` request (protocol 0.4).
    pub fn draft_verdict(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
        source: &str,
    ) -> Result<DraftVerdict, SessionError> {
        let snapshot = self.draft_snapshot(scope, mapping, source)?;
        Ok(draft_verdict(&snapshot, mapping)?)
    }

    /// Set the draft overlay and take the snapshot of the resulting world,
    /// as one request scoped to that overlay: setting the overlay cancels
    /// whatever earlier request was still composing for the same mapping,
    /// and this request's own token is polled between the composition
    /// phases.  The coordinator is serial today, so the token is never
    /// tripped mid-flight; the wiring is what a worker pool will need.
    fn draft_snapshot(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
        source: &str,
    ) -> Result<std::sync::Arc<AnalysisSnapshot>, SessionError> {
        let host = self.ide_in(scope)?;
        host.set_definition_draft(mapping, source);
        let (request, token) =
            host.begin_request(CancelScope::Overlay(OverlayKey::MappingDefinition {
                mapping,
            }));
        let snapshot = host.snapshot_cancellable(&token);
        host.end_request(request);
        Ok(snapshot.map_err(QueryError::from)?)
    }

    /// The draft is gone (revert, reload, detach): later queries on every
    /// surface see the committed definition again.  True if there was one.
    pub fn discard_draft(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
    ) -> Result<bool, SessionError> {
        Ok(self.ide_in(scope)?.clear_definition_draft(mapping))
    }

    /// Completion candidates at a byte offset into the draft.
    pub fn draft_completion(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
        source: &str,
        offset: u32,
    ) -> Result<Vec<SemanticCompletion>, SessionError> {
        let snapshot = self.draft_snapshot(scope, mapping, source)?;
        if !snapshot.effective().design.mappings.contains_key(&mapping) {
            return Err(QueryError::UnknownEntity {
                entity: EntityRef::Mapping(mapping),
            }
            .into());
        }
        Ok(completion(
            &snapshot,
            &CompletionContext::Formula { mapping, offset },
        ))
    }

    /// The Formula Composer's view of the mapping's effective definition
    /// (its draft overlay when one exists, else the committed formula);
    /// sets no overlay.
    pub fn formula_projection(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
    ) -> Result<bdl_ide::FormulaProjection, SessionError> {
        let snapshot = self.ide_in(scope)?.snapshot();
        Ok(bdl_ide::formula_projection(&snapshot, mapping)?)
    }

    /// What a slot of the draft expects and what fits.
    pub fn formula_slot(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
        source: &str,
        node: &str,
    ) -> Result<bdl_ide::SlotInfo, SessionError> {
        let snapshot = self.draft_snapshot(scope, mapping, source)?;
        Ok(bdl_ide::formula_slot(&snapshot, mapping, node)?)
    }

    /// A structured action on the draft, answered with the text it makes;
    /// the overlay is *not* moved to the answer — Studio puts it into the
    /// draft and the ordinary analysis follows.
    pub fn compose_formula(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
        source: &str,
        op: &bdl_ide::ComposeOp,
    ) -> Result<bdl_ide::ComposeResult, SessionError> {
        let snapshot = self.draft_snapshot(scope, mapping, source)?;
        Ok(bdl_ide::compose(&snapshot, mapping, source, op)?)
    }

    /// The concept named at a byte offset into the draft, explained by the
    /// IDE service; `None` when nothing semantic is under the cursor.
    pub fn draft_hover(
        &mut self,
        scope: Option<ComponentId>,
        mapping: DeclId,
        source: &str,
        offset: u32,
    ) -> Result<Option<(TextRange, SemanticHover)>, SessionError> {
        let snapshot = self.draft_snapshot(scope, mapping, source)?;
        if !snapshot.effective().design.mappings.contains_key(&mapping) {
            return Err(QueryError::UnknownEntity {
                entity: EntityRef::Mapping(mapping),
            }
            .into());
        }
        let Some((range, _)) = snapshot.index().formula_name_at(mapping, offset) else {
            return Ok(None);
        };
        let Some(entity) = entity_at_formula(&snapshot, mapping, offset) else {
            return Ok(None);
        };
        Ok(hover(&snapshot, entity).map(|h| (range, h)))
    }

    /// The current world's snapshot (committed + whatever overlays exist),
    /// for entity queries that set no overlay of their own.
    pub fn ide_snapshot(&mut self) -> Result<std::sync::Arc<AnalysisSnapshot>, SessionError> {
        Ok(self.ide()?.snapshot())
    }

    pub fn simulation_mut(&mut self) -> Result<&mut Option<SimulationRun>, SessionError> {
        Ok(&mut self.project_mut()?.simulation)
    }

    /// Layout is not semantics: it does not create a revision.
    pub fn set_layout(&mut self, layout: Layout) -> Result<(), SessionError> {
        self.project_mut()?.layout = layout;
        Ok(())
    }
}

/// The outcome of several model steps committed as one revision: the
/// strongest kind, every invalidation, every origin, the first creation.
fn merge_outcomes(mut acc: EditOutcome, next: EditOutcome) -> EditOutcome {
    if next.kind == Some(EditKind::Edit) || acc.kind.is_none() {
        acc.kind = next.kind.or(acc.kind);
    }
    acc.invalidates.extend(next.invalidates);
    acc.origin_decls.extend(next.origin_decls);
    acc.created_concept = acc.created_concept.or(next.created_concept);
    acc.created_mapping = acc.created_mapping.or(next.created_mapping);
    acc.created_clock = acc.created_clock.or(next.created_clock);
    acc.created_output = acc.created_output.or(next.created_output);
    acc.created_device = acc.created_device.or(next.created_device);
    acc
}

fn merge_system_outcomes(mut acc: SystemEditOutcome, next: SystemEditOutcome) -> SystemEditOutcome {
    if next.kind == Some(EditKind::Edit) || acc.kind.is_none() {
        acc.kind = next.kind.or(acc.kind);
    }
    acc.invalidates.extend(next.invalidates);
    acc.origin_decls.extend(next.origin_decls);
    acc.instances.extend(next.instances);
    acc.bindings.extend(next.bindings);
    acc.created_component = acc.created_component.or(next.created_component);
    acc.created_instance = acc.created_instance.or(next.created_instance);
    acc.created_port = acc.created_port.or(next.created_port);
    acc.created_binding = acc.created_binding.or(next.created_binding);
    acc.created_export = acc.created_export.or(next.created_export);
    acc.inner = match (acc.inner.take(), next.inner) {
        (Some(a), Some(b)) => Some(merge_outcomes(a, b)),
        (a, b) => a.or(b),
    };
    acc
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::surface::Signature;

    fn concept(name: &str) -> EditOp {
        EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: None,
        }
    }

    #[test]
    fn edits_undo_redo_keep_revision_monotone() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new("test");
        s.init(dir.path(), "lamp").unwrap();
        let c = s.apply(Revision::INITIAL, &concept("Tilt")).unwrap();
        assert_eq!(c.snapshot.revision, Revision::from_raw(1));
        assert!(s.project().unwrap().dirty());

        // stale base is refused, project untouched
        let err = s
            .apply(Revision::INITIAL, &concept("Brightness"))
            .unwrap_err();
        assert!(matches!(err, SessionError::StaleRevision { .. }));
        assert_eq!(s.project().unwrap().current.revision, Revision::from_raw(1));

        let u = s.undo().unwrap();
        assert_eq!(u.snapshot.revision, Revision::from_raw(2));
        assert!(u.snapshot.design.concepts.is_empty());
        assert!(!s.project().unwrap().dirty());
        let r = s.redo().unwrap();
        assert_eq!(r.snapshot.revision, Revision::from_raw(3));
        assert_eq!(r.snapshot.design.concepts.len(), 1);
        assert!(s.redo().is_err());
    }

    #[test]
    fn save_then_reopen_preserves_unresolved_mapping() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new("test");
        s.init(dir.path(), "lamp").unwrap();
        let c = s.apply(Revision::from_raw(0), &concept("Tilt")).unwrap();
        let tilt = c.outcome.unwrap().created_concept.unwrap();
        let c = s
            .apply(Revision::from_raw(1), &concept("Brightness"))
            .unwrap();
        let bright = c.outcome.unwrap().created_concept.unwrap();
        let c = s
            .apply(
                Revision::from_raw(2),
                &EditOp::CreateMapping {
                    name: "dimByTilt".into(),
                    description: String::new(),
                    signature: Signature {
                        inputs: vec![tilt],
                        output: bright,
                    },
                    definition: None,
                    clock: None,
                },
            )
            .unwrap();
        let id = c.outcome.unwrap().created_mapping.unwrap();
        s.save_with(false).unwrap();
        assert!(!s.project().unwrap().dirty());
        s.close().unwrap();

        let mut s2 = Session::new("test");
        let p = s2.open(dir.path()).unwrap();
        assert_eq!(p.current.design, c.snapshot.design);
        assert!(p.current.design.mappings[&id].is_unresolved());
        assert_eq!(p.current.revision, Revision::INITIAL);
    }

    #[test]
    fn drafts_are_overlays_over_the_committed_revision() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new("test");
        s.init(dir.path(), "lamp").unwrap();
        let rep = |d| Some(bdl_model::Representation::Quantity { dim: d });
        let mk = |name: &str, r| EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: r,
        };
        let tilt = s
            .apply(
                Revision::from_raw(0),
                &mk("Tilt", rep(bdl_model::Dim::ANGLE)),
            )
            .unwrap()
            .outcome
            .unwrap()
            .created_concept
            .unwrap();
        let bright = s
            .apply(
                Revision::from_raw(1),
                &mk("Brightness", rep(bdl_model::Dim::ZERO)),
            )
            .unwrap()
            .outcome
            .unwrap()
            .created_concept
            .unwrap();
        let id = s
            .apply(
                Revision::from_raw(2),
                &EditOp::CreateMapping {
                    name: "dimByTilt".into(),
                    description: String::new(),
                    signature: Signature {
                        inputs: vec![tilt],
                        output: bright,
                    },
                    definition: None,
                    clock: None,
                },
            )
            .unwrap()
            .outcome
            .unwrap()
            .created_mapping
            .unwrap();

        // The draft is judged; the project is not touched.
        let v = s.draft_verdict(None, id, "Tilt / 90 deg").unwrap();
        assert_eq!(v.status, bdl_compiler::MappingStatus::ClockConsistent);
        assert_eq!(v.stamp.revision, Revision::from_raw(3));
        let p = s.project().unwrap();
        assert_eq!(p.current.revision, Revision::from_raw(3));
        assert!(p.current.design.mappings[&id].is_unresolved());
        assert_eq!(p.ide.overlays().len(), 1);

        // A newer draft supersedes; its verdict carries a newer stamp.
        let v2 = s.draft_verdict(None, id, "Tilt + 1 s").unwrap();
        assert_eq!(v2.status, bdl_compiler::MappingStatus::Invalid);
        assert!(v2.stamp > v.stamp);
        assert!(!v2.diagnostics.is_empty());

        // Committing the draft's text drops the overlay; committing
        // something else keeps it (the designer's text is not lost).
        s.apply(
            Revision::from_raw(3),
            &EditOp::AttachDefinition {
                id,
                definition: bdl_model::Definition::Formula {
                    source: "Tilt + 1 s".into(),
                },
            },
        )
        .unwrap();
        assert!(s.project().unwrap().ide.overlays().is_empty());
        s.draft_verdict(None, id, "Tilt / 45 deg").unwrap();
        s.undo().unwrap();
        assert_eq!(s.project().unwrap().ide.overlays().len(), 1);
        assert!(matches!(
            s.draft_verdict(None, DeclId::from_raw(99), "1"),
            Err(SessionError::Ide(QueryError::UnknownEntity { .. }))
        ));
    }

    #[test]
    fn open_twice_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new("test");
        s.init(dir.path(), "lamp").unwrap();
        assert!(matches!(
            s.open(dir.path()),
            Err(SessionError::AlreadyOpen(_))
        ));
    }
}
