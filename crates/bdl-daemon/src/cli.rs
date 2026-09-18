//! The headless front end: `bdld check | compile | simulate <project>`.
//!
//! Every command opens the project through the session — the one loader
//! Studio, the LSP and this front end share (`docs/spec/project-format.md`)
//! — so a text project is read, reconciled and analysed exactly as an
//! editor reads it, and what checks here checks there.  Output is for a
//! person on a terminal, or JSON with `--json` for a script.

use crate::session::Session;
use bdl_diagnostics::{Code, Diagnostic, Entity, Severity};
use bdl_model::surface::Design;
use bdl_model::DeclId;
use bdl_reactive::Value;
use std::path::Path;

/// Why a command did not succeed; the process exit code says so.
#[derive(Debug)]
pub enum Failure {
    /// The project could not be opened.
    Open(String),
    /// The project opened and has errors.
    Errors(usize),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Open(e) => write!(f, "{e}"),
            Failure::Errors(n) => write!(f, "{n} error(s)"),
        }
    }
}

impl std::error::Error for Failure {}

fn open(root: &Path, version: &str) -> Result<Session, Failure> {
    let mut session = Session::new(version);
    session
        .open(root)
        .map_err(|e| Failure::Open(e.to_string()))?;
    Ok(session)
}

/// Load faults of a text project — what the sources say that the model
/// cannot mean — as diagnostics with a file position.
fn text_faults(session: &Session) -> Vec<(String, Diagnostic)> {
    let Ok(p) = session.project() else {
        return Vec::new();
    };
    let Some(loaded) = p.system.as_ref().and_then(|s| s.text.as_ref()) else {
        return Vec::new();
    };
    let line_of = |file: usize, offset: u32| -> String {
        let text = &loaded.files[file].text;
        let upto = &text[..(offset as usize).min(text.len())];
        let line = upto.matches('\n').count() + 1;
        let col = upto
            .rsplit('\n')
            .next()
            .map(|s| s.chars().count())
            .unwrap_or(0)
            + 1;
        format!("{}:{line}:{col}", loaded.files[file].path)
    };
    loaded
        .build
        .faults
        .iter()
        .map(|f| match f {
            bdl_text::TextFault::Syntax { file, error } => (
                line_of(*file, error.span.start),
                Diagnostic::error(error.code.as_str(), Entity::Project, error.message.clone()),
            ),
            bdl_text::TextFault::Load(l) => (
                line_of(l.file, l.span.start),
                if l.open {
                    Diagnostic::warning(Code(l.code.clone()), Entity::Project, l.message.clone())
                } else {
                    Diagnostic::error(Code(l.code.clone()), Entity::Project, l.message.clone())
                },
            ),
        })
        .collect()
}

fn entity_name(design: &Design, e: &Entity) -> String {
    match e {
        Entity::Project => "project".into(),
        Entity::Concept { id } => design
            .concepts
            .get(id)
            .map(|c| format!("concept {}", c.name))
            .unwrap_or_else(|| format!("concept #{}", id.raw())),
        Entity::Mapping { id } => design
            .mappings
            .get(id)
            .map(|m| format!("mapping {}", m.name))
            .unwrap_or_else(|| format!("mapping #{}", id.raw())),
    }
}

