//! The reference evaluator: the kernel's `MEv` made operational.
//!
//! One global tick, several domains.  For each domain active at the tick,
//! its declarations are evaluated (lazily, memoised, in causal order by
//! construction) reading only
//!
//! * inputs for unresolved declarations at this tick,
//! * values of declarations evaluated *this* tick in the *same* domain (or
//!   agnostic ones), and
//! * **state cells** committed at the end of *earlier* ticks.
//!
//! `delay init e` at path `p` in declaration `d` is the cell `(d, p)`: read
//! → the cell's value if it was ever written, else `init`; write → `e` now,
//! into the *next* state.  `sync src init e` is the same cell written when
//! `src` is active and read when the owner's domain is active — so a domain
//! active at the same tick as its source sees the source's *previous*
//! activation only, whatever order a host processes domains in.
//!
//! A tick has two phases.  **Read**: every due declaration is evaluated with
//! temporal forms yielding their cell's committed value.  **Write**: every
//! temporal site whose writing domain is active evaluates its operand (in
//! the same read mode) into the *next* state.  Reads never see writes of
//! the same tick; nothing is mutated in place.

use crate::value::Value;
use bdl_check::ExprPath;
use bdl_ir::{DesignIr, Expr, Prim};
use bdl_model::{ClockId, DeclId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Stable identity of a delay/sync cell: the declaration and the path of
/// the temporal form inside its realization.  Survives unrelated edits and
/// re-elaboration as long as the declaration and its temporal structure do.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StateCellId {
    pub decl: DeclId,
    pub path: ExprPath,
}

/// Values supplied for unresolved declarations at one tick.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TickInput {
    pub values: BTreeMap<DeclId, Value>,
}

/// Everything that survives from one tick to the next.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct State {
    /// Cells that have been written at least once.
    pub cells: BTreeMap<StateCellId, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TickOutcome {
    pub next: State,
    /// Every declaration due this tick (active domains + agnostic) and its
    /// value — an unresolved declaration's is the input it was fed, echoed
    /// so a trace reads the same way for every declaration (ADR-0033).
    pub values: BTreeMap<DeclId, Value>,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error, Serialize, Deserialize)]
pub enum RuntimeError {
    #[error("no input value for unresolved declaration {decl} at tick {tick}")]
    MissingInput { decl: DeclId, tick: u64 },
    #[error("{op} produced a non-finite number in {decl} (tick {tick})")]
    NonFinite { decl: DeclId, tick: u64, op: String },
    #[error("division by zero in {decl} (tick {tick})")]
    DivisionByZero { decl: DeclId, tick: u64 },
    #[error("unknown declaration {decl}")]
    UnknownDeclaration { decl: DeclId },
    #[error("instantaneous cycle at runtime through {decl} (causality should have rejected it)")]
    InstantaneousCycle { decl: DeclId },
    #[error("internal: {0}")]
    Internal(String),
}

/// A temporal form anywhere in the design and the domain whose activation
/// writes its cell: the owner's domain for `delay`, `src` for `sync`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct TemporalSite {
    cell: StateCellId,
    /// `None`: the owner is domain-agnostic (ill-clocked; never written).
    writer: Option<ClockId>,
}

