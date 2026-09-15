//! The kernel's `Causal`: the instantaneous dependency graph is acyclic,
//! witnessed by a rank.  A structural cycle every path of which passes
//! through a delayed/transported operand is causal and runs at every tick;
//! a cycle with an instantaneous path is not.
//!
//! Algorithm: Tarjan's SCC over the instantaneous graph (deterministic —
//! nodes and successors are visited in `DeclId` order).  Every SCC of size
//! larger than one, and every node with an instantaneous self-edge, is an
//! instantaneous cycle.  When there is none, Kahn's algorithm gives the evaluation order
//! and each node's rank (its longest instantaneous path to a source).

use crate::graph::DependencyGraph;
use bdl_diagnostics::{Diagnostic, Entity};
use bdl_ir::DesignIr;
use bdl_model::DeclId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalityAnalysis {
    pub valid: bool,
    /// Instantaneous cycles, each as its members in `DeclId` order.
    pub cycles: Vec<Vec<DeclId>>,
    /// `rank d < rank a` whenever `a` instantaneously depends on `d`.
    /// Empty when invalid.
    pub rank: BTreeMap<DeclId, u32>,
    /// A total order compatible with the ranks: an implementation artefact
    /// of the graph, never of the canvas.
    pub order: Vec<DeclId>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CausalityAnalysis {
    pub fn in_cycle(&self, d: DeclId) -> bool {
        self.cycles.iter().any(|c| c.contains(&d))
    }
}

pub fn check_causality(ir: &DesignIr, g: &DependencyGraph) -> CausalityAnalysis {
    let cycles = instantaneous_cycles(g);
    let mut diagnostics = Vec::new();
    let name = |d: DeclId| {
        ir.decls
            .get(&d)
            .map(|x| x.name.clone())
            .unwrap_or_else(|| d.to_string())
    };
    for cycle in &cycles {
        let names: Vec<String> = cycle.iter().map(|d| name(*d)).collect();
        let message = if cycle.len() == 1 {
            format!("{} depends on its own current value.", names[0])
        } else {
            format!(
                "{} depend on each other without any remembered value between them.",
                join_names(&names)
            )
        };
        for d in cycle {
            diagnostics.push(
                Diagnostic::error("reactive.instantaneous_cycle", Entity::Mapping { id: *d }, message.clone())
                    .explain(
                        "A value cannot be computed from itself in the same instant. Where the loop should \
                         carry a value from one moment to the next, say so with `previous`, or restructure \
                         the relationships so one of them no longer reads the other directly.",
                    )
                    .technical(format!(
                        "instantaneous cycle (SCC) {}",
                        cycle.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(" → ")
                    )),
            );
        }
    }
    if !cycles.is_empty() {
        bdl_diagnostics::sort_diagnostics(&mut diagnostics);
        return CausalityAnalysis {
            valid: false,
            cycles,
            rank: BTreeMap::new(),
            order: Vec::new(),
            diagnostics,
        };
    }
    let (rank, order) = ranks(g);
    CausalityAnalysis {
        valid: true,
        cycles,
        rank,
        order,
        diagnostics,
    }
}

/// Tarjan's SCC, deterministic.  Returns the non-trivial components (size > 1
/// or self-loop), each sorted, in order of their smallest member.
fn instantaneous_cycles(g: &DependencyGraph) -> Vec<Vec<DeclId>> {
    struct Tarjan<'a> {
        g: &'a DependencyGraph,
        index: u32,
        indices: BTreeMap<DeclId, u32>,
        low: BTreeMap<DeclId, u32>,
        on_stack: BTreeSet<DeclId>,
        stack: Vec<DeclId>,
        out: Vec<Vec<DeclId>>,
    }
    impl Tarjan<'_> {
        fn visit(&mut self, v: DeclId) {
            self.indices.insert(v, self.index);
            self.low.insert(v, self.index);
            self.index += 1;
            self.stack.push(v);
            self.on_stack.insert(v);
            let succ: Vec<DeclId> = self
                .g
                .instantaneous
                .get(&v)
                .into_iter()
                .flatten()
                .copied()
                .collect();
            for w in succ {
                if !self.indices.contains_key(&w) {
                    self.visit(w);
                    let lw = self.low[&w];
                    let lv = self.low[&v];
                    self.low.insert(v, lv.min(lw));
                } else if self.on_stack.contains(&w) {
                    let iw = self.indices[&w];
                    let lv = self.low[&v];
                    self.low.insert(v, lv.min(iw));
                }
            }
            if self.low[&v] == self.indices[&v] {
                let mut comp = Vec::new();
                while let Some(w) = self.stack.pop() {
                    self.on_stack.remove(&w);
                    comp.push(w);
                    if w == v {
                        break;
                    }
                }
                comp.sort();
                let self_loop = comp.len() == 1 && self.g.inst_depends_on(v, v);
                if comp.len() > 1 || self_loop {
                    self.out.push(comp);
                }
            }
        }
    }
    let mut t = Tarjan {
        g,
        index: 0,
        indices: BTreeMap::new(),
        low: BTreeMap::new(),
        on_stack: BTreeSet::new(),
        stack: Vec::new(),
        out: Vec::new(),
    };
    for v in &g.nodes {
        if !t.indices.contains_key(v) {
            t.visit(*v);
        }
    }
    t.out.sort();
    t.out
}