fn print_diagnostics(design: &Design, where_: &[(String, Diagnostic)], json: bool) -> usize {
    let mut errors = 0;
    if json {
        let items: Vec<serde_json::Value> = where_
            .iter()
            .map(|(at, d)| {
                serde_json::json!({
                    "at": at,
                    "code": d.code.as_str(),
                    "severity": format!("{:?}", d.severity).to_lowercase(),
                    "entity": entity_name(design, &d.entity),
                    "message": d.message,
                    "explanation": d.explanation,
                })
            })
            .collect();
        println!("{}", serde_json::json!({ "diagnostics": items }));
    }
    for (at, d) in where_ {
        let sev = match d.severity {
            Severity::Error => {
                errors += 1;
                "error"
            }
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        if !json {
            let place = if at.is_empty() {
                entity_name(design, &d.entity)
            } else {
                at.clone()
            };
            println!("{sev}[{}] {place}: {}", d.code.as_str(), d.message);
            if !d.explanation.is_empty() {
                println!("    {}", d.explanation);
            }
        }
    }
    errors
}

/// `check`: open, analyse, report.  Exit 1 when anything is an error.
pub fn check(root: &Path, version: &str, json: bool) -> Result<(), Failure> {
    let session = open(root, version)?;
    let project = session
        .project()
        .map_err(|e| Failure::Open(e.to_string()))?;
    let analysis = bdl_compiler::analyze(&project.current);
    let mut all = text_faults(&session);
    all.extend(
        analysis
            .diagnostics
            .iter()
            .map(|d| (String::new(), d.clone())),
    );
    let errors = print_diagnostics(&project.current.design, &all, json);
    if !json {
        let d = &project.current.design;
        println!(
            "{}: {} concept(s), {} relationship(s), {} output(s); {}",
            d.name,
            d.concepts.len(),
            d.mappings.len(),
            d.outputs.len(),
            if errors == 0 {
                if analysis.output_complete {
                    "checks, outputs complete".to_owned()
                } else {
                    "checks, outputs not yet complete".to_owned()
                }
            } else {
                format!("{errors} error(s)")
            }
        );
    }
    if errors > 0 {
        Err(Failure::Errors(errors))
    } else {
        Ok(())
    }
}

/// `compile`: generate the Rust crate into `out`.
pub fn compile(root: &Path, version: &str, out: &Path, json: bool) -> Result<(), Failure> {
    let session = open(root, version)?;
    let project = session
        .project()
        .map_err(|e| Failure::Open(e.to_string()))?;
    let faults = text_faults(&session);
    let fault_errors = print_diagnostics(&project.current.design, &faults, json);
    if fault_errors > 0 {
        return Err(Failure::Errors(fault_errors));
    }
    let options = bdl_compiler::CompileOptions {
        require_complete: true,
        codegen: Default::default(),
    };
    let artifact = bdl_compiler::compile(&project.current, &options);
    let all: Vec<(String, Diagnostic)> = artifact
        .diagnostics
        .iter()
        .map(|d| (String::new(), d.clone()))
        .collect();
    let errors = print_diagnostics(&project.current.design, &all, json);
    let Some(generated) = artifact.generated else {
        return Err(Failure::Errors(errors.max(1)));
    };
    for (path, text) in &generated.files {
        let target = out.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Failure::Open(e.to_string()))?;
        }
        std::fs::write(&target, text).map_err(|e| Failure::Open(e.to_string()))?;
    }
    if json {
        println!(
            "{}",
            serde_json::json!({ "package": generated.package, "files": generated.files.keys().collect::<Vec<_>>(), "out": out })
        );
    } else {
        println!(
            "{}: {} file(s) written to {}",
            generated.package,
            generated.files.len(),
            out.display()
        );
    }
    Ok(())
}

/// One `--input name=value` argument: a relationship of the design and a
/// constant for it — `true`/`false` for a Boolean concept, a number in
/// the concept's base unit otherwise.
pub fn parse_input(design: &Design, spec: &str) -> Result<(DeclId, Value), String> {
    let (name, value) = spec
        .split_once('=')
        .ok_or_else(|| format!("`{spec}`: expected name=value"))?;
    let m = design
        .mappings
        .values()
        .find(|m| m.name == name.trim())
        .ok_or_else(|| format!("no relationship named `{}`", name.trim()))?;
    let concept = design
        .concepts
        .get(&m.signature.output)
        .ok_or_else(|| format!("`{}` has no concept", m.name))?;
    let repr = match &concept.representation {
        Some(r) => r,
        None => return Err(format!("concept `{}` has no value form yet", concept.name)),
    };
    let inner = parse_value(repr, value.trim())?;
    Ok((m.id, Value::sem(concept.id, inner)))
}

