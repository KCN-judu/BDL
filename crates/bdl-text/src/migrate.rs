//! Migration of a legacy JSON project to the unified layout (ADR-0023 §6).
//!
//! A manifest of schema 1 names a legacy project by its `kind`.  Opening
//! it — from any tool — migrates it in place, once:
//!
//! 1. the JSON model is read by the legacy reader it was written with;
//! 2. its textual projection is rendered once into `src/main.bdl`;
//! 3. `.bdl/identities.json` is seeded from the ids the model already
//!    has, so layout, hues and every reference keep their identity, and
//!    the allocators carry over so no id is ever reused;
//! 4. `.bdl/authoring.json` takes the model's groups;
//! 5. the sources are read back and compared with the model — a
//!    difference refuses the migration before anything is written;
//! 6. the JSON design file is renamed `*.migrated` (recoverable, never
//!    read again) and the manifest is rewritten without a kind.
//!
//! Layout is not touched.  There is no path back.

use crate::build::BuildResult;
use crate::names::{identifier_from, is_identifier, unique_identifier};
use crate::print;
use crate::splice::table_of;
use crate::workspace::{
    load_workspace, write_authoring, write_identities, SourceFile, TextError, SOURCE_DIR,
};
use bdl_model::persist::{self, Manifest, ProjectKind};
use bdl_system::BehaviorSystem;
use std::path::{Path, PathBuf};

/// The default source file a migrated project's items are rendered into.
pub const MIGRATED_SOURCE: &str = "src/main.bdl";

/// What a migration did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationReport {
    pub from: ProjectKind,
    /// The source file written (empty when only the manifest changed).
    pub source: Option<String>,
    /// Legacy files renamed out of the way.
    pub renamed: Vec<String>,
    /// Display names the source cannot spell, mapped to identifiers
    /// (`(before, after)`), so the report can say so.
    pub renamed_names: Vec<(String, String)>,
}

