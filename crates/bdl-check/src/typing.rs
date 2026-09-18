//! The typing judgment `Θ; Δ; G; Γ ⊢ e : τ` as an inference function.

use bdl_ir::{DesignIr, Expr, Ty};
use bdl_model::{DeclId, SemanticId};
use std::collections::BTreeSet;

/// Which semantic concepts a term may *construct* with `mk`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Grant {
    /// Client code: nothing may be constructed.
    None,
    /// The concepts in result position of a signature (`Ty::grant`).
    Of(BTreeSet<SemanticId>),
    /// The inlined executable program, after every construction was
    /// authorised at its own declaration.
    All,
}

impl Grant {
    pub fn of(ty: &Ty) -> Grant {
        Grant::Of(ty.grant().into_iter().collect())
    }
    pub fn permits(&self, s: SemanticId) -> bool {
        match self {
            Grant::None => false,
            Grant::Of(set) => set.contains(&s),
            Grant::All => true,
        }
    }
}

/// Position of a sub-expression: the sequence of child indices from the
/// root (`App{f,a}`: f = 0, a = 1; `Delay{init,e}`: init = 0, e = 1; …).
pub type ExprPath = Vec<u8>;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{kind:?} at {path:?}")]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub path: ExprPath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeErrorKind {
    UnboundVariable {
        index: u32,
    },
    UnknownDeclaration {
        id: DeclId,
    },
    ExpectedFunction {
        found: Ty,
    },
    ArgumentMismatch {
        expected: Ty,
        found: Ty,
    },
    RepOfNonSemantic {
        found: Ty,
    },
    /// `Θ s = none`: the concept has no representation yet.
    UnboundRepresentation {
        concept: SemanticId,
    },
    ConstructionNotGranted {
        concept: SemanticId,
    },
    ConstructionMismatch {
        concept: SemanticId,
        expected: Ty,
        found: Ty,
    },
    /// `delay`/`sync` operand and initial value differ.
    TemporalMismatch {
        init: Ty,
        value: Ty,
    },
    /// `delay`/`sync` of a function-typed value.
    TemporalNotData {
        found: Ty,
    },
    /// `delay`/`sync` under a binder.
    TemporalUnderBinder,
    /// The realization's type is not the interface's expected type.
    RealizationMismatch {
        expected: Ty,
        found: Ty,
    },
    /// `eq` at a type that is not data (a function type): the kernel's one
    /// capability proof (`eq τ (h : τ.Data)`), checked here.
    EqualityNotData {
        found: Ty,
    },
    /// `fold f z l`: `f` is not `τ → σ → σ` for the list's `τ` and `z`'s `σ`.
    FoldMismatch {
        step: Ty,
        init: Ty,
        list: Ty,
    },
}

/// `infer Θ Δ G Γ e`.  `ctx` is the de Bruijn context, innermost first.
pub fn infer(ir: &DesignIr, grant: &Grant, ctx: &[Ty], expr: &Expr) -> Result<Ty, TypeError> {
    let mut path = Vec::new();
    infer_at(ir, grant, ctx, expr, &mut path)
}

fn err(kind: TypeErrorKind, path: &[u8]) -> TypeError {
    TypeError {
        kind,
        path: path.to_vec(),
    }
}

