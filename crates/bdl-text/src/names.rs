//! Names are identifiers (docs/spec/textual-syntax.md §2.5; ADR-0023).
//!
//! The text is the semantic source of every project, so a name the text
//! cannot spell is not a name the model may hold.  A create or rename from
//! any surface is refused with [`why_not_identifier`]; a legacy display
//! name is mapped once, deterministically, by [`identifier_from`] when a
//! JSON project migrates.

use bdl_model::edit::EditOp;
use bdl_syntax::SyntaxKind;
use bdl_system::SystemEditOp;

/// `[A-Za-z_][A-Za-z0-9_]*`, not the lone `_`, not a keyword.
pub fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    name != "_" && SyntaxKind::from_keyword(name).is_none()
}

/// Why `name` is not a name the source can spell, in product language;
/// `None` when it is one.
pub fn why_not_identifier(name: &str) -> Option<String> {
    if is_identifier(name) {
        return None;
    }
    Some(if name.trim().is_empty() {
        "A name is needed.".into()
    } else if name == "_" {
        "`_` means \"no name\" in the source; choose a word.".into()
    } else if SyntaxKind::from_keyword(name).is_some() {
        format!("`{name}` is a word of the language; choose another name.")
    } else if name.chars().any(char::is_whitespace) {
        format!(
            "A name is one word, without spaces: `{}`.",
            identifier_from(name)
        )
    } else {
        format!(
            "A name is letters, digits and underscores, starting with a letter: `{}`.",
            identifier_from(name)
        )
    })
}

/// The identifier a free display name becomes: every run of characters
/// outside `[A-Za-z0-9_]` is one underscore, a leading digit gets an
/// underscore before it, and a keyword or empty result gets a trailing
/// underscore.  Deterministic, so the same legacy name always migrates to
/// the same source key.
pub fn identifier_from(display: &str) -> String {
    let mut out = String::with_capacity(display.len());
    let mut pending_gap = false;
    for c in display.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if pending_gap && !out.is_empty() {
                out.push('_');
            }
            pending_gap = false;
            out.push(c);
        } else {
            pending_gap = true;
        }
    }
    if out.is_empty() {
        out.push_str("unnamed");
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if !is_identifier(&out) {
        out.push('_');
    }
    out
}

/// A name that is unique among `taken` — `name`, else `name_2`, `name_3`, …
pub fn unique_identifier(name: &str, taken: &[String]) -> String {
    if !taken.iter().any(|t| t == name) {
        return name.to_owned();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{name}_{n}");
        if !taken.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// The names a flat edit introduces, if any.
pub fn names_in_edit(op: &EditOp) -> Vec<&str> {
    match op {
        EditOp::CreateConcept { name, .. }
        | EditOp::RenameConcept { name, .. }
        | EditOp::CreateMapping { name, .. }
        | EditOp::RenameMapping { name, .. }
        | EditOp::CreateClockDomain { name, .. }
        | EditOp::RenameClockDomain { name, .. }
        | EditOp::CreateOutput { name, .. }
        | EditOp::RenameOutput { name, .. }
        | EditOp::CreateDevice { name, .. }
        | EditOp::RenameDevice { name, .. } => vec![name.as_str()],
        _ => Vec::new(),
    }
}

/// The names a system edit introduces, if any.
pub fn names_in_system_edit(op: &SystemEditOp) -> Vec<&str> {
    match op {
        SystemEditOp::Base { op } | SystemEditOp::EditComponentBody { op, .. } => names_in_edit(op),
        SystemEditOp::CreateComponent { name, .. }
        | SystemEditOp::RenameComponent { name, .. }
        | SystemEditOp::DuplicateComponent { name, .. }
        | SystemEditOp::DeclarePort { name, .. }
        | SystemEditOp::RenamePort { name, .. }
        | SystemEditOp::CreateInstance { name, .. }
        | SystemEditOp::RenameInstance { name, .. } => vec![name.as_str()],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_and_their_reasons() {
        assert!(is_identifier("Tilt"));
        assert!(is_identifier("dim_by_tilt2"));
        assert!(is_identifier("_private"));
        assert!(!is_identifier("_"));
        assert!(!is_identifier("Light Output"));
        assert!(!is_identifier("2fast"));
        assert!(!is_identifier("mapping"));
        assert!(!is_identifier(""));
        assert!(why_not_identifier("Light Output")
            .unwrap()
            .contains("`Light_Output`"));
        assert!(why_not_identifier("mapping")
            .unwrap()
            .contains("word of the language"));
        assert_eq!(why_not_identifier("Tilt"), None);
    }

    #[test]
    fn display_names_map_deterministically() {
        assert_eq!(identifier_from("Light Output"), "Light_Output");
        assert_eq!(identifier_from("PWM light"), "PWM_light");
        assert_eq!(identifier_from("  Ambient — light  "), "Ambient_light");
        assert_eq!(identifier_from("2nd tilt"), "_2nd_tilt");
        assert_eq!(identifier_from("mapping"), "mapping_");
        assert_eq!(identifier_from("···"), "unnamed");
        assert_eq!(
            unique_identifier("Light", &["Light".into(), "Light_2".into()]),
            "Light_3"
        );
    }
}
