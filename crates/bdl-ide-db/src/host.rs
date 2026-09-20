//! The IDE host: the one mutable place, from which immutable snapshots
//! are taken.
//!
//! ```text
//!      committed ProjectSnapshot ─┐
//!      OverlaySet                 ├─ IdeHost ── snapshot() ──▶ Arc<AnalysisSnapshot>
//!      documents (uri ↔ id)       │                                 │
//!      in-flight requests         ┘                          queries run here
//! ```
//!
//! Ownership and threads: the host is `Send` and is meant to be owned by
//! one coordinator (bdld's single request loop) or shared behind a mutex
//! (the LSP adapter).  Snapshots are `Arc`, `Send + Sync` and immutable, so
//! any number of queries on any threads may hold one while the host moves
//! on.  A query never sees half-updated state because it never sees the
//! host at all — only a snapshot taken between two host mutations.
//!
//! Every mutation moves the stamp ([`IdeHost::stamp`]) and cancels the
//! in-flight requests it makes obsolete.  The snapshot for the current
//! stamp is cached, so repeated queries against an unchanged world do not
//! re-run the compiler.

use crate::cancel::{CancelScope, CancellationToken, Cancelled, RequestId, RequestTracker};
use crate::overlay::{Overlay, OverlayGeneration, OverlayId, OverlayKey, OverlaySet};
use crate::snapshot::AnalysisSnapshot;
use crate::stamp::SnapshotStamp;
use crate::text::{DocumentId, DocumentUri};
use crate::workspace::{file_uri, TextGround};
use bdl_compiler::ProjectAnalysis;
use bdl_model::surface::{Definition, Design, ProjectSnapshot};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Host configuration.  Small on purpose; grows with real needs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IdeConfig {
    /// Drop a definition draft as soon as the committed definition equals
    /// it (the commit succeeded).  Off only for tests that want to observe
    /// the redundant overlay.
    pub clear_redundant_drafts: bool,
}

impl IdeConfig {
    pub fn standard() -> IdeConfig {
        IdeConfig {
            clear_redundant_drafts: true,
        }
    }
}

/// What [`IdeHost::set_committed`] did to the overlay set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitEffect {
    /// Drafts dropped because their mapping is gone or their text is now
    /// what is committed.
    pub dropped: Vec<OverlayKey>,
    /// Requests cancelled by the change.
    pub cancelled_requests: usize,
}

/// The mutable ground state.
#[derive(Debug)]
pub struct IdeHost {
    committed: Arc<ProjectSnapshot>,
    committed_analysis: Option<Arc<ProjectAnalysis>>,
    /// Present for a text workspace: the sources on disk are the ground,
    /// and `committed` is their flattening (ADR-0020).
    text: Option<TextGround>,
    overlays: OverlaySet,
    uris: BTreeMap<DocumentId, DocumentUri>,
    ids: BTreeMap<DocumentUri, DocumentId>,
    next_document: u32,
    config: IdeConfig,
    requests: RequestTracker,
    cache: Option<Arc<AnalysisSnapshot>>,
    /// For a host over a component body: the body declarations that back a
    /// port, with the port's kind.  A port-backed declaration presents its
    /// port role, never the Source role (ADR-0032).
    port_backed: BTreeMap<bdl_model::DeclId, bdl_system::PortKind>,
}

impl IdeHost {
    pub fn new(committed: ProjectSnapshot) -> IdeHost {
        IdeHost::with_config(committed, IdeConfig::standard())
    }

    pub fn with_config(committed: ProjectSnapshot, config: IdeConfig) -> IdeHost {
        IdeHost {
            committed: Arc::new(committed),
            committed_analysis: None,
            text: None,
            overlays: OverlaySet::default(),
            uris: BTreeMap::new(),
            ids: BTreeMap::new(),
            next_document: 0,
            config,
            requests: RequestTracker::default(),
            cache: None,
            port_backed: BTreeMap::new(),
        }
    }

    /// The port-backed declarations of the body this host is over (a
    /// system's own host has none).  Changes the snapshot's roles only.
    pub fn set_port_backed(&mut self, ports: BTreeMap<bdl_model::DeclId, bdl_system::PortKind>) {
        if self.port_backed != ports {
            self.port_backed = ports;
            self.cache = None;
        }
    }

    /// A host over an empty design: text documents alone will populate it.
    pub fn empty(name: &str) -> IdeHost {
        IdeHost::new(ProjectSnapshot::new(Design::empty(name)))
    }

    /// A host over a text workspace: every source file becomes a document
    /// (`bdl-file:<path>`) whether or not a buffer is open for it, so
    /// navigation and rename reach unopened files too.
    pub fn text_workspace(
        name: &str,
        files: Vec<bdl_text::SourceFile>,
        table: bdl_text::IdentityTable,
    ) -> IdeHost {
        let mut host = IdeHost::empty(name);
        host.set_text_ground(files, table);
        host
    }