fn infer_at(
    ir: &DesignIr,
    grant: &Grant,
    ctx: &[Ty],
    expr: &Expr,
    path: &mut ExprPath,
) -> Result<Ty, TypeError> {
    // Descend into child `i`, restoring the path afterwards.
    fn child(
        ir: &DesignIr,
        grant: &Grant,
        ctx: &[Ty],
        e: &Expr,
        path: &mut ExprPath,
        i: u8,
    ) -> Result<Ty, TypeError> {
        path.push(i);
        let r = infer_at(ir, grant, ctx, e, path);
        path.pop();
        r
    }

    match expr {
        Expr::Var { index } => ctx
            .get(*index as usize)
            .cloned()
            .ok_or_else(|| err(TypeErrorKind::UnboundVariable { index: *index }, path)),
        Expr::BoolLit { .. } => Ok(Ty::Bool),
        Expr::NatLit { .. } => Ok(Ty::Nat),
        Expr::Lam { dom, body } => {
            let mut inner = Vec::with_capacity(ctx.len() + 1);
            inner.push(dom.clone());
            inner.extend_from_slice(ctx);
            let cod = child(ir, grant, &inner, body, path, 0)?;
            Ok(Ty::arr(dom.clone(), cod))
        }
        Expr::App { f, a } => {
            let tf = child(ir, grant, ctx, f, path, 0)?;
            let Ty::Arr { dom, cod } = tf else {
                path.push(0);
                let e = err(TypeErrorKind::ExpectedFunction { found: tf }, path);
                path.pop();
                return Err(e);
            };
            let ta = child(ir, grant, ctx, a, path, 1)?;
            if ta != *dom {
                path.push(1);
                let e = err(
                    TypeErrorKind::ArgumentMismatch {
                        expected: *dom,
                        found: ta,
                    },
                    path,
                );
                path.pop();
                return Err(e);
            }
            Ok(*cod)
        }
        Expr::DeclRef { id } => ir
            .ty_view(*id)
            .cloned()
            .ok_or_else(|| err(TypeErrorKind::UnknownDeclaration { id: *id }, path)),
        Expr::Rep { e } => {
            let te = child(ir, grant, ctx, e, path, 0)?;
            let Ty::Sem { id } = te else {
                return Err(err(TypeErrorKind::RepOfNonSemantic { found: te }, path));
            };
            ir.representation_of(id)
                .cloned()
                .ok_or_else(|| err(TypeErrorKind::UnboundRepresentation { concept: id }, path))
        }
        Expr::Mk { s, e } => {
            if !grant.permits(*s) {
                return Err(err(
                    TypeErrorKind::ConstructionNotGranted { concept: *s },
                    path,
                ));
            }
            let rep = ir
                .representation_of(*s)
                .cloned()
                .ok_or_else(|| err(TypeErrorKind::UnboundRepresentation { concept: *s }, path))?;
            let te = child(ir, grant, ctx, e, path, 0)?;
            if te != rep {
                return Err(err(
                    TypeErrorKind::ConstructionMismatch {
                        concept: *s,
                        expected: rep,
                        found: te,
                    },
                    path,
                ));
            }
            Ok(Ty::sem(*s))
        }
        Expr::Prim { p } => {
            if let bdl_ir::Prim::Eq { ty } = p {
                if !ty.is_data() {
                    return Err(err(
                        TypeErrorKind::EqualityNotData { found: ty.clone() },
                        path,
                    ));
                }
            }
            Ok(p.ty())
        }
        Expr::Fold { f, z, l } => {
            let tf = child(ir, grant, ctx, f, path, 0)?;
            let tz = child(ir, grant, ctx, z, path, 1)?;
            let tl = child(ir, grant, ctx, l, path, 2)?;
            let ok = match (&tf, &tl) {
                (Ty::Arr { dom, cod }, Ty::List { elem }) => match cod.as_ref() {
                    Ty::Arr { dom: acc, cod: res } => **dom == **elem && **acc == tz && **res == tz,
                    _ => false,
                },
                _ => false,
            };
            if !ok {
                return Err(err(
                    TypeErrorKind::FoldMismatch {
                        step: tf,
                        init: tz,
                        list: tl,
                    },
                    path,
                ));
            }
            Ok(tz)
        }
        Expr::Delay { init, e } | Expr::Sync { init, e, .. } => {
            if !ctx.is_empty() {
                return Err(err(TypeErrorKind::TemporalUnderBinder, path));
            }
            let ti = child(ir, grant, ctx, init, path, 0)?;
            let te = child(ir, grant, ctx, e, path, 1)?;
            if ti != te {
                return Err(err(
                    TypeErrorKind::TemporalMismatch {
                        init: ti,
                        value: te,
                    },
                    path,
                ));
            }
            if !ti.is_data() {
                return Err(err(TypeErrorKind::TemporalNotData { found: ti }, path));
            }
            Ok(ti)
        }
    }
}