fn temporal_sites(ir: &DesignIr) -> Vec<TemporalSite> {
    fn walk(
        owner: DeclId,
        own: Option<ClockId>,
        e: &Expr,
        path: &mut ExprPath,
        out: &mut Vec<TemporalSite>,
    ) {
        match e {
            Expr::Sync { src, init, e } => {
                out.push(TemporalSite {
                    cell: StateCellId {
                        decl: owner,
                        path: path.clone(),
                    },
                    writer: Some(*src),
                });
                path.push(0);
                walk(owner, own, init, path, out);
                path.pop();
                path.push(1);
                walk(owner, own, e, path, out);
                path.pop();
            }
            Expr::Delay { init, e } => {
                out.push(TemporalSite {
                    cell: StateCellId {
                        decl: owner,
                        path: path.clone(),
                    },
                    writer: own,
                });
                path.push(0);
                walk(owner, own, init, path, out);
                path.pop();
                path.push(1);
                walk(owner, own, e, path, out);
                path.pop();
            }
            Expr::Lam { body, .. } => {
                path.push(0);
                walk(owner, own, body, path, out);
                path.pop();
            }
            Expr::App { f, a } => {
                path.push(0);
                walk(owner, own, f, path, out);
                path.pop();
                path.push(1);
                walk(owner, own, a, path, out);
                path.pop();
            }
            Expr::Rep { e } | Expr::Mk { e, .. } => {
                path.push(0);
                walk(owner, own, e, path, out);
                path.pop();
            }
            Expr::Fold { f, z, l } => {
                for (i, sub) in [(0u8, f), (1, z), (2, l)] {
                    path.push(i);
                    walk(owner, own, sub, path, out);
                    path.pop();
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for (id, d) in &ir.decls {
        if let Some(b) = &d.realization {
            walk(
                *id,
                ir.clocks.get(id).copied(),
                b,
                &mut Vec::new(),
                &mut out,
            );
        }
    }
    out
}

/// Which declarations are evaluated at a tick: those whose domain is active,
/// and domain-agnostic ones whenever anything is active (or every tick in
/// a design with no domains at all).
pub fn evaluated_this_tick(ir: &DesignIr, active: &BTreeSet<ClockId>) -> BTreeSet<DeclId> {
    let anything = !active.is_empty() || ir.clocks.is_empty();
    ir.decls
        .keys()
        .filter(|d| match ir.clocks.get(d) {
            Some(c) => active.contains(c),
            None => anything,
        })
        .copied()
        .collect()
}

/// Evaluate one global tick.  Pure: `state` is read only; the outcome holds
/// the next state.
pub fn step(
    ir: &DesignIr,
    tick: u64,
    active: &BTreeSet<ClockId>,
    state: &State,
    input: &TickInput,
) -> Result<TickOutcome, RuntimeError> {
    let order: Vec<DeclId> = evaluated_this_tick(ir, active).into_iter().collect();
    step_in_order(ir, tick, active, state, input, &order)
}

/// [`step`] with an explicit host processing order for the declarations of
/// the active domains.  The result does not depend on it — that is the
/// property the tests check — but a runtime may pass whatever order its
/// scheduler produced.
pub fn step_in_order(
    ir: &DesignIr,
    tick: u64,
    active: &BTreeSet<ClockId>,
    state: &State,
    input: &TickInput,
    order: &[DeclId],
) -> Result<TickOutcome, RuntimeError> {
    let mut cx = Cx {
        ir,
        tick,
        prev: state,
        input,
        memo: BTreeMap::new(),
        in_progress: BTreeSet::new(),
        next: state.cells.clone(),
    };
    // Read phase.  Laziness + memo makes the host order irrelevant to the
    // result (causality guarantees termination); every declaration due this
    // tick is evaluated even if the order omits it.
    let due = evaluated_this_tick(ir, active);
    for d in order.iter().filter(|d| due.contains(d)).chain(due.iter()) {
        cx.decl_value(*d)?;
    }
    // Write phase: every temporal site whose writing domain is active
    // evaluates its operand — in read mode, so nested temporal forms still
    // see committed state — into the next state.
    for site in temporal_sites(ir) {
        if !site.writer.is_some_and(|w| active.contains(&w)) {
            continue;
        }
        let Some(body) = ir.realization_of(site.cell.decl) else {
            continue;
        };
        let operand = match at_path(body, &site.cell.path) {
            Some(Expr::Sync { e, .. }) | Some(Expr::Delay { e, .. }) => e,
            _ => {
                return Err(RuntimeError::Internal(format!(
                    "temporal site {:?} not found",
                    site.cell
                )))
            }
        };
        let mut p = site.cell.path.clone();
        p.push(1);
        let v = cx.eval(site.cell.decl, &p, operand, &[])?;
        cx.next.insert(site.cell.clone(), v);
    }
    Ok(TickOutcome {
        next: State { cells: cx.next },
        values: cx.memo,
    })
}

fn at_path<'a>(mut e: &'a Expr, path: &[u8]) -> Option<&'a Expr> {
    for &i in path {
        e = match (e, i) {
            (Expr::Lam { body, .. }, 0) => body,
            (Expr::App { f, .. }, 0) => f,
            (Expr::App { a, .. }, 1) => a,
            (Expr::Rep { e }, 0) | (Expr::Mk { e, .. }, 0) => e,
            (Expr::Delay { init, .. }, 0) | (Expr::Sync { init, .. }, 0) => init,
            (Expr::Delay { e, .. }, 1) | (Expr::Sync { e, .. }, 1) => e,
            (Expr::Fold { f, .. }, 0) => f,
            (Expr::Fold { z, .. }, 1) => z,
            (Expr::Fold { l, .. }, 2) => l,
            _ => return None,
        };
    }
    Some(e)
}

struct Cx<'a> {
    ir: &'a DesignIr,
    tick: u64,
    prev: &'a State,
    input: &'a TickInput,
    memo: BTreeMap<DeclId, Value>,
    in_progress: BTreeSet<DeclId>,
    next: BTreeMap<StateCellId, Value>,
}

impl Cx<'_> {
    /// `declRef d`: input for unresolved, memoised body otherwise.
    fn decl_value(&mut self, d: DeclId) -> Result<Value, RuntimeError> {
        if let Some(v) = self.memo.get(&d) {
            return Ok(v.clone());
        }
        let decl = self
            .ir
            .decls
            .get(&d)
            .ok_or(RuntimeError::UnknownDeclaration { decl: d })?;
        let Some(body) = &decl.realization else {
            let v = self
                .input
                .values
                .get(&d)
                .cloned()
                .ok_or(RuntimeError::MissingInput {
                    decl: d,
                    tick: self.tick,
                })?;
            self.memo.insert(d, v.clone());
            return Ok(v);
        };
        if !self.in_progress.insert(d) {
            return Err(RuntimeError::InstantaneousCycle { decl: d });
        }
        let v = self.eval(d, &[], body, &[])?;
        self.in_progress.remove(&d);
        self.memo.insert(d, v.clone());
        Ok(v)
    }

    /// Evaluate `e` belonging to declaration `owner` at `path`, in de Bruijn
    /// environment `env` (innermost first).
    fn eval(
        &mut self,
        owner: DeclId,
        path: &[u8],
        e: &Expr,
        env: &[Value],
    ) -> Result<Value, RuntimeError> {
        let mut p = path.to_vec();
        match e {
            Expr::Var { index } => env
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| RuntimeError::Internal(format!("unbound variable {index}"))),
            Expr::BoolLit { value } => Ok(Value::Bool { value: *value }),
            Expr::NatLit { value } => Ok(Value::Nat { value: *value }),
            Expr::Lam { body, .. } => Ok(Value::Closure {
                env: env.to_vec(),
                body: (**body).clone(),
            }),
            Expr::App { f, a } => {
                p.push(0);
                let fv = self.eval(owner, &p, f, env)?;
                p.pop();
                p.push(1);
                let av = self.eval(owner, &p, a, env)?;
                p.pop();
                self.apply(owner, fv, av)
            }
            // `fold f z [x₁, …, xₙ] = f x₁ (… (f xₙ z))`, computed by finite
            // iteration from the last element: never a stack of `n` frames
            // and never more steps than the list has elements.
            Expr::Fold { f, z, l } => {
                p.push(0);
                let fv = self.eval(owner, &p, f, env)?;
                p.pop();
                p.push(1);
                let zv = self.eval(owner, &p, z, env)?;
                p.pop();
                p.push(2);
                let lv = self.eval(owner, &p, l, env)?;
                p.pop();
                let Value::List { items } = lv else {
                    return Err(RuntimeError::Internal(format!(
                        "fold over a non-list {lv:?}"
                    )));
                };
                let mut acc = zv;
                for x in items.iter_from_last() {
                    let step = self.apply(owner, fv.clone(), x.clone())?;
                    acc = self.apply(owner, step, acc)?;
                }
                Ok(acc)
            }
            Expr::DeclRef { id } => self.decl_value(*id),
            Expr::Rep { e } => {
                p.push(0);
                let v = self.eval(owner, &p, e, env)?;
                match v {
                    Value::Semantic { repr, .. } => Ok(*repr),
                    other => Err(RuntimeError::Internal(format!(
                        "rep of non-semantic {other:?}"
                    ))),
                }
            }
            Expr::Mk { s, e } => {
                p.push(0);
                let v = self.eval(owner, &p, e, env)?;
                Ok(Value::sem(*s, v))
            }
            Expr::Prim { p: prim } => self.apply_prim(owner, prim.clone(), Vec::new()),
            // Read mode: the committed value of the cell, else the initial
            // value.  Writes happen in `step`'s write phase.
            Expr::Delay { init, .. } | Expr::Sync { init, .. } => {
                let cell = StateCellId {
                    decl: owner,
                    path: p.clone(),
                };
                match self.prev.cells.get(&cell) {
                    Some(v) => Ok(v.clone()),
                    None => {
                        p.push(0);
                        self.eval(owner, &p, init, env)
                    }
                }
            }
        }
    }

    /// Apply a function value to an argument: enter a closure's body, or
    /// extend a partial primitive.
    fn apply(&mut self, owner: DeclId, fv: Value, av: Value) -> Result<Value, RuntimeError> {
        match fv {
            Value::Closure { env: cenv, body } => {
                let mut inner = Vec::with_capacity(cenv.len() + 1);
                inner.push(av);
                inner.extend(cenv);
                // A closure body has no temporal forms (typing), so its
                // path is irrelevant to state.
                self.eval(owner, &[], &body, &inner)
            }
            Value::Prim { prim, mut args } => {
                args.push(av);
                self.apply_prim(owner, prim, args)
            }
            other => Err(RuntimeError::Internal(format!(
                "applied a non-function {other:?}"
            ))),
        }
    }

    fn apply_prim(
        &self,
        owner: DeclId,
        prim: Prim,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if args.len() < prim.arity() {
            return Ok(Value::Prim { prim, args });
        }
        let tick = self.tick;
        let non_finite = |op: &str| RuntimeError::NonFinite {
            decl: owner,
            tick,
            op: op.to_owned(),
        };
        let q = |v: &Value| {
            v.as_quantity()
                .ok_or_else(|| RuntimeError::Internal(format!("expected a quantity, got {v:?}")))
        };
        let b = |v: &Value| {
            v.as_bool()
                .ok_or_else(|| RuntimeError::Internal(format!("expected a boolean, got {v:?}")))
        };
        let finite = |dim, x: f64, op: &str| {
            if x.is_finite() {
                Ok(Value::q(dim, x))
            } else {
                Err(non_finite(op))
            }
        };
        Ok(match (&prim, args.as_slice()) {
            (Prim::Lit { dim, value }, []) => Value::q(*dim, value.0),
            (Prim::Add { dim }, [a, c]) => finite(*dim, q(a)?.1 + q(c)?.1, "add")?,
            (Prim::Sub { dim }, [a, c]) => finite(*dim, q(a)?.1 - q(c)?.1, "sub")?,
            (Prim::Mul { d1, d2 }, [a, c]) => finite(*d1 + *d2, q(a)?.1 * q(c)?.1, "mul")?,
            (Prim::Div { d1, d2 }, [a, c]) => {
                let (x, y) = (q(a)?.1, q(c)?.1);
                if y == 0.0 {
                    return Err(RuntimeError::DivisionByZero { decl: owner, tick });
                }
                finite(*d1 - *d2, x / y, "div")?
            }
            (Prim::Lt { .. }, [a, c]) => Value::boolean(q(a)?.1 < q(c)?.1),
            (Prim::Eq { .. }, [a, c]) => Value::boolean(a.structurally_equal(c)),
            (Prim::Not, [a]) => Value::boolean(!b(a)?),
            (Prim::And, [a, c]) => Value::boolean(b(a)? && b(c)?),
            (Prim::Or, [a, c]) => Value::boolean(b(a)? || b(c)?),
            (Prim::Ite { .. }, [c, x, y]) => {
                if b(c)? {
                    x.clone()
                } else {
                    y.clone()
                }
            }
            (Prim::None { .. }, []) => Value::None,
            (Prim::Some { .. }, [x]) => Value::some(x.clone()),
            (Prim::IsSome { .. }, [x]) => Value::boolean(matches!(x, Value::Some { .. })),
            (Prim::GetD { .. }, [x, dflt]) => match x {
                Value::Some { value } => (**value).clone(),
                _ => dflt.clone(),
            },
            (Prim::Nil { .. }, []) => Value::list([]),
            (Prim::Cons { .. }, [x, Value::List { items }]) => Value::List {
                items: crate::value::List::cons(x.clone(), items.clone()),
            },
            (Prim::Length { .. }, [Value::List { items }]) => Value::scalar(items.len() as f64),
            (Prim::Take { .. }, [k, Value::List { items }]) => Value::List {
                items: items.take(count(q(k)?.1)),
            },
            (Prim::Drop { .. }, [k, Value::List { items }]) => Value::List {
                items: items.drop(count(q(k)?.1)),
            },
            (Prim::Reverse { .. }, [Value::List { items }]) => Value::List {
                items: items.reversed(),
            },
            (Prim::Head { .. }, [Value::List { items }]) => match items.first() {
                Some(x) => Value::some(x.clone()),
                None => Value::None,
            },
            (Prim::ToList { .. }, [Value::Some { value }]) => Value::list([(**value).clone()]),
            (Prim::ToList { .. }, [Value::None]) => Value::list([]),
            (Prim::Pair { .. }, [a, c]) => Value::pair(a.clone(), c.clone()),
            (Prim::Fst { .. }, [Value::Pair { fst, .. }]) => (**fst).clone(),
            (Prim::Snd { .. }, [Value::Pair { snd, .. }]) => (**snd).clone(),
            _ => {
                return Err(RuntimeError::Internal(format!(
                    "ill-shaped primitive application {prim:?} {args:?}"
                )))
            }
        })
    }
}