    /// Whether this host is over a text workspace.
    pub fn is_text_workspace(&self) -> bool {
        self.text.is_some()
    }

    /// The text ground as last loaded from disk.
    pub fn text_ground(&self) -> Option<&TextGround> {
        self.text.as_ref()
    }

    /// (Re)load the text ground from disk: the files, the identity table.
    /// Open buffers survive as overlays over the new files; the revision
    /// moves so results stamped before the reload are stale.
    pub fn set_text_ground(
        &mut self,
        files: Vec<bdl_text::SourceFile>,
        table: bdl_text::IdentityTable,
    ) {
        let revision = match &self.text {
            Some(g) => g.revision.next(),
            None => bdl_model::Revision::INITIAL,
        };
        let name = self.committed.design.name.clone();
        for f in &files {
            self.document_id(&file_uri(&f.path));
        }
        self.text = Some(TextGround {
            name,
            files,
            table,
            revision,
        });
        self.requests.project_changed();
        self.committed_analysis = None;
        self.cache = None;
    }

    /// The document of a workspace file by its relative path.
    pub fn file_document(&self, path: &str) -> Option<DocumentId> {
        self.known_document(&file_uri(path))
    }

    pub fn config(&self) -> &IdeConfig {
        &self.config
    }

    // ---- committed state -------------------------------------------------

    pub fn committed(&self) -> &ProjectSnapshot {
        &self.committed
    }

    /// The analysis of the committed project alone (no overlays), cached.
    pub fn committed_analysis(&mut self) -> Arc<ProjectAnalysis> {
        if let Some(a) = &self.committed_analysis {
            return a.clone();
        }
        let a = Arc::new(bdl_compiler::analyze(&self.committed));
        self.committed_analysis = Some(a.clone());
        a
    }

    /// A new committed revision arrived (an edit, undo, redo, reopen).
    /// Everything in flight is obsolete; drafts that the commit made
    /// redundant or orphaned are dropped.
    pub fn set_committed(&mut self, snapshot: ProjectSnapshot) -> CommitEffect {
        let cancelled_requests = self.requests.project_changed();
        self.committed = Arc::new(snapshot);
        self.committed_analysis = None;
        self.cache = None;
        let mut dropped = Vec::new();
        let design = &self.committed.design;
        let clear_redundant = self.config.clear_redundant_drafts;
        let mut to_drop = Vec::new();
        for e in self.overlays.iter() {
            if let Overlay::MappingDefinitionDraft { mapping, source } = &e.overlay {
                match design.mappings.get(mapping) {
                    None => to_drop.push(e.overlay.key()),
                    Some(m) if clear_redundant => {
                        let committed = m.definition.as_ref().and_then(Definition::formula_source);
                        if committed == Some(source.as_str()) {
                            to_drop.push(e.overlay.key());
                        }
                    }
                    Some(_) => {}
                }
            }
        }
        for key in to_drop {
            if self.overlays.remove(key).is_some() {
                dropped.push(key);
            }
        }
        CommitEffect {
            dropped,
            cancelled_requests,
        }
    }

    // ---- overlays ----------------------------------------------------------

    pub fn overlays(&self) -> &OverlaySet {
        &self.overlays
    }

    /// The current world: committed revision + overlay generation.
    pub fn stamp(&self) -> SnapshotStamp {
        let revision = match &self.text {
            Some(g) => g.revision,
            None => self.committed.revision,
        };
        SnapshotStamp::new(revision, self.overlays.generation())
    }

    /// Insert or replace an overlay.  Cancels requests scoped to its key.
    pub fn set_overlay(&mut self, overlay: Overlay) -> (OverlayId, OverlayGeneration) {
        let key = overlay.key();
        self.requests.overlay_changed(key);
        self.cache = None;
        self.overlays.upsert(overlay)
    }

    /// Remove an overlay.  Cancels requests scoped to its key.
    pub fn remove_overlay(&mut self, key: OverlayKey) -> bool {
        let removed = self.overlays.remove(key).is_some();
        if removed {
            self.requests.overlay_changed(key);
            self.cache = None;
        }
        removed
    }

    /// The Studio formula editor's overlay for one mapping.
    pub fn set_definition_draft(
        &mut self,
        mapping: bdl_model::DeclId,
        source: impl Into<String>,
    ) -> (OverlayId, OverlayGeneration) {
        self.set_overlay(Overlay::MappingDefinitionDraft {
            mapping,
            source: source.into(),
        })
    }

