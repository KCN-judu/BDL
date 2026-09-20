//! `drive light by brightness` is the spelling the language prefers; the
//! legacy `drive light = brightness` still parses to the same drive edge.
//! What the service does about it (docs/spec/textual-syntax.md §14.1): a
//! hint with the quick fix *Write `by`* on the legacy form, nothing on the
//! preferred one, `by` classified as a keyword only where the grammar
//! makes it one, `by` offered after the output, the formatter keeping
//! both spellings as authored, and the rendered projection writing `by`.

use bdl_ide::tokens::{semantic_tokens, TokenType};
use bdl_ide::*;
use bdl_ide_db::workspace::file_uri;
use bdl_ide_db::TextEdit;
use bdl_text::{IdentityTable, SourceFile};

fn host(text: &str) -> IdeHost {
    IdeHost::text_workspace(
        "lamp",
        vec![SourceFile {
            path: "src/main.bdl".into(),
            text: text.into(),
        }],
        IdentityTable::default(),
    )
}

const PREFERRED: &str = "\
concept Brightness : Scalar
clock main
mapping level : () -> Brightness @main
level() = 5
output light : Brightness @main
drive light by level
";

const LEGACY: &str = "\
concept Brightness : Scalar
clock main
mapping level : () -> Brightness @main
level() = 5
output light : Brightness @main
drive light   =   level   // the edge
";

fn legacy_hints(set: &DiagnosticSet) -> Vec<&SemanticDiagnostic> {
    set.items
        .iter()
        .filter(|d| d.code == "text.legacy_drive")
        .collect()
}

#[test]
fn both_spellings_are_one_drive_edge_and_only_the_legacy_one_is_hinted() {
    let mut a = host(PREFERRED);
    let mut b = host(LEGACY);
    let (sa, sb) = (a.snapshot(), b.snapshot());
    let edge = |s: &AnalysisSnapshot| {
        let design = &s.effective().design;
        let m = design
            .mappings
            .values()
            .find(|m| m.name == "level")
            .unwrap();
        let o = design.outputs.values().find(|o| o.name == "light").unwrap();
        (m.drives, o.id)
    };
    let (da, oa) = edge(&sa);
    let (db, ob) = edge(&sb);
    assert_eq!(da, Some(oa));
    assert_eq!(db, Some(ob));
    // the output analysis — DriveWF, SingleDriver — and the whole Design IR
    // (so lowering, simulation and realization) are the same judgment
    let aa = bdl_compiler::analyze(sa.effective());
    let ab = bdl_compiler::analyze(sb.effective());
    assert_eq!(aa.outputs, ab.outputs);
    assert_eq!(aa.ir, ab.ir);
    assert!(aa.outputs.valid_bindings.len() == 1);

    let doc_a = a.file_document("src/main.bdl").unwrap();
    let doc_b = b.file_document("src/main.bdl").unwrap();
    assert!(legacy_hints(&diagnostics(&sa, DiagnosticScope::Document(doc_a))).is_empty());
    let set = diagnostics(&sb, DiagnosticScope::Document(doc_b));
    let hints = legacy_hints(&set);
    assert_eq!(hints.len(), 1, "{:?}", set.items);
    let h = hints[0];
    assert_eq!(h.severity, SemanticSeverity::Hint);
    assert!(!h.is_error());
    assert_eq!(
        h.message,
        "Prefer `by` for output driving: `drive light by level` says that `level` drives `light`."
    );
    let range = h.primary.source.expect("placed").range;
    assert_eq!(&LEGACY[range.start as usize..range.end as usize], "=");
    let level = sb
        .effective()
        .design
        .mappings
        .values()
        .find(|m| m.name == "level")
        .unwrap()
        .id;
    assert_eq!(h.primary.entity, EntityRef::Mapping(level));
    assert_eq!(h.primary.role, EntityRole::DriveEdge);

    // the quick fix rewrites the relation token only
    let actions = actions_for(&sb, h);
    let fix = actions
        .iter()
        .find(|a| a.title == "Write `by`")
        .expect("quick fix");
    assert!(fix.is_ready());
    let edits: Vec<TextEdit> = fix
        .plan
        .as_ref()
        .unwrap()
        .text_edits(doc_b)
        .into_iter()
        .cloned()
        .collect();
    assert_eq!(edits.len(), 1);
    let fixed = TextEdit::apply_all(LEGACY, &edits).unwrap();
    assert_eq!(
        fixed,
        LEGACY.replace(
            "drive light   =   level   // the edge",
            "drive light   by   level   // the edge"
        )
    );
    b.set_text_document(&file_uri("src/main.bdl"), fixed.clone());
    let sb2 = b.snapshot();
    let doc_b2 = b.file_document("src/main.bdl").unwrap();
    assert!(legacy_hints(&diagnostics(&sb2, DiagnosticScope::Document(doc_b2))).is_empty());

    // the formatter normalises spacing and keeps the spelling as authored
    assert_eq!(
        bdl_syntax::format::format_module(&fixed).as_deref(),
        Some(
            PREFERRED
                .replace("drive light by level", "drive light by level  // the edge")
                .as_str()
        )
    );
    assert_eq!(
        bdl_syntax::format::format_module(LEGACY).as_deref(),
        Some(
            PREFERRED
                .replace("drive light by level", "drive light = level  // the edge")
                .as_str()
        )
    );
    let once = bdl_syntax::format::format_module(LEGACY).unwrap();
    assert_eq!(
        bdl_syntax::format::format_module(&once).as_deref(),
        Some(once.as_str())
    );
    let once = bdl_syntax::format::format_module(PREFERRED).unwrap();
    assert_eq!(
        bdl_syntax::format::format_module(&once).as_deref(),
        Some(once.as_str())
    );

    // what the model prints back into a source is the preferred spelling
    let design = &sb.effective().design;
    let level = design
        .mappings
        .values()
        .find(|m| m.name == "level")
        .unwrap();
    assert_eq!(
        bdl_text::print::drive(design, level).as_deref(),
        Some("drive light by level")
    );
}