/// A list position from a dimensionless quantity: the kernel's `Nat` is a
/// count; production's `f64` (ADR-0011) is read as a whole number, rounding
/// towards zero, and never below zero.  `take 2.7 xs` is `take 2 xs`.
pub fn count(k: f64) -> usize {
    if k.is_finite() && k > 0.0 {
        k.trunc() as usize
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_designs::*;
    use bdl_ir::Ty;
    use bdl_model::Dim;

    fn active(cs: &[u64]) -> BTreeSet<ClockId> {
        cs.iter().map(|n| c(*n)).collect()
    }

    fn run(ir: &DesignIr, ticks: &[(TickInput, BTreeSet<ClockId>)]) -> Vec<TickOutcome> {
        let mut state = State::default();
        let mut out = Vec::new();
        for (t, (input, act)) in ticks.iter().enumerate() {
            let o = step(ir, t as u64, act, &state, input).unwrap();
            state = o.next.clone();
            out.push(o);
        }
        out
    }

    fn input(pairs: &[(u64, Value)]) -> TickInput {
        TickInput {
            values: pairs.iter().map(|(n, v)| (d(*n), v.clone())).collect(),
        }
    }

    #[test]
    fn literals_arithmetic_and_dimensions() {
        let e = Expr::apps(
            Expr::prim(Prim::Mul {
                d1: Dim::LENGTH,
                d2: Dim::TIME,
            }),
            [lit_dim(Dim::LENGTH, 2.0), lit_dim(Dim::TIME, 3.5)],
        );
        let ir = ir_with_decls(&[(0, Some(e))]);
        let o = run(&ir, &[(input(&[]), active(&[0]))]);
        assert_eq!(o[0].values[&d(0)], Value::q(Dim::LENGTH + Dim::TIME, 7.0));
    }

    #[test]
    fn booleans_comparisons_options_and_if() {
        let cmp = Expr::apps(
            Expr::prim(Prim::Lt { dim: Dim::ZERO }),
            [lit(1.0), lit(2.0)],
        );
        let ite = Expr::apps(
            Expr::prim(Prim::Ite {
                ty: Ty::q(Dim::ZERO),
            }),
            [Expr::app(Expr::prim(Prim::Not), cmp), lit(10.0), lit(20.0)],
        );
        let opt = Expr::apps(
            Expr::prim(Prim::GetD {
                ty: Ty::q(Dim::ZERO),
            }),
            [
                Expr::app(
                    Expr::prim(Prim::Some {
                        ty: Ty::q(Dim::ZERO),
                    }),
                    lit(5.0),
                ),
                lit(0.0),
            ],
        );
        let none = Expr::apps(
            Expr::prim(Prim::GetD {
                ty: Ty::q(Dim::ZERO),
            }),
            [
                Expr::prim(Prim::None {
                    ty: Ty::q(Dim::ZERO),
                }),
                lit(9.0),
            ],
        );
        let ir = ir_with_decls(&[(0, Some(ite)), (1, Some(opt)), (2, Some(none))]);
        let o = run(&ir, &[(input(&[]), active(&[0]))]);
        assert_eq!(o[0].values[&d(0)], Value::scalar(20.0));
        assert_eq!(o[0].values[&d(1)], Value::scalar(5.0));
        assert_eq!(o[0].values[&d(2)], Value::scalar(9.0));
    }

    #[test]
    fn lists_pairs_and_the_recursor() {
        let q0 = Ty::q(Dim::ZERO);
        // sum := fold add 0 [1, 2, 3]  — foldr: 1 + (2 + (3 + 0))
        let sum = Expr::fold(
            Expr::prim(Prim::Add { dim: Dim::ZERO }),
            lit(0.0),
            list_lit([lit(1.0), lit(2.0), lit(3.0)]),
        );
        // order := fold (λx. λacc. cons x acc) [] [1, 2, 3] — a copy, in order
        let copy = Expr::fold(
            Expr::lam(
                q0.clone(),
                Expr::lam(
                    Ty::list(q0.clone()),
                    Expr::apps(
                        Expr::prim(Prim::Cons { ty: q0.clone() }),
                        [Expr::var(1), Expr::var(0)],
                    ),
                ),
            ),
            Expr::prim(Prim::Nil { ty: q0.clone() }),
            list_lit([lit(1.0), lit(2.0), lit(3.0)]),
        );
        // empty := fold add 7 []  — the initial value
        let empty = Expr::fold(
            Expr::prim(Prim::Add { dim: Dim::ZERO }),
            lit(7.0),
            list_lit([]),
        );
        // ops := (length, take 2, drop 2, reverse, head, head [])
        let xs = list_lit([lit(1.0), lit(2.0), lit(3.0)]);
        let length = Expr::app(Expr::prim(Prim::Length { ty: q0.clone() }), xs.clone());
        let take = Expr::apps(
            Expr::prim(Prim::Take { ty: q0.clone() }),
            [lit(2.0), xs.clone()],
        );
        let drop = Expr::apps(
            Expr::prim(Prim::Drop { ty: q0.clone() }),
            [lit(2.0), xs.clone()],
        );
        let rev = Expr::app(Expr::prim(Prim::Reverse { ty: q0.clone() }), xs.clone());
        let head = Expr::app(Expr::prim(Prim::Head { ty: q0.clone() }), xs.clone());
        let head_nil = Expr::app(Expr::prim(Prim::Head { ty: q0.clone() }), list_lit([]));
        let to_list = Expr::app(
            Expr::prim(Prim::ToList { ty: q0.clone() }),
            Expr::app(Expr::prim(Prim::Some { ty: q0.clone() }), lit(4.0)),
        );
        let pair = Expr::apps(
            Expr::prim(Prim::Pair {
                fst: q0.clone(),
                snd: Ty::Bool,
            }),
            [lit(5.0), Expr::BoolLit { value: true }],
        );
        let fst = Expr::app(
            Expr::prim(Prim::Fst {
                fst: q0.clone(),
                snd: Ty::Bool,
            }),
            pair.clone(),
        );
        let snd = Expr::app(
            Expr::prim(Prim::Snd {
                fst: q0.clone(),
                snd: Ty::Bool,
            }),
            pair.clone(),
        );
        let ir = ir_with_decls(&[
            (0, Some(sum)),
            (1, Some(copy)),
            (2, Some(empty)),
            (3, Some(length)),
            (4, Some(take)),
            (5, Some(drop)),
            (6, Some(rev)),
            (7, Some(head)),
            (8, Some(head_nil)),
            (9, Some(to_list)),
            (10, Some(pair)),
            (11, Some(fst)),
            (12, Some(snd)),
        ]);
        let o = run(&ir, &[(input(&[]), active(&[0]))]);
        let v = |n: u64| o[0].values[&d(n)].clone();
        let q = |x: f64| Value::scalar(x);
        assert_eq!(v(0), q(6.0));
        assert_eq!(v(1), Value::list([q(1.0), q(2.0), q(3.0)]));
        assert_eq!(v(2), q(7.0));
        assert_eq!(v(3), q(3.0));
        assert_eq!(v(4), Value::list([q(1.0), q(2.0)]));
        assert_eq!(v(5), Value::list([q(3.0)]));
        assert_eq!(v(6), Value::list([q(3.0), q(2.0), q(1.0)]));
        assert_eq!(v(7), Value::some(q(1.0)));
        assert_eq!(v(8), Value::None);
        assert_eq!(v(9), Value::list([q(4.0)]));
        assert_eq!(v(10), Value::pair(q(5.0), Value::boolean(true)));
        assert_eq!(v(11), q(5.0));
        assert_eq!(v(12), Value::boolean(true));
    }

    #[test]
    fn fold_is_finite_iteration_not_a_stack() {
        // a 100 000-element input list folds without a frame per element
        let sum = Expr::fold(
            Expr::prim(Prim::Add { dim: Dim::ZERO }),
            lit(0.0),
            Expr::decl(d(0)),
        );
        let mut ir = ir_with_decls(&[(0, None), (1, Some(sum))]);
        ir.decls.get_mut(&d(0)).unwrap().interface.expected_type = Ty::list(Ty::q(Dim::ZERO));
        let xs = Value::list((0..100_000).map(|_| Value::scalar(1.0)));
        let o = run(&ir, &[(input(&[(0, xs)]), active(&[0]))]);
        assert_eq!(o[0].values[&d(1)], Value::scalar(100_000.0));
    }

    #[test]
    fn structural_equality_on_data_and_no_order() {
        let q0 = Ty::q(Dim::ZERO);
        let eq = |ty: Ty, a: Expr, b: Expr| Expr::apps(Expr::prim(Prim::Eq { ty }), [a, b]);
        let pair = |a: f64, b: f64| {
            Expr::apps(
                Expr::prim(Prim::Pair {
                    fst: q0.clone(),
                    snd: q0.clone(),
                }),
                [lit(a), lit(b)],
            )
        };
        let some = |a: f64| Expr::app(Expr::prim(Prim::Some { ty: q0.clone() }), lit(a));
        let ir = ir_with_decls(&[
            (
                0,
                Some(eq(
                    Ty::list(q0.clone()),
                    list_lit([lit(1.0), lit(2.0)]),
                    list_lit([lit(1.0), lit(2.0)]),
                )),
            ),
            (
                1,
                Some(eq(
                    Ty::list(q0.clone()),
                    list_lit([lit(1.0), lit(2.0)]),
                    list_lit([lit(2.0), lit(1.0)]),
                )),
            ),
            (
                2,
                Some(eq(
                    Ty::prod(q0.clone(), q0.clone()),
                    pair(1.0, 2.0),
                    pair(1.0, 2.0),
                )),
            ),
            (
                3,
                Some(eq(
                    Ty::prod(q0.clone(), q0.clone()),
                    pair(1.0, 2.0),
                    pair(2.0, 1.0),
                )),
            ),
            (4, Some(eq(Ty::opt(q0.clone()), some(1.0), some(1.0)))),
            (
                5,
                Some(eq(
                    Ty::opt(q0.clone()),
                    some(1.0),
                    Expr::prim(Prim::None { ty: q0.clone() }),
                )),
            ),
            (
                6,
                Some(eq(
                    Ty::Bool,
                    Expr::BoolLit { value: false },
                    Expr::BoolLit { value: false },
                )),
            ),
        ]);
        let o = run(&ir, &[(input(&[]), active(&[0]))]);
        let b = |n: u64| o[0].values[&d(n)].as_bool().unwrap();
        assert!(b(0) && !b(1) && b(2) && !b(3) && b(4) && !b(5) && b(6));
        // semantic values compare by concept and representation
        assert!(Value::sem(s(0), Value::scalar(1.0))
            .structurally_equal(&Value::sem(s(0), Value::scalar(1.0))));
        assert!(!Value::sem(s(0), Value::scalar(1.0))
            .structurally_equal(&Value::sem(s(1), Value::scalar(1.0))));
        // no order anywhere but on quantities: `Lt` is dimension-indexed
        assert!(matches!(
            bdl_ir::Prim::Lt { dim: Dim::ZERO }.ty(),
            Ty::Arr { dom, .. } if matches!(*dom, Ty::Q { .. })
        ));
    }

    #[test]
    fn lists_and_pairs_are_delayed_and_transported_like_any_data() {
        // fast (c0): log := cons x (delay [] log)   — the 9a accumulator
        // slow (c1): seen := sync c0 [] log         — strictly before
        let q0 = Ty::q(Dim::ZERO);
        let nil = Expr::prim(Prim::Nil { ty: q0.clone() });
        let mut ir = ir_with_decls(&[
            (0, None),
            (
                1,
                Some(Expr::apps(
                    Expr::prim(Prim::Cons { ty: q0.clone() }),
                    [Expr::decl(d(0)), Expr::delay(nil.clone(), Expr::decl(d(1)))],
                )),
            ),
            (2, Some(Expr::sync(c(0), nil, Expr::decl(d(1))))),
        ]);
        ir.clocks.insert(d(2), c(1));
        let ticks: Vec<_> = [1.0, 2.0, 3.0]
            .into_iter()
            .map(|v| (input(&[(0, Value::scalar(v))]), active(&[0, 1])))
            .collect();
        let o = run(&ir, &ticks);
        let q = |x: f64| Value::scalar(x);
        assert_eq!(o[0].values[&d(1)], Value::list([q(1.0)]));
        assert_eq!(o[2].values[&d(1)], Value::list([q(3.0), q(2.0), q(1.0)]));
        // the slow domain sees the log of the previous fast activation only
        assert_eq!(o[0].values[&d(2)], Value::list([]));
        assert_eq!(o[1].values[&d(2)], Value::list([q(1.0)]));
        assert_eq!(o[2].values[&d(2)], Value::list([q(2.0), q(1.0)]));
        // a pair delayed: (x, previous x)
        let pair = Expr::apps(
            Expr::prim(Prim::Pair {
                fst: q0.clone(),
                snd: q0.clone(),
            }),
            [Expr::decl(d(0)), Expr::delay(lit(0.0), Expr::decl(d(0)))],
        );
        let ir = ir_with_decls(&[(0, None), (1, Some(pair))]);
        let ticks: Vec<_> = [1.0, 2.0]
            .into_iter()
            .map(|v| (input(&[(0, Value::scalar(v))]), active(&[0])))
            .collect();
        let o = run(&ir, &ticks);
        assert_eq!(o[1].values[&d(1)], Value::pair(q(2.0), q(1.0)));
    }

    #[test]
    fn closures_apply_and_keep_semantic_identity() {
        let mut ir = ir_with_decls(&[(0, None)]);
        lamp_concepts(&mut ir);
        // dimByTilt = λx:Tilt. mk Brightness (div (rep x) 90deg);  brightness = dimByTilt tilt
        let f = Expr::Lam {
            dom: Ty::sem(s(0)),
            body: Box::new(Expr::mk(
                s(1),
                Expr::apps(
                    Expr::prim(Prim::Div {
                        d1: Dim::ANGLE,
                        d2: Dim::ANGLE,
                    }),
                    [Expr::rep(Expr::Var { index: 0 }), lit_dim(Dim::ANGLE, 90.0)],
                ),
            )),
        };
        ir.decls.get_mut(&d(0)).unwrap().interface.expected_type = Ty::sem(s(0));
        ir.decls.insert(
            d(1),
            bdl_ir::Declaration {
                id: d(1),
                name: "dimByTilt".into(),
                interface: bdl_ir::Interface {
                    expected_type: Ty::arr(Ty::sem(s(0)), Ty::sem(s(1))),
                    commitments: vec![],
                },
                realization: Some(f),
            },
        );
        ir.decls.insert(
            d(2),
            bdl_ir::Declaration {
                id: d(2),
                name: "brightness".into(),
                interface: bdl_ir::Interface {
                    expected_type: Ty::sem(s(1)),
                    commitments: vec![],
                },
                realization: Some(Expr::app(Expr::decl(d(1)), Expr::decl(d(0)))),
            },
        );
        ir.clocks.insert(d(2), c(0));
        let o = run(
            &ir,
            &[(
                input(&[(0, Value::sem(s(0), Value::q(Dim::ANGLE, 45.0)))]),
                active(&[0]),
            )],
        );
        assert_eq!(o[0].values[&d(2)], Value::sem(s(1), Value::scalar(0.5)));
        assert!(matches!(o[0].values[&d(1)], Value::Closure { .. }));
    }

    #[test]
    fn delay_reads_previous_activation_and_never_the_same_tick() {
        // prev := delay 0 x ; x := input
        let ir = ir_with_decls(&[
            (0, None),
            (1, Some(Expr::delay(lit(0.0), Expr::decl(d(0))))),
        ]);
        let ticks: Vec<_> = [10.0, 20.0, 30.0]
            .into_iter()
            .map(|v| (input(&[(0, Value::scalar(v))]), active(&[0])))
            .collect();
        let o = run(&ir, &ticks);
        assert_eq!(o[0].values[&d(1)], Value::scalar(0.0));
        assert_eq!(o[1].values[&d(1)], Value::scalar(10.0));
        assert_eq!(o[2].values[&d(1)], Value::scalar(20.0));
        // the cell is addressed by (decl, path) and lives in the committed state
        assert_eq!(
            o[2].next.cells[&StateCellId {
                decl: d(1),
                path: vec![]
            }],
            Value::scalar(30.0)
        );
    }

    #[test]
    fn delayed_cycle_runs_as_an_accumulator() {
        // a := delay 0 b ; b := a + 1
        let ir = ir_with_decls(&[
            (0, Some(Expr::delay(lit(0.0), Expr::decl(d(1))))),
            (
                1,
                Some(Expr::apps(
                    Expr::prim(Prim::Add { dim: Dim::ZERO }),
                    [Expr::decl(d(0)), lit(1.0)],
                )),
            ),
        ]);
        let ticks: Vec<_> = (0..4).map(|_| (input(&[]), active(&[0]))).collect();
        let o = run(&ir, &ticks);
        let bs: Vec<f64> = o
            .iter()
            .map(|t| t.values[&d(1)].as_quantity().unwrap().1)
            .collect();
        assert_eq!(bs, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn missing_input_is_a_structured_error_not_a_default() {
        let ir = ir_with_decls(&[(0, None)]);
        let e = step(&ir, 0, &active(&[0]), &State::default(), &input(&[])).unwrap_err();
        assert_eq!(
            e,
            RuntimeError::MissingInput {
                decl: d(0),
                tick: 0
            }
        );
    }

    #[test]
    fn division_by_zero_and_non_finite_fail_the_tick() {
        let ir = ir_with_decls(&[(
            0,
            Some(Expr::apps(
                Expr::prim(Prim::Div {
                    d1: Dim::ZERO,
                    d2: Dim::ZERO,
                }),
                [lit(1.0), lit(0.0)],
            )),
        )]);
        assert_eq!(
            step(&ir, 0, &active(&[0]), &State::default(), &input(&[])).unwrap_err(),
            RuntimeError::DivisionByZero {
                decl: d(0),
                tick: 0
            }
        );
        let ir = ir_with_decls(&[(
            0,
            Some(Expr::apps(
                Expr::prim(Prim::Mul {
                    d1: Dim::ZERO,
                    d2: Dim::ZERO,
                }),
                [lit(1e308), lit(1e308)],
            )),
        )]);
        assert!(matches!(
            step(&ir, 0, &active(&[0]), &State::default(), &input(&[])).unwrap_err(),
            RuntimeError::NonFinite { .. }
        ));
    }

    #[test]
    fn sync_reads_the_source_strictly_before_the_current_tick() {
        // fast (c0): x := input;   slow (c1): y := sync c0 (-1) x
        let mut ir = ir_with_decls(&[
            (0, None),
            (1, Some(Expr::sync(c(0), lit(-1.0), Expr::decl(d(0))))),
        ]);
        ir.clocks.insert(d(1), c(1));
        // both active every tick: y sees x from the previous tick, never this one
        let ticks: Vec<_> = [1.0, 2.0, 3.0]
            .into_iter()
            .map(|v| (input(&[(0, Value::scalar(v))]), active(&[0, 1])))
            .collect();
        let o = run(&ir, &ticks);
        let ys: Vec<f64> = o
            .iter()
            .map(|t| t.values[&d(1)].as_quantity().unwrap().1)
            .collect();
        assert_eq!(ys, vec![-1.0, 1.0, 2.0]);
        // slow domain active only at tick 2: it sees x from tick 1 (the last source activation before)
        let ticks = vec![
            (input(&[(0, Value::scalar(1.0))]), active(&[0])),
            (input(&[(0, Value::scalar(2.0))]), active(&[0])),
            (input(&[(0, Value::scalar(3.0))]), active(&[0, 1])),
        ];
        let o = run(&ir, &ticks);
        assert!(!o[0].values.contains_key(&d(1)));
        assert_eq!(o[2].values[&d(1)], Value::scalar(2.0));
    }

    proptest::proptest! {
        #[test]
        fn same_tick_domains_are_order_independent_and_deterministic(xs in proptest::collection::vec(-100.0f64..100.0, 1..8)) {
            // c0: x := input ; c1: y := sync c0 0 x ; c0: z := sync c1 0 y  (a two-way crossing)
            let mut ir = ir_with_decls(&[
                (0, None),
                (1, Some(Expr::sync(c(0), lit(0.0), Expr::decl(d(0))))),
                (2, Some(Expr::sync(c(1), lit(0.0), Expr::decl(d(1))))),
            ]);
            ir.clocks.insert(d(1), c(1));
            let ticks: Vec<_> = xs.iter().map(|v| (input(&[(0, Value::scalar(*v))]), active(&[0, 1]))).collect();
            let a = run(&ir, &ticks);
            let b = run(&ir, &ticks);
            proptest::prop_assert_eq!(&a, &b);
            // a host that processes the domains' declarations in the opposite order
            let mut state = State::default();
            let mut rev = Vec::new();
            for (t, (inp, act)) in ticks.iter().enumerate() {
                let order: Vec<DeclId> = evaluated_this_tick(&ir, act).into_iter().rev().collect();
                let o = step_in_order(&ir, t as u64, act, &state, inp, &order).unwrap();
                state = o.next.clone();
                rev.push(o);
            }
            proptest::prop_assert_eq!(&a, &rev);
            // y lags x by one activation, z lags y by one: never the same tick
            for (t, o) in a.iter().enumerate() {
                let y = o.values[&d(1)].as_quantity().unwrap().1;
                let z = o.values[&d(2)].as_quantity().unwrap().1;
                proptest::prop_assert_eq!(y, if t == 0 { 0.0 } else { xs[t - 1] });
                proptest::prop_assert_eq!(z, if t < 2 { 0.0 } else { xs[t - 2] });
            }
        }
    }
}