    pub fn clear_definition_draft(&mut self, mapping: bdl_model::DeclId) -> bool {
        self.remove_overlay(OverlayKey::MappingDefinition { mapping })
    }

    // ---- documents ---------------------------------------------------------

    /// The id of a document, allocating one on first sight.  Ids are never
    /// reused within a host, so a closed-and-reopened document is a new
    /// document.
    pub fn document_id(&mut self, uri: &DocumentUri) -> DocumentId {
        if let Some(id) = self.ids.get(uri) {
            return *id;
        }
        let id = DocumentId(self.next_document);
        self.next_document += 1;
        self.ids.insert(uri.clone(), id);
        self.uris.insert(id, uri.clone());
        id
    }

    pub fn known_document(&self, uri: &DocumentUri) -> Option<DocumentId> {
        self.ids.get(uri).copied()
    }

    pub fn document_uri(&self, id: DocumentId) -> Option<&DocumentUri> {
        self.uris.get(&id)
    }

    /// An editor buffer's current text (didOpen / didChange).
    pub fn set_text_document(
        &mut self,
        uri: &DocumentUri,
        source: impl Into<String>,
    ) -> (DocumentId, OverlayGeneration) {
        let id = self.document_id(uri);
        let (_, g) = self.set_overlay(Overlay::TextDocument {
            document: id,
            source: source.into(),
        });
        (id, g)
    }

    /// The buffer is gone (didClose): the document reverts to whatever the
    /// committed model says.  The id is retired.
    pub fn close_text_document(&mut self, uri: &DocumentUri) -> bool {
        if crate::workspace::file_path(uri).is_some() && self.text.is_some() {
            // A workspace file stays a document; only its buffer goes.
            let Some(id) = self.ids.get(uri).copied() else {
                return false;
            };
            return self.remove_overlay(OverlayKey::TextDocument { document: id });
        }
        let Some(id) = self.ids.remove(uri) else {
            return false;
        };
        self.uris.remove(&id);
        self.remove_overlay(OverlayKey::TextDocument { document: id });
        true
    }

    // ---- requests and cancellation ------------------------------------------

    pub fn begin_request(&mut self, scope: CancelScope) -> (RequestId, CancellationToken) {
        self.requests.begin(scope)
    }

    pub fn end_request(&mut self, id: RequestId) {
        self.requests.end(id);
    }

    pub fn cancel_request(&mut self, id: RequestId) -> bool {
        self.requests.cancel(id)
    }

    pub fn live_requests(&self) -> usize {
        self.requests.live_count()
    }

    // ---- snapshots -----------------------------------------------------------

    /// The snapshot of the current world, built on demand and cached until
    /// the world changes.
    pub fn snapshot(&mut self) -> Arc<AnalysisSnapshot> {
        match self.snapshot_cancellable(&CancellationToken::never()) {
            Ok(s) => s,
            Err(Cancelled) => unreachable!("a never-cancelled token cannot cancel"),
        }
    }

    /// As [`IdeHost::snapshot`], polling `token` between phases.
    pub fn snapshot_cancellable(
        &mut self,
        token: &CancellationToken,
    ) -> Result<Arc<AnalysisSnapshot>, Cancelled> {
        let stamp = self.stamp();
        if let Some(s) = &self.cache {
            if s.stamp() == stamp {
                return Ok(s.clone());
            }
        }
        let s = match &self.text {
            Some(ground) => {
                AnalysisSnapshot::compose_text(ground, &self.overlays, &self.uris, token)?
            }
            None => AnalysisSnapshot::compose(
                self.committed.clone(),
                &self.overlays,
                &self.uris,
                token,
            )?,
        };
        let s = Arc::new(s.with_port_backed(self.port_backed.clone()));
        self.cache = Some(s.clone());
        Ok(s)
    }

    /// The committed world alone, with no overlay applied: what a saved
    /// formula's render reads while the designer may be drafting another.
    /// Composed on demand; the text ground, when one exists, is the
    /// committed text.
    pub fn committed_snapshot(&mut self) -> Arc<AnalysisSnapshot> {
        if self.overlays.is_empty() {
            return self.snapshot();
        }
        let none = OverlaySet::default();
        let token = CancellationToken::never();
        let s = match &self.text {
            Some(ground) => AnalysisSnapshot::compose_text(ground, &none, &self.uris, &token),
            None => AnalysisSnapshot::compose(self.committed.clone(), &none, &self.uris, &token),
        };
        match s {
            Ok(s) => Arc::new(s.with_port_backed(self.port_backed.clone())),
            Err(Cancelled) => unreachable!("a never-cancelled token cannot cancel"),
        }
    }

