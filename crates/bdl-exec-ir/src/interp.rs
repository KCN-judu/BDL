//! An interpreter for the executable IR, structured exactly like the
//! generated `step`: read phase in plan order, write phase in cell order,
//! commit.  It speaks the reference evaluator's `Value` and `RuntimeError`
//! so a trace can be compared with the reference by `==`.

use crate::{Activation, ClockSlot, DeclKind, ExecExpr, ExecIr, LocalId, PrimOp};
use bdl_model::DeclId;
use bdl_reactive::eval::{count, RuntimeError};
use bdl_reactive::Value;
use std::collections::BTreeMap;

/// Committed cell values, `None` until first written.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct CellState {
    pub cells: Vec<Option<Value>>,
}

impl CellState {
    pub fn init(ir: &ExecIr) -> CellState {
        CellState {
            cells: vec![None; ir.cells.len()],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TickResult {
    pub next: CellState,
    /// Per declaration in plan order; `None` when not due this tick.
    pub values: Vec<Option<Value>>,
    /// Per output slot; `None` when the driver was not due.
    pub outputs: Vec<Option<Value>>,
    /// Per machine sink; `None` when the driver was not due.  Downstream
    /// of `values` and `outputs`, which never depend on it.
    pub commands: Vec<Option<Value>>,
}

/// The input half of the adapter, in the interpreter: every provider's
/// value from its raw reading, in input-slot order, ready for [`step`]'s
/// `inputs`.  `raw` is indexed like `ir.providers`; a slot whose reading
/// was not taken stays `None`.  Pure: the provider's term reads its raw
/// local and nothing else (`ProviderPlan`), so no state, tick or
/// declaration is consulted — which is what lets a host and a board give
/// the same value for the same reading.
pub fn provide(ir: &ExecIr, raw: &[Option<Value>]) -> Result<Vec<Option<Value>>, RuntimeError> {
    let mut inputs: Vec<Option<Value>> = vec![None; ir.inputs.len()];
    let empty = CellState { cells: Vec::new() };
    for (i, p) in ir.providers.iter().enumerate() {
        let Some(reading) = raw.get(i).cloned().flatten() else {
            continue;
        };
        let owner = ir
            .decl(p.decl)
            .map(|d| d.id)
            .ok_or_else(|| RuntimeError::Internal(format!("provider decl {:?} missing", p.decl)))?;
        let mut cx = Cx {
            owner,
            tick: 0,
            prev: &empty,
            values: &[],
            locals: BTreeMap::from([(p.local, reading)]),
        };
        let v = cx.eval(&p.provide)?;
        let slot = inputs
            .get_mut(p.slot.0 as usize)
            .ok_or_else(|| RuntimeError::Internal(format!("provider slot {:?} missing", p.slot)))?;
        *slot = Some(v);
    }
    Ok(inputs)
}

/// One global tick.  `inputs` is indexed by input slot.
pub fn step(
    ir: &ExecIr,
    tick: u64,
    active: &[ClockSlot],
    state: &CellState,
    inputs: &[Option<Value>],
) -> Result<TickResult, RuntimeError> {
    let mut values: Vec<Option<Value>> = vec![None; ir.decls.len()];
    for d in &ir.decls {
        if !ir.due(d.activation, active) {
            continue;
        }
        let v = match &d.kind {
            DeclKind::Input { slot } => inputs
                .get(slot.0 as usize)
                .cloned()
                .flatten()
                .ok_or(RuntimeError::MissingInput { decl: d.id, tick })?,
            DeclKind::Computed { body } => {
                let mut cx = Cx {
                    owner: d.id,
                    tick,
                    prev: state,
                    values: &values,
                    locals: BTreeMap::new(),
                };
                cx.eval(body)?
            }
        };
        values[d.index.0 as usize] = Some(v);
    }
    let mut next = state.clone();
    for cell in &ir.cells {
        if !active.contains(&cell.writer) {
            continue;
        }
        let owner = ir.decl(cell.owner).map(|d| d.id).ok_or_else(|| {
            RuntimeError::Internal(format!("cell owner {:?} missing", cell.owner))
        })?;
        let mut cx = Cx {
            owner,
            tick,
            prev: state,
            values: &values,
            locals: BTreeMap::new(),
        };
        let v = cx.eval(&cell.operand)?;
        next.cells[cell.slot.0 as usize] = Some(v);
    }
    let outputs = ir
        .outputs
        .iter()
        .map(|o| values[o.driver.0 as usize].clone())
        .collect();
    let mut commands = Vec::with_capacity(ir.sinks.len());
    for s in &ir.sinks {
        let driver = ir
            .decl(s.driver)
            .ok_or_else(|| RuntimeError::Internal(format!("sink driver {:?} missing", s.driver)))?;
        if values[s.driver.0 as usize].is_none() {
            commands.push(None);
            continue;
        }
        let mut cx = Cx {
            owner: driver.id,
            tick,
            prev: state,
            values: &values,
            locals: BTreeMap::new(),
        };
        commands.push(Some(cx.eval(&s.command)?));
    }
    Ok(TickResult {
        next,
        values,
        outputs,
        commands,
    })
}

struct Cx<'a> {
    owner: DeclId,
    tick: u64,
    prev: &'a CellState,
    values: &'a [Option<Value>],
    locals: BTreeMap<LocalId, Value>,
}

impl Cx<'_> {
    fn internal(&self, msg: String) -> RuntimeError {
        RuntimeError::Internal(msg)
    }

    fn eval(&mut self, e: &ExecExpr) -> Result<Value, RuntimeError> {
        Ok(match e {
            ExecExpr::Bool { value } => Value::Bool { value: *value },
            ExecExpr::Nat { value } => Value::Nat { value: *value },
            ExecExpr::Quantity { dim, value } => Value::q(*dim, value.0),
            ExecExpr::Local { id } => self
                .locals
                .get(id)
                .cloned()
                .ok_or_else(|| self.internal(format!("unbound local {id:?}")))?,
            ExecExpr::Let { local, value, body } => {
                let v = self.eval(value)?;
                self.locals.insert(*local, v);
                self.eval(body)?
            }
            ExecExpr::ReadDecl { decl } => self
                .values
                .get(decl.0 as usize)
                .cloned()
                .flatten()
                .ok_or_else(|| self.internal(format!("decl {decl:?} read before evaluation")))?,
            ExecExpr::Wrap { sem, e } => Value::sem(*sem, self.eval(e)?),
            ExecExpr::Unwrap { e } => match self.eval(e)? {
                Value::Semantic { repr, .. } => *repr,
                other => return Err(self.internal(format!("rep of non-semantic {other:?}"))),
            },
            ExecExpr::ReadCell { slot, init } => match self.prev.cells.get(slot.0 as usize) {
                Some(Some(v)) => v.clone(),
                Some(None) => self.eval(init)?,
                None => return Err(self.internal(format!("state slot {slot:?} out of range"))),
            },
            ExecExpr::Prim { op, args } => {
                // Strict: every argument first, left to right.
                let mut vs = Vec::with_capacity(args.len());
                for a in args {
                    vs.push(self.eval(a)?);
                }
                self.apply(op, vs)?
            }
            ExecExpr::Fold {
                elem,
                acc,
                step,
                init,
                list,
            } => {
                let mut value = self.eval(init)?;
                let Value::List { items } = self.eval(list)? else {
                    return Err(self.internal("fold over a non-list".into()));
                };
                for x in items.iter_from_last() {
                    self.locals.insert(*elem, x.clone());
                    self.locals.insert(*acc, value);
                    value = self.eval(step)?;
                }
                value
            }
        })
    }

    fn apply(&self, op: &PrimOp, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let (owner, tick) = (self.owner, self.tick);
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
                Err(RuntimeError::NonFinite {
                    decl: owner,
                    tick,
                    op: op.to_owned(),
                })
            }
        };
        Ok(match (op, args.as_slice()) {
            (PrimOp::Add { dim }, [a, c]) => finite(*dim, q(a)?.1 + q(c)?.1, "add")?,
            (PrimOp::Sub { dim }, [a, c]) => finite(*dim, q(a)?.1 - q(c)?.1, "sub")?,
            (PrimOp::Mul { d1, d2 }, [a, c]) => finite(*d1 + *d2, q(a)?.1 * q(c)?.1, "mul")?,
            (PrimOp::Div { d1, d2 }, [a, c]) => {
                let (x, y) = (q(a)?.1, q(c)?.1);
                if y == 0.0 {
                    return Err(RuntimeError::DivisionByZero { decl: owner, tick });
                }
                finite(*d1 - *d2, x / y, "div")?
            }
            (PrimOp::Lt, [a, c]) => Value::boolean(q(a)?.1 < q(c)?.1),
            (PrimOp::Eq, [a, c]) => Value::boolean(a.structurally_equal(c)),
            (PrimOp::Not, [a]) => Value::boolean(!b(a)?),
            (PrimOp::And, [a, c]) => Value::boolean(b(a)? && b(c)?),
            (PrimOp::Or, [a, c]) => Value::boolean(b(a)? || b(c)?),
            (PrimOp::Ite { .. }, [c, x, y]) => {
                if b(c)? {
                    x.clone()
                } else {
                    y.clone()
                }
            }
            (PrimOp::None { .. }, []) => Value::None,
            (PrimOp::Some { .. }, [x]) => Value::some(x.clone()),
            (PrimOp::IsSome { .. }, [x]) => Value::boolean(matches!(x, Value::Some { .. })),
            (PrimOp::GetD { .. }, [x, dflt]) => match x {
                Value::Some { value } => (**value).clone(),
                _ => dflt.clone(),
            },
            (PrimOp::Nil { .. }, []) => Value::list([]),
            (PrimOp::Cons { .. }, [x, Value::List { items }]) => Value::List {
                items: bdl_reactive::value::List::cons(x.clone(), items.clone()),
            },
            (PrimOp::Length { .. }, [Value::List { items }]) => Value::scalar(items.len() as f64),
            (PrimOp::Take { .. }, [k, Value::List { items }]) => Value::List {
                items: items.take(count(q(k)?.1)),
            },
            (PrimOp::Drop { .. }, [k, Value::List { items }]) => Value::List {
                items: items.drop(count(q(k)?.1)),
            },
            (PrimOp::Reverse { .. }, [Value::List { items }]) => Value::List {
                items: items.reversed(),
            },
            (PrimOp::Head { .. }, [Value::List { items }]) => match items.first() {
                Some(x) => Value::some(x.clone()),
                None => Value::None,
            },
            (PrimOp::ToList { .. }, [Value::Some { value }]) => Value::list([(**value).clone()]),
            (PrimOp::ToList { .. }, [Value::None]) => Value::list([]),
            (PrimOp::Pair { .. }, [a, c]) => Value::pair(a.clone(), c.clone()),
            (PrimOp::Fst { .. }, [Value::Pair { fst, .. }]) => (**fst).clone(),
            (PrimOp::Snd { .. }, [Value::Pair { snd, .. }]) => (**snd).clone(),
            _ => {
                return Err(RuntimeError::Internal(format!(
                    "ill-shaped primitive application {op:?} {args:?}"
                )))
            }
        })
    }
}

/// Convenience: run a whole schedule.  `active_at(tick)` gives the active
/// slots, `inputs_at(tick)` the input slot values.  Stops at the first
/// error, which is returned with its tick.
#[allow(clippy::type_complexity)]
pub fn run(
    ir: &ExecIr,
    ticks: u64,
    active_at: &dyn Fn(u64) -> Vec<ClockSlot>,
    inputs_at: &dyn Fn(u64) -> Vec<Option<Value>>,
) -> (Vec<TickResult>, Option<(u64, RuntimeError)>) {
    let mut state = CellState::init(ir);
    let mut out = Vec::new();
    for t in 0..ticks {
        match step(ir, t, &active_at(t), &state, &inputs_at(t)) {
            Ok(r) => {
                state = r.next.clone();
                out.push(r);
            }
            Err(e) => return (out, Some((t, e))),
        }
    }
    (out, None)
}

/// Whether `activation` runs at a tick with `active` slots — re-exported
/// for hosts that build traces.
pub fn is_due(ir: &ExecIr, activation: Activation, active: &[ClockSlot]) -> bool {
    ir.due(activation, active)
}