/// Check a declaration's realization against its interface: typed in the
/// empty context under `Grant::of(expected_type)`, and equal to it.
/// `Ok(None)` for an unresolved declaration — a legal state, not an error.
pub fn check_realization(ir: &DesignIr, decl: DeclId) -> Result<Option<Ty>, TypeError> {
    let Some(d) = ir.decls.get(&decl) else {
        return Err(err(TypeErrorKind::UnknownDeclaration { id: decl }, &[]));
    };
    let Some(body) = &d.realization else {
        return Ok(None);
    };
    let expected = &d.interface.expected_type;
    let found = infer(ir, &Grant::of(expected), &[], body)?;
    if &found != expected {
        return Err(err(
            TypeErrorKind::RealizationMismatch {
                expected: expected.clone(),
                found,
            },
            &[],
        ));
    }
    Ok(Some(found))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_ir::{ConceptBinding, Declaration, Interface, Prim, Scalar};
    use bdl_model::Dim;

    fn sem(n: u64) -> SemanticId {
        SemanticId::from_raw(n)
    }
    fn decl(n: u64) -> DeclId {
        DeclId::from_raw(n)
    }

    /// Tilt : q Angle, Brightness : q 1, Motor : q Angle, Open : unbound.
    fn ir() -> DesignIr {
        let mut ir = DesignIr::default();
        for (id, name, rep) in [
            (0, "Tilt", Some(Ty::q(Dim::ANGLE))),
            (1, "Brightness", Some(Ty::q(Dim::ZERO))),
            (2, "MotorAngle", Some(Ty::q(Dim::ANGLE))),
            (3, "Open", None),
        ] {
            ir.concepts.insert(
                sem(id),
                ConceptBinding {
                    id: sem(id),
                    name: name.into(),
                    representation: rep,
                    ordered: false,
                },
            );
        }
        ir
    }

    fn lit(dim: Dim, v: f64) -> Expr {
        Expr::prim(Prim::Lit {
            dim,
            value: Scalar(v),
        })
    }

    /// λx:Tilt. mk Brightness (div (rep x) (lit angle 90))
    fn dim_by_tilt() -> Expr {
        Expr::Lam {
            dom: Ty::sem(sem(0)),
            body: Box::new(Expr::mk(
                sem(1),
                Expr::apps(
                    Expr::prim(Prim::Div {
                        d1: Dim::ANGLE,
                        d2: Dim::ANGLE,
                    }),
                    [Expr::rep(Expr::Var { index: 0 }), lit(Dim::ANGLE, 90.0)],
                ),
            )),
        }
    }

    #[test]
    fn rep_then_arithmetic_then_mk_types_to_the_signature() {
        let sig = Ty::arr(Ty::sem(sem(0)), Ty::sem(sem(1)));
        let t = infer(&ir(), &Grant::of(&sig), &[], &dim_by_tilt()).unwrap();
        assert_eq!(t, sig);
    }

    #[test]
    fn construction_requires_the_grant() {
        // λx:Tilt. mk MotorAngle (rep x) — well typed at Tilt→MotorAngle, but
        // under the grant of a Tilt→Brightness declaration it is refused.
        let hidden = Expr::Lam {
            dom: Ty::sem(sem(0)),
            body: Box::new(Expr::mk(sem(2), Expr::rep(Expr::Var { index: 0 }))),
        };
        let own = Ty::arr(Ty::sem(sem(0)), Ty::sem(sem(2)));
        assert_eq!(infer(&ir(), &Grant::of(&own), &[], &hidden).unwrap(), own);
        let other = Ty::arr(Ty::sem(sem(0)), Ty::sem(sem(1)));
        let e = infer(&ir(), &Grant::of(&other), &[], &hidden).unwrap_err();
        assert_eq!(
            e.kind,
            TypeErrorKind::ConstructionNotGranted { concept: sem(2) }
        );
        assert_eq!(e.path, vec![0]);
        // client code constructs nothing
        assert!(matches!(
            infer(
                &ir(),
                &Grant::None,
                &[],
                &Expr::mk(sem(1), lit(Dim::ZERO, 1.0))
            )
            .unwrap_err()
            .kind,
            TypeErrorKind::ConstructionNotGranted { .. }
        ));
    }

    #[test]
    fn dimensions_fall_out_of_typing() {
        // add(length) applied to a time literal
        let bad = Expr::apps(
            Expr::prim(Prim::Add { dim: Dim::LENGTH }),
            [lit(Dim::LENGTH, 1.0), lit(Dim::TIME, 1.0)],
        );
        let e = infer(&ir(), &Grant::None, &[], &bad).unwrap_err();
        assert_eq!(
            e.kind,
            TypeErrorKind::ArgumentMismatch {
                expected: Ty::q(Dim::LENGTH),
                found: Ty::q(Dim::TIME)
            }
        );
        assert_eq!(e.path, vec![1]);
        // mul produces the sum of dimensions
        let ok = Expr::apps(
            Expr::prim(Prim::Mul {
                d1: Dim::LENGTH,
                d2: Dim::TIME,
            }),
            [lit(Dim::LENGTH, 2.0), lit(Dim::TIME, 3.0)],
        );
        assert_eq!(
            infer(&ir(), &Grant::None, &[], &ok).unwrap(),
            Ty::q(Dim::LENGTH + Dim::TIME)
        );
    }

    #[test]
    fn mk_with_the_wrong_representation_is_refused() {
        let sig = Ty::arr(Ty::sem(sem(0)), Ty::sem(sem(1)));
        // returns the angle itself instead of a dimensionless level
        let wrong = Expr::Lam {
            dom: Ty::sem(sem(0)),
            body: Box::new(Expr::mk(sem(1), Expr::rep(Expr::Var { index: 0 }))),
        };
        let e = infer(&ir(), &Grant::of(&sig), &[], &wrong).unwrap_err();
        assert_eq!(
            e.kind,
            TypeErrorKind::ConstructionMismatch {
                concept: sem(1),
                expected: Ty::q(Dim::ZERO),
                found: Ty::q(Dim::ANGLE)
            }
        );
    }

    #[test]
    fn unbound_representation_is_a_precise_error() {
        let e = infer(
            &ir(),
            &Grant::All,
            &[Ty::sem(sem(3))],
            &Expr::rep(Expr::Var { index: 0 }),
        )
        .unwrap_err();
        assert_eq!(
            e.kind,
            TypeErrorKind::UnboundRepresentation { concept: sem(3) }
        );
    }

    #[test]
    fn application_and_function_errors() {
        let e = infer(
            &ir(),
            &Grant::None,
            &[],
            &Expr::app(lit(Dim::ZERO, 1.0), lit(Dim::ZERO, 1.0)),
        )
        .unwrap_err();
        assert!(matches!(e.kind, TypeErrorKind::ExpectedFunction { .. }));
        let e = infer(&ir(), &Grant::None, &[], &Expr::Var { index: 3 }).unwrap_err();
        assert_eq!(e.kind, TypeErrorKind::UnboundVariable { index: 3 });
    }

    #[test]
    fn declarations_are_typed_through_the_type_view_only() {
        let mut ir = ir();
        let sig = Ty::arr(Ty::sem(sem(0)), Ty::sem(sem(1)));
        // B is unresolved; A refers to B and is typed anyway.
        ir.decls.insert(
            decl(1),
            Declaration {
                id: decl(1),
                name: "B".into(),
                interface: Interface {
                    expected_type: sig.clone(),
                    commitments: vec![],
                },
                realization: None,
            },
        );
        ir.decls.insert(
            decl(0),
            Declaration {
                id: decl(0),
                name: "A".into(),
                interface: Interface {
                    expected_type: sig.clone(),
                    commitments: vec![],
                },
                realization: Some(Expr::decl(decl(1))),
            },
        );
        assert_eq!(check_realization(&ir, decl(0)).unwrap(), Some(sig.clone()));
        assert_eq!(check_realization(&ir, decl(1)).unwrap(), None);
        // realizing B with a garbage body changes nothing about A's typing
        ir.decls.get_mut(&decl(1)).unwrap().realization = Some(Expr::BoolLit { value: true });
        assert_eq!(check_realization(&ir, decl(0)).unwrap(), Some(sig.clone()));
        assert!(matches!(
            check_realization(&ir, decl(1)).unwrap_err().kind,
            TypeErrorKind::RealizationMismatch { .. }
        ));
    }

    #[test]
    fn delay_and_sync_type_like_the_kernel() {
        let d = Expr::delay(lit(Dim::ZERO, 0.0), lit(Dim::ZERO, 1.0));
        assert_eq!(
            infer(&ir(), &Grant::None, &[], &d).unwrap(),
            Ty::q(Dim::ZERO)
        );
        let s = Expr::sync(
            bdl_model::ClockId::from_raw(0),
            Expr::BoolLit { value: false },
            Expr::BoolLit { value: true },
        );
        assert_eq!(infer(&ir(), &Grant::None, &[], &s).unwrap(), Ty::Bool);
        let mismatch = Expr::delay(Expr::BoolLit { value: false }, lit(Dim::ZERO, 1.0));
        assert!(matches!(
            infer(&ir(), &Grant::None, &[], &mismatch).unwrap_err().kind,
            TypeErrorKind::TemporalMismatch { .. }
        ));
        let under = Expr::Lam {
            dom: Ty::Bool,
            body: Box::new(Expr::delay(
                Expr::BoolLit { value: false },
                Expr::Var { index: 0 },
            )),
        };
        assert_eq!(
            infer(&ir(), &Grant::None, &[], &under).unwrap_err().kind,
            TypeErrorKind::TemporalUnderBinder
        );
        let closure = Expr::delay(
            Expr::Lam {
                dom: Ty::Bool,
                body: Box::new(Expr::Var { index: 0 }),
            },
            Expr::Lam {
                dom: Ty::Bool,
                body: Box::new(Expr::Var { index: 0 }),
            },
        );
        assert!(matches!(
            infer(&ir(), &Grant::None, &[], &closure).unwrap_err().kind,
            TypeErrorKind::TemporalNotData { .. }
        ));
    }

    #[test]
    fn fold_types_like_the_kernel_and_equality_needs_data() {
        let q0 = Ty::q(Dim::ZERO);
        // fold (add) 0 [1, 2]  : q 0
        let list = Expr::apps(
            Expr::prim(Prim::Cons { ty: q0.clone() }),
            [
                lit(Dim::ZERO, 1.0),
                Expr::apps(
                    Expr::prim(Prim::Cons { ty: q0.clone() }),
                    [
                        lit(Dim::ZERO, 2.0),
                        Expr::prim(Prim::Nil { ty: q0.clone() }),
                    ],
                ),
            ],
        );
        let sum = Expr::fold(
            Expr::prim(Prim::Add { dim: Dim::ZERO }),
            lit(Dim::ZERO, 0.0),
            list.clone(),
        );
        assert_eq!(infer(&ir(), &Grant::None, &[], &sum).unwrap(), q0);
        // the step must consume the element type and thread the accumulator
        let bad = Expr::fold(Expr::prim(Prim::And), Expr::BoolLit { value: true }, list);
        assert!(matches!(
            infer(&ir(), &Grant::None, &[], &bad).unwrap_err().kind,
            TypeErrorKind::FoldMismatch { .. }
        ));
        // structural equality at any data type, refused at a function type
        let pair = Expr::apps(
            Expr::prim(Prim::Pair {
                fst: Ty::Bool,
                snd: q0.clone(),
            }),
            [Expr::BoolLit { value: true }, lit(Dim::ZERO, 1.0)],
        );
        let eq_pair = Expr::apps(
            Expr::prim(Prim::Eq {
                ty: Ty::prod(Ty::Bool, q0.clone()),
            }),
            [pair.clone(), pair],
        );
        assert_eq!(infer(&ir(), &Grant::None, &[], &eq_pair).unwrap(), Ty::Bool);
        let eq_fn = Expr::prim(Prim::Eq {
            ty: Ty::arr(Ty::Bool, Ty::Bool),
        });
        assert!(matches!(
            infer(&ir(), &Grant::None, &[], &eq_fn).unwrap_err().kind,
            TypeErrorKind::EqualityNotData { .. }
        ));
        // a pair of two concepts keeps both identities in its type
        let two = Expr::apps(
            Expr::prim(Prim::Pair {
                fst: Ty::sem(sem(0)),
                snd: Ty::sem(sem(1)),
            }),
            [Expr::Var { index: 0 }, Expr::Var { index: 1 }],
        );
        assert_eq!(
            infer(
                &ir(),
                &Grant::None,
                &[Ty::sem(sem(0)), Ty::sem(sem(1))],
                &two
            )
            .unwrap(),
            Ty::prod(Ty::sem(sem(0)), Ty::sem(sem(1)))
        );
    }

    proptest::proptest! {
        #[test]
        fn saturated_prim_applications_infer_their_declared_type(a in 0.0f64..1e6, b in 0.0f64..1e6) {
            let p = Prim::Mul { d1: Dim::LENGTH, d2: Dim::TIME };
            let e = Expr::apps(Expr::prim(p.clone()), [lit(Dim::LENGTH, a), lit(Dim::TIME, b)]);
            let t = infer(&ir(), &Grant::None, &[], &e).unwrap();
            let Ty::Arr { cod, .. } = p.ty() else { unreachable!() };
            let Ty::Arr { cod, .. } = *cod else { unreachable!() };
            proptest::prop_assert_eq!(t, *cod);
        }

        #[test]
        fn mk_yields_the_requested_semantic_type(v in -1e6f64..1e6) {
            let e = Expr::mk(sem(1), lit(Dim::ZERO, v));
            proptest::prop_assert_eq!(infer(&ir(), &Grant::All, &[], &e).unwrap(), Ty::sem(sem(1)));
        }
    }
}