    /// Whether a result stamped `stamp` still describes the current world.
    pub fn is_current(&self, stamp: SnapshotStamp) -> bool {
        stamp == self.stamp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cancel::CancelScope;
    use bdl_model::edit::{apply_edit, EditOp};
    use bdl_model::surface::Signature;
    use bdl_model::{DeclId, Dim, Representation};

    fn lamp() -> (ProjectSnapshot, DeclId) {
        let mut s = ProjectSnapshot::new(Design::empty("lamp"));
        let mut ids = Vec::new();
        for (name, rep) in [
            ("Tilt", Representation::Quantity { dim: Dim::ANGLE }),
            ("Brightness", Representation::Quantity { dim: Dim::ZERO }),
        ] {
            let a = apply_edit(
                &s,
                &EditOp::CreateConcept {
                    name: name.into(),
                    description: String::new(),
                    representation: Some(rep),
                },
            )
            .unwrap();
            ids.push(a.outcome.created_concept.unwrap());
            s = a.snapshot;
        }
        let a = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![ids[0]],
                    output: ids[1],
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        (a.snapshot, a.outcome.created_mapping.unwrap())
    }

    #[test]
    fn snapshot_is_cached_per_stamp_and_rebuilt_on_change() {
        let (s, id) = lamp();
        let mut host = IdeHost::new(s);
        let a = host.snapshot();
        let b = host.snapshot();
        assert!(Arc::ptr_eq(&a, &b));
        host.set_definition_draft(id, "Tilt / 90 deg");
        let c = host.snapshot();
        assert!(!Arc::ptr_eq(&a, &c));
        assert!(c.stamp() > a.stamp());
        assert!(c.is_drafted(id));
        assert!(!a.is_drafted(id));
        // The old snapshot is untouched by the change.
        assert!(a.effective().design.mappings[&id].is_unresolved());
        assert!(!c.effective().design.mappings[&id].is_unresolved());
        assert!(c.committed().design.mappings[&id].is_unresolved());
    }

    #[test]
    fn commit_drops_redundant_and_orphaned_drafts_and_cancels_requests() {
        let (s, id) = lamp();
        let mut host = IdeHost::new(s.clone());
        host.set_definition_draft(id, "Tilt / 90 deg");
        let (_, token) = host.begin_request(CancelScope::Overlay(OverlayKey::MappingDefinition {
            mapping: id,
        }));
        let committed = apply_edit(
            &s,
            &EditOp::AttachDefinition {
                id,
                definition: Definition::Formula {
                    source: "Tilt / 90 deg".into(),
                },
            },
        )
        .unwrap()
        .snapshot;
        let effect = host.set_committed(committed.clone());
        assert!(token.is_cancelled());
        assert_eq!(effect.cancelled_requests, 1);
        assert_eq!(
            effect.dropped,
            vec![OverlayKey::MappingDefinition { mapping: id }]
        );
        assert!(host.overlays().is_empty());

        // A draft that differs from the commit survives it.
        host.set_definition_draft(id, "Tilt / 45 deg");
        let effect = host.set_committed(committed);
        assert!(effect.dropped.is_empty());
        assert_eq!(host.overlays().len(), 1);

        // A draft for a deleted mapping does not.
        let without = apply_edit(host.committed(), &EditOp::DeleteMapping { id })
            .unwrap()
            .snapshot;
        let effect = host.set_committed(without);
        assert_eq!(effect.dropped.len(), 1);
    }

    #[test]
    fn text_documents_get_stable_ids_and_close_reverts() {
        let (s, _) = lamp();
        let mut host = IdeHost::new(s);
        let uri = DocumentUri::new("file:///lamp.bdl");
        let (id, g1) = host.set_text_document(&uri, "concept Extra : Scalar\n");
        let (id2, g2) = host.set_text_document(&uri, "concept Extra : Angle\n");
        assert_eq!(id, id2);
        assert!(g2 > g1);
        let snap = host.snapshot();
        assert_eq!(snap.effective().design.concepts.len(), 3);
        assert_eq!(snap.committed().design.concepts.len(), 2);
        assert_eq!(snap.document_by_uri(&uri), Some(id));
        assert!(host.close_text_document(&uri));
        assert!(!host.close_text_document(&uri));
        let snap = host.snapshot();
        assert_eq!(snap.effective().design.concepts.len(), 2);
        assert_ne!(host.document_id(&uri), id, "a reopened document is new");
    }

    #[test]
    fn draft_for_unknown_mapping_is_a_fault_not_a_panic() {
        let (s, _) = lamp();
        let mut host = IdeHost::new(s);
        host.set_definition_draft(DeclId::from_raw(999), "1");
        let snap = host.snapshot();
        assert_eq!(snap.overlays().len(), 1);
        assert!(snap.overlays()[0].fault.is_some());
        assert!(!snap.is_drafted(DeclId::from_raw(999)));
    }
}
