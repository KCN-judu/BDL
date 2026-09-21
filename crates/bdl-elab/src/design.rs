//! Surface model → Design IR.
//!
//! Concepts become `ConceptBinding`s (`Θ`); every mapping becomes a
//! `Declaration` whose interface is `sem A₁ → … → sem B` with no commitments
//! (the surface does not author commitments yet), and whose realization is
//! the elaborated formula when there is one and it elaborates.

use crate::formula::{elaborate_formula, elaborate_formula_in, Realized};
use crate::names::InputEnv;
use bdl_diagnostics::{Diagnostic, Entity};
use bdl_ir::{ConceptBinding, Declaration, DesignIr, Expr, Interface, OutputSpec, Ty};
use bdl_model::surface::{Definition, Design, MappingBlock, Representation, Transport};
use bdl_model::DeclId;
use std::collections::BTreeMap;

/// `Quantity dim → q dim`, `Boolean → bool`, `Count → nat`, `Optional →
/// opt`, `List → list`, `Pair → prod`.  Every case is concept-free data,
/// as `ConceptEnv.WF` requires.
pub fn representation_ty(r: &Representation) -> Ty {
    match r {
        Representation::Quantity { dim } => Ty::q(*dim),
        Representation::Boolean => Ty::Bool,
        Representation::Count => Ty::Nat,
        Representation::Optional { inner } => Ty::opt(representation_ty(inner)),
        Representation::List { element } => Ty::list(representation_ty(element)),
        Representation::Pair { first, second } => {
            Ty::prod(representation_ty(first), representation_ty(second))
        }
    }
}

/// The kernel interface of a relationship: its canonical type
/// `domain(inputs) -> B` (`Ty::of_signature`; `() -> B` for no inputs) in
/// the kernel's encoding — curried, the unit domain eliminated:
/// `sem A₁ → … → sem Aₙ → sem B`, and `sem B` when there are no inputs
/// (`bdl_ir::ty`, Lean's `expectedType`).  No commitments.
pub fn elaborate_interface(mapping: &MappingBlock) -> Interface {
    let sig = &mapping.signature;
    Interface {
        expected_type: Ty::kernel_of_signature(&sig.inputs, sig.output),
        commitments: Vec::new(),
    }
}

