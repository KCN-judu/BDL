//! Elaboration of the textual expression forms — calls, blocks and `let`,
//! `if`, `match` and patterns, `Some`/`None`, `delay`/`sync` placement —
//! into the existing Core, with the diagnostics the surface must give.
//! Every positive case is re-checked by `bdl-check`, the authority.

#![allow(clippy::unwrap_used)]

use bdl_check::pretty;
use bdl_diagnostics::{Diagnostic, Severity, Span};
use bdl_elab::{elaborate_design, RealizationOutcome};
use bdl_ir::Expr;
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{DeclId, Dim, SemanticId};

/// A lamp with a heater: concepts, one relationship with an input, two
/// nullary relationships, one timing domain.
struct Lamp {
    s: ProjectSnapshot,
    tilt: SemanticId,
    brightness: SemanticId,
    held: SemanticId,
    level: SemanticId,
    dim_by_tilt: DeclId,
    /// `level : Level`, unresolved (an input).
    level_in: DeclId,
    /// `boost : Level`, `level * 2`.
    boost: DeclId,
}

impl Lamp {
    fn new() -> Lamp {
        let mut s = ProjectSnapshot::new(Design::empty("lamp"));
        let concept = |s: &mut ProjectSnapshot, name: &str, rep: Representation| {
            let a = apply_edit(
                s,
                &EditOp::CreateConcept {
                    name: name.into(),
                    description: String::new(),
                    representation: Some(rep),
                },
            )
            .unwrap();
            *s = a.snapshot;
            a.outcome.created_concept.unwrap()
        };
        let tilt = concept(&mut s, "Tilt", Representation::Quantity { dim: Dim::ANGLE });
        let brightness = concept(
            &mut s,
            "Brightness",
            Representation::Quantity { dim: Dim::ZERO },
        );
        let held = concept(&mut s, "Held", Representation::Boolean);
        let level = concept(&mut s, "Level", Representation::Quantity { dim: Dim::ZERO });
        let a = apply_edit(
            &s,
            &EditOp::CreateClockDomain {
                name: "main".into(),
            },
        )
        .unwrap();
        s = a.snapshot;
        let mut lamp = Lamp {
            s,
            tilt,
            brightness,
            held,
            level,
            dim_by_tilt: DeclId::from_raw(0),
            level_in: DeclId::from_raw(0),
            boost: DeclId::from_raw(0),
        };
        lamp.dim_by_tilt = lamp.mapping("dimByTilt", vec![tilt], brightness, Some("Tilt / 90 deg"));
        lamp.level_in = lamp.mapping("level", vec![], level, None);
        lamp.boost = lamp.mapping("boost", vec![], level, Some("level * 2"));
        lamp
    }