/// Kahn's algorithm over the (acyclic) instantaneous graph, taking ready
/// nodes in `DeclId` order.  Rank = longest path from a node with no
/// instantaneous dependencies.
fn ranks(g: &DependencyGraph) -> (BTreeMap<DeclId, u32>, Vec<DeclId>) {
    let mut remaining: BTreeMap<DeclId, usize> = g
        .nodes
        .iter()
        .map(|d| (*d, g.instantaneous.get(d).map_or(0, |s| s.len())))
        .collect();
    let mut rank: BTreeMap<DeclId, u32> = BTreeMap::new();
    let mut order = Vec::new();
    let mut ready: BTreeSet<DeclId> = remaining
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(d, _)| *d)
        .collect();
    while let Some(d) = ready.iter().next().copied() {
        ready.remove(&d);
        let r = g
            .instantaneous
            .get(&d)
            .into_iter()
            .flatten()
            .map(|dep| rank[dep] + 1)
            .max()
            .unwrap_or(0);
        rank.insert(d, r);
        order.push(d);
        for a in g.reverse_instantaneous.get(&d).into_iter().flatten() {
            let n = remaining.get_mut(a).expect("reverse edge target is a node");
            *n -= 1;
            if *n == 0 {
                ready.insert(*a);
            }
        }
    }
    debug_assert_eq!(order.len(), g.nodes.len(), "acyclic graph is fully ordered");
    (rank, order)
}

fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [a] => a.clone(),
        [a, b] => format!("{a} and {b}"),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::analyze_dependencies;
    use crate::test_designs::*;
    use bdl_ir::Expr;

    fn analysis(decls: &[(u64, Option<Expr>)]) -> CausalityAnalysis {
        let ir = ir_with_decls(decls);
        check_causality(&ir, &analyze_dependencies(&ir))
    }

    #[test]
    fn acyclic_graph_orders_by_rank_deterministically() {
        // 2 := 0 + 1 ; 1 := 0 ; 0 := input
        let a = analysis(&[
            (
                2,
                Some(Expr::apps(
                    Expr::prim(bdl_ir::Prim::Add {
                        dim: bdl_model::Dim::ZERO,
                    }),
                    [Expr::decl(d(0)), Expr::decl(d(1))],
                )),
            ),
            (1, Some(Expr::decl(d(0)))),
            (0, None),
        ]);
        assert!(a.valid);
        assert_eq!(a.order, vec![d(0), d(1), d(2)]);
        assert_eq!(a.rank[&d(0)], 0);
        assert_eq!(a.rank[&d(1)], 1);
        assert_eq!(a.rank[&d(2)], 2);
        let b = analysis(&[
            (
                2,
                Some(Expr::apps(
                    Expr::prim(bdl_ir::Prim::Add {
                        dim: bdl_model::Dim::ZERO,
                    }),
                    [Expr::decl(d(0)), Expr::decl(d(1))],
                )),
            ),
            (1, Some(Expr::decl(d(0)))),
            (0, None),
        ]);
        assert_eq!(a, b);
    }

    #[test]
    fn two_node_instantaneous_cycle_is_rejected_by_name() {
        let a = analysis(&[(0, Some(Expr::decl(d(1)))), (1, Some(Expr::decl(d(0))))]);
        assert!(!a.valid);
        assert_eq!(a.cycles, vec![vec![d(0), d(1)]]);
        assert_eq!(a.diagnostics.len(), 2);
        assert_eq!(
            a.diagnostics[0].code.as_str(),
            "reactive.instantaneous_cycle"
        );
        assert!(a.diagnostics[0]
            .message
            .contains("decl0 and decl1 depend on each other"));
        assert!(a.diagnostics[0].technical.contains("decl#0 → decl#1"));
    }

    #[test]
    fn self_instantaneous_cycle_is_rejected() {
        let a = analysis(&[(0, Some(Expr::decl(d(0))))]);
        assert!(!a.valid);
        assert_eq!(a.cycles, vec![vec![d(0)]]);
        assert!(a.diagnostics[0].message.contains("its own current value"));
    }

    #[test]
    fn cycle_through_delay_is_accepted() {
        // a := delay 0 b ; b := a
        let a = analysis(&[
            (0, Some(Expr::delay(lit(0.0), Expr::decl(d(1))))),
            (1, Some(Expr::decl(d(0)))),
        ]);
        assert!(a.valid, "{:?}", a.diagnostics);
        assert_eq!(a.order, vec![d(0), d(1)]);
        // self-delayed accumulator
        let acc = analysis(&[(0, Some(Expr::delay(lit(0.0), Expr::decl(d(0)))))]);
        assert!(acc.valid);
    }

    #[test]
    fn cycle_through_sync_transported_operand_is_accepted_but_through_init_is_not() {
        let ok = analysis(&[
            (0, Some(Expr::sync(c(0), lit(0.0), Expr::decl(d(1))))),
            (1, Some(Expr::decl(d(0)))),
        ]);
        assert!(ok.valid);
        let bad = analysis(&[
            (0, Some(Expr::sync(c(0), Expr::decl(d(1)), lit(0.0)))),
            (1, Some(Expr::decl(d(0)))),
        ]);
        assert!(!bad.valid, "the initial value is read instantaneously");
    }

    #[test]
    fn partially_delayed_cycle_is_rejected() {
        // a := delay 0 b ; b := c ; c := a  (delayed) — fine;  add c := b too → instantaneous b↔c
        let a = analysis(&[
            (0, Some(Expr::delay(lit(0.0), Expr::decl(d(1))))),
            (1, Some(Expr::decl(d(2)))),
            (2, Some(Expr::decl(d(1)))),
        ]);
        assert!(!a.valid);
        assert_eq!(a.cycles, vec![vec![d(1), d(2)]]);
    }

    #[test]
    fn unresolved_declarations_take_part() {
        let a = analysis(&[(0, None), (1, Some(Expr::decl(d(0))))]);
        assert!(a.valid);
        assert_eq!(a.order, vec![d(0), d(1)]);
    }
}