/// Where a mapping's realization stands after elaboration.
#[derive(Clone, Debug, PartialEq)]
pub enum RealizationOutcome {
    /// No definition attached: a legal, open declaration.
    Unresolved,
    /// A definition is attached and elaborated to Core.
    Elaborated(Realized),
    /// A definition is attached but did not elaborate (diagnostics say why).
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MappingElab {
    pub id: DeclId,
    pub interface: Interface,
    pub outcome: RealizationOutcome,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Elaboration {
    pub ir: DesignIr,
    pub mappings: BTreeMap<DeclId, MappingElab>,
}

/// Elaborate a whole design.  Pure; deterministic (ordered maps in, ordered
/// maps out).  Concepts first, so formulas can consult `Θ`.
pub fn elaborate_design(design: &Design) -> Elaboration {
    let mut ir = DesignIr::default();
    for c in design.concepts.values() {
        ir.concepts.insert(
            c.id,
            ConceptBinding {
                id: c.id,
                name: c.name.clone(),
                representation: c.representation.as_ref().map(representation_ty),
                ordered: c.ordered,
            },
        );
    }
    // Interfaces first: a formula may later refer to other declarations by
    // identity, and typing consults their type view.
    for m in design.mappings.values() {
        ir.decls.insert(
            m.id,
            Declaration {
                id: m.id,
                name: m.name.clone(),
                interface: elaborate_interface(m),
                realization: None,
            },
        );
        if let Some(c) = m.clock {
            ir.clocks.insert(m.id, c);
        }
    }
    for c in design.clocks.values() {
        ir.clock_names.insert(c.id, c.name.clone());
    }
    // Outputs: `OutputSpec` needs a domain; an output without one is open
    // (not yet a kernel output) and stays out of `Δ.outputs`.  Every drive
    // edge is recorded as authored — `DriveWF` is what rejects one whose
    // sink is open or does not fit, and the output pass says so.
    for o in design.outputs.values() {
        ir.output_names.insert(o.id, o.name.clone());
        if let Some(clock) = o.clock {
            ir.outputs.insert(
                o.id,
                OutputSpec {
                    id: o.id,
                    name: o.name.clone(),
                    accepts: Ty::sem(o.accepts),
                    clock,
                },
            );
        }
    }
    for m in design.mappings.values() {
        if let Some(o) = m.drives {
            ir.drives.insert(m.id, o);
        }
    }
    let mut mappings = BTreeMap::new();
    for m in design.mappings.values() {
        let interface = elaborate_interface(m);
        let (outcome, diagnostics) = match &m.definition {
            None => (RealizationOutcome::Unresolved, Vec::new()),
            Some(Definition::Formula { source }) => match elaborate_formula(design, &ir, m, source)
            {
                Ok((r, warnings)) => (RealizationOutcome::Elaborated(r), warnings),
                Err(d) => (RealizationOutcome::Failed, d),
            },
            Some(Definition::ScopedFormula { source, scope }) => {
                let env = InputEnv::scoped(&m.signature.inputs, scope);
                match elaborate_formula_in(design, &ir, m, source, env) {
                    Ok((r, warnings)) => (RealizationOutcome::Elaborated(r), warnings),
                    Err(d) => (RealizationOutcome::Failed, d),
                }
            }
            Some(Definition::Reference { target, transport }) => {
                match elaborate_reference(design, &ir, m, *target, transport.as_ref()) {
                    Ok(r) => (RealizationOutcome::Elaborated(r), Vec::new()),
                    Err(d) => (RealizationOutcome::Failed, d),
                }
            }
        };
        if let RealizationOutcome::Elaborated(r) = &outcome {
            if let Some(d) = ir.decls.get_mut(&m.id) {
                d.realization = Some(r.expr.clone());
            }
        }
        mappings.insert(
            m.id,
            MappingElab {
                id: m.id,
                interface,
                outcome,
                diagnostics,
            },
        );
    }
    Elaboration { ir, mappings }
}

/// `Definition::Reference`: the kernel's `declRef target`, or
/// `sync src init (declRef target)`.  No name is resolved; the target is an
/// identity.  The transport's initial value is a closed formula: it is
/// elaborated as a formula of this mapping's own (unit-domain) signature and
/// must mention no relationship and no temporal form, so a display name
/// can never influence a binding.  Typing is left to the checker, as for
/// every other realization.
pub fn elaborate_reference(
    design: &Design,
    ir: &DesignIr,
    mapping: &MappingBlock,
    target: DeclId,
    transport: Option<&Transport>,
) -> Result<Realized, Vec<Diagnostic>> {
    let entity = Entity::Mapping { id: mapping.id };
    if !ir.decls.contains_key(&target) {
        return Err(vec![Diagnostic::error(
            "reference.unknown_target",
            entity,
            format!(
                "{} refers to a relationship that does not exist.",
                mapping.name
            ),
        )
        .technical(format!("declRef {target}: not in Δ"))]);
    }
    let Some(t) = transport else {
        return Ok(Realized {
            expr: Expr::decl(target),
            spans: BTreeMap::new(),
        });
    };
    if !mapping.signature.is_unit_domain() {
        return Err(vec![Diagnostic::error(
            "reference.transport_of_relationship",
            entity,
            format!("{} has inputs, so its value cannot be carried across timing domains.", mapping.name),
        )
        .explain("Only a value can be remembered and transported: a relationship whose domain is `()` is read as one; a relationship with inputs is not.")
        .technical("sync needs a data operand (TemporalNotData)")]);
    }
    let closed = MappingBlock {
        definition: None,
        ..mapping.clone()
    };
    let (init, _) = elaborate_formula(design, ir, &closed, &t.init)?;
    if !init.expr.refs().is_empty() || !init.expr.is_delay_free() {
        return Err(vec![Diagnostic::error(
            "reference.init_not_closed",
            entity,
            format!("The initial value of {} must be a constant.", mapping.name),
        )
        .explain("A transported binding starts from a stated value; it cannot read other relationships or remember anything.")
        .technical(format!("init refs {:?}, delay-free {}", init.expr.refs(), init.expr.is_delay_free()))]);
    }
    Ok(Realized {
        expr: Expr::sync(t.source, init.expr, Expr::decl(target)),
        spans: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_diagnostics::Span;
    use bdl_ir::{Expr, Prim};
    use bdl_model::edit::{apply_edit, EditOp};
    use bdl_model::surface::{ProjectSnapshot, Signature};
    use bdl_model::{ConceptId, Dim};

    /// Tilt : angle, Brightness : dimensionless, Held : boolean, Open : unbound,
    /// and dimByTilt : (Tilt, Held) -> Brightness with `formula`.
    fn lamp(formula: Option<&str>) -> (Design, DeclId, [ConceptId; 4]) {
        let mut s = ProjectSnapshot::new(Design::empty("lamp"));
        let mut ids = Vec::new();
        for (name, rep) in [
            ("Tilt", Some(Representation::Quantity { dim: Dim::ANGLE })),
            (
                "Brightness",
                Some(Representation::Quantity { dim: Dim::ZERO }),
            ),
            ("Held", Some(Representation::Boolean)),
            ("Open", None),
        ] {
            let a = apply_edit(
                &s,
                &EditOp::CreateConcept {
                    name: name.into(),
                    description: String::new(),
                    representation: rep,
                },
            )
            .unwrap();
            ids.push(a.outcome.created_concept.unwrap());
            s = a.snapshot;
        }
        let a = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![ids[0], ids[2]],
                    output: ids[1],
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        let id = a.outcome.created_mapping.unwrap();
        s = a.snapshot;
        if let Some(f) = formula {
            s = apply_edit(
                &s,
                &EditOp::AttachDefinition {
                    id,
                    definition: Definition::Formula { source: f.into() },
                },
            )
            .unwrap()
            .snapshot;
        }
        (s.design, id, [ids[0], ids[1], ids[2], ids[3]])
    }

    fn codes(d: &[Diagnostic]) -> Vec<&str> {
        d.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn interface_is_the_signature_over_sem_types() {
        let (design, id, [tilt, bright, held, _]) = lamp(None);
        let e = elaborate_design(&design);
        let m = &e.mappings[&id];
        assert_eq!(
            m.interface.expected_type,
            Ty::arrows([Ty::sem(tilt), Ty::sem(held)], Ty::sem(bright))
        );
        assert!(m.interface.commitments.is_empty());
        assert_eq!(m.outcome, RealizationOutcome::Unresolved);
        assert!(m.diagnostics.is_empty());
        assert_eq!(e.ir.decls[&id].realization, None);
        assert_eq!(e.ir.representation_of(tilt), Some(&Ty::q(Dim::ANGLE)));
        assert!(e.ir.concepts_well_formed());
    }

    #[test]
    fn a_dimension_correct_formula_elaborates_to_rep_arith_mk() {
        let (design, id, [tilt, bright, held, _]) = lamp(Some("if Held then Tilt / 90 deg else 0"));
        let e = elaborate_design(&design);
        let m = &e.mappings[&id];
        let RealizationOutcome::Elaborated(r) = &m.outcome else {
            panic!("{:?}", m.diagnostics)
        };
        // λ tilt. λ held. mk Brightness (ite (rep held) (div (rep tilt) 90deg) 0)
        let Expr::Lam { dom, body } = &r.expr else {
            panic!()
        };
        assert_eq!(*dom, Ty::sem(tilt));
        let Expr::Lam { dom, body } = &**body else {
            panic!()
        };
        assert_eq!(*dom, Ty::sem(held));
        let Expr::Mk { s, e: inner } = &**body else {
            panic!()
        };
        assert_eq!(*s, bright);
        let Expr::App { f, a: els } = &**inner else {
            panic!()
        };
        assert_eq!(
            **els,
            Expr::prim(Prim::Lit {
                dim: Dim::ZERO,
                value: bdl_ir::Scalar(0.0)
            })
        );
        let Expr::App { f, a: then } = &**f else {
            panic!()
        };
        let Expr::App { f: ite, a: cond } = &**f else {
            panic!()
        };
        assert_eq!(
            **ite,
            Expr::prim(Prim::Ite {
                ty: Ty::q(Dim::ZERO)
            })
        );
        // `held` is the innermost binder: index 0; `tilt` is index 1
        assert_eq!(**cond, Expr::rep(Expr::Var { index: 0 }));
        let Expr::App { f, a: ninety } = &**then else {
            panic!()
        };
        let Expr::App { f: div, a: t } = &**f else {
            panic!()
        };
        assert_eq!(
            **div,
            Expr::prim(Prim::Div {
                d1: Dim::ANGLE,
                d2: Dim::ANGLE
            })
        );
        assert_eq!(**t, Expr::rep(Expr::Var { index: 1 }));
        let Expr::Prim {
            p: Prim::Lit { dim, value },
        } = &**ninety
        else {
            panic!()
        };
        assert_eq!(*dim, Dim::ANGLE);
        assert!((value.0 - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
        // the checker agrees, under the declaration's own grant
        assert_eq!(
            bdl_check::check_realization(&e.ir, id).unwrap(),
            Some(m.interface.expected_type.clone())
        );
        // spans: the whole body, and the division
        assert_eq!(r.spans[&vec![0, 0, 0]], Span::new(0, 33));
    }

    #[test]
    fn unknown_name_is_reported_with_the_available_inputs() {
        let (design, id, _) = lamp(Some("tilt + angle"));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(m.outcome, RealizationOutcome::Failed);
        assert_eq!(codes(&m.diagnostics), vec!["formula.name.unknown"]);
        assert_eq!(m.diagnostics[0].span, Some(Span::new(7, 12)));
        assert!(m.diagnostics[0].fixes[0].contains("Tilt, Held"));
    }

    #[test]
    fn case_insensitive_name_resolves_but_a_project_concept_that_is_not_an_input_is_explained() {
        let (design, id, _) = lamp(Some("tilt / 1 rad"));
        let m = &elaborate_design(&design).mappings[&id];
        assert!(matches!(m.outcome, RealizationOutcome::Elaborated(_)));
        let (design, id, _) = lamp(Some("Open / 1 rad"));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(codes(&m.diagnostics), vec!["formula.name.not_an_input"]);
    }

    /// The Sem-block model (ADR-0043) changes no resolution: a name is an
    /// input, then a relationship of the design (a Sem block or a rule),
    /// then a concept that is not an input — never a value looked up by
    /// concept.  A rule reading one concept twice reaches each input by
    /// the name the model derived for it.
    #[test]
    fn resolution_is_input_then_mapping_then_not_an_input_and_never_by_concept() {
        let (design, id, ids) = lamp(None);
        let m = &design.mappings[&id];
        let env = crate::names::InputEnv::for_mapping(&design, m);
        assert_eq!(env.resolve(&design, "Tilt"), crate::names::Lookup::Input(0));
        assert_eq!(env.resolve(&design, "Held"), crate::names::Lookup::Input(1));
        assert_eq!(
            env.resolve(&design, "dimByTilt"),
            crate::names::Lookup::Mapping(id)
        );
        assert_eq!(
            env.resolve(&design, "Open"),
            crate::names::Lookup::NotAnInput(ids[3], "Open".into())
        );
        assert_eq!(
            env.resolve(&design, "nothing"),
            crate::names::Lookup::Unknown
        );
        // the same concept twice: the derived names tell the inputs apart
        let a = apply_edit(
            &ProjectSnapshot::new(design.clone()),
            &EditOp::SetMappingSignature {
                id,
                signature: Signature {
                    inputs: vec![ids[0], ids[0]],
                    output: ids[1],
                },
            },
        )
        .unwrap();
        let design = a.snapshot.design;
        let m = &design.mappings[&id];
        assert_eq!(m.parameters, vec!["tilt1", "tilt2"]);
        let env = crate::names::InputEnv::for_mapping(&design, m);
        assert_eq!(
            env.resolve(&design, "tilt1"),
            crate::names::Lookup::Input(0)
        );
        assert_eq!(
            env.resolve(&design, "tilt2"),
            crate::names::Lookup::Input(1)
        );
        // the concept's own name is no input now: it is the concept
        assert_eq!(
            env.resolve(&design, "Tilt"),
            crate::names::Lookup::NotAnInput(ids[0], "Tilt".into())
        );
        // without names (a hand-written text `f(Tilt, Tilt)`), the exact
        // spelling takes the first input and the loose one is ambiguous —
        // the second input is unreachable, which is why the model names them
        let unnamed = crate::names::InputEnv::with_parameters(&design, &[ids[0], ids[0]], &[]);
        assert_eq!(
            unnamed.resolve(&design, "Tilt"),
            crate::names::Lookup::Input(0)
        );
        assert_eq!(
            unnamed.resolve(&design, "tilt"),
            crate::names::Lookup::Ambiguous(vec!["Tilt".into(), "Tilt".into()])
        );
    }

    #[test]
    fn malformed_formula() {
        let (design, id, _) = lamp(Some("Tilt / "));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(
            codes(&m.diagnostics),
            vec!["formula.parse.unexpected_token"]
        );
        assert_eq!(m.diagnostics[0].span, Some(Span::new(7, 7)));
    }

    #[test]
    fn adding_incompatible_dimensions() {
        let (design, id, _) = lamp(Some("(Tilt + 1 s) / 1 rad"));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(codes(&m.diagnostics), vec!["dimension.mismatch"]);
        assert!(m.diagnostics[0].message.contains("an angle and a time"));
        assert_eq!(m.diagnostics[0].span, Some(Span::new(0, 12)));
    }

    #[test]
    fn wrong_representation_for_the_output() {
        let (design, id, _) = lamp(Some("Tilt"));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(codes(&m.diagnostics), vec!["realization.type_mismatch"]);
        assert!(m.diagnostics[0]
            .message
            .contains("Brightness is a dimensionless quantity"));
        assert!(m.diagnostics[0].message.contains("produces an angle"));
    }

    #[test]
    fn unbound_representation_is_open_not_an_error() {
        let (mut design, id, [_, _, _, open]) = lamp(Some("Tilt / 1 rad"));
        design.mappings.get_mut(&id).unwrap().signature.output = open;
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(m.outcome, RealizationOutcome::Failed);
        assert_eq!(
            codes(&m.diagnostics),
            vec!["concept.unbound_representation"]
        );
        assert!(!m.diagnostics[0].is_error());
    }

    #[test]
    fn kind_and_branch_errors() {
        let (design, id, _) = lamp(Some("Held + 1"));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(codes(&m.diagnostics), vec!["type.operand_kind"]);
        let (design, id, _) = lamp(Some("if Held then 1 else Held"));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(codes(&m.diagnostics), vec!["type.branch_mismatch"]);
        let (design, id, _) = lamp(Some("3 furlong / 1 rad"));
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(codes(&m.diagnostics), vec!["formula.unit.unknown"]);
    }

    #[test]
    fn comparisons_and_booleans_elaborate_to_kernel_prims() {
        let (design, id, _) = lamp(Some(
            "if Tilt >= 45 deg && !Held || Tilt != 0 rad then 1 else 0",
        ));
        let m = &elaborate_design(&design).mappings[&id];
        assert!(
            matches!(m.outcome, RealizationOutcome::Elaborated(_)),
            "{:?}",
            m.diagnostics
        );
        let e = elaborate_design(&design);
        assert!(bdl_check::check_realization(&e.ir, id).is_ok());
    }

    #[test]
    fn representation_rebinding_changes_the_analysis() {
        let (mut design, id, [tilt, ..]) = lamp(Some("Tilt / 90 deg"));
        assert!(matches!(
            elaborate_design(&design).mappings[&id].outcome,
            RealizationOutcome::Elaborated(_)
        ));
        // Tilt rebound to a length: the same formula now divides a length by an angle
        design.concepts.get_mut(&tilt).unwrap().representation =
            Some(Representation::Quantity { dim: Dim::LENGTH });
        let m = &elaborate_design(&design).mappings[&id];
        assert_eq!(codes(&m.diagnostics), vec!["realization.type_mismatch"]);
    }

    proptest::proptest! {
        #[test]
        fn elaboration_never_panics(src in "\\PC{0,48}") {
            let (design, _, _) = lamp(Some(&src));
            let _ = elaborate_design(&design);
        }
    }
}
