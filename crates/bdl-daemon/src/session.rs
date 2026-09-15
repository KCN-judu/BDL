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
use bdl_model::edit::{apply_edit, EditError, EditOp, EditOutcome};
use bdl_model::layout::Layout;
use bdl_model::persist::{self, PersistError};
use bdl_model::surface::{Design, ProjectSnapshot};
use bdl_model::{DeclId, Revision};
use bdl_reactive::Simulation;
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
    /// Designs before the current one, oldest first.
    undo: Vec<Design>,
    /// Designs undone, most recently undone last.
    redo: Vec<Design>,
    saved: Design,
    saved_layout: Layout,
}

impl OpenProject {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn dirty(&self) -> bool {
        self.current.design != self.saved || self.layout != self.saved_layout
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

    pub fn open(&mut self, root: &Path) -> Result<&OpenProject, SessionError> {
        self.ensure_closed()?;
        let loaded = persist::load_project(root)?;
        self.install(root, loaded.snapshot, loaded.layout);
        self.project()
    }

    pub fn init(&mut self, root: &Path, name: &str) -> Result<&OpenProject, SessionError> {
        self.ensure_closed()?;
        let created = persist::init_project(root, name, &self.compiler_version)?;
        self.install(root, created.snapshot, created.layout);
        self.project()
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
            undo: Vec::new(),
            redo: Vec::new(),
        });
    }

    pub fn close(&mut self) -> Result<(), SessionError> {
        self.project()?;
        self.project = None;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), SessionError> {
        let version = self.compiler_version.clone();
        let p = self.project_mut()?;
        persist::save_project(&p.root, &p.current, &p.layout, &version)?;
        p.saved = p.current.design.clone();
        p.saved_layout = p.layout.clone();
        Ok(())
    }

    /// Apply one edit against `base`; refused if the project has moved on.
    pub fn apply(&mut self, base: Revision, op: &EditOp) -> Result<Committed, SessionError> {
        let p = self.project_mut()?;
        if p.current.revision != base {
            return Err(SessionError::StaleRevision {
                expected: base,
                actual: p.current.revision,
            });
        }
        let applied = apply_edit(&p.current, op)?;
        let previous = std::mem::replace(&mut p.current, applied.snapshot);
        p.undo.push(previous.design);
        p.redo.clear();
        p.simulation = None;
        p.ide.set_committed(p.current.clone());
        Ok(Committed {
            snapshot: p.current.clone(),
            outcome: Some(applied.outcome),
        })
    }

    pub fn undo(&mut self) -> Result<Committed, SessionError> {
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

    /// The IDE ground state of the open project.
    pub fn ide(&mut self) -> Result<&mut IdeHost, SessionError> {
        Ok(&mut self.project_mut()?.ide)
    }

    /// Studio typed in the definition editor: `source` becomes the draft
    /// overlay of `mapping` and the compiler's verdict on the resulting
    /// world is returned.  The project, its revision and its history are
    /// untouched; the overlay stays until a commit makes it the committed
    /// definition, the mapping is deleted, or a newer draft replaces it.
    /// Served by the `AnalyzeDefinitionDraft` request (protocol 0.4).
    pub fn draft_verdict(
        &mut self,
        mapping: DeclId,
        source: &str,
    ) -> Result<DraftVerdict, SessionError> {
        let snapshot = self.draft_snapshot(mapping, source)?;
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
        mapping: DeclId,
        source: &str,
    ) -> Result<std::sync::Arc<AnalysisSnapshot>, SessionError> {
        let host = self.ide()?;
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
    pub fn discard_draft(&mut self, mapping: DeclId) -> Result<bool, SessionError> {
        Ok(self.ide()?.clear_definition_draft(mapping))
    }

    /// Completion candidates at a byte offset into the draft.
    pub fn draft_completion(
        &mut self,
        mapping: DeclId,
        source: &str,
        offset: u32,
    ) -> Result<Vec<SemanticCompletion>, SessionError> {
        let snapshot = self.draft_snapshot(mapping, source)?;
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

    /// The concept named at a byte offset into the draft, explained by the
    /// IDE service; `None` when nothing semantic is under the cursor.
    pub fn draft_hover(
        &mut self,
        mapping: DeclId,
        source: &str,
        offset: u32,
    ) -> Result<Option<(TextRange, SemanticHover)>, SessionError> {
        let snapshot = self.draft_snapshot(mapping, source)?;
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
                },
            )
            .unwrap();
        let id = c.outcome.unwrap().created_mapping.unwrap();
        s.save().unwrap();
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
                },
            )
            .unwrap()
            .outcome
            .unwrap()
            .created_mapping
            .unwrap();

        // The draft is judged; the project is not touched.
        let v = s.draft_verdict(id, "Tilt / 90 deg").unwrap();
        assert_eq!(v.status, bdl_compiler::MappingStatus::ClockConsistent);
        assert_eq!(v.stamp.revision, Revision::from_raw(3));
        let p = s.project().unwrap();
        assert_eq!(p.current.revision, Revision::from_raw(3));
        assert!(p.current.design.mappings[&id].is_unresolved());
        assert_eq!(p.ide.overlays().len(), 1);

        // A newer draft supersedes; its verdict carries a newer stamp.
        let v2 = s.draft_verdict(id, "Tilt + 1 s").unwrap();
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
        s.draft_verdict(id, "Tilt / 45 deg").unwrap();
        s.undo().unwrap();
        assert_eq!(s.project().unwrap().ide.overlays().len(), 1);
        assert!(matches!(
            s.draft_verdict(DeclId::from_raw(99), "1"),
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
