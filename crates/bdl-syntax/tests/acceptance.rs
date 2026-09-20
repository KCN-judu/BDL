//! The acceptance programs (task §40–§42) through the typed AST and the
//! lowering, as a future project loader would use them.

use bdl_syntax::ast::{self, AstNode, AstToken, LiteralKind};
use bdl_syntax::kind::SyntaxKind;
use bdl_syntax::lower::{ExprKind, PatternKind, SurfaceItem, TypeKind};
use bdl_syntax::{lower_module, parse_module};

const LAMP: &str = "// Lamp behaviour

concept Tilt : Angle
concept Brightness : Scalar
concept Held : Bool

mapping dimByTilt : Tilt -> Brightness
dimByTilt(tilt) =
  clamp(tilt / (90 deg), 0, 1)

mapping chooseBrightness : Held -> Tilt -> Brightness
chooseBrightness(held, tilt) =
  if held then
    dimByTilt(tilt)
  else
    0
";

const LAMP_MODE: &str = "enum LampMode {
  Off,
  Automatic,
  Manual(Brightness),
}

mapping resolve : LampMode -> Tilt -> Brightness
resolve(mode, tilt) =
  match mode {
    Off => 0,
    Automatic => dimByTilt(tilt),
    Manual(value) => value,
  }
";

fn named(t: &ast::Type) -> String {
    match t {
        ast::Type::Named(n) => n.name().map(|n| n.as_str()).unwrap_or_default(),
        other => panic!("not a named type: {}", other.text()),
    }
}

#[test]
fn lamp_program_is_lossless_and_fully_recoverable_from_the_typed_ast() {
    let parse = parse_module(LAMP);
    assert!(parse.is_ok(), "{:?}", parse.errors());
    assert_eq!(parse.text(), LAMP);
    let module = parse.tree();

    // Comments and whitespace are in the tree.
    let root = module.syntax();
    let first = root.first_token().expect("token");
    assert_eq!(first.kind(), SyntaxKind::LineComment);
    assert_eq!(first.text(), "// Lamp behaviour");
    let whitespace = root
        .descendants_with_tokens()
        .filter_map(|el| el.into_token())
        .filter(|t| t.kind() == SyntaxKind::Whitespace)
        .count();
    assert!(whitespace > 20);

    let concepts: Vec<String> = module
        .concepts()
        .map(|c| c.name().expect("name").as_str())
        .collect();
    assert_eq!(concepts, vec!["Tilt", "Brightness", "Held"]);

    let mappings: Vec<ast::MappingDecl> = module.mappings().collect();
    assert_eq!(mappings.len(), 2);

    // mapping dimByTilt
    let dim = &mappings[0];
    assert_eq!(dim.name().expect("name").as_str(), "dimByTilt");
    let (inputs, output) = dim.signature().expect("signature").uncurry();
    assert_eq!(inputs.iter().map(named).collect::<Vec<_>>(), vec!["Tilt"]);
    assert_eq!(named(&output), "Brightness");
    let def = dim.definition().expect("definition");
    assert_eq!(def.name().expect("name").as_str(), "dimByTilt");
    let params: Vec<String> = def.params().map(|p| p.text()).collect();
    assert_eq!(params, vec!["tilt"]);
    let ast::Expr::Call(call) = def.body().expect("body") else {
        panic!("call expected")
    };
    assert_eq!(call.callee().expect("callee").text(), "clamp");
    let args: Vec<ast::Expr> = call.arguments().collect();
    assert_eq!(args.len(), 3);
    // clamp(tilt / (90 deg), 0, 1): the unit literal is in there.
    let ast::Expr::Binary(div) = &args[0] else {
        panic!("division expected")
    };
    assert_eq!(div.op(), Some(ast::BinaryOp::Div));
    let ast::Expr::Paren(paren) = div.rhs().expect("rhs") else {
        panic!("parenthesised unit literal")
    };
    let ast::Expr::Literal(lit) = paren.inner().expect("inner") else {
        panic!("literal")
    };
    let Some(LiteralKind::Number(n)) = lit.kind() else {
        panic!("number")
    };
    assert_eq!(n.text(), "90");
    assert_eq!(n.literal().decimal().expect("decimal").to_string(), "90e0");
    assert_eq!(lit.unit().expect("unit").as_str(), "deg");

    // mapping chooseBrightness
    let choose = &mappings[1];
    assert_eq!(choose.name().expect("name").as_str(), "chooseBrightness");
    let (inputs, output) = choose.signature().expect("signature").uncurry();
    assert_eq!(
        inputs.iter().map(named).collect::<Vec<_>>(),
        vec!["Held", "Tilt"]
    );
    assert_eq!(named(&output), "Brightness");
    let def = choose.definition().expect("definition");
    let params: Vec<String> = def.params().map(|p| p.text()).collect();
    assert_eq!(params, vec!["held", "tilt"]);
    let ast::Expr::If(if_) = def.body().expect("body") else {
        panic!("if expected")
    };
    assert_eq!(if_.condition().expect("cond").text(), "held");
    let ast::Expr::Call(call) = if_.then_branch().expect("then") else {
        panic!("call expected")
    };
    assert_eq!(call.callee().expect("callee").text(), "dimByTilt");
    assert_eq!(
        call.arguments().map(|a| a.text()).collect::<Vec<_>>(),
        vec!["tilt"]
    );
    assert_eq!(if_.else_branch().expect("else").text(), "0");

    // Node spans exclude the trivia around them.
    assert_eq!(
        dim.text().lines().next(),
        Some("mapping dimByTilt : Tilt -> Brightness")
    );
}

