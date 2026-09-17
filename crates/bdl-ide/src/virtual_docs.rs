//! Read-only virtual documents (docs/IDE_SERVICE_ARCHITECTURE.md
//! "Virtual documents"): the explanation of an entity, the kernel Core
//! of the design, the generated Rust.  Each is a rendering of the
//! semantic model or of the compiler's output; none is ever a source of
//! truth, and none is edited.

use crate::explain::{explain, Explanation};
use bdl_check::pretty;
use bdl_ide_db::{AnalysisSnapshot, DocumentUri, EntityRef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VirtualKind {
    Explain,
    Core,
    Rust,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualDocument {
    pub uri: DocumentUri,
    /// A language id for the client's highlighter: `markdown`,
    /// `bdl-core`, `rust`.
    pub language: String,
    pub text: String,
}

/// The virtual document of `kind`; `entity` narrows an explanation to
/// one thing (the whole project otherwise).
pub fn virtual_document(
    snapshot: &AnalysisSnapshot,
    kind: VirtualKind,
    entity: Option<EntityRef>,
) -> VirtualDocument {
    let name = snapshot.effective().design.name.clone();
    match kind {
        VirtualKind::Explain => {
            let (uri, text) = match entity {
                Some(e) => (
                    format!(
                        "bdl-explain://{name}/{}",
                        snapshot.name_of(e).unwrap_or("entity")
                    ),
                    explain(snapshot, e)
                        .map(|x| x.markdown())
                        .unwrap_or_else(|| "Nothing here to explain.\n".into()),
                ),
                None => (format!("bdl-explain://{name}"), explain_project(snapshot)),
            };
            VirtualDocument {
                uri: DocumentUri::new(uri),
                language: "markdown".into(),
                text,
            }
        }
        VirtualKind::Core => VirtualDocument {
            uri: DocumentUri::new(format!("bdl-core://{name}")),
            language: "bdl-core".into(),
            text: core_document(snapshot),
        },
        VirtualKind::Rust => VirtualDocument {
            uri: DocumentUri::new(format!("bdl-generated://{name}/src/lib.rs")),
            language: "rust".into(),
            text: rust_document(snapshot),
        },
    }
}

impl Explanation {
    /// The explanation as Markdown: title, status, one section per
    /// heading, the diagnostics last.
    pub fn markdown(&self) -> String {
        let mut md = format!("# {}\n\n*{}*\n", self.title, self.status.label());
        for s in &self.sections {
            md.push_str(&format!("\n## {}\n\n", s.heading));
            for (k, v) in &s.lines {
                md.push_str(&format!("- **{k}**: {v}\n"));
            }
        }
        if !self.diagnostics.is_empty() {
            md.push_str("\n## Diagnostics\n\n");
            for d in &self.diagnostics {
                md.push_str(&format!("- `{}` {}\n", d.code, d.message));
            }
        }
        md
    }
}

fn explain_project(snapshot: &AnalysisSnapshot) -> String {
    let design = &snapshot.effective().design;
    let mut md = format!("# {}\n", design.name);
    let entities = design
        .concepts
        .keys()
        .map(|c| EntityRef::Concept(*c))
        .chain(design.mappings.keys().map(|m| EntityRef::Mapping(*m)))
        .chain(design.outputs.keys().map(|o| EntityRef::Output(*o)));
    for e in entities {
        if let Some(x) = explain(snapshot, e) {
            md.push('\n');
            // Demote the entity's own headings under the project's.
            for line in x.markdown().lines() {
                if let Some(rest) = line.strip_prefix('#') {
                    md.push_str("##");
                    md.push_str(rest);
                } else {
                    md.push_str(line);
                }
                md.push('\n');
            }
        }
    }
    md
}

/// The kernel Core of the design: every concept with its kernel type,
/// every declaration with its interface, domain, drive edge and the
/// realization the elaborator produced.
pub fn core_document(snapshot: &AnalysisSnapshot) -> String {
    let design = &snapshot.effective().design;
    let analysis = snapshot.analysis();
    let ir = &analysis.ir;
    let mut out = format!(
        "// Core of `{}` — kernel terms as the checker sees them. Read-only; generated from the model.\n",
        design.name
    );
    if !ir.concepts.is_empty() {
        out.push('\n');
    }
    for c in ir.concepts.values() {
        match &c.representation {
            Some(t) => out.push_str(&format!("sem {} : {}\n", c.name, pretty::kernel(t))),
            None => out.push_str(&format!("sem {} : ?  // open\n", c.name)),
        }
    }
    for k in design.clocks.values() {
        out.push_str(&format!("clock {}\n", k.name));
    }
    for d in ir.decls.values() {
        out.push('\n');
        out.push_str(&format!(
            "decl {} : {}\n",
            d.name,
            pretty::kernel(&d.interface.expected_type)
        ));
        if let Some(c) = ir.clocks.get(&d.id) {
            let name = ir
                .clock_names
                .get(c)
                .cloned()
                .unwrap_or_else(|| format!("clock#{}", c.raw()));
            out.push_str(&format!("  @ {name}\n"));
        }
        match analysis
            .mappings
            .get(&d.id)
            .and_then(|m| m.realization.as_ref())
            .or(d.realization.as_ref())
        {
            Some(e) => out.push_str(&format!("  = {}\n", pretty::expr(e))),
            None => out.push_str("  = ?  // unresolved\n"),
        }
        if let Some(o) = ir.drives.get(&d.id) {
            let name = ir
                .output_names
                .get(o)
                .cloned()
                .unwrap_or_else(|| format!("output#{}", o.raw()));
            out.push_str(&format!("  drives {name}\n"));
        }
    }
    for o in ir.outputs.values() {
        out.push_str(&format!(
            "\noutput {} : {}\n  @ {}\n",
            o.name,
            pretty::kernel(&o.accepts),
            ir.clock_names
                .get(&o.clock)
                .cloned()
                .unwrap_or_else(|| format!("clock#{}", o.clock.raw()))
        ));
    }
    out
}

/// The generated semantic core (`src/lib.rs`) of the design, or the
/// reasons it cannot be generated yet, as Rust comments.
pub fn rust_document(snapshot: &AnalysisSnapshot) -> String {
    let options = bdl_compiler::CompileOptions {
        require_complete: false,
        codegen: Default::default(),
    };
    let artifact = bdl_compiler::compile(snapshot.effective(), &options);
    match &artifact.generated {
        Some(g) => g.core_source().to_owned(),
        None => {
            let mut out = String::from("// No code is generated for this design yet.\n");
            for d in &artifact.diagnostics {
                out.push_str(&format!("// {}: {}\n", d.code.as_str(), d.message));
                if !d.explanation.is_empty() {
                    out.push_str(&format!("//   {}\n", d.explanation));
                }
            }
            out
        }
    }
}