/// A constant input as written on the command line: `true`, `3`, `0.5`,
/// `[1, 2, 3]`, `(20, 45)`, `none`, `some(1)` — following the value form.
fn parse_value(repr: &bdl_model::surface::Representation, value: &str) -> Result<Value, String> {
    use bdl_model::surface::Representation;
    let value = value.trim();
    Ok(match repr {
        Representation::Boolean => match value {
            "true" => Value::boolean(true),
            "false" => Value::boolean(false),
            other => return Err(format!("`{other}`: expected true or false")),
        },
        Representation::Count => Value::Nat {
            value: value
                .parse()
                .map_err(|_| format!("`{value}`: expected a whole number"))?,
        },
        Representation::Quantity { dim } => Value::q(
            *dim,
            value
                .parse()
                .map_err(|_| format!("`{value}`: expected a number"))?,
        ),
        Representation::Optional { inner } => {
            if value == "none" {
                Value::None
            } else if let Some(x) = value
                .strip_prefix("some(")
                .and_then(|v| v.strip_suffix(')'))
            {
                Value::some(parse_value(inner, x)?)
            } else {
                return Err(format!("`{value}`: expected none or some(…)"));
            }
        }
        Representation::List { element } => {
            let Some(body) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) else {
                return Err(format!("`{value}`: expected a collection [a, b, …]"));
            };
            let mut items = Vec::new();
            for part in split_top_level(body) {
                items.push(parse_value(element, part)?);
            }
            Value::list(items)
        }
        Representation::Pair { first, second } => {
            let Some(body) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) else {
                return Err(format!("`{value}`: expected a grouped value (a, b)"));
            };
            let parts = split_top_level(body);
            let [a, b] = parts.as_slice() else {
                return Err(format!("`{value}`: a grouped value has two parts"));
            };
            Value::pair(parse_value(first, a)?, parse_value(second, b)?)
        }
    })
}

/// Split on commas outside brackets and parentheses; nothing for an
/// empty body.
fn split_top_level(body: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, ch) in body.char_indices() {
        match ch {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(&body[start..i]);
                start = i + ch.len_utf8();
            }
            _ => {}
        }
    }
    if !body[start..].trim().is_empty() {
        out.push(&body[start..]);
    }
    out
}

/// `simulate`: run `ticks` activations with constant inputs and print
/// every relationship's value per tick.
pub fn simulate(
    root: &Path,
    version: &str,
    ticks: u64,
    inputs: &[String],
    json: bool,
) -> Result<(), Failure> {
    let session = open(root, version)?;
    let project = session
        .project()
        .map_err(|e| Failure::Open(e.to_string()))?;
    let design = &project.current.design;
    let faults = text_faults(&session);
    if print_diagnostics(design, &faults, json) > 0 {
        return Err(Failure::Errors(1));
    }
    let analysis = bdl_compiler::analyze(&project.current);
    if !analysis.causality.valid {
        return Err(Failure::Open(
            "the design has relationships that depend on each other in the same instant".into(),
        ));
    }
    let mut trace = bdl_reactive::InputTrace::default();
    for spec in inputs {
        let (decl, value) = parse_input(design, spec).map_err(Failure::Open)?;
        trace.series(decl, std::iter::repeat_n(value, ticks as usize));
    }
    let schedule = bdl_reactive::Schedule::always(&analysis.ir);
    let names: std::collections::BTreeMap<_, _> = analysis
        .ir
        .concepts
        .values()
        .map(|c| (c.id, c.name.clone()))
        .collect();
    let mut sim = bdl_reactive::Simulation::new(analysis.ir, &analysis.causality, schedule, trace)
        .map_err(|e| Failure::Open(e.to_string()))?;
    let name =
        |id: bdl_model::SemanticId| names.get(&id).cloned().unwrap_or_else(|| id.to_string());
    for _ in 0..ticks {
        sim.step().map_err(|e| Failure::Open(e.to_string()))?;
    }
    let decl_name = |d: &DeclId| {
        design
            .mappings
            .get(d)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| format!("#{}", d.raw()))
    };
    if json {
        let ticks: Vec<serde_json::Value> = sim
            .trace()
            .ticks
            .iter()
            .map(|t| {
                serde_json::json!({
                    "tick": t.tick,
                    "values": t.values.iter().map(|(d, v)| (decl_name(d), v.render(&name))).collect::<std::collections::BTreeMap<_, _>>(),
                })
            })
            .collect();
        println!("{}", serde_json::json!({ "ticks": ticks }));
    } else {
        for t in &sim.trace().ticks {
            let cells: Vec<String> = t
                .values
                .iter()
                .map(|(d, v)| format!("{} = {}", decl_name(d), v.render(&name)))
                .collect();
            println!("tick {}: {}", t.tick, cells.join("  "));
        }
    }
    Ok(())
}