#[test]
fn lamp_program_lowers_to_the_surface_tree() {
    let parse = parse_module(LAMP);
    let (module, errors) = lower_module(&parse);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(module.items.len(), 5);
    let mappings: Vec<_> = module.mappings().collect();
    let (inputs, output) = mappings[1].signature.uncurry();
    let names = |ts: &[&bdl_syntax::SurfaceType]| -> Vec<String> {
        ts.iter()
            .map(|t| match &t.kind {
                TypeKind::Named { name, .. } => name.clone(),
                TypeKind::Function { .. } => "fn".into(),
                TypeKind::Unit => "()".into(),
                TypeKind::Tuple(_) => "tuple".into(),
            })
            .collect()
    };
    assert_eq!(names(&inputs), vec!["Held", "Tilt"]);
    assert_eq!(names(&[output]), vec!["Brightness"]);
    let def = mappings[0].definition.as_ref().expect("definition");
    assert_eq!(def.params.len(), 1);
    assert_eq!(def.params[0].kind, PatternKind::Ident("tilt".into()));
    let ExprKind::Call { callee, args } = &def.body.kind else {
        panic!("call")
    };
    assert_eq!(callee.kind, ExprKind::Name("clamp".into()));
    let ExprKind::Binary { rhs, .. } = &args[0].kind else {
        panic!("div")
    };
    let ExprKind::Number { literal, unit } = &rhs.kind else {
        panic!("unit literal")
    };
    assert_eq!(literal.as_str(), "90");
    assert_eq!(unit.as_ref().expect("unit").name, "deg");
    // Parentheses are dropped in lowering but their span is kept.
    assert_eq!(
        &LAMP[rhs.span.start as usize..rhs.span.end as usize],
        "(90 deg)"
    );
}

