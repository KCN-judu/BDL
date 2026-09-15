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

use bdl_model::edit::{apply_edit, EditError, EditOp, EditOutcome};
use bdl_model::layout::Layout;
use bdl_model::persist::{self, PersistError};
use bdl_model::surface::{Design, ProjectSnapshot};
use bdl_model::Revision;
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
}

pub struct OpenProject {
    pub root: PathBuf,
    pub current: ProjectSnapshot,
    pub layout: Layout,
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
            current: snapshot,
            layout,
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
        Ok(Committed {
            snapshot: p.current.clone(),
            outcome: None,
        })
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
