//! The kernel's `Clocked` judgment (`Clock.lean` §2): a realization is
//! well-clocked in its declaration's domain when every reference stays in
//! that domain or is domain-agnostic, every `delay` has a domain, and every
//! `sync src` switches the domain of its transported operand to `src`.
//! Typing is blind to domains; this is the only place they are checked.
//! No transport is ever inferred.

use bdl_check::ExprPath;
use bdl_diagnostics::{Diagnostic, Entity};
use bdl_ir::{DesignIr, Expr};
use bdl_model::{ClockId, DeclId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossDomainReference {
    /// The declaration whose realization reads across.
    pub from: DeclId,
    pub from_domain: Option<ClockId>,
    /// The declaration read.
    pub to: DeclId,
    pub to_domain: Option<ClockId>,
    pub path: ExprPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockAnalysis {
    pub valid: bool,
    /// Declarations whose realization fails the judgment.
    pub ill_clocked: BTreeSet<DeclId>,
    pub crossings: Vec<CrossDomainReference>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn check_clocks(ir: &DesignIr) -> ClockAnalysis {
    let mut a = ClockAnalysis {
        valid: true,
        ill_clocked: BTreeSet::new(),
        crossings: Vec::new(),
        diagnostics: Vec::new(),
    };
    for (id, decl) in &ir.decls {
        let Some(body) = &decl.realization else {
            continue;
        };
        let own = ir.clocks.get(id).copied();
        let mut path = Vec::new();
        let mut problems = Vec::new();
        clocked(ir, *id, own, body, &mut path, &mut problems);
        if problems.is_empty() {
            continue;
        }
        a.valid = false;
        a.ill_clocked.insert(*id);
        for p in problems {
            match p {
                Problem::Cross(x) => {
                    let from_name = &decl.name;
                    let to_name = ir.decls.get(&x.to).map(|d| d.name.as_str()).unwrap_or("?");
                    let dom = |c: Option<ClockId>| match c {
                        Some(c) => ir
                            .clock_names
                            .get(&c)
                            .cloned()
                            .unwrap_or_else(|| c.to_string()),
                        None => "no domain".to_string(),
                    };
                    a.diagnostics.push(
                        Diagnostic::error(
                            "clock.cross_domain_reference",
                            Entity::Mapping { id: *id },
                            format!(
                                "{from_name} updates in a different timing domain from {to_name}. Choose how this relationship should observe the source value."
                            ),
                        )
                        .explain(format!(
                            "{to_name} moves with {}, {from_name} with {}. Either read the last value {to_name} produced — with a stated value to use before it has ever reported — or move {from_name} into {to_name}'s domain and accept its timing.",
                            dom(x.to_domain),
                            dom(x.from_domain)
                        ))
                        .technical(format!(
                            "Clocked Κ {} (declRef {}) fails: Κ = {}; needs sync {} init e",
                            dom(x.from_domain),
                            x.to,
                            dom(x.to_domain),
                            x.to_domain.map(|c| c.to_string()).unwrap_or_default()
                        )),
                    );
                    a.crossings.push(x);
                }
                Problem::TemporalWithoutDomain(path) => {
                    a.diagnostics.push(
                        Diagnostic::error(
                            "clock.temporal_without_domain",
                            Entity::Mapping { id: *id },
                            format!("{} remembers a value over time but has no timing domain.", decl.name),
                        )
                        .explain("Remembering a value means reading a domain at its previous activation; assign this relationship to a timing domain.")
                        .technical(format!("delay/sync at path {path:?} with Κ = none")),
                    );
                }
            }
        }
    }
    bdl_diagnostics::sort_diagnostics(&mut a.diagnostics);
    a
}

enum Problem {
    Cross(CrossDomainReference),
    TemporalWithoutDomain(ExprPath),
}

/// `clockedB Κ c e`, collecting every violation with its path.
fn clocked(
    ir: &DesignIr,
    owner: DeclId,
    c: Option<ClockId>,
    e: &Expr,
    path: &mut ExprPath,
    out: &mut Vec<Problem>,
) {
    match e {
        Expr::Var { .. } | Expr::BoolLit { .. } | Expr::NatLit { .. } | Expr::Prim { .. } => {}
        Expr::Lam { body, .. } => {
            path.push(0);
            clocked(ir, owner, c, body, path, out);
            path.pop();
        }
        Expr::App { f, a } => {
            path.push(0);
            clocked(ir, owner, c, f, path, out);
            path.pop();
            path.push(1);
            clocked(ir, owner, c, a, path, out);
            path.pop();
        }
        Expr::Rep { e } | Expr::Mk { e, .. } => {
            path.push(0);
            clocked(ir, owner, c, e, path, out);
            path.pop();
        }
        Expr::DeclRef { id } => {
            let target = ir.clocks.get(id).copied();
            if !(target.is_none() || target == c) {
                out.push(Problem::Cross(CrossDomainReference {
                    from: owner,
                    from_domain: c,
                    to: *id,
                    to_domain: target,
                    path: path.clone(),
                }));
            }
        }
        Expr::Delay { init, e } => match c {
            None => out.push(Problem::TemporalWithoutDomain(path.clone())),
            Some(_) => {
                path.push(0);
                clocked(ir, owner, c, init, path, out);
                path.pop();
                path.push(1);
                clocked(ir, owner, c, e, path, out);
                path.pop();
            }
        },
        Expr::Sync { src, init, e } => match c {
            None => out.push(Problem::TemporalWithoutDomain(path.clone())),
            Some(_) => {
                path.push(0);
                clocked(ir, owner, c, init, path, out);
                path.pop();
                // the transported operand lives in the source domain
                path.push(1);
                clocked(ir, owner, Some(*src), e, path, out);
                path.pop();
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_designs::*;
    use bdl_ir::Ty;

    #[test]
    fn same_domain_and_agnostic_references_are_accepted() {
        let mut ir = ir_with_decls(&[
            (0, None),
            (1, Some(Expr::decl(d(0)))),
            (2, Some(Expr::decl(d(3)))),
            (3, Some(lit(1.0))),
        ]);
        ir.clocks.remove(&d(3)); // pure, domain-agnostic
        let a = check_clocks(&ir);
        assert!(a.valid, "{:?}", a.diagnostics);
        // the agnostic declaration is usable from another domain too
        ir.clocks.insert(d(2), c(1));
        assert!(check_clocks(&ir).valid);
    }

    #[test]
    fn direct_cross_domain_reference_is_rejected_and_explained() {
        let mut ir = ir_with_decls(&[(0, None), (1, Some(Expr::decl(d(0))))]);
        ir.clocks.insert(d(0), c(1));
        ir.clock_names.insert(c(1), "ambient".into());
        let a = check_clocks(&ir);
        assert!(!a.valid);
        assert_eq!(a.ill_clocked, [d(1)].into_iter().collect());
        assert_eq!(a.crossings.len(), 1);
        assert_eq!(a.crossings[0].path, Vec::<u8>::new());
        let dg = &a.diagnostics[0];
        assert_eq!(dg.code.as_str(), "clock.cross_domain_reference");
        assert!(dg
            .message
            .contains("decl1 updates in a different timing domain from decl0"));
        assert!(dg.explanation.contains("ambient") && dg.explanation.contains("main"));
    }

    #[test]
    fn explicit_sync_is_accepted_and_its_init_is_read_locally() {
        let mut ir = ir_with_decls(&[
            (0, None),
            (1, Some(Expr::sync(c(1), lit(0.0), Expr::decl(d(0))))),
        ]);
        ir.clocks.insert(d(0), c(1));
        assert!(check_clocks(&ir).valid);
        // an init that reads the *other* domain directly is still a crossing
        ir.decls.get_mut(&d(1)).unwrap().realization =
            Some(Expr::sync(c(1), Expr::decl(d(0)), Expr::decl(d(0))));
        let a = check_clocks(&ir);
        assert!(!a.valid);
        assert_eq!(a.crossings[0].path, vec![0]);
    }

    #[test]
    fn equal_rate_but_distinct_identity_still_needs_sync() {
        // Rates are not part of this judgment at all: two domains are two
        // domains whatever schedule they will get.
        let mut ir = ir_with_decls(&[(0, None), (1, Some(Expr::decl(d(0))))]);
        ir.clocks.insert(d(0), c(7));
        assert!(!check_clocks(&ir).valid);
    }

    #[test]
    fn delay_needs_a_domain() {
        let mut ir = ir_with_decls(&[(0, Some(Expr::delay(lit(0.0), lit(1.0))))]);
        assert!(check_clocks(&ir).valid);
        ir.clocks.remove(&d(0));
        let a = check_clocks(&ir);
        assert!(!a.valid);
        assert_eq!(
            a.diagnostics[0].code.as_str(),
            "clock.temporal_without_domain"
        );
        // a pure lambda without temporal forms is fine without a domain
        ir.decls.get_mut(&d(0)).unwrap().realization = Some(Expr::Lam {
            dom: Ty::Bool,
            body: Box::new(Expr::Var { index: 0 }),
        });
        assert!(check_clocks(&ir).valid);
    }
}