/// Migrate `root` if its manifest names a legacy project; `Ok(None)` when
/// it is already unified.
pub fn migrate_legacy(
    root: &Path,
    compiler_version: &str,
) -> Result<Option<MigrationReport>, TextError> {
    let manifest = persist::read_manifest(root)?;
    let Some(kind) = manifest.legacy_kind() else {
        return Ok(None);
    };
    let (mut system, design_file): (BehaviorSystem, Option<PathBuf>) = match kind {
        ProjectKind::Text => {
            // Already the unified layout; only the manifest is old.
            persist::write_manifest(root, &Manifest::unified(&manifest.name, compiler_version))?;
            return Ok(Some(MigrationReport {
                from: kind,
                source: None,
                renamed: Vec::new(),
                renamed_names: Vec::new(),
            }));
        }
        ProjectKind::Flat => {
            let loaded = persist::load_project(root)?;
            (
                BehaviorSystem::from_flat(loaded.snapshot.design),
                Some(root.join(persist::DESIGN_FILE)),
            )
        }
        ProjectKind::System => {
            let loaded = bdl_system::persist::load_system_project(root)?;
            (
                loaded.snapshot.system,
                Some(root.join(bdl_system::persist::SYSTEM_FILE)),
            )
        }
    };
    let src = root.join(SOURCE_DIR);
    if src.is_dir()
        && std::fs::read_dir(&src)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        return Err(TextError::Migration {
            path: root.to_path_buf(),
            kind,
            message: format!(
                "a {kind:?} project that already has files under {SOURCE_DIR}/: which is the design?"
            ),
        });
    }

    // Names the source cannot spell become identifiers, once and
    // deterministically (docs/spec/textual-syntax.md §2.5).
    let renamed_names = identifier_names(&mut system);

    // Render once, then prove the text reads back as the model before
    // touching the disk.
    let mut text = print::render_system(&system);
    if text.trim().is_empty() {
        text = format!("// {}\n", system.base.name);
    }
    let files = vec![SourceFile {
        path: MIGRATED_SOURCE.to_owned(),
        text,
    }];
    let probe: BuildResult = load_workspace(&system.base.name, &files, &Default::default());
    let table = table_of(&system, &probe, &files);
    let build = load_workspace(&system.base.name, &files, &table);
    if let Some(f) = build.faults.iter().find(|f| !f.is_open()) {
        return Err(TextError::Migration {
            path: root.to_path_buf(),
            kind,
            message: format!("the rendered source does not build: {f:?}"),
        });
    }
    if let Some(difference) = first_difference(&system, &build.system) {
        return Err(TextError::Migration {
            path: root.to_path_buf(),
            kind,
            message: format!("the rendered source does not read back as the design: {difference}"),
        });
    }

    persist::write_atomic(&root.join(MIGRATED_SOURCE), files[0].text.as_bytes())?;
    write_identities(root, &build.table)?;
    write_authoring(root, &system)?;
    let mut renamed = Vec::new();
    if let Some(path) = design_file {
        if path.is_file() {
            let to = path.with_extension("json.migrated");
            std::fs::rename(&path, &to).map_err(|source| TextError::Io {
                path: path.clone(),
                source,
            })?;
            renamed.push(
                to.strip_prefix(root)
                    .unwrap_or(&to)
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    persist::write_manifest(
        root,
        &Manifest::unified(&system.base.name, compiler_version),
    )?;
    Ok(Some(MigrationReport {
        from: kind,
        source: Some(MIGRATED_SOURCE.to_owned()),
        renamed,
        renamed_names,
    }))
}

/// Map every name the text cannot spell to an identifier, unique within
/// its namespace; returns what changed.
fn identifier_names(system: &mut BehaviorSystem) -> Vec<(String, String)> {
    let mut changed = Vec::new();
    fn fix(names: Vec<&mut String>, changed: &mut Vec<(String, String)>) {
        let mut taken: Vec<String> = names
            .iter()
            .filter(|n| is_identifier(n))
            .map(|n| (*n).clone())
            .collect();
        for n in names {
            if is_identifier(n) {
                continue;
            }
            let candidate = unique_identifier(&identifier_from(n), &taken);
            taken.push(candidate.clone());
            changed.push((std::mem::replace(n, candidate.clone()), candidate));
        }
    }
    let base = &mut system.base;
    fix(
        base.concepts.values_mut().map(|c| &mut c.name).collect(),
        &mut changed,
    );
    fix(
        base.clocks.values_mut().map(|c| &mut c.name).collect(),
        &mut changed,
    );
    fix(
        base.mappings.values_mut().map(|m| &mut m.name).collect(),
        &mut changed,
    );
    fix(
        base.outputs.values_mut().map(|o| &mut o.name).collect(),
        &mut changed,
    );
    fix(
        base.devices.values_mut().map(|d| &mut d.name).collect(),
        &mut changed,
    );
    fix(
        system
            .components
            .values_mut()
            .map(|c| &mut c.name)
            .collect(),
        &mut changed,
    );
    for c in system.components.values_mut() {
        fix(
            c.body.concepts.values_mut().map(|x| &mut x.name).collect(),
            &mut changed,
        );
        fix(
            c.body.clocks.values_mut().map(|x| &mut x.name).collect(),
            &mut changed,
        );
        fix(
            c.body.mappings.values_mut().map(|x| &mut x.name).collect(),
            &mut changed,
        );
        fix(
            c.body.outputs.values_mut().map(|x| &mut x.name).collect(),
            &mut changed,
        );
        fix(
            c.body.devices.values_mut().map(|x| &mut x.name).collect(),
            &mut changed,
        );
        fix(
            c.interface
                .ports
                .values_mut()
                .map(|p| &mut p.name)
                .collect(),
            &mut changed,
        );
    }
    fix(
        system.instances.values_mut().map(|i| &mut i.name).collect(),
        &mut changed,
    );
    fix(
        system.exports.values_mut().map(|e| &mut e.name).collect(),
        &mut changed,
    );
    changed
}

/// The first semantic difference between the model that was migrated and
/// the one its rendered source builds to, or `None` when they agree.
/// Lexical parameter names are not compared: a JSON relationship had
/// none and the printer spells its inputs by concept name, which resolves
/// to the same formula (ADR-0020).  Groups live in the authoring sidecar
/// and are compared by the caller's tests, not here.
pub fn first_difference(expected: &BehaviorSystem, actual: &BehaviorSystem) -> Option<String> {
    let norm = |s: &BehaviorSystem| {
        let mut s = s.clone();
        for m in s.base.mappings.values_mut() {
            m.parameters.clear();
        }
        for c in s.components.values_mut() {
            for m in c.body.mappings.values_mut() {
                m.parameters.clear();
            }
        }
        s.groups.clear();
        s
    };
    let (a, b) = (norm(expected), norm(actual));
    if a == b {
        return None;
    }
    if a.base.concepts != b.base.concepts {
        return Some("concepts".into());
    }
    if a.base.clocks != b.base.clocks {
        return Some("timing domains".into());
    }
    if a.base.mappings != b.base.mappings {
        for (id, m) in &a.base.mappings {
            match b.base.mappings.get(id) {
                Some(n) if n == m => {}
                _ => return Some(format!("relationship {}", m.name)),
            }
        }
        return Some("relationships".into());
    }
    if a.base.outputs != b.base.outputs {
        return Some("outputs".into());
    }
    if a.base.devices != b.base.devices {
        return Some("devices".into());
    }
    if a.base.ids != b.base.ids {
        return Some("id allocators".into());
    }
    if a.components != b.components {
        return Some("components".into());
    }
    if a.instances != b.instances {
        return Some("instances".into());
    }
    if a.bindings != b.bindings {
        return Some("bindings".into());
    }
    if a.exports != b.exports {
        return Some("exports".into());
    }
    Some("system identities".into())
}

// ---- the explicit unit domain (docs/spec/textual-syntax.md §4.1) ----------------

/// What [`make_unit_domains_explicit`] did, or would do.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct UnitDomainMigration {
    /// `(path, signatures rewritten)` for every file that changes.
    pub files: Vec<(String, usize)>,
}

impl UnitDomainMigration {
    pub fn total(&self) -> usize {
        self.files.iter().map(|(_, n)| n).sum()
    }
}

/// Rewrite every legacy output-only signature `mapping f : A` of the
/// project's sources into the preferred `mapping f : () -> A`, losslessly:
/// one insertion per signature, nothing else touched — comments, blank
/// lines, spacing and definitions stay byte for byte.  Opt-in: nothing
/// runs it but a designer's request.  The rewritten sources are built
/// against the project's identity table before anything is written: a
/// system or a table that differs refuses the migration (no semantic
/// change, no identity moved).  With `dry_run` nothing is written.
pub fn make_unit_domains_explicit(
    root: &Path,
    dry_run: bool,
) -> Result<UnitDomainMigration, TextError> {
    rewrite_sources(
        root,
        dry_run,
        bdl_syntax::migrate::make_unit_domains_explicit,
        "the explicit unit domains",
    )
}

/// Rewrite every legacy `drive o = m` of the project's sources into the
/// preferred `drive o by m` (docs/spec/textual-syntax.md §14.1): the `=`
/// token becomes `by`, nothing else touched.  The same guards as
/// [`make_unit_domains_explicit`]: opt-in, refused if the design or an
/// identity would change, nothing written with `dry_run`.
pub fn make_drives_by(root: &Path, dry_run: bool) -> Result<UnitDomainMigration, TextError> {
    rewrite_sources(
        root,
        dry_run,
        bdl_syntax::migrate::make_drives_by,
        "the `by` drives",
    )
}

/// One spelling migration over every source: `rewrite` gives a file's
/// migrated text and how many sites it touched; the rewritten sources are
/// built against the project's identity table before anything is written,
/// and a system or a table that differs refuses the whole migration.
fn rewrite_sources(
    root: &Path,
    dry_run: bool,
    rewrite: fn(&str) -> (String, usize),
    what: &str,
) -> Result<UnitDomainMigration, TextError> {
    let manifest = persist::read_manifest(root)?;
    let files = crate::workspace::discover_sources(root)?;
    let table = crate::workspace::load_identities(root)?;
    let before = load_workspace(&manifest.name, &files, &table);
    let mut report = UnitDomainMigration::default();
    let mut after_files = Vec::with_capacity(files.len());
    for f in &files {
        let (text, n) = rewrite(&f.text);
        if n > 0 {
            report.files.push((f.path.clone(), n));
        }
        after_files.push(SourceFile {
            path: f.path.clone(),
            text,
        });
    }
    if report.files.is_empty() {
        return Ok(report);
    }
    let after = load_workspace(&manifest.name, &after_files, &table);
    if after.system != before.system || after.table != before.table {
        return Err(TextError::Rewrite {
            path: root.to_path_buf(),
            message: format!("{what} would change the design or an identity; nothing was written"),
        });
    }
    if after.faults.len() != before.faults.len() {
        return Err(TextError::Rewrite {
            path: root.to_path_buf(),
            message: "the rewritten sources report different faults; nothing was written".into(),
        });
    }
    if !dry_run {
        for f in &after_files {
            if report.files.iter().any(|(p, _)| p == &f.path) {
                let path = root.join(&f.path);
                std::fs::write(&path, &f.text).map_err(|source| TextError::Io { path, source })?;
            }
        }
    }
    Ok(report)
}
