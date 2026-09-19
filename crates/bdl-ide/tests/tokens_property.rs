//! Property tests on the lexical layer: whatever the text, the token
//! stream is well formed (sorted, disjoint, non-empty, on character
//! boundaries), every keyword/number/comment the lexer sees is
//! classified, and the LSP encoding round-trips for every position
//! encoding.  This is the invariant a client leans on when it keeps the
//! last tokens through edits (`docs/architecture/syntax-highlighting.md`).

use bdl_ide::tokens::encode::{byte_range, decode_data, encode_data, PositionEncoding};
use bdl_ide::{lexical_tokens, SemanticToken, TokenType};
use bdl_syntax::{parse_formula, parse_module, SyntaxKind};
use proptest::prelude::*;

fn well_formed(text: &str, tokens: &[SemanticToken]) -> Result<(), TestCaseError> {
    for t in tokens {
        prop_assert!(t.range.start < t.range.end, "empty token {t:?}");
        prop_assert!(t.range.end as usize <= text.len(), "out of bounds {t:?}");
        prop_assert!(text.is_char_boundary(t.range.start as usize), "{t:?}");
        prop_assert!(text.is_char_boundary(t.range.end as usize), "{t:?}");
    }
    for w in tokens.windows(2) {
        prop_assert!(
            w[0].range.end <= w[1].range.start,
            "overlap {:?} {:?}",
            w[0],
            w[1]
        );
    }
    Ok(())
}

fn round_trips(text: &str, tokens: &[SemanticToken]) -> Result<(), TestCaseError> {
    for enc in [
        PositionEncoding::Utf8,
        PositionEncoding::Utf16,
        PositionEncoding::Utf32,
    ] {
        let data = encode_data(tokens, text, enc);
        // multi-line tokens split; single-line ones come back exactly
        let single: Vec<_> = tokens
            .iter()
            .filter(|t| !text[t.range.start as usize..t.range.end as usize].contains('\n'))
            .collect();
        let rows = decode_data(&data);
        prop_assert!(rows.len() >= single.len());
        for t in single {
            let found = rows.iter().any(|r| {
                byte_range(text, enc, r.0, r.1, r.2) == Some(t.range) && r.3 == t.ty.index()
            });
            prop_assert!(found, "{enc:?} lost {t:?}");
        }
    }
    Ok(())
}

/// Text drawn from the language's own alphabet, so the parser is
/// exercised on near-programs rather than on noise alone.
fn bdl_soup() -> impl Strategy<Value = String> {
    let atoms = prop_oneof![
        Just("concept "),
        Just("mapping "),
        Just("enum "),
        Just("component "),
        Just("instance "),
        Just("output "),
        Just("clock "),
        Just("device "),
        Just("port "),
        Just("export "),
        Just("bind "),
        Just("drive "),
        Just("if "),
        Just("then "),
        Just("else "),
        Just("all "),
        Just("any "),
        Just("in "),
        Just("delay "),
        Just("Tilt"),
        Just("Angle"),
        Just("dimByTilt"),
        Just("clamp"),
        Just("x"),
        Just("日本"),
        Just("ångström"),
        Just("𝛼"),
        Just("90"),
        Just("0.5"),
        Just("1e3"),
        Just("deg"),
        Just("ms"),
        Just(":"),
        Just("->"),
        Just("=>"),
        Just("="),
        Just("("),
        Just(")"),
        Just("["),
        Just("]"),
        Just("{"),
        Just("}"),
        Just(","),
        Just("?"),
        Just("??"),
        Just(".."),
        Just("@"),
        Just("."),
        Just("+"),
        Just("-"),
        Just("*"),
        Just("/"),
        Just("!"),
        Just("&&"),
        Just("||"),
        Just("=="),
        Just("!="),
        Just("<="),
        Just(">"),
        Just("// comment"),
        Just("/* block */"),
        Just("/* open"),
        Just("\n"),
        Just(" "),
        Just("§"),
        Just("\t"),
    ];
    prop::collection::vec(atoms, 0..40).prop_map(|v| v.concat())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn lexical_tokens_are_well_formed_on_any_text(text in "\\PC{0,200}") {
        let parse = parse_module(&text);
        let tokens = lexical_tokens(&parse.syntax_node());
        well_formed(&text, &tokens)?;
        round_trips(&text, &tokens)?;
    }

    #[test]
    fn lexical_tokens_cover_every_keyword_number_and_comment(text in bdl_soup()) {
        let parse = parse_module(&text);
        let root = parse.syntax_node();
        let tokens = lexical_tokens(&root);
        well_formed(&text, &tokens)?;
        round_trips(&text, &tokens)?;
        for t in root.descendants_with_tokens().filter_map(|e| e.into_token()) {
            let want = match t.kind() {
                SyntaxKind::Number => Some(TokenType::Number),
                SyntaxKind::LineComment | SyntaxKind::BlockComment => Some(TokenType::Comment),
                k if k.is_keyword() => Some(TokenType::Keyword),
                _ => None,
            };
            if let Some(want) = want {
                let start: u32 = t.text_range().start().into();
                let hit = tokens.iter().find(|s| s.range.start == start);
                prop_assert!(hit.is_some(), "{:?} {:?} unclassified in {text:?}", t.kind(), t.text());
                if want != TokenType::Keyword {
                    prop_assert_eq!(hit.map(|s| s.ty), Some(want), "{:?}", t.text());
                }
            }
        }
    }

    #[test]
    fn formula_lexical_tokens_are_well_formed_on_any_text(text in bdl_soup()) {
        let parse = parse_formula(&text);
        let tokens = lexical_tokens(&parse.syntax_node());
        well_formed(&text, &tokens)?;
        round_trips(&text, &tokens)?;
    }
}
