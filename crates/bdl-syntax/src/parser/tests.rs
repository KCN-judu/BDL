//! The parser test matrix (`docs/spec/textual-syntax.md`; task §31–§33).

use crate::ast::{self, AstNode};
use crate::kind::SyntaxKind;
use crate::lower::{ExprKind, SurfaceExpr, UnaryOp};
use crate::syntax::{debug_tree, SyntaxErrorCode};
use crate::{parse, parse_formula, parse_module};
use bdl_diagnostics::Span;

/// Fully parenthesised rendering of a lowered expression.
fn show(e: &SurfaceExpr) -> String {
    match &e.kind {
        ExprKind::Name(n) => n.clone(),
        ExprKind::Number { literal, unit } => match unit {
            Some(u) => format!("{}{}", literal.as_str(), u.name),
            None => literal.as_str().to_owned(),
        },
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Unary { op, expr } => format!(
            "({}{})",
            if *op == UnaryOp::Not { "!" } else { "-" },
            show(expr)
        ),
        ExprKind::Binary { op, lhs, rhs } => {
            format!("({} {} {})", show(lhs), op.symbol(), show(rhs))
        }
        ExprKind::If { cond, then, els } => {
            format!("(if {} then {} else {})", show(cond), show(then), show(els))
        }
        ExprKind::Call { callee, args } => format!(
            "{}[{}]",
            show(callee),
            args.iter().map(show).collect::<Vec<_>>().join(", ")
        ),
        ExprKind::Match { scrutinee, arms } => format!(
            "(match {} {{{}}})",
            show(scrutinee),
            arms.iter()
                .map(|a| format!("{:?} => {}", a.pattern.kind, show(&a.body)))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        ExprKind::Block { lets, tail } => format!(
            "{{{} {}}}",
            lets.iter()
                .map(|l| format!("let {:?} = {};", l.pattern.kind, show(&l.value)))
                .collect::<Vec<_>>()
                .join(" "),
            show(tail)
        ),
        ExprKind::List(items) => {
            format!(
                "[{}]",
                items.iter().map(show).collect::<Vec<_>>().join(", ")
            )
        }
        ExprKind::Tuple(items) => {
            format!(
                "<{}>",
                items.iter().map(show).collect::<Vec<_>>().join(", ")
            )
        }
        ExprKind::Lambda { params, body } => format!(
            "(\\{} => {})",
            params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(","),
            show(body)
        ),
        ExprKind::Hole => "?".into(),
        ExprKind::Unit => "()".into(),
        ExprKind::Binder {
            form,
            param,
            collection,
            body,
        } => format!(
            "({} {} in {}: {})",
            form.word(),
            param.name,
            show(collection),
            show(body)
        ),
        ExprKind::Range { lo, hi } => format!("({} .. {})", show(lo), show(hi)),
    }
}

fn ok(src: &str) -> String {
    let e = parse(src).unwrap_or_else(|e| panic!("{src:?}: {e}"));
    show(&e)
}

fn errors(src: &str) -> Vec<(SyntaxErrorCode, Span, String)> {
    parse_module(src)
        .errors()
        .iter()
        .map(|e| (e.code, e.span, e.message.clone()))
        .collect()
}

fn module_ok(src: &str) -> ast::Module {
    let p = parse_module(src);
    assert!(p.is_ok(), "{src:?}: {:?}", p.errors());
    assert_eq!(p.text(), src);
    p.tree()
}

fn tree(src: &str) -> String {
    debug_tree(&parse_module(src).syntax_node())
}

fn type_text(t: &ast::Type) -> String {
    match t {
        ast::Type::Named(n) => {
            let args: Vec<String> = n.type_args().map(|a| type_text(&a)).collect();
            if args.is_empty() {
                n.name().map(|n| n.as_str()).unwrap_or_default()
            } else {
                format!(
                    "{}<{}>",
                    n.name().map(|n| n.as_str()).unwrap_or_default(),
                    args.join(", ")
                )
            }
        }
        ast::Type::Function(f) => format!(
            "({} -> {})",
            f.domain().map(|d| type_text(&d)).unwrap_or_default(),
            f.codomain().map(|c| type_text(&c)).unwrap_or_default()
        ),
        ast::Type::Paren(p) => p.inner().map(|i| type_text(&i)).unwrap_or_default(),
        ast::Type::Unit(_) => "()".into(),
        ast::Type::Tuple(t) => format!(
            "({})",
            t.parts()
                .map(|p| type_text(&p))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

// ---- declarations ----------------------------------------------------------

#[test]
fn unresolved_mapping() {
    let m = module_ok("mapping f : A -> B");
    let d = m.mappings().next().expect("mapping");
    assert_eq!(d.name().expect("name").as_str(), "f");
    assert_eq!(type_text(&d.signature().expect("type")), "(A -> B)");
    assert!(d.definition().is_none());
}

#[test]
fn mapping_with_definition() {
    let m = module_ok("mapping f : A -> B\nf(x) = g(x)");
    let d = m.mappings().next().expect("mapping");
    let def = d.definition().expect("definition");
    assert_eq!(def.name().expect("name").as_str(), "f");
    let params: Vec<String> = def.params().map(|p| p.text()).collect();
    assert_eq!(params, vec!["x"]);
    assert!(matches!(def.body(), Some(ast::Expr::Call(_))));
}

#[test]
fn open_and_represented_concepts() {
    let m = module_ok("concept Open\nconcept Tilt : Angle");
    let cs: Vec<ast::ConceptDecl> = m.concepts().collect();
    assert_eq!(cs[0].name().expect("name").as_str(), "Open");
    assert!(cs[0].representation().is_none());
    assert_eq!(type_text(&cs[1].representation().expect("type")), "Angle");
}

#[test]
fn enum_declaration() {
    let m = module_ok("enum LampMode<T> { Off, Automatic, Manual(Brightness, T), }");
    let e = m.enums().next().expect("enum");
    assert_eq!(e.name().expect("name").as_str(), "LampMode");
    let tp: Vec<String> = e.type_params().map(|n| n.as_str()).collect();
    assert_eq!(tp, vec!["T"]);
    let vs: Vec<(String, Vec<String>)> = e
        .variants()
        .map(|v| {
            (
                v.name().expect("name").as_str(),
                v.fields().map(|t| type_text(&t)).collect(),
            )
        })
        .collect();
    assert_eq!(
        vs,
        vec![
            ("Off".to_owned(), vec![]),
            ("Automatic".to_owned(), vec![]),
            (
                "Manual".to_owned(),
                vec!["Brightness".to_owned(), "T".to_owned()]
            ),
        ]
    );
    module_ok("enum Empty {}");
    module_ok("enum One { A }");
}

// ---- types -------------------------------------------------------------------

#[test]
fn types() {
    let cases = [
        ("A", "A"),
        ("A -> B", "(A -> B)"),
        ("A -> B -> C", "(A -> (B -> C))"),
        ("Option<A>", "Option<A>"),
        ("Result<A, B>", "Result<A, B>"),
        ("(A -> B) -> C", "((A -> B) -> C)"),
        ("Option<Result<A, B>>", "Option<Result<A, B>>"),
        ("Option<A -> B>", "Option<(A -> B)>"),
    ];
    for (src, expected) in cases {
        let m = module_ok(&format!("mapping f : {src}"));
        let d = m.mappings().next().expect("mapping");
        assert_eq!(type_text(&d.signature().expect("type")), expected, "{src}");
    }
    let (inputs, output) = module_ok("mapping f : A -> B -> C")
        .mappings()
        .next()
        .and_then(|d| d.signature())
        .expect("type")
        .uncurry();
    assert_eq!(
        inputs.iter().map(type_text).collect::<Vec<_>>(),
        vec!["A", "B"]
    );
    assert_eq!(type_text(&output), "C");
}

// ---- calls -----------------------------------------------------------------

#[test]
fn calls() {
    assert_eq!(ok("f()"), "f[]");
    assert_eq!(ok("f(x)"), "f[x]");
    assert_eq!(ok("f(x, y)"), "f[x, y]");
    assert_eq!(ok("f(x, y,)"), "f[x, y]");
    assert_eq!(ok("f(x)(y)"), "f[x][y]");
    assert_eq!(ok("f(g(x), h(y, z))"), "f[g[x], h[y, z]]");
    assert_eq!(ok("-f(x)"), "(-f[x])");
    assert_eq!(ok("f(x) + 1"), "(f[x] + 1)");
    assert_eq!(
        ok("clamp(tilt / (90 deg), 0, 1)"),
        "clamp[(tilt / 90deg), 0, 1]"
    );
}

// ---- units -----------------------------------------------------------------

#[test]
fn units() {
    assert_eq!(ok("90 deg"), "90deg");
    assert_eq!(ok("25.4 mm"), "25.4mm");
    assert_eq!(ok("x / (2 s)"), "(x / 2s)");
    assert_eq!(ok("90 deg / 2"), "(90deg / 2)");
    // Unknown units are a semantic matter, not a syntactic one.
    assert_eq!(ok("90 foobar"), "90foobar");
    let e = parse("Tilt / 90 deg").expect("parses");
    let ExprKind::Binary { rhs, .. } = &e.kind else {
        panic!("binary")
    };
    let ExprKind::Number { literal, unit } = &rhs.kind else {
        panic!("number")
    };
    assert_eq!(literal.as_str(), "90");
    assert_eq!(unit.as_ref().map(|u| u.name.as_str()), Some("deg"));
    assert_eq!(unit.as_ref().map(|u| u.span), Some(Span::new(10, 13)));
}

#[test]
fn units_attach_only_to_number_literals() {
    for src in ["x deg", "f(x) mm", "(1 + 2) s"] {
        let e = parse(src).expect_err(src);
        assert_eq!(e.code, SyntaxErrorCode::Unexpected, "{src}");
        assert!(
            e.hint.as_deref().is_some_and(|h| h.contains("unit")),
            "{src}: {e:?}"
        );
    }
}

// ---- expressions -----------------------------------------------------------

#[test]
fn precedence() {
    assert_eq!(ok("1 + 2 * 3"), "(1 + (2 * 3))");
    assert_eq!(ok("(1 + 2) * 3"), "((1 + 2) * 3)");
    assert_eq!(ok("!x"), "(!x)");
    assert_eq!(ok("x < y"), "(x < y)");
    assert_eq!(ok("if x then y else z"), "(if x then y else z)");
    assert_eq!(
        ok("a || b && c == (d < e + f * -g)"),
        "(a || (b && (c == (d < (e + (f * (-g)))))))"
    );
    assert_eq!(
        ok("a || b && c < e + f * -g"),
        "(a || (b && (c < (e + (f * (-g))))))"
    );
    assert_eq!(ok("1 - 2 - 3"), "((1 - 2) - 3)");
    assert_eq!(ok("a / b * c"), "((a / b) * c)");
    assert_eq!(ok("a && b && c"), "((a && b) && c)");
    assert_eq!(ok("-x * y"), "((-x) * y)");
    assert_eq!(ok("- -x"), "(-(-x))");
    assert_eq!(ok("!a && b"), "((!a) && b)");
    assert_eq!(
        ok("if a < 1 then b else if c then d else e"),
        "(if (a < 1) then b else (if c then d else e))"
    );
    assert_eq!(ok("if c then a else b + 1"), "(if c then a else (b + 1))");
    assert_eq!(ok("1 + if c then a else b"), "(1 + (if c then a else b))");
    assert_eq!(ok("(a < b) == c"), "((a < b) == c)");
    assert_eq!(ok("true || false"), "(true || false)");
}

#[test]
fn blocks_and_let() {
    assert_eq!(
        ok("{ let n = tilt / (90 deg); let b = clamp(n, 0, 1); b }"),
        "{let Ident(\"n\") = (tilt / 90deg); let Ident(\"b\") = clamp[n, 0, 1]; b}"
    );
    assert_eq!(ok("{ x }"), "{ x}");
    assert_eq!(ok("{ x } + 1"), "({ x} + 1)");
}

#[test]
fn match_expression() {
    assert_eq!(
        ok("match mode { Off => 0, Automatic => dimByTilt(tilt), Manual(value) => value, }"),
        "(match mode {Ident(\"Off\") => 0; Ident(\"Automatic\") => dimByTilt[tilt]; \
         Constructor { name: \"Manual\", fields: [SurfacePattern { kind: Ident(\"value\"), \
         span: Span { start: 60, end: 65 } }] } => value})"
    );
    // Trailing comma optional on the last arm, and `match` is a primary.
    assert_eq!(
        ok("match x { _ => 1 } + 2"),
        "((match x {Wildcard => 1}) + 2)"
    );
    assert_eq!(ok("match x {}"), "(match x {})");
}

#[test]
fn patterns() {
    let e = parse("match v { _ => 0, x => 1, None => 2, Some(x) => 3, Point(x, y) => 4, true => 5, -1 => 6, 7 => 8 }")
        .expect("parses");
    let ExprKind::Match { arms, .. } = &e.kind else {
        panic!("match")
    };
    let kinds: Vec<String> = arms
        .iter()
        .map(|a| format!("{:?}", a.pattern.kind))
        .collect();
    assert_eq!(
        kinds,
        vec![
            "Wildcard",
            "Ident(\"x\")",
            "Ident(\"None\")",
            "Constructor { name: \"Some\", fields: [SurfacePattern { kind: Ident(\"x\"), span: Span { start: 42, end: 43 } }] }",
            "Constructor { name: \"Point\", fields: [SurfacePattern { kind: Ident(\"x\"), span: Span { start: 57, end: 58 } }, SurfacePattern { kind: Ident(\"y\"), span: Span { start: 60, end: 61 } }] }",
            "Bool(true)",
            "Number { negative: true, literal: NumberLiteral { text: \"1\" } }",
            "Number { negative: false, literal: NumberLiteral { text: \"7\" } }",
        ]
    );
    let e = parse("match v { 1.5 => 0 }").expect_err("fractional literal pattern");
    assert_eq!(e.code, SyntaxErrorCode::LiteralPattern);
}

// ---- spans -------------------------------------------------------------------

#[test]
fn spans_cover_the_source() {
    let e = parse("tilt / 90 deg").expect("parses");
    assert_eq!(e.span, Span::new(0, 13));
    let ExprKind::Binary { rhs, .. } = &e.kind else {
        panic!("binary")
    };
    assert_eq!(rhs.span, Span::new(7, 13));
    let e = parse("(a + b)").expect("parses");
    assert_eq!(e.span, Span::new(0, 7));
    let e = parse("  f(x)  ").expect("parses");
    assert_eq!(e.span, Span::new(2, 6));
}

// ---- malformed -------------------------------------------------------------

#[test]
fn malformed_formulas() {
    let e = parse("1 +").expect_err("1 +");
    assert_eq!(e.span, Span::new(3, 3));
    assert!(
        e.message.contains("expected an expression after `+`"),
        "{e}"
    );
    let e = parse("(1 + 2").expect_err("(1 + 2");
    assert!(e.message.contains("`)`"), "{e}");
    let e = parse("1 2").expect_err("1 2");
    assert_eq!(e.span, Span::new(2, 3));
    let e = parse("if a then b").expect_err("if a then b");
    assert!(e.message.contains("`else`"), "{e}");
    let e = parse("a ) b").expect_err("a ) b");
    assert_eq!(e.span, Span::new(2, 3));
    let e = parse("").expect_err("empty");
    assert_eq!(e.code, SyntaxErrorCode::Empty);
    let e = parse("a # b").expect_err("a # b");
    assert_eq!(e.code, SyntaxErrorCode::InvalidCharacter);
    assert_eq!(e.span, Span::new(2, 3));
    let e = parse("a & b").expect_err("a & b");
    assert_eq!(e.found, "`&`");
    let e = parse("Tilt / ").expect_err("Tilt / ");
    assert_eq!(e.span, Span::new(7, 7));
}

#[test]
fn chained_comparisons_are_rejected_with_a_suggestion() {
    for src in [
        "a < b < c",
        "a == b == c",
        "a == b < c",
        "a < b == c",
        "a <= b > c",
    ] {
        let p = parse_formula(src);
        let e = p
            .errors()
            .iter()
            .find(|e| e.code == SyntaxErrorCode::ChainedComparison)
            .unwrap_or_else(|| panic!("{src}: {:?}", p.errors()));
        assert!(e
            .hint
            .as_deref()
            .is_some_and(|h| h.contains("a < b && b < c")));
        assert_eq!(p.errors().len(), 1, "{src}: one error, {:?}", p.errors());
    }
    // The tree is still complete: the parse failed, the structure did not.
    let p = parse_formula("a < b < c");
    let expr = p.tree().expr().expect("expr");
    assert!(matches!(expr, ast::Expr::Binary(_)));
    assert_eq!(p.text(), "a < b < c");
    // Parenthesised comparisons are fine.
    assert!(parse("(a < b) < c").is_ok());
}

#[test]
fn missing_closers_missing_eq_missing_arrow() {
    let e = errors("mapping f : A -> B\nf(x = x");
    assert_eq!(e[0].0, SyntaxErrorCode::Expected);
    assert!(e[0].2.contains("`)` to close the parameter list"), "{e:?}");

    let e = errors("mapping f : A -> B\nf(x) = { let y = x; y");
    assert!(
        e.iter().any(|e| e.2.contains("`}` to close this block")),
        "{e:?}"
    );

    let e = errors("mapping f : A -> B\nf(x) x");
    assert!(e.iter().any(|e| e.2.contains("expected `=`")), "{e:?}");

    let e = errors("mapping f : A B\nf(x) = x");
    assert!(!e.is_empty(), "missing arrow must be diagnosed");

    let e = errors("mapping f A -> B");
    assert!(e[0].2.contains("expected `:`"), "{e:?}");

    let e = errors("concept Tilt :");
    assert!(e[0].2.contains("after `:`"), "{e:?}");
}

#[test]
fn malformed_match_arms_recover_at_comma_and_brace() {
    let src = "mapping f : A -> B\nf(x) = match x { Off => , Auto 1, => 2, On => 3 }";
    let p = parse_module(src);
    assert_eq!(p.text(), src);
    assert!(!p.is_ok());
    let m = p.tree();
    let def = m
        .mappings()
        .next()
        .and_then(|d| d.definition())
        .expect("def");
    let ast::Expr::Match(mx) = def.body().expect("body") else {
        panic!("match")
    };
    let arms: Vec<String> = mx
        .arms()
        .filter_map(|a| a.pattern().map(|p| p.text()))
        .collect();
    assert!(
        arms.contains(&"Off".to_owned()) && arms.contains(&"On".to_owned()),
        "{arms:?}"
    );
}

#[test]
fn unexpected_tokens_at_top_level_are_skipped_to_the_next_item() {
    let src = "concept A\n) ) garbage 1 2 3\nconcept B\nx(y) = 1\nmapping m : A -> B";
    let p = parse_module(src);
    assert_eq!(p.text(), src);
    let names: Vec<String> = p
        .tree()
        .items()
        .filter_map(|i| match i {
            ast::Item::Concept(c) => c.name().map(|n| n.as_str()),
            ast::Item::Mapping(m) => m.name().map(|n| n.as_str()),
            ast::Item::Enum(e) => e.name().map(|n| n.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["A", "B", "m"]);
    assert_eq!(p.errors().len(), 2, "{:?}", p.errors());
    assert!(p.errors()[1]
        .message
        .contains("needs its `mapping` signature"));
}

#[test]
fn invalid_acceptance_program() {
    let src = "concept Tilt : Angle\n\nmapping broken : Tilt -> Brightness\nbroken(tilt =\n  if tilt < 10 deg < 20 deg then\n    foo(tilt\n  else\n    0\n\nconcept Later : Scalar\n";
    let p = parse_module(src);
    assert_eq!(p.text(), src);
    let codes: Vec<SyntaxErrorCode> = p.errors().iter().map(|e| e.code).collect();
    assert_eq!(
        codes,
        vec![
            SyntaxErrorCode::Expected,
            SyntaxErrorCode::ChainedComparison,
            SyntaxErrorCode::Expected
        ]
    );
    let m = p.tree();
    let concepts: Vec<String> = m
        .concepts()
        .filter_map(|c| c.name())
        .map(|n| n.as_str())
        .collect();
    assert_eq!(concepts, vec!["Tilt", "Later"]);
    let broken = m.mappings().next().expect("broken");
    let def = broken.definition().expect("definition survives");
    assert!(matches!(def.body(), Some(ast::Expr::If(_))));
}

#[test]
fn reserved_words_are_diagnosed_not_fatal() {
    let e = errors("concept context");
    assert_eq!(e[0].0, SyntaxErrorCode::ReservedWord);
    assert_eq!(e.len(), 1);
    let e = parse("context + 1").expect_err("reserved");
    assert_eq!(e.code, SyntaxErrorCode::ReservedWord);
}

#[test]
fn a_block_needs_a_final_expression() {
    let e = parse("{ let x = 1; }").expect_err("no tail");
    assert!(e.message.contains("no final expression"), "{e}");
    let e = parse("{ x; y }").expect_err("stray semicolon");
    assert!(e.message.contains("only a `let` ends with `;`"), "{e}");
}

// ---- trivia and losslessness -----------------------------------------------

#[test]
fn comments_and_whitespace_are_retained() {
    let src = "// head\n\nconcept A // trailing\n/* block */ concept B\n";
    let t = tree(src);
    assert!(t.contains("LineComment@0..7 \"// head\""), "{t}");
    assert!(t.contains("LineComment@19..30 \"// trailing\""), "{t}");
    assert!(t.contains("BlockComment@31..42 \"/* block */\""), "{t}");
    assert!(t.contains("Whitespace@7..9 \"\\n\\n\""), "{t}");
    assert_eq!(parse_module(src).text(), src);
}

#[test]
fn a_comment_directly_above_an_item_is_attached_to_it() {
    let src = "// far\n\n// near\nconcept A\n";
    let m = parse_module(src).tree();
    let c = m.concepts().next().expect("concept");
    assert_eq!(c.text(), "// near\nconcept A");
    assert_eq!(c.span(), Span::new(8, 25));
    // The far comment stays in the module.
    let first = m.syntax().first_token().expect("token");
    assert_eq!(first.kind(), SyntaxKind::LineComment);
    assert_eq!(first.text(), "// far");
}

#[test]
fn a_comment_directly_above_a_port_keeps_the_port() {
    use crate::lower::{lower_module, ComponentBodyItem, PortWord, SurfaceItem};
    let cases: [(&str, &str, ast::PortWord, PortWord); 5] = [
        (
            "// c",
            "requires tiltValue : Tilt @main",
            ast::PortWord::Requires,
            PortWord::Requires,
        ),
        (
            "/// doc",
            "requires tiltValue : Tilt @main",
            ast::PortWord::Requires,
            PortWord::Requires,
        ),
        (
            "// c",
            "provides brightness : Brightness @main\n  brightness() = 1",
            ast::PortWord::Provides,
            PortWord::Provides,
        ),
        (
            "/// doc",
            "provides brightness : Brightness @main\n  brightness() = 1",
            ast::PortWord::Provides,
            PortWord::Provides,
        ),
        (
            "// c",
            "param gain : Gain",
            ast::PortWord::Param,
            PortWord::Param,
        ),
    ];
    for (comment, decl, word, lowered_word) in cases {
        for blank in ["", "\n"] {
            let src =
                format!("component L {{\n  use concept Tilt\n{blank}  {comment}\n  {decl}\n}}\n");
            // The CST is lossless: the comment is there …
            let t = tree(&src);
            assert!(t.contains("LineComment@"), "{src:?}\n{t}");
            assert!(t.contains(comment), "{src:?}\n{t}");
            // … and so is the declaration, with the comment attached to it.
            let m = module_ok(&src);
            let comp = match m.items().next().expect("an item") {
                ast::Item::Component(c) => c,
                other => panic!("{src:?}: {other:?}"),
            };
            let ports: Vec<ast::PortDecl> = comp
                .items()
                .filter_map(|i| match i {
                    ast::ComponentItem::Port(p) => Some(p),
                    _ => None,
                })
                .collect();
            assert_eq!(ports.len(), 1, "{src:?}: one port declaration in the AST");
            assert!(
                ports[0].text().starts_with(comment),
                "{src:?}: {:?}",
                ports[0].text()
            );
            assert_eq!(ports[0].word(), Some(word), "{src:?}: the port's word");
            assert_eq!(
                ports[0].name().map(|n| n.as_str()).as_deref(),
                decl.split(' ').nth(1)
            );
            // Lowering keeps it: the same item with or without the comment.
            let (with, errors) = lower_module(&parse_module(&src));
            assert!(errors.is_empty(), "{src:?}: {errors:?}");
            let bare = src.replace(&format!("  {comment}\n"), "");
            let (without, _) = lower_module(&parse_module(&bare));
            let ports_of = |m: &crate::lower::SurfaceModule| -> Vec<(PortWord, String)> {
                match &m.items[0] {
                    SurfaceItem::Component(c) => c
                        .items
                        .iter()
                        .filter_map(|i| match i {
                            ComponentBodyItem::Port(p) => Some((p.word, p.name.name.clone())),
                            _ => None,
                        })
                        .collect(),
                    other => panic!("{other:?}"),
                }
            };
            assert_eq!(ports_of(&with), ports_of(&without), "{src:?}");
            assert_eq!(
                ports_of(&with),
                vec![(lowered_word, decl.split(' ').nth(1).unwrap().into())]
            );
        }
    }
}

#[test]
fn a_comment_directly_above_a_body_mapping_keeps_the_mapping() {
    use crate::lower::{lower_module, ComponentBodyItem, SurfaceItem};
    let src = "component L {\n  // c\n  mapping d : Tilt -> Brightness\n  d(t) = t\n  /// doc\n  requires t : Tilt\n}\n";
    let (m, errors) = lower_module(&parse_module(src));
    assert!(errors.is_empty(), "{errors:?}");
    let SurfaceItem::Component(c) = &m.items[0] else {
        panic!("{:?}", m.items[0]);
    };
    let kinds: Vec<&str> = c
        .items
        .iter()
        .map(|i| match i {
            ComponentBodyItem::Mapping(_) => "mapping",
            ComponentBodyItem::Port(_) => "port",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, ["mapping", "port"]);
}

#[test]
fn a_trailing_comment_before_the_closing_brace_declares_nothing() {
    use crate::lower::{lower_module, SurfaceItem};
    let src = "component L {\n  requires t : Tilt\n  // last\n}\nconcept A\n";
    let p = parse_module(src);
    assert!(p.is_ok(), "{:?}", p.errors());
    assert_eq!(p.text(), src);
    let t = tree(src);
    assert!(t.contains("LineComment@"), "{t}");
    assert!(t.contains("\"// last\""), "{t}");
    let (m, errors) = lower_module(&p);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(m.items.len(), 2, "{:?}", m.items);
    let SurfaceItem::Component(c) = &m.items[0] else {
        panic!("{:?}", m.items[0]);
    };
    assert_eq!(c.items.len(), 1, "{:?}", c.items);
    assert!(
        matches!(&m.items[1], SurfaceItem::Concept(_)),
        "{:?}",
        m.items[1]
    );
}

#[test]
fn round_trip_valid_and_malformed() {
    let cases = [
        "",
        "   ",
        "// only a comment",
        "concept Tilt : Angle\n",
        "mapping f : A -> B\nf(x) =\n  g(x) + 1 // done\n",
        "mapping broken : Tilt -> Brightness\nbroken(tilt =\n  if tilt < 10 deg < 20 deg then foo(tilt else 0",
        "enum E { A, B(C,), }",
        "}}} ((( ,,, => -> § 😀 /* unterminated",
        "match { let ; if then else",
        "1 + + 2",
        "a § b",
    ];
    for src in cases {
        assert_eq!(parse_module(src).text(), src, "module {src:?}");
        assert_eq!(parse_formula(src).text(), src, "formula {src:?}");
    }
}

#[test]
fn numbers_keep_their_spelling() {
    for spelling in ["0.1", "1.0", "1", "1e999", ".5", "1.5E-3", "007"] {
        let e = parse(spelling).expect("number");
        let ExprKind::Number { literal, .. } = &e.kind else {
            panic!("number")
        };
        assert_eq!(literal.as_str(), spelling);
    }
}

// ---- property tests ----------------------------------------------------------

proptest::proptest! {
    #[test]
    fn arbitrary_strings_never_panic_the_lexer(s in "\\PC{0,64}") {
        let (tokens, _) = crate::lexer::lex(&s);
        let joined: String = tokens.iter().map(|t| t.text(&s)).collect();
        proptest::prop_assert_eq!(joined, s);
    }

    #[test]
    fn arbitrary_strings_never_panic_the_parsers_and_round_trip(s in "\\PC{0,96}") {
        let m = parse_module(&s);
        proptest::prop_assert_eq!(m.text(), s.clone());
        let f = parse_formula(&s);
        proptest::prop_assert_eq!(f.text(), s.clone());
        let _ = crate::lower::lower_module(&m);
        let _ = crate::lower::lower_formula(&f);
    }

    /// Token soup from the whole vocabulary, so every grammar loop meets
    /// every token: the parser must terminate and stay lossless.
    #[test]
    fn arbitrary_token_streams_never_panic_and_always_progress(
        toks in proptest::collection::vec(
            proptest::sample::select(vec![
                "concept", "mapping", "enum", "match", "let", "if", "then", "else", "true",
                "false", "context", "x", "Some", "_", "1", "2.5", "(", ")", "{", "}", "<", ">",
                "<=", ">=", "==", "!=", ":", ",", ";", "=", "->", "=>", "+", "-", "*", "/", "!",
                "&&", "||", "deg", "§", "//c\n", "/* c */",
            ]),
            0..40,
        )
    ) {
        let s = toks.join(" ");
        let m = parse_module(&s);
        proptest::prop_assert_eq!(m.text(), s.clone());
        let f = parse_formula(&s);
        proptest::prop_assert_eq!(f.text(), s.clone());
        // Every error has a span inside the source.
        for e in m.errors().iter().chain(f.errors()) {
            proptest::prop_assert!(e.span.end as usize <= s.len(), "{:?}", e);
        }
    }

    #[test]
    fn well_formed_arithmetic_round_trips(a in 0u32..1000, b in 0u32..1000, c in 0u32..1000) {
        let e = parse(&format!("{a} + {b} * {c}")).expect("parses");
        proptest::prop_assert_eq!(show(&e), format!("({a} + ({b} * {c}))"));
    }
}

#[test]
fn collections_grouped_values_rules_and_membership() {
    assert_eq!(ok("[1, 2, 3]"), "[1, 2, 3]");
    assert_eq!(ok("[]"), "[]");
    assert_eq!(ok("[a, ]"), "[a]");
    assert_eq!(ok("(a, b)"), "<a, b>");
    assert_eq!(ok("(a, b, c)"), "<a, b, c>");
    assert_eq!(ok("(a)"), "a");
    assert_eq!(ok("mode in [1, 2]"), "(mode in [1, 2])");
    assert_eq!(ok("x in xs && y"), "((x in xs) && y)");
    // a rule extends as far right as possible and is an argument
    assert_eq!(
        ok("any(xs, x => x < 30 deg)"),
        "any[xs, (\\x => (x < 30deg))]"
    );
    assert_eq!(
        ok("foldr(xs, 0, (x, acc) => x + acc)"),
        "foldr[xs, 0, (\\x,acc => (x + acc))]"
    );
    assert_eq!(
        ok("map(xs, x => (x, x * 2))"),
        "map[xs, (\\x => <x, (x * 2)>)]"
    );
    // `in` is a comparison: it does not chain
    assert!(errors("mapping f : A -> B\nf(a) = a in b in c\n")
        .iter()
        .any(|(c, _, _)| *c == SyntaxErrorCode::ChainedComparison));
    // `ordered concept` is an item
    let m = parse_module("ordered concept Brightness : Scalar\nconcept Mode : Count\n");
    assert!(m.errors().is_empty());
    let items: Vec<bool> = ast::Module::cast(m.syntax_node())
        .unwrap()
        .concepts()
        .map(|c| c.is_ordered())
        .collect();
    assert_eq!(items, vec![true, false]);
    assert!(errors("ordered mapping f : A\n")
        .iter()
        .any(|(_, _, m)| m.contains("expected `concept` after `ordered`")));
}

/// `?` is a slot: an expression not yet written, legal wherever a value
/// may stand (the Formula Composer's hole), formatted as itself.
#[test]
fn a_slot_parses_wherever_a_value_may_stand_and_formats_as_itself() {
    assert_eq!(ok("?"), "?");
    assert_eq!(ok("tilt / ?"), "(tilt / ?)");
    assert_eq!(ok("clamp(? / (90 deg), ?, 1)"), "clamp[(? / 90deg), ?, 1]");
    assert_eq!(ok("? * ?"), "(? * ?)");
    assert_eq!(ok("-?"), "(-?)");
    // never an operator, never a unit
    assert!(!errors(
        "mapping f : A
f() = 1 ? 2
"
    )
    .is_empty());
    assert_eq!(
        crate::format::format_module(
            "mapping f : A
f() = tilt/?
"
        )
        .as_deref(),
        Some(
            "mapping f : A
f() = tilt / ?
"
        )
    );
}

/// The natural forms (P11): `all x in xs: body` and `lo .. hi` are surface
/// syntax over the same rules and equations — the parser builds them as
/// their own nodes; nothing else about the grammar changes.
#[test]
fn binders_are_contextual_and_extend_as_far_right_as_a_rule() {
    assert_eq!(
        ok("all reading in readings: reading < limit"),
        "(all reading in readings: (reading < limit))"
    );
    assert_eq!(
        ok("any x in xs: x > 1 && y"),
        "(any x in xs: ((x > 1) && y))"
    );
    assert_eq!(ok("map x in xs: x * 2"), "(map x in xs: (x * 2))");
    assert_eq!(
        ok("filter x in xs: x in a .. b"),
        "(filter x in xs: (x in (a .. b)))"
    );
    // a binder is an operand only in parentheses
    assert_eq!(ok("(all x in xs: x) && ok"), "((all x in xs: x) && ok)");
    assert_eq!(ok("!(any x in xs: x)"), "(!(any x in xs: x))");
    // nested, with shadowing left to the elaborator's lexical scopes
    assert_eq!(
        ok("all row in grid: all x in row: x < 1"),
        "(all row in grid: (all x in row: (x < 1)))"
    );
    assert_eq!(
        ok("any x in xs: all x in ys: x"),
        "(any x in xs: (all x in ys: x))"
    );
    // the collection is an expression short of `:` — a call, an index, a
    // sum; a membership test or a range there needs parentheses
    assert_eq!(ok("all x in f(a): x"), "(all x in f[a]: x)");
    assert_eq!(ok("all x in xs + ys: x"), "(all x in (xs + ys): x)");
    assert_eq!(ok("all x in (a .. b): x"), "(all x in (a .. b): x)");
    // the words stay ordinary names elsewhere: a mapping called `map`
    assert_eq!(ok("map(xs, x => x)"), "map[xs, (\\x => x)]");
    assert_eq!(ok("map(x)"), "map[x]");
    assert_eq!(ok("all(x, y)"), "all[x, y]");
    assert_eq!(ok("all + 1"), "(all + 1)");
    assert_eq!(ok("filter"), "filter");
    assert_eq!(ok("all in xs"), "(all in xs)");
    // the binder body is the whole rest, like a rule's
    assert_eq!(
        ok("any(xs, x => all y in ys: x < y)"),
        "any[xs, (\\x => (all y in ys: (x < y)))]"
    );
}

#[test]
fn malformed_binders_say_what_is_missing() {
    let says = |src: &str, what: &str| {
        let es = errors(&format!("mapping f : A\nf() = {src}\n"));
        assert!(
            es.iter().any(|(_, _, m)| m.contains(what)),
            "{src:?}: {es:?}"
        );
    };
    says("all x in xs x < 1", "expected `:` before the body");
    says("all x in : x", "expected the collection after `in`");
    says("all x in xs:", "expected the body after `:`");
    // `all in xs` is a membership test on a name called `all`: the colon
    // is the surprise, not the word
    let es = errors("mapping f : A\nf() = all in xs: x\n");
    assert_eq!(es.len(), 1, "{es:?}");
    assert_eq!(es[0].1, Span::new(29, 30));
}

#[test]
fn ranges_sit_between_comparison_and_arithmetic_and_do_not_chain() {
    assert_eq!(ok("x in lo .. hi"), "(x in (lo .. hi))");
    assert_eq!(ok("x + y in lo .. hi"), "((x + y) in (lo .. hi))");
    assert_eq!(ok("x in lo + d .. hi - d"), "(x in ((lo + d) .. (hi - d)))");
    assert_eq!(ok("x in -45 deg .. 45 deg"), "(x in ((-45deg) .. 45deg))");
    assert_eq!(ok("x in a * 2 .. b / 2"), "(x in ((a * 2) .. (b / 2)))");
    assert_eq!(ok("x in lo .. hi && y"), "((x in (lo .. hi)) && y)");
    assert_eq!(ok("x in xs"), "(x in xs)");
    // the lexer: a number never swallows the dots
    assert_eq!(ok("1.0..2.0"), "(1.0 .. 2.0)");
    assert_eq!(ok("1..2"), "(1 .. 2)");
    assert_eq!(ok("0.5 .. 1.5"), "(0.5 .. 1.5)");
    assert_eq!(ok("-1.0 .. 1.0"), "((-1.0) .. 1.0)");
    assert_eq!(ok("1 mm..2 mm"), "(1mm .. 2mm)");
    // the tokens of `-1.0 .. +1.0`: signs are operators, dots are one
    // operator, numbers are whole (there is no unary `+` in BDL, so the
    // parser then says so; the lexer never mis-splits)
    let (tokens, errs) = crate::lexer::lex("-1.0 .. +1.0");
    assert!(errs.is_empty());
    let kinds: Vec<SyntaxKind> = tokens
        .iter()
        .map(|t| t.kind)
        .filter(|k| *k != SyntaxKind::Whitespace)
        .collect();
    assert_eq!(
        kinds,
        vec![
            SyntaxKind::Minus,
            SyntaxKind::Number,
            SyntaxKind::DotDot,
            SyntaxKind::Plus,
            SyntaxKind::Number
        ]
    );
    // a range is one span, not a chain
    assert!(errors("mapping f : A\nf() = 1 .. 2 .. 3\n")
        .iter()
        .any(|(c, _, _)| *c == SyntaxErrorCode::ChainedComparison));
    // lossless: the formatter normalises the spaces and nothing else
    let fmt = |src: &str| crate::format::format_module(&format!("mapping f : A\nf() = {src}\n"));
    assert_eq!(
        fmt("x in 1.0..2.0").as_deref(),
        Some("mapping f : A\nf() = x in 1.0 .. 2.0\n")
    );
    assert_eq!(
        fmt("all reading in readings:reading<limit").as_deref(),
        Some("mapping f : A\nf() = all reading in readings: reading < limit\n")
    );
    // old call syntax is kept as written: never rewritten to the natural form
    assert_eq!(
        fmt("all(readings, reading => reading < limit)").as_deref(),
        Some("mapping f : A\nf() = all(readings, reading => reading < limit)\n")
    );
    assert_eq!(
        fmt("inRange(x, 1, 2)").as_deref(),
        Some("mapping f : A\nf() = inRange(x, 1, 2)\n")
    );
}

/// `x ?? d` is a default for a value that may be absent: it binds tighter
/// than a range or a comparison and weaker than arithmetic, and
/// associates to the right.
#[test]
fn coalesce_binds_between_comparison_and_arithmetic() {
    assert_eq!(ok("x ?? 0"), "(x ?? 0)");
    assert_eq!(ok("x ?? d + 1"), "(x ?? (d + 1))");
    assert_eq!(ok("x ?? 0 < 1"), "((x ?? 0) < 1)");
    assert_eq!(ok("x ?? 0 in lo .. hi"), "((x ?? 0) in (lo .. hi))");
    assert_eq!(ok("x in a ?? 0 .. b"), "(x in ((a ?? 0) .. b))");
    assert_eq!(ok("a ?? b ?? c"), "(a ?? (b ?? c))");
    assert_eq!(ok("? ?? ?"), "(? ?? ?)");
    assert_eq!(ok("-x ?? 1"), "((-x) ?? 1)");
    assert_eq!(
        crate::format::format_module("mapping f : A\nf() = x??0\n").as_deref(),
        Some("mapping f : A\nf() = x ?? 0\n")
    );
}

/// `()` is the empty product: as a type it opens a signature (`mapping f
/// : () -> B`, the same as the shorthand `mapping f : B`), as a value it is
/// the argument of a relationship without inputs (`f(())`).  A product
/// domain `(A, B) -> C` reads as `A -> B -> C`.  None of it disturbs
/// grouping, tuples, empty argument lists or calls.
#[test]
fn the_empty_product_is_a_type_and_a_value_and_a_product_domain_is_a_signature() {
    let sig = |src: &str| {
        let m = module_ok(src);
        let d = m.mappings().next().expect("mapping");
        let (inputs, output) = d.signature().expect("type").uncurry();
        (
            inputs.iter().map(type_text).collect::<Vec<_>>(),
            type_text(&output),
        )
    };
    assert_eq!(
        sig("mapping TempSensor : RoomTemp"),
        (vec![], "RoomTemp".into())
    );
    assert_eq!(
        sig("mapping TempSensor : () -> RoomTemp"),
        (vec![], "RoomTemp".into())
    );
    assert_eq!(sig("mapping f : () -> ()"), (vec![], "()".into()));
    assert_eq!(
        sig("mapping f : (Angle, Time) -> Speed"),
        (
            vec!["Angle".to_string(), "Time".to_string()],
            "Speed".into()
        )
    );
    assert_eq!(
        sig("mapping f : Angle -> Time -> Speed"),
        (
            vec!["Angle".to_string(), "Time".to_string()],
            "Speed".into()
        )
    );
    // a parenthesised type is still a grouping, not a product
    assert_eq!(
        sig("mapping f : (Angle) -> Speed"),
        (vec!["Angle".into()], "Speed".into())
    );
    // `()` after an input stays for lowering to refuse
    assert_eq!(
        sig("mapping f : Angle -> () -> Speed"),
        (vec!["Angle".into(), "()".into()], "Speed".into())
    );
    let m = module_ok("mapping f : () -> RoomTemp");
    let d = m.mappings().next().expect("mapping");
    assert_eq!(type_text(&d.signature().expect("type")), "(() -> RoomTemp)");
    // the value: an argument, never confused with an empty argument list,
    // a grouping, a grouped value or a slot
    assert_eq!(ok("f(())"), "f[()]");
    assert_eq!(ok("f()"), "f[]");
    assert_eq!(ok("(a)"), "a");
    assert_eq!(ok("(a, b)"), "<a, b>");
    assert_eq!(ok("f(?)"), "f[?]");
    assert_eq!(ok("() == ()"), "(() == ())");
    // the formatter keeps the spelling that was written
    for src in [
        "mapping TempSensor : RoomTemp\n",
        "mapping TempSensor : () -> RoomTemp\n",
        "mapping f : (Angle, Time) -> Speed\n",
        "mapping f : Angle -> Time -> Speed\n",
        "mapping f : () -> ()\nf() = ()\n",
        "mapping f : A\nf() = g(())\n",
    ] {
        assert_eq!(
            crate::format::format_module(src).as_deref(),
            Some(src),
            "{src:?}"
        );
    }
    assert_eq!(
        crate::format::format_module("mapping f : (  Angle,Time )->Speed\n").as_deref(),
        Some("mapping f : (Angle, Time) -> Speed\n")
    );
    assert_eq!(
        crate::format::format_module("mapping f : (  ) -> Speed\n").as_deref(),
        Some("mapping f : () -> Speed\n")
    );
}