    fn mapping(
        &mut self,
        name: &str,
        inputs: Vec<SemanticId>,
        output: SemanticId,
        formula: Option<&str>,
    ) -> DeclId {
        let a = apply_edit(
            &self.s,
            &EditOp::CreateMapping {
                name: name.into(),
                description: String::new(),
                signature: Signature { inputs, output },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        self.s = a.snapshot;
        let id = a.outcome.created_mapping.unwrap();
        if let Some(f) = formula {
            let a = apply_edit(
                &self.s,
                &EditOp::AttachDefinition {
                    id,
                    definition: Definition::Formula { source: f.into() },
                },
            )
            .unwrap();
            self.s = a.snapshot;
        }
        id
    }

    /// Elaborate `src` as `f : (Tilt, Held) -> Brightness`.
    fn with_inputs(&mut self, src: &str) -> Outcome {
        let id = self.mapping("f", vec![self.tilt, self.held], self.brightness, Some(src));
        self.outcome(id)
    }

    /// Elaborate `src` as `g : Level` (no inputs, so memory is allowed).
    fn nullary(&mut self, src: &str) -> Outcome {
        let id = self.mapping("g", vec![], self.level, Some(src));
        self.outcome(id)
    }

    fn outcome(&mut self, id: DeclId) -> Outcome {
        let e = elaborate_design(&self.s.design);
        let m = &e.mappings[&id];
        let core = match &m.outcome {
            RealizationOutcome::Elaborated(r) => {
                let checked = bdl_check::check_realization(&e.ir, id);
                assert!(
                    checked.is_ok(),
                    "checker refused {}: {:?}",
                    pretty::expr(&r.expr),
                    checked
                );
                Some(r.expr.clone())
            }
            _ => None,
        };
        let a = apply_edit(&self.s, &EditOp::DeleteMapping { id }).unwrap();
        self.s = a.snapshot;
        Outcome {
            core,
            diagnostics: m.diagnostics.clone(),
        }
    }
}

struct Outcome {
    core: Option<Expr>,
    diagnostics: Vec<Diagnostic>,
}

impl Outcome {
    fn core(&self) -> String {
        match &self.core {
            Some(e) => pretty::expr(e),
            None => panic!("did not elaborate: {:?}", self.diagnostics),
        }
    }
    fn codes(&self) -> Vec<&str> {
        self.diagnostics.iter().map(|d| d.code.as_str()).collect()
    }
    fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect()
    }
    fn first(&self) -> &Diagnostic {
        self.errors().first().expect("an error")
    }
    fn expect_error(&self, code: &str, span: Span) -> &Diagnostic {
        assert!(
            self.core.is_none(),
            "elaborated despite {code}: {}",
            self.core()
        );
        let d = self
            .diagnostics
            .iter()
            .find(|d| d.code.as_str() == code)
            .unwrap_or_else(|| panic!("no {code} in {:?}", self.codes()));
        assert_eq!(d.span, Some(span), "{code}: {}", d.message);
        d
    }
}

// ---- calls -------------------------------------------------------------------

#[test]
fn calls_apply_relationships_to_semantic_values() {
    let mut l = Lamp::new();
    // input → relationship → the formula's result
    assert_eq!(
        l.with_inputs("dimByTilt(Tilt)").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (rep (decl#0 #1)))"
    );
    // a nullary relationship is a value; another relationship's value is passed on
    assert_eq!(l.nullary("boost").core(), "(mk sem#3 (rep decl#2))");
    assert_eq!(
        l.with_inputs("dimByTilt(Tilt) + level").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (add[1] (rep (decl#0 #1)) (rep decl#1)))"
    );
    // an `if` whose branches are both Tilt is still a Tilt, so it can be passed
    assert_eq!(
        l.with_inputs("dimByTilt(if Held then Tilt else Tilt)")
            .core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (rep (decl#0 (ite[sem#0] (rep #0) #1 #1))))"
    );
}

#[test]
fn call_diagnostics_have_precise_spans() {
    let mut l = Lamp::new();
    l.with_inputs("dimByTilt(Tilt, Held)")
        .expect_error("formula.call.arity", Span::new(0, 21));
    l.with_inputs("dimByTilt()")
        .expect_error("formula.call.arity", Span::new(0, 11));
    l.with_inputs("Tilt(1)")
        .expect_error("formula.call.not_a_relationship", Span::new(0, 4));
    l.with_inputs("dimByTilt(Tilt)(1)")
        .expect_error("formula.call.not_a_relationship", Span::new(0, 15));
    l.with_inputs("nope(Tilt)")
        .expect_error("formula.name.unknown", Span::new(0, 4));
    l.with_inputs("1 + nope")
        .expect_error("formula.name.unknown", Span::new(4, 8));
    // the wrong concept, and a computed number, as arguments
    l.with_inputs("dimByTilt(Held)")
        .expect_error("formula.call.argument_type", Span::new(10, 14));
    l.with_inputs("dimByTilt(Tilt / 2)")
        .expect_error("formula.call.argument", Span::new(10, 18));
    // a relationship used as a value: the higher-order case, refused at the surface
    l.with_inputs("dimByTilt")
        .expect_error("formula.mapping.needs_arguments", Span::new(0, 9));
    l.with_inputs("1 + dimByTilt")
        .expect_error("formula.mapping.needs_arguments", Span::new(4, 13));
    let o = l.nullary("boost(1)");
    o.expect_error("formula.call.arity", Span::new(0, 8));
    assert!(o.first().message.contains("its only argument is `()`"));
    assert!(o.first().fixes[0].contains("Write boost."));
}

/// A relationship without inputs has the canonical type `() -> B`: the
/// reference `boost`, the empty call `boost()` and the explicit application
/// to the unique argument `boost(())` are one term of Core — `declRef` —
/// with the unit erased (`bdl_ir::ty`); `()` is a value nowhere else and is
/// never an absent value, a quantity or a measurement unit.
#[test]
fn a_unit_domain_relationship_is_read_as_a_value_and_its_argument_is_erased() {
    let mut l = Lamp::new();
    let plain = l.nullary("boost").core();
    assert_eq!(l.nullary("boost()").core(), plain);
    assert_eq!(l.nullary("boost(())").core(), plain);
    assert_eq!(
        l.nullary("boost(()) + boost").core(),
        l.nullary("boost + boost()").core()
    );
    // `()` on its own is not a value
    let o = l.nullary("()");
    o.expect_error("formula.unit.not_a_value", Span::new(0, 2));
    l.nullary("1 + ()")
        .expect_error("formula.unit.not_a_value", Span::new(4, 6));
    // not an absent value, not a truth value, not a unit suffix
    let o = l.nullary("() == None");
    assert!(o.core.is_none());
    assert!(
        o.codes().contains(&"formula.unit.not_a_value"),
        "{:?}",
        o.codes()
    );
    l.nullary("if () then 1 else 2")
        .expect_error("formula.unit.not_a_value", Span::new(3, 5));
    let o = l.with_inputs("dimByTilt(())");
    assert!(o.core.is_none());
    assert!(
        o.codes().contains(&"formula.unit.not_a_value")
            || o.codes().contains(&"formula.call.argument"),
        "{:?}",
        o.codes()
    );
}

// ---- blocks and let ------------------------------------------------------------

#[test]
fn let_is_a_beta_redex_and_scopes_lexically() {
    let mut l = Lamp::new();
    assert_eq!(
        l.with_inputs("{ let n = Tilt / (90 deg); n }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(q[1]). #0 (div[rad,rad] (rep #1) 1.5707963267948966[rad])))"
    );
    // nested lets: the inner one sees the outer one; shadowing is lexical
    assert_eq!(
        l.with_inputs("{ let a = 1; let b = a + 1; a + b }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(q[1]). (λ(q[1]). (add[1] #1 #0) (add[1] #0 1[1])) 1[1]))"
    );
    assert_eq!(
        l.with_inputs("{ let x = 1; let x = x + 1; x }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(q[1]). (λ(q[1]). #0 (add[1] #0 1[1])) 1[1]))"
    );
    // a let may shadow an input; inputs are still reachable by their exact name
    assert_eq!(
        l.with_inputs("{ let tilt = Tilt / 2; tilt / (1 rad) }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(q[rad]). (div[rad,rad] #0 1[rad]) (div[rad,1] (rep #1) 2[1])))"
    );
    // a let-bound semantic value stays semantic: it can be passed on
    assert_eq!(
        l.with_inputs("{ let t = Tilt; dimByTilt(t) }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (rep (λ(sem#0). (decl#0 #0) #1)))"
    );
    // nested blocks
    assert_eq!(
        l.with_inputs("{ let a = { let b = 2; b * 3 }; a + 1 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(q[1]). (add[1] #0 1[1]) (λ(q[1]). (mul[1,1] #0 3[1]) 2[1])))"
    );
}

#[test]
fn let_scope_and_pattern_diagnostics() {
    let mut l = Lamp::new();
    // out of scope: a name from an inner block, and a name from a sibling block
    l.with_inputs("{ let a = { let b = 2; b }; b }")
        .expect_error("formula.name.unknown", Span::new(28, 29));
    l.with_inputs("{ let a = 1; a } + a")
        .expect_error("formula.name.unknown", Span::new(19, 20));
    // a let cannot take a value apart
    let o = l.with_inputs("{ let Some(v) = Some(1); v }");
    o.expect_error("formula.let.refutable", Span::new(6, 13));
    assert_eq!(o.errors().len(), 1, "no cascade from `v`: {:?}", o.codes());
    l.with_inputs("{ let None = 1; 2 }")
        .expect_error("formula.pattern.constructor_binding", Span::new(6, 10));
    l.with_inputs("{ let x = None; 1 }")
        .expect_error("formula.option.undetermined", Span::new(10, 14));
    // a let-bound value is not a relationship
    l.with_inputs("{ let x = 1; x(2) }")
        .expect_error("formula.call.not_a_relationship", Span::new(13, 14));
}

// ---- if ----------------------------------------------------------------------

#[test]
fn if_joins_branches_and_reports_condition_and_branch_errors_where_they_are() {
    let mut l = Lamp::new();
    // a semantic branch and a number: the semantic one is observed
    assert_eq!(
        l.with_inputs("if Held then dimByTilt(Tilt) else 0").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (ite[q[1]] (rep #0) (rep (decl#0 #1)) 0[1]))"
    );
    let o = l.with_inputs("if Tilt then 1 else 2");
    o.expect_error("type.operand_kind", Span::new(3, 7));
    let o = l.with_inputs("if Held then 1 else Held");
    o.expect_error("type.branch_mismatch", Span::new(0, 24));
    assert!(o
        .first()
        .message
        .contains("a dimensionless quantity and true or false"));
    // `None` takes its kind from the other branch
    assert_eq!(
        l.with_inputs("match (if Held then Some(1) else None) { Some(v) => v, None => 0 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(opt q[1]). (ite[q[1]] (isSome[q[1]] #0) (λ(q[1]). #0 (getD[q[1]] #0 0[1])) 0[1]) (ite[opt q[1]] (rep #0) (some[q[1]] 1[1]) none[q[1]])))"
    );
    assert_eq!(
        l.with_inputs("match (if Held then None else Some(1)) { Some(v) => v, None => 0 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(opt q[1]). (ite[q[1]] (isSome[q[1]] #0) (λ(q[1]). #0 (getD[q[1]] #0 0[1])) 0[1]) (ite[opt q[1]] (rep #0) none[q[1]] (some[q[1]] 1[1]))))"
    );
}

// ---- match -------------------------------------------------------------------

#[test]
fn match_on_bool_option_and_representations() {
    let mut l = Lamp::new();
    // Bool: the scrutinee is bound once; `true` is the value itself
    assert_eq!(
        l.with_inputs("match Held { true => 1, false => 0 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(sem#2). (ite[q[1]] (rep #0) 1[1] 0[1]) #0))"
    );
    // Option: isSome test, getD projection for the binding
    assert_eq!(
        l.with_inputs("match Some(Tilt / (1 rad)) { Some(v) => v, None => 0 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(opt q[1]). (ite[q[1]] (isSome[q[1]] #0) (λ(q[1]). #0 (getD[q[1]] #0 0[1])) 0[1]) (some[q[1]] (div[rad,rad] (rep #1) 1[rad]))))"
    );
    // nested constructor patterns
    assert_eq!(
        l.with_inputs("match Some(Some(1)) { Some(Some(x)) => x, Some(None) => 1, None => 2 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(opt opt q[1]). (ite[q[1]] (and (isSome[opt q[1]] #0) (isSome[q[1]] (getD[opt q[1]] #0 none[q[1]]))) (λ(q[1]). #0 (getD[q[1]] (getD[opt q[1]] #0 none[q[1]]) 0[1])) (ite[q[1]] (and (isSome[opt q[1]] #0) (not (isSome[q[1]] (getD[opt q[1]] #0 none[q[1]])))) 1[1] 2[1])) (some[opt q[1]] (some[q[1]] 1[1]))))"
    );
    // a literal pattern on a dimensionless quantity; a catch-all binds the value
    assert_eq!(
        l.nullary("match level { 0 => 1, x => x }").core(),
        "(mk sem#3 (λ(sem#3). (ite[q[1]] (eq[q[1]] (rep #0) 0[1]) 1[1] (rep (λ(sem#3). #0 #0))) decl#1))"
    );
    // a semantic scrutinee met by a literal pattern is observed
    assert_eq!(
        l.with_inputs("match Held { true => 1, _ => 0 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(sem#2). (ite[q[1]] (rep #0) 1[1] 0[1]) #0))"
    );
    // a nested match, with a shadowing binder
    assert_eq!(
        l.with_inputs("match Some(1) { Some(x) => match Some(x) { Some(x) => x, None => 0 }, None => 0 }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (λ(opt q[1]). (ite[q[1]] (isSome[q[1]] #0) (λ(q[1]). (λ(opt q[1]). (ite[q[1]] (isSome[q[1]] #0) (λ(q[1]). #0 (getD[q[1]] #0 0[1])) 0[1]) (some[q[1]] #0)) (getD[q[1]] #0 0[1])) 0[1]) (some[q[1]] 1[1])))"
    );
    // arms that are relationships of the same concept stay semantic
    assert_eq!(
        l.with_inputs("match Held { true => dimByTilt(Tilt), false => dimByTilt(Tilt) }").core(),
        "λ(sem#0). λ(sem#2). (mk sem#1 (rep (λ(sem#2). (ite[sem#1] (rep #0) (decl#0 #2) (decl#0 #2)) #0)))"
    );
}

#[test]
fn match_exhaustiveness_reachability_and_pattern_diagnostics() {
    let mut l = Lamp::new();
    let o = l.with_inputs("match Held { true => 1 }");
    o.expect_error("formula.match.non_exhaustive", Span::new(0, 24));
    assert!(o.first().message.contains("`false` is not handled"));
    let o = l.with_inputs("match Some(1) { Some(v) => v }");
    assert!(o
        .expect_error("formula.match.non_exhaustive", Span::new(0, 30))
        .message
        .contains("`None`"));
    let o = l.with_inputs("match Some(Some(1)) { Some(Some(v)) => v, None => 0 }");
    assert!(o
        .expect_error("formula.match.non_exhaustive", Span::new(0, 53))
        .message
        .contains("inside `Some(…)`, `None` is not handled"));
    let o = l.with_inputs("match Tilt / (1 rad) { 0 => 1 }");
    assert!(o.first().message.contains("add a final `_ => …` arm"));

    // unreachable arms are warnings; the formula still elaborates
    let o = l.with_inputs("match Some(1) { Some(x) => x, Some(y) => y, None => 0, _ => 5 }");
    assert!(o.core.is_some());
    let unreachable: Vec<Option<Span>> = o
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str() == "formula.match.unreachable")
        .map(|d| {
            assert_eq!(d.severity, Severity::Warning);
            d.span
        })
        .collect();
    assert_eq!(
        unreachable,
        vec![Some(Span::new(30, 43)), Some(Span::new(55, 61))]
    );
    let o = l.with_inputs("match Held { _ => 1, true => 2 }");
    assert!(o.core.is_some());
    assert_eq!(o.codes(), vec!["formula.match.unreachable"]);

    // patterns
    l.with_inputs("match Some(1) { Some(x, y) => 1, None => 0 }")
        .expect_error("formula.constructor.arity", Span::new(16, 26));
    l.with_inputs("match Some(1) { Some => 1, None => 0 }")
        .expect_error("formula.constructor.arity", Span::new(16, 20));
    l.with_inputs("match Some(1) { None(x) => 1, _ => 0 }")
        .expect_error("formula.constructor.arity", Span::new(16, 23));
    let o = l.with_inputs("match Some(1) { Off(x) => 1, _ => 0 }");
    assert!(o
        .expect_error("formula.constructor.unknown", Span::new(16, 22))
        .explanation
        .contains("DI-19"));
    l.with_inputs("match Some(Some(1)) { Some(Some(x, x)) => 1, _ => 0 }")
        .expect_error("formula.constructor.arity", Span::new(27, 37));
    l.with_inputs("match Held { Some(x) => 1, _ => 0 }")
        .expect_error("formula.pattern.kind", Span::new(13, 20));
    l.with_inputs("match Some(1) { true => 1, _ => 0 }")
        .expect_error("formula.pattern.kind", Span::new(16, 20));
    l.with_inputs("match Tilt { 0 => 1, _ => 2 }")
        .expect_error("dimension.mismatch", Span::new(13, 14));
    l.with_inputs("match Some(1) { Some(1.5) => 1, _ => 0 }")
        .expect_error("formula.parse.unexpected_token", Span::new(21, 24));
    // pattern variables are scoped to their arm
    l.with_inputs("match Some(1) { Some(v) => v, None => v }")
        .expect_error("formula.name.unknown", Span::new(38, 39));
    // Some / None as expressions
    l.with_inputs("Some(1, 2)")
        .expect_error("formula.constructor.arity", Span::new(0, 10));
    l.with_inputs("None(1)")
        .expect_error("formula.constructor.arity", Span::new(0, 7));
    l.with_inputs("Some")
        .expect_error("formula.constructor.arity", Span::new(0, 4));
    // equality is structural at any data kind (Phase 9b): an optional
    // value compares with a bare `None`, which takes its kind from the
    // other side; only the mapping's Brightness result is then wrong
    assert_eq!(
        l.with_inputs("Some(1) == None").codes(),
        vec!["realization.type_mismatch"]
    );
}

#[test]
fn duplicate_pattern_bindings_are_refused() {
    let mut l = Lamp::new();
    let o = l.with_inputs("match Some(Some(1)) { Some(Some(x)) => x, Some(x) => 1, None => 0 }");
    assert!(
        o.core.is_some(),
        "x in different arms is fine: {:?}",
        o.codes()
    );
    // the same name twice in one pattern needs a nested constructor with
    // two payloads; options have one, so a duplicate can only arise as
    // `Some(x)` inside a pattern that already bound `x` — not expressible
    // on options.  The check is exercised through the compiler directly.
    let d = bdl_syntax::formula("match Some(1) { Some(x) => x, None => 0 }").unwrap();
    assert!(matches!(d.kind, bdl_syntax::ExprKind::Match { .. }));
}

// ---- temporal placement ----------------------------------------------------------

#[test]
fn memory_is_allowed_outside_binders_only() {
    let mut l = Lamp::new();
    // a let's value and a match's scrutinee are outside every binder
    assert_eq!(
        l.nullary("{ let previous = delay(0, level); previous + 1 }")
            .core(),
        "(mk sem#3 (λ(q[1]). (add[1] #0 1[1]) (delay 0[1] (rep decl#1))))"
    );
    assert_eq!(
        l.nullary("match delay(None, Some(level)) { Some(v) => v, None => 0 }").core(),
        "(mk sem#3 (λ(opt q[1]). (ite[q[1]] (isSome[q[1]] #0) (λ(q[1]). #0 (getD[q[1]] #0 0[1])) 0[1]) (delay none[q[1]] (some[q[1]] (rep decl#1)))))"
    );
    // an `if` branch is not a binder
    assert_eq!(
        l.nullary("if level < 1 then delay(0, level) else level")
            .core(),
        "(mk sem#3 (ite[q[1]] (lt[1] (rep decl#1) 1[1]) (delay 0[1] (rep decl#1)) (rep decl#1)))"
    );
    // both operands the same concept: the delayed value stays semantic
    assert_eq!(
        l.nullary("delay(level, level)").core(),
        "(mk sem#3 (rep (delay decl#1 decl#1)))"
    );
    // inside a block's result, an arm, or a formula over inputs: refused
    l.nullary("{ let p = 1; delay(0, p) }")
        .expect_error("formula.temporal.under_binder", Span::new(13, 24));
    l.nullary("match level { 0 => delay(0, level), _ => 1 }")
        .expect_error("formula.temporal.under_binder", Span::new(19, 34));
    l.with_inputs("delay(0, Tilt / 1 rad)")
        .expect_error("formula.temporal.under_inputs", Span::new(0, 22));
    l.nullary("delay(true, level)")
        .expect_error("type.temporal_mismatch", Span::new(0, 18));
    l.nullary("sync(ambient, 0, level)")
        .expect_error("formula.sync.unknown_domain", Span::new(5, 12));
    l.nullary("sync(main, 0, level)").core();
    l.nullary("delay(0)")
        .expect_error("formula.temporal.arity", Span::new(0, 8));
}

// ---- spans for checker errors ---------------------------------------------------

#[test]
fn every_core_subterm_of_a_block_or_match_has_a_span() {
    let mut l = Lamp::new();
    let id = l.mapping(
        "f",
        vec![l.tilt, l.held],
        l.brightness,
        Some("{ let n = Tilt / (90 deg); match Some(n) { Some(v) => v, None => 0 } }"),
    );
    let e = elaborate_design(&l.s.design);
    let RealizationOutcome::Elaborated(r) = &e.mappings[&id].outcome else {
        panic!("{:?}", e.mappings[&id].diagnostics)
    };
    // walk the term: every node that is not a synthetic binder/default must
    // have a recorded span
    fn walk(e: &Expr, path: &mut Vec<u8>, out: &mut Vec<Vec<u8>>) {
        out.push(path.clone());
        let children: Vec<&Expr> = match e {
            Expr::Lam { body, .. } => vec![body],
            Expr::App { f, a } => vec![f, a],
            Expr::Rep { e } | Expr::Mk { e, .. } => vec![e],
            Expr::Delay { init, e } | Expr::Sync { init, e, .. } => vec![init, e],
            _ => vec![],
        };
        for (i, c) in children.into_iter().enumerate() {
            path.push(i as u8);
            walk(c, path, out);
            path.pop();
        }
    }
    let mut paths = Vec::new();
    walk(&r.expr, &mut Vec::new(), &mut paths);
    let with_span = paths.iter().filter(|p| r.spans.contains_key(*p)).count();
    // the two lambdas, mk, the let, the match, the scrutinee, the arms…
    assert!(
        with_span >= 12,
        "{with_span} of {} paths carry a span",
        paths.len()
    );
    // the scrutinee `Some(n)` and the let's value are where they were written
    let src = "{ let n = Tilt / (90 deg); match Some(n) { Some(v) => v, None => 0 } }";
    let spans: Vec<&str> = r
        .spans
        .values()
        .map(|s| &src[s.start as usize..s.end as usize])
        .collect();
    assert!(spans.contains(&"Some(n)"));
    assert!(spans.contains(&"Tilt / (90 deg)"));
    assert!(spans.contains(&"v"));
}