#[test]
fn lamp_mode_program_is_unambiguous() {
    let parse = parse_module(LAMP_MODE);
    assert!(parse.is_ok(), "{:?}", parse.errors());
    assert_eq!(parse.text(), LAMP_MODE);
    let module = parse.tree();
    let e = module.enums().next().expect("enum");
    assert_eq!(e.name().expect("name").as_str(), "LampMode");
    let variants: Vec<(String, usize)> = e
        .variants()
        .map(|v| (v.name().expect("name").as_str(), v.fields().count()))
        .collect();
    assert_eq!(
        variants,
        vec![
            ("Off".into(), 0),
            ("Automatic".into(), 0),
            ("Manual".into(), 1)
        ]
    );
    let resolve = module.mappings().next().expect("mapping");
    let ast::Expr::Match(m) = resolve.definition().and_then(|d| d.body()).expect("body") else {
        panic!("match")
    };
    assert_eq!(m.scrutinee().expect("scrutinee").text(), "mode");
    let arms: Vec<ast::MatchArm> = m.arms().collect();
    assert_eq!(arms.len(), 3);
    assert!(arms.iter().all(|a| a.has_trailing_comma()));
    assert!(matches!(arms[0].pattern(), Some(ast::Pattern::Ident(_))));
    assert!(matches!(arms[1].pattern(), Some(ast::Pattern::Ident(_))));
    let Some(ast::Pattern::Constructor(c)) = arms[2].pattern() else {
        panic!("constructor pattern")
    };
    assert_eq!(c.name().expect("name").as_str(), "Manual");
    assert_eq!(
        c.fields().map(|f| f.text()).collect::<Vec<_>>(),
        vec!["value"]
    );

    let (lowered, errors) = lower_module(&parse);
    assert!(errors.is_empty());
    let SurfaceItem::Enum(en) = &lowered.items[0] else {
        panic!("enum")
    };
    assert_eq!(en.variants[2].fields.len(), 1);
    let SurfaceItem::Mapping(mp) = &lowered.items[1] else {
        panic!("mapping")
    };
    let ExprKind::Match { arms, .. } = &mp.definition.as_ref().expect("def").body.kind else {
        panic!("match")
    };
    assert_eq!(
        arms[2].pattern.kind,
        PatternKind::Constructor {
            name: "Manual".into(),
            fields: vec![bdl_syntax::SurfacePattern {
                kind: PatternKind::Ident("value".into()),
                span: bdl_diagnostics::Span::new(206, 211),
            }],
        }
    );
}

#[test]
fn lowering_reports_definition_name_and_arity_mismatches() {
    let src = "mapping f : A -> B\ng(x) = x\nmapping h : A -> B -> C\nh(x) = x\n";
    let parse = parse_module(src);
    assert!(parse.is_ok());
    let (module, errors) = lower_module(&parse);
    assert_eq!(module.mappings().count(), 2);
    let messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
    assert_eq!(
        messages,
        vec![
            "this definition is named `g`, but the mapping declared above it is `f`",
            "`h` reads 2 inputs by its signature, but its definition names 1",
        ]
    );
}

/// A device body carries the realization profile beside the pin choices,
/// in either order; a body without one leaves the profile open (the
/// spelling of every project written before realization existed).
#[test]
fn a_device_body_carries_its_realization_profile() {
    let src = "device lamp : pwm_channel for light { realization pwm_duty8, pin 0 = D3 }\n\
               device relay : digital_output for coil { pin 0 = D4, realization gpio_level }\n\
               device old : pwm_channel for light { pin 0 = D5 }\n\
               device bare : pwm_channel for light\n";
    let parse = parse_module(src);
    assert!(parse.is_ok(), "{:?}", parse.errors());
    let (module, errors) = lower_module(&parse);
    assert!(errors.is_empty(), "{errors:?}");
    type Device = (String, Option<String>, Vec<(u16, String)>);
    let devices: Vec<Device> = module
        .items
        .iter()
        .map(|i| match i {
            SurfaceItem::Device(d) => (
                d.name.name.clone(),
                d.realization.as_ref().map(|r| r.name.clone()),
                d.pins.iter().map(|(i, p)| (*i, p.name.clone())).collect(),
            ),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        devices,
        vec![
            (
                "lamp".into(),
                Some("pwm_duty8".into()),
                vec![(0, "D3".into())]
            ),
            (
                "relay".into(),
                Some("gpio_level".into()),
                vec![(0, "D4".into())]
            ),
            ("old".into(), None, vec![(0, "D5".into())]),
            ("bare".into(), None, vec![]),
        ]
    );
    // `realization` without a profile is an error that names what is expected.
    let parse = parse_module("device x : pwm_channel for light { realization }\n");
    assert!(!parse.is_ok());
    assert!(parse.errors()[0]
        .message
        .contains("realization profile's id"));
}
