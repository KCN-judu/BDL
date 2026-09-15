//! Dependency graphs over declarations, derived once from realizations
//! (`Dependency.lean`: `DependsOn`, `InstDependsOn`).
//!
//! `A → B` when `A`'s realization references `B`.  The instantaneous graph
//! omits references under the delayed operand of `delay` and the
//! transported operand of `sync`; initial values are read at the first
//! activation and stay instantaneous.  Unresolved declarations have no
//! outgoing edges but may be depended upon.

use bdl_ir::DesignIr;
use bdl_model::DeclId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type Edges = BTreeMap<DeclId, BTreeSet<DeclId>>;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Every declaration of the design, including those with no edges.
    pub nodes: BTreeSet<DeclId>,
    /// `a → b`: `a` references `b` anywhere in its realization.
    pub all: Edges,
    /// `a → b`: `a` references `b` outside any delayed/transported operand.
    pub instantaneous: Edges,
    /// `b → {a}`: who references `b` (any reference).
    pub reverse_all: Edges,
    /// `b → {a}`: who references `b` instantaneously.
    pub reverse_instantaneous: Edges,
    /// Referenced declarations that do not exist in the design.
    pub dangling: BTreeMap<DeclId, BTreeSet<DeclId>>,
}

pub fn analyze_dependencies(ir: &DesignIr) -> DependencyGraph {
    let mut g = DependencyGraph::default();
    for (id, d) in &ir.decls {
        g.nodes.insert(*id);
        g.all.entry(*id).or_default();
        g.instantaneous.entry(*id).or_default();
        g.reverse_all.entry(*id).or_default();
        g.reverse_instantaneous.entry(*id).or_default();
        let Some(body) = &d.realization else { continue };
        for b in body.refs() {
            if !ir.decls.contains_key(&b) {
                g.dangling.entry(*id).or_default().insert(b);
                continue;
            }
            g.all.entry(*id).or_default().insert(b);
            g.reverse_all.entry(b).or_default().insert(*id);
        }
        for b in body.inst_refs() {
            if !ir.decls.contains_key(&b) {
                continue;
            }
            g.instantaneous.entry(*id).or_default().insert(b);
            g.reverse_instantaneous.entry(b).or_default().insert(*id);
        }
    }
    g
}

impl DependencyGraph {
    pub fn depends_on(&self, a: DeclId, b: DeclId) -> bool {
        self.all.get(&a).is_some_and(|s| s.contains(&b))
    }
    pub fn inst_depends_on(&self, a: DeclId, b: DeclId) -> bool {
        self.instantaneous.get(&a).is_some_and(|s| s.contains(&b))
    }
    /// Everything `a` depends on, transitively, through any edge.
    pub fn reachable(&self, a: DeclId) -> BTreeSet<DeclId> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![a];
        while let Some(x) = stack.pop() {
            for y in self.all.get(&x).into_iter().flatten() {
                if seen.insert(*y) {
                    stack.push(*y);
                }
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_designs::*;
    use bdl_ir::Expr;

    #[test]
    fn instantaneous_edges_skip_delayed_operands_but_keep_initial_values() {
        // a := delay (declRef c) (declRef b);  b := a;  c := 0
        let mut ir = ir_with_decls(&[
            (0, Some(Expr::delay(Expr::decl(d(2)), Expr::decl(d(1))))),
            (1, Some(Expr::decl(d(0)))),
            (2, Some(lit(0.0))),
        ]);
        ir.clocks.clear();
        let g = analyze_dependencies(&ir);
        assert!(g.depends_on(d(0), d(1)) && g.depends_on(d(0), d(2)));
        assert!(
            !g.inst_depends_on(d(0), d(1)),
            "the delayed operand is not instantaneous"
        );
        assert!(
            g.inst_depends_on(d(0), d(2)),
            "the initial value is instantaneous"
        );
        assert!(g.inst_depends_on(d(1), d(0)));
        assert_eq!(g.reverse_instantaneous[&d(0)], [d(1)].into_iter().collect());
        assert_eq!(g.reverse_all[&d(1)], [d(0)].into_iter().collect());
    }

    #[test]
    fn unresolved_declarations_are_nodes_that_can_be_depended_upon() {
        let ir = ir_with_decls(&[(0, None), (1, Some(Expr::decl(d(0))))]);
        let g = analyze_dependencies(&ir);
        assert!(g.nodes.contains(&d(0)));
        assert!(g.all[&d(0)].is_empty());
        assert!(g.inst_depends_on(d(1), d(0)));
    }

    #[test]
    fn dangling_references_are_recorded_not_edges() {
        let ir = ir_with_decls(&[(0, Some(Expr::decl(d(9))))]);
        let g = analyze_dependencies(&ir);
        assert!(g.all[&d(0)].is_empty());
        assert_eq!(g.dangling[&d(0)], [d(9)].into_iter().collect());
    }
}