#[test]
fn by_is_a_keyword_in_a_drive_and_a_name_anywhere_else() {
    let text = "\
concept Level : Scalar
clock main
mapping by : () -> Level @main
by() = 5
mapping level : () -> Level @main
level() = by + 1
output light : Level @main
drive light by by
";
    let mut h = host(text);
    let doc = h.file_document("src/main.bdl").unwrap();
    let snap = h.snapshot();
    let toks = semantic_tokens(&snap, doc);
    let bys: Vec<TokenType> = toks
        .iter()
        .filter(|t| &text[t.range.start as usize..t.range.end as usize] == "by")
        .map(|t| t.ty)
        .collect();
    // declaration, definition head, use in a formula, the relation word,
    // the driver: only the relation word is a keyword
    assert_eq!(bys.len(), 5, "{bys:?}");
    assert_eq!(bys[3], TokenType::Keyword);
    assert!(
        bys.iter()
            .enumerate()
            .all(|(i, k)| i == 3 || *k != TokenType::Keyword),
        "{bys:?}"
    );
    // and it is one drive edge from `by` to `light`
    let design = &snap.effective().design;
    let by = design.mappings.values().find(|m| m.name == "by").unwrap();
    let light = design.outputs.values().find(|o| o.name == "light").unwrap();
    assert_eq!(by.drives, Some(light.id));
    assert!(diagnostics(&snap, DiagnosticScope::Document(doc))
        .items
        .iter()
        .all(|d| !d.is_error()));
}

#[test]
fn completion_offers_by_after_the_output_and_the_drivers_after_by() {
    let mut h = host(PREFERRED);
    let complete = |h: &mut IdeHost, text: &str, needle: &str| -> Vec<String> {
        h.set_text_document(&file_uri("src/main.bdl"), text.to_owned());
        let doc = h.file_document("src/main.bdl").unwrap();
        let offset = (text.find(needle).unwrap() + needle.len()) as u32;
        let snap = h.snapshot();
        completion(
            &snap,
            &CompletionContext::Document {
                document: doc,
                offset,
            },
        )
        .into_iter()
        .map(|c| c.label)
        .collect()
    };
    let typing = PREFERRED.replace("drive light by level\n", "drive light \n");
    let labels = complete(&mut h, &typing, "drive light ");
    assert_eq!(labels, vec!["by"], "{labels:?}");
    let typing = PREFERRED.replace("drive light by level\n", "drive light by \n");
    let labels = complete(&mut h, &typing, "drive light by ");
    assert!(labels.contains(&"level".to_string()), "{labels:?}");
    assert!(!labels.contains(&"=".to_string()) && !labels.contains(&"by".to_string()));
    let typing = PREFERRED.replace("drive light by level\n", "drive \n");
    let labels = complete(&mut h, &typing, "drive ");
    assert_eq!(labels, vec!["light"], "{labels:?}");
}
