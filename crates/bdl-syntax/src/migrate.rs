//! Source migrations: lossless rewrites of one spelling into the preferred
//! one, computed over the tree and applied as byte-range insertions, so
//! that everything outside the edited ranges — comments, blank lines, the
//! author's spacing — is untouched.

use crate::ast::{self, AstNode};
use bdl_diagnostics::Span;

/// One insertion or replacement of a byte range of the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceEdit {
    pub span: Span,
    pub text: String,
}

/// A `mapping` whose signature is written in the legacy output-only
/// shorthand (`mapping f : A` for the canonical `mapping f : () -> A`,
/// docs/spec/textual-syntax.md §4.1): the declaration and the span of its
/// signature, in source order.  Only `mapping` declarations — a bare type
/// is what the shorthand means there and nowhere else.
pub fn legacy_unit_domain_signatures(src: &str) -> Vec<(ast::MappingDecl, Span)> {
    let parse = crate::parser::parse_module(src);
    parse
        .syntax_node()
        .descendants()
        .filter_map(ast::MappingDecl::cast)
        .filter_map(|m| {
            let sig = m.signature()?;
            match sig {
                ast::Type::Function(_) => None,
                _ => Some((m.clone(), sig.span())),
            }
        })
        .collect()
}

/// The edits that make every legacy output-only signature explicit:
/// `mapping f : A` → `mapping f : () -> A`, an insertion of `() -> ` before
/// each such signature.  Nothing else changes; a source with syntax errors
/// yields no edits (the tree's shape is not what the author means yet).
pub fn explicit_unit_domain_edits(src: &str) -> Vec<SourceEdit> {
    if !crate::parser::parse_module(src).errors().is_empty() {
        return Vec::new();
    }
    legacy_unit_domain_signatures(src)
        .into_iter()
        .map(|(_, span)| SourceEdit {
            span: Span::new(span.start, span.start),
            text: "() -> ".to_string(),
        })
        .collect()
}

/// Apply [`explicit_unit_domain_edits`]: the migrated source and how many
/// signatures were rewritten.
pub fn make_unit_domains_explicit(src: &str) -> (String, usize) {
    let edits = explicit_unit_domain_edits(src);
    (apply(src, &edits), edits.len())
}

/// Every drive declaration written in the legacy spelling `drive o = m`
/// (the preferred one is `drive o by m`, docs/spec/textual-syntax.md
/// §14.1): the declaration and the span of its `=`, in source order.
pub fn legacy_drive_decls(src: &str) -> Vec<(ast::DriveDecl, Span)> {
    let parse = crate::parser::parse_module(src);
    parse
        .syntax_node()
        .descendants()
        .filter_map(ast::DriveDecl::cast)
        .filter_map(|d| {
            let eq = d.legacy_eq()?;
            Some((d, crate::syntax::span_of(eq.text_range())))
        })
        .collect()
}

/// The edits that rewrite every legacy `drive o = m` as `drive o by m`:
/// the `=` token becomes `by`, nothing else moves.  A source with syntax
/// errors yields no edits.
pub fn drive_by_edits(src: &str) -> Vec<SourceEdit> {
    if !crate::parser::parse_module(src).errors().is_empty() {
        return Vec::new();
    }
    legacy_drive_decls(src)
        .into_iter()
        .map(|(_, span)| SourceEdit {
            span,
            text: "by".to_string(),
        })
        .collect()
}

/// Apply [`drive_by_edits`]: the migrated source and how many drives were
/// rewritten.
pub fn make_drives_by(src: &str) -> (String, usize) {
    let edits = drive_by_edits(src);
    (apply(src, &edits), edits.len())
}

/// Apply non-overlapping edits, in any order.
pub fn apply(src: &str, edits: &[SourceEdit]) -> String {
    let mut sorted: Vec<&SourceEdit> = edits.iter().collect();
    sorted.sort_by_key(|e| e.span.start);
    let mut out = String::with_capacity(src.len() + edits.len() * 6);
    let mut at = 0usize;
    for e in sorted {
        let (s, end) = (e.span.start as usize, e.span.end as usize);
        out.push_str(&src[at..s]);
        out.push_str(&e.text);
        at = end;
    }
    out.push_str(&src[at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shorthand becomes explicit; nothing else moves — comments, blank
    /// lines, definitions, the explicit form, ports, and a concept's type
    /// (a bare type that is not the shorthand).
    #[test]
    fn the_migration_inserts_the_empty_domain_and_touches_nothing_else() {
        let src = "// the lamp\nconcept Tilt : Angle\n\nmapping tilt : Tilt @main   // a source\ntilt() = 0 deg\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(t) = t / (90 deg)\nmapping level : () -> Brightness\n\ncomponent C {\n  requires lean : Tilt\n  mapping inner : Tilt\n}\n";
        let errs = crate::parser::parse_module(src);
        assert!(errs.errors().is_empty(), "{:?}", errs.errors());
        let (out, n) = make_unit_domains_explicit(src);
        assert_eq!(n, 2);
        assert_eq!(
            out,
            "// the lamp\nconcept Tilt : Angle\n\nmapping tilt : () -> Tilt @main   // a source\ntilt() = 0 deg\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(t) = t / (90 deg)\nmapping level : () -> Brightness\n\ncomponent C {\n  requires lean : Tilt\n  mapping inner : () -> Tilt\n}\n"
        );
        // idempotent
        assert_eq!(make_unit_domains_explicit(&out), (out.clone(), 0));
        // a broken source is left alone
        assert_eq!(make_unit_domains_explicit("mapping f : A ->\n").1, 0);
        // the parenthesised and product forms are explicit already or not the shorthand
        assert_eq!(make_unit_domains_explicit("mapping f : (A) -> B\n").1, 0);
        assert_eq!(make_unit_domains_explicit("mapping f : (A, B) -> C\n").1, 0);
    }

    /// `=` becomes `by`, token for token; comments, spacing, the preferred
    /// form and every other `=` are untouched.
    #[test]
    fn the_drive_migration_replaces_only_the_relation_token() {
        let src = "output light : Brightness @main\nmapping level : () -> Brightness @main\nlevel() = 5\ndrive light   =  level   // the edge\ndrive glow by level\ndrive /* out */ other = level\n";
        let (out, n) = make_drives_by(src);
        assert_eq!(n, 2);
        assert_eq!(
            out,
            "output light : Brightness @main\nmapping level : () -> Brightness @main\nlevel() = 5\ndrive light   by  level   // the edge\ndrive glow by level\ndrive /* out */ other by level\n"
        );
        assert_eq!(make_drives_by(&out), (out.clone(), 0));
        assert_eq!(make_drives_by("drive light =\n").1, 0);
    }
}
