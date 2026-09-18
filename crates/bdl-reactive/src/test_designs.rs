//! Shared fixtures for tests: small Design IRs built directly.

use bdl_ir::{ConceptBinding, Declaration, DesignIr, Expr, Interface, Prim, Scalar, Ty};
use bdl_model::{ClockId, DeclId, Dim, SemanticId};

pub fn d(n: u64) -> DeclId {
    DeclId::from_raw(n)
}
pub fn c(n: u64) -> ClockId {
    ClockId::from_raw(n)
}
pub fn s(n: u64) -> SemanticId {
    SemanticId::from_raw(n)
}
pub fn lit(v: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim: Dim::ZERO,
        value: Scalar(v),
    })
}
pub fn lit_dim(dim: Dim, v: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim,
        value: Scalar(v),
    })
}

/// Declarations of type `q[1]`, all in clock 0, with the given realizations.
pub fn ir_with_decls(decls: &[(u64, Option<Expr>)]) -> DesignIr {
    let mut ir = DesignIr::default();
    for (n, r) in decls {
        ir.decls.insert(
            d(*n),
            Declaration {
                id: d(*n),
                name: format!("decl{n}"),
                interface: Interface {
                    expected_type: Ty::q(Dim::ZERO),
                    commitments: vec![],
                },
                realization: r.clone(),
            },
        );
        ir.clocks.insert(d(*n), c(0));
    }
    ir.clock_names.insert(c(0), "main".into());
    ir
}

/// Tilt : q[rad] (sem 0), Brightness : q[1] (sem 1).
pub fn lamp_concepts(ir: &mut DesignIr) {
    ir.concepts.insert(
        s(0),
        ConceptBinding {
            id: s(0),
            name: "Tilt".into(),
            representation: Some(Ty::q(Dim::ANGLE)),
            ordered: false,
        },
    );
    ir.concepts.insert(
        s(1),
        ConceptBinding {
            id: s(1),
            name: "Brightness".into(),
            representation: Some(Ty::q(Dim::ZERO)),
            ordered: false,
        },
    );
}

/// `[e₁, …, eₙ]` at `q[1]`.
pub fn list_lit(items: impl IntoIterator<Item = Expr>) -> Expr {
    let q0 = Ty::q(Dim::ZERO);
    items
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .fold(Expr::prim(Prim::Nil { ty: q0.clone() }), |tail, x| {
            Expr::apps(Expr::prim(Prim::Cons { ty: q0.clone() }), [x, tail])
        })
}
