//! Reactive lowering: a checked Design IR becomes an executable plan.
//!
//! What lowering decides — and codegen must never rediscover:
//!
//! * **Clock slots.** Every `ClockId` in the design (from `Κ`, `Ω`, and
//!   every `sync` source) gets a dense slot, in `ClockId` order.
//! * **State slots.** Every `delay`/`sync` site — `StateCellId { decl,
//!   path }` — gets a dense slot, in `StateCellId` order (declaration, then
//!   path), which is also the reference evaluator's write order.  The
//!   reverse map is kept for diagnostics.
//! * **Writers.** A `delay` cell is written when its owner's domain is
//!   active; a `sync src` cell when `src` is active.  Operands are lowered
//!   in read mode: nested temporal forms read committed state.
//! * **Input slots.** Unresolved declarations, in `DeclId` order.
//! * **Evaluation order.** The reference evaluator's traversal: for each
//!   declaration in `DeclId` order, a depth-first walk of instantaneous
//!   references in expression order, emitting a declaration after the
//!   declarations it reads (post-order).  Causality guarantees this
//!   terminates; the order is a valid topological order and, when several
//!   declarations fail at one tick, both engines usually name the same one
//!   (DI-25).
//! * **Agnostic declarations** keep the reference's rule: evaluated
//!   whenever any domain is active (every tick when the design has no
//!   domains).  They are never assigned a domain (DI-16).
//! * **Higher-order forms are inlined.** A formula lambda applied to
//!   arguments becomes `Let`s; a reference to a declaration whose
//!   realization is a lambda is inlined at each saturated application.
//!   Anything else that would need a closure at runtime is refused with
//!   `backend.unsupported_higher_order` (option A of the brief; DI-24).
//!
//! Lowering trusts the analysis that ran before it: types, causality,
//! clock consistency and `DriveWF`/`SingleDriver` are inputs, not re-derived.
//! Where an invariant it relies on is nonetheless missing it returns
//! `backend.internal_lowering`, never a panic.

#![forbid(unsafe_code)]

use bdl_check::{infer, ExprPath, Grant};
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_exec_ir::{
    Activation, CellPlan, ClockPlan, ClockSlot, ConceptPlan, DeclIndex, DeclKind, DeclPlan,
    ExecExpr, ExecIr, FunctionPlan, InputPlan, InputSlot, LocalId, OutputPlan, OutputSlot, PrimOp,
    StateSlot, EXEC_IR_VERSION,
};
use bdl_ir::{DesignIr, Expr, Prim, Ty};
use bdl_model::{ClockId, DeclId, OutputId, SemanticId};
use bdl_reactive::StateCellId;
use std::collections::{BTreeMap, BTreeSet};

/// The validated drive edges (`OutputAnalysis::valid_bindings`): lowering
/// projects exactly these, never `β` itself.
pub type ValidBindings = BTreeMap<DeclId, OutputId>;

/// Lower a checked design.  On failure every diagnostic is returned (in
/// the documented order); nothing partial is produced.
pub fn lower(
    ir: &DesignIr,
    name: &str,
    bindings: &ValidBindings,
) -> Result<ExecIr, Vec<Diagnostic>> {
    let mut lw = Lowerer::new(ir, name, bindings);
    match lw.run() {
        Ok(exec) if lw.diagnostics.is_empty() => Ok(exec),
        _ => {
            sort_diagnostics(&mut lw.diagnostics);
            Err(lw.diagnostics)
        }
    }
}

/// The dense slot of every state cell, for source maps and debugging.
pub fn state_slots(exec: &ExecIr) -> BTreeMap<StateCellId, StateSlot> {
    exec.cells
        .iter()
        .map(|c| (c.cell.clone(), c.slot))
        .collect()
}

struct Lowerer<'a> {
    ir: &'a DesignIr,
    name: String,
    bindings: &'a ValidBindings,
    diagnostics: Vec<Diagnostic>,
    clocks: BTreeMap<ClockId, ClockSlot>,
    cells: BTreeMap<StateCellId, StateSlot>,
    cell_plans: BTreeMap<StateSlot, CellPlan>,
    next_local: u32,
    /// Declarations that are values (not functions), by plan position.
    index: BTreeMap<DeclId, DeclIndex>,
}

enum Head<'e> {
    Prim(&'e Prim),
    /// The lambda itself (its body is entered per bound argument).
    Lam {
        body: &'e Expr,
    },
    Other(&'e Expr),
}

impl<'a> Lowerer<'a> {
    fn new(ir: &'a DesignIr, name: &str, bindings: &'a ValidBindings) -> Self {
        Lowerer {
            ir,
            name: name.to_owned(),
            bindings,
            diagnostics: Vec::new(),
            clocks: BTreeMap::new(),
            cells: BTreeMap::new(),
            cell_plans: BTreeMap::new(),
            next_local: 0,
            index: BTreeMap::new(),
        }
    }

    fn internal(&mut self, decl: Option<DeclId>, what: impl Into<String>) {
        let what = what.into();
        let entity = decl
            .map(|id| Entity::Mapping { id })
            .unwrap_or(Entity::Project);
        self.diagnostics.push(
            Diagnostic::error("backend.internal_lowering", entity, "Code generation hit an internal inconsistency.")
                .explain("The design passed analysis but lowering found something analysis should have established. This is a compiler bug; the design itself is fine.")
                .technical(what),
        );
    }

    fn unsupported(&mut self, decl: DeclId, path: &ExprPath, what: &str) {
        self.diagnostics.push(
            Diagnostic::error(
                "backend.unsupported_higher_order",
                Entity::Mapping { id: decl },
                "This relationship cannot be generated yet.",
            )
            .explain("Its definition uses a relationship as a value — passed along, stored, or applied only partly — which the first backend does not support. Apply relationships directly to their inputs.")
            .technical(format!("backend.unsupported_higher_order at {decl}, path {path:?}: {what}")),
        );
    }

    fn fresh_local(&mut self) -> LocalId {
        let l = LocalId(self.next_local);
        self.next_local += 1;
        l
    }

    fn run(&mut self) -> Result<ExecIr, ()> {
        let ir = self.ir;
        // Clock slots: every domain mentioned anywhere, in ClockId order.
        let mut clock_ids: BTreeSet<ClockId> = ir.clocks.values().copied().collect();
        clock_ids.extend(ir.outputs.values().map(|o| o.clock));
        for d in ir.decls.values() {
            if let Some(b) = &d.realization {
                collect_sync_sources(b, &mut clock_ids);
            }
        }
        if clock_ids.len() > usize::from(u16::MAX) {
            self.internal(None, "more than 65535 clock domains");
            return Err(());
        }
        for (i, c) in clock_ids.iter().enumerate() {
            self.clocks.insert(*c, ClockSlot(i as u16));
        }
        let clocks: Vec<ClockPlan> = clock_ids
            .iter()
            .map(|c| ClockPlan {
                slot: self.clocks[c],
                id: *c,
                name: ir
                    .clock_names
                    .get(c)
                    .cloned()
                    .unwrap_or_else(|| c.to_string()),
            })
            .collect();

        // Value declarations vs functions.
        let mut functions = Vec::new();
        let mut values: Vec<DeclId> = Vec::new();
        for d in ir.decls.values() {
            let is_function = matches!(d.interface.expected_type, Ty::Arr { .. });
            match (&d.realization, is_function) {
                (_, false) => values.push(d.id),
                (Some(Expr::Lam { .. }), true) | (Some(Expr::DeclRef { .. }), true) => functions
                    .push(FunctionPlan {
                        id: d.id,
                        name: d.name.clone(),
                        ty: d.interface.expected_type.clone(),
                    }),
                (Some(_), true) => {
                    self.unsupported(
                        d.id,
                        &vec![],
                        "a function-typed declaration whose definition is not a lambda",
                    );
                }
                (None, true) => {
                    self.unsupported(
                        d.id,
                        &vec![],
                        "a function-typed unresolved declaration (an input that is a relationship)",
                    );
                }
            }
        }

        // State slots: every temporal site, in StateCellId order.
        let mut sites: BTreeMap<StateCellId, ()> = BTreeMap::new();
        for d in ir.decls.values() {
            if let Some(b) = &d.realization {
                collect_sites(d.id, b, &mut Vec::new(), &mut sites);
            }
        }
        for (i, cell) in sites.keys().enumerate() {
            self.cells.insert(cell.clone(), StateSlot(i as u32));
        }

        // Evaluation order: the reference traversal.
        let order = evaluation_order(ir, &values);
        for (i, id) in order.iter().enumerate() {
            self.index.insert(*id, DeclIndex(i as u32));
        }

        // Inputs: unresolved value declarations in DeclId order.
        let mut inputs = Vec::new();
        let mut input_slot: BTreeMap<DeclId, InputSlot> = BTreeMap::new();
        for id in &values {
            if ir.decls[id].realization.is_none() {
                let slot = InputSlot(inputs.len() as u32);
                input_slot.insert(*id, slot);
                inputs.push(InputPlan {
                    slot,
                    decl: self.index[id],
                });
            }
        }

        // Declarations.
        let mut decls = Vec::new();
        for id in &order {
            let d = &ir.decls[id];
            let activation = match ir.clocks.get(id) {
                Some(c) => Activation::Domain {
                    clock: self.clocks[c],
                },
                None => Activation::Agnostic,
            };
            let kind = match &d.realization {
                None => DeclKind::Input {
                    slot: input_slot[id],
                },
                Some(body) => {
                    let mut cx = ExprCx {
                        owner: *id,
                        grant: Grant::of(&d.interface.expected_type),
                        env: Vec::new(),
                    };
                    match self.expr(&mut cx, body, &mut Vec::new()) {
                        Some(e) => DeclKind::Computed { body: e },
                        None => continue,
                    }
                }
            };
            decls.push(DeclPlan {
                index: self.index[id],
                id: *id,
                name: d.name.clone(),
                ty: d.interface.expected_type.clone(),
                activation,
                kind,
            });
        }

        // Outputs: the validated bindings, in OutputId order.
        let mut by_output: BTreeMap<OutputId, DeclId> = BTreeMap::new();
        for (d, o) in self.bindings {
            if by_output.insert(*o, *d).is_some() {
                self.internal(Some(*d), format!("two drivers for {o} reached lowering"));
            }
        }
        let mut outputs = Vec::new();
        for (i, (o, d)) in by_output.iter().enumerate() {
            let (Some(spec), Some(driver)) = (ir.outputs.get(o), self.index.get(d)) else {
                self.internal(
                    Some(*d),
                    format!("drive edge {d} → {o} to an unknown output or a function"),
                );
                continue;
            };
            outputs.push(OutputPlan {
                slot: OutputSlot(i as u32),
                id: *o,
                name: spec.name.clone(),
                driver: *driver,
                ty: spec.accepts.clone(),
            });
        }

        if !self.diagnostics.is_empty() {
            return Err(());
        }
        let cells = std::mem::take(&mut self.cell_plans)
            .into_values()
            .collect::<Vec<_>>();
        if cells.len() != self.cells.len() {
            self.internal(
                None,
                format!(
                    "{} temporal sites but {} cell plans",
                    self.cells.len(),
                    cells.len()
                ),
            );
            return Err(());
        }
        // Concepts: every one a carried type mentions, transitively through
        // representations.
        let mut mentioned: BTreeSet<SemanticId> = BTreeSet::new();
        for t in decls
            .iter()
            .map(|d| &d.ty)
            .chain(cells.iter().map(|c| &c.ty))
            .chain(outputs.iter().map(|o| &o.ty))
        {
            collect_sems(t, &mut mentioned);
        }
        let mut concepts = Vec::new();
        let mut queue: Vec<SemanticId> = mentioned.iter().copied().collect();
        while let Some(s) = queue.pop() {
            let Some(rep) = ir.representation_of(s) else {
                self.internal(None, format!("concept {s} has no representation"));
                return Err(());
            };
            let mut inner = BTreeSet::new();
            collect_sems(rep, &mut inner);
            for i in inner {
                if mentioned.insert(i) {
                    queue.push(i);
                }
            }
        }
        for s in &mentioned {
            let Some(cb) = ir.concepts.get(s) else {
                continue;
            };
            let Some(rep) = &cb.representation else {
                continue;
            };
            concepts.push(ConceptPlan {
                id: *s,
                name: cb.name.clone(),
                representation: rep.clone(),
            });
        }
        Ok(ExecIr {
            version: EXEC_IR_VERSION,
            name: self.name.clone(),
            has_domains: !ir.clocks.is_empty(),
            concepts,
            clocks,
            inputs,
            decls,
            cells,
            outputs,
            functions,
        })
    }

    /// Lower one expression of `cx.owner` at `path`.  `None` after a
    /// diagnostic was recorded.
    fn expr(&mut self, cx: &mut ExprCx, e: &Expr, path: &mut ExprPath) -> Option<ExecExpr> {
        match e {
            Expr::BoolLit { value } => Some(ExecExpr::Bool { value: *value }),
            Expr::NatLit { value } => Some(ExecExpr::Nat { value: *value }),
            Expr::Var { index } => {
                let n = cx.env.len();
                match n
                    .checked_sub(1 + *index as usize)
                    .and_then(|i| cx.env.get(i))
                {
                    Some(l) => Some(ExecExpr::Local { id: *l }),
                    None => {
                        self.internal(
                            Some(cx.owner),
                            format!("unbound variable {index} at {path:?}"),
                        );
                        None
                    }
                }
            }
            Expr::Lam { .. } => {
                self.unsupported(cx.owner, path, "a lambda used as a value");
                None
            }
            Expr::Prim { p } => {
                if p.arity() == 0 {
                    self.prim(cx, p, Vec::new(), path)
                } else {
                    self.unsupported(cx.owner, path, "an unapplied primitive");
                    None
                }
            }
            Expr::App { .. } => self.app(cx, e, path),
            Expr::DeclRef { id } => match self.index.get(id) {
                Some(i) => Some(ExecExpr::ReadDecl { decl: *i }),
                None => {
                    // A function referenced without application.
                    if self.ir.decls.contains_key(id) {
                        self.unsupported(cx.owner, path, "a relationship referenced as a value");
                    } else {
                        self.internal(
                            Some(cx.owner),
                            format!("reference to unknown declaration {id}"),
                        );
                    }
                    None
                }
            },
            Expr::Rep { e } => {
                path.push(0);
                let inner = self.expr(cx, e, path);
                path.pop();
                inner.map(ExecExpr::unwrap)
            }
            Expr::Mk { s, e } => {
                path.push(0);
                let inner = self.expr(cx, e, path);
                path.pop();
                inner.map(|i| ExecExpr::wrap(*s, i))
            }
            Expr::Delay { init, e: operand }
            | Expr::Sync {
                init, e: operand, ..
            } => {
                let cell = StateCellId {
                    decl: cx.owner,
                    path: path.clone(),
                };
                let Some(slot) = self.cells.get(&cell).copied() else {
                    self.internal(
                        Some(cx.owner),
                        format!("temporal site {cell:?} was not collected"),
                    );
                    return None;
                };
                let writer = match e {
                    Expr::Sync { src, .. } => self.clocks.get(src).copied(),
                    _ => self
                        .ir
                        .clocks
                        .get(&cx.owner)
                        .and_then(|c| self.clocks.get(c).copied()),
                };
                let Some(writer) = writer else {
                    self.internal(
                        Some(cx.owner),
                        format!("temporal form at {path:?} has no writing domain (ill-clocked)"),
                    );
                    return None;
                };
                if !cx.env.is_empty() {
                    self.internal(
                        Some(cx.owner),
                        format!("temporal form under a binder at {path:?}"),
                    );
                    return None;
                }
                let ty = match infer(self.ir, &cx.grant, &[], e) {
                    Ok(t) => t,
                    Err(err) => {
                        self.internal(
                            Some(cx.owner),
                            format!("cell {cell:?} does not type: {:?}", err.kind),
                        );
                        return None;
                    }
                };
                path.push(0);
                let init_e = self.expr(cx, init, path);
                path.pop();
                path.push(1);
                let op_e = self.expr(cx, operand, path);
                path.pop();
                let (init_e, op_e) = (init_e?, op_e?);
                let owner = self.index.get(&cx.owner).copied();
                let Some(owner) = owner else {
                    self.internal(Some(cx.owner), "temporal form in a function");
                    return None;
                };
                self.cell_plans.insert(
                    slot,
                    CellPlan {
                        slot,
                        cell,
                        owner,
                        ty,
                        writer,
                        operand: op_e,
                    },
                );
                Some(ExecExpr::read_cell(slot, init_e))
            }
        }
    }

    /// A (possibly nested) application: flatten to head + arguments, then
    /// saturate a primitive, or inline a lambda / function declaration.
    fn app(&mut self, cx: &mut ExprCx, e: &Expr, path: &mut ExprPath) -> Option<ExecExpr> {
        // Flatten.  Arguments keep their own paths for temporal identity.
        let mut args: Vec<(&Expr, ExprPath)> = Vec::new();
        let mut head = e;
        let mut hp = path.clone();
        while let Expr::App { f, a } = head {
            let mut ap = hp.clone();
            ap.push(1);
            args.push((a, ap));
            hp.push(0);
            head = f;
        }
        args.reverse();
        let (head, closed) = match head {
            Expr::Prim { p } => (Head::Prim(p), false),
            lam @ Expr::Lam { .. } => (Head::Lam { body: lam }, false),
            Expr::DeclRef { id } if !self.index.contains_key(id) => {
                match resolve_function(self.ir, *id) {
                    Some(lam @ Expr::Lam { .. }) => (Head::Lam { body: lam }, true),
                    _ => {
                        self.unsupported(
                            cx.owner,
                            &hp,
                            "an application of something that is not a lambda or a primitive",
                        );
                        return None;
                    }
                }
            }
            other => (Head::Other(other), false),
        };
        match head {
            Head::Prim(p) => {
                if args.len() != p.arity() {
                    self.unsupported(cx.owner, &hp, "a partially applied primitive");
                    return None;
                }
                let mut lowered = Vec::with_capacity(args.len());
                for (a, mut ap) in args {
                    lowered.push(self.expr(cx, a, &mut ap)?);
                }
                self.prim(cx, p, lowered, &hp)
            }
            Head::Lam { body } => {
                // Arguments are lowered in the caller's environment and
                // bound outermost first; the body runs in an environment
                // of its parameters over the caller's (a literal lambda)
                // or over nothing (an inlined declaration is closed).
                let mut env = if closed { Vec::new() } else { cx.env.clone() };
                let mut bound = Vec::new();
                let mut lambda: &Expr = body;
                let n = args.len();
                for (i, (a, mut ap)) in args.into_iter().enumerate() {
                    let value = self.expr(cx, a, &mut ap)?;
                    let l = self.fresh_local();
                    bound.push((l, value));
                    env.push(l);
                    if i + 1 < n {
                        match lambda {
                            Expr::Lam { body: inner, .. } => lambda = inner,
                            _ => {
                                self.unsupported(
                                    cx.owner,
                                    &hp,
                                    "more arguments than lambda parameters",
                                );
                                return None;
                            }
                        }
                    }
                }
                let Expr::Lam { body: last, .. } = lambda else {
                    self.unsupported(cx.owner, &hp, "more arguments than lambda parameters");
                    return None;
                };
                let mut inner = ExprCx {
                    owner: cx.owner,
                    grant: cx.grant.clone(),
                    env,
                };
                // Paths inside an inlined body carry no temporal identity
                // (typing forbids temporal forms under binders).
                let mut body_path = hp.clone();
                body_path.push(0);
                let mut result = self.expr(&mut inner, last, &mut body_path)?;
                for (l, v) in bound.into_iter().rev() {
                    result = ExecExpr::let_(l, v, result);
                }
                Some(result)
            }
            Head::Other(h) => {
                let what = match h {
                    Expr::Var { .. } => "an application of a relationship passed as an argument",
                    _ => "an application of a non-function",
                };
                self.unsupported(cx.owner, &hp, what);
                None
            }
        }
    }

    fn prim(
        &mut self,
        cx: &mut ExprCx,
        p: &Prim,
        args: Vec<ExecExpr>,
        path: &ExprPath,
    ) -> Option<ExecExpr> {
        let op = match p {
            Prim::Lit { dim, value } => {
                return Some(ExecExpr::Quantity {
                    dim: *dim,
                    value: *value,
                })
            }
            Prim::Add { dim } => PrimOp::Add { dim: *dim },
            Prim::Sub { dim } => PrimOp::Sub { dim: *dim },
            Prim::Mul { d1, d2 } => PrimOp::Mul { d1: *d1, d2: *d2 },
            Prim::Div { d1, d2 } => PrimOp::Div { d1: *d1, d2: *d2 },
            Prim::Lt { .. } => PrimOp::Lt,
            Prim::Eq { .. } => PrimOp::Eq,
            Prim::Not => PrimOp::Not,
            Prim::And => PrimOp::And,
            Prim::Or => PrimOp::Or,
            Prim::Ite { ty } => PrimOp::Ite { ty: ty.clone() },
            Prim::None { ty } => PrimOp::None { ty: ty.clone() },
            Prim::Some { ty } => PrimOp::Some { ty: ty.clone() },
            Prim::IsSome { ty } => PrimOp::IsSome { ty: ty.clone() },
            Prim::GetD { ty } => PrimOp::GetD { ty: ty.clone() },
        };
        if let Some(t) = prim_result_ty(&op) {
            if matches!(t, Ty::Arr { .. }) {
                self.unsupported(cx.owner, path, "a primitive over relationships");
                return None;
            }
        }
        Some(ExecExpr::Prim { op, args })
    }
}

#[derive(Clone)]
struct ExprCx {
    owner: DeclId,
    grant: Grant,
    /// De Bruijn environment: the local bound by each enclosing lambda
    /// parameter, outermost first.
    env: Vec<LocalId>,
}

fn prim_result_ty(op: &PrimOp) -> Option<Ty> {
    match op {
        PrimOp::Ite { ty }
        | PrimOp::GetD { ty }
        | PrimOp::Some { ty }
        | PrimOp::None { ty }
        | PrimOp::IsSome { ty } => Some(ty.clone()),
        _ => None,
    }
}

/// The lambda a function declaration stands for, through alias chains.
fn resolve_function(ir: &DesignIr, id: DeclId) -> Option<&Expr> {
    let mut seen = BTreeSet::new();
    let mut cur = id;
    loop {
        if !seen.insert(cur) {
            return None;
        }
        match ir.decls.get(&cur)?.realization.as_ref()? {
            lam @ Expr::Lam { .. } => return Some(lam),
            Expr::DeclRef { id } => cur = *id,
            _ => return None,
        }
    }
}

fn collect_sems(t: &Ty, out: &mut BTreeSet<SemanticId>) {
    match t {
        Ty::Sem { id } => {
            out.insert(*id);
        }
        Ty::Arr { dom, cod } => {
            collect_sems(dom, out);
            collect_sems(cod, out);
        }
        Ty::Opt { inner } => collect_sems(inner, out),
        Ty::Bool | Ty::Nat | Ty::Q { .. } => {}
    }
}

fn collect_sync_sources(e: &Expr, out: &mut BTreeSet<ClockId>) {
    match e {
        Expr::Sync { src, init, e } => {
            out.insert(*src);
            collect_sync_sources(init, out);
            collect_sync_sources(e, out);
        }
        Expr::Delay { init, e } => {
            collect_sync_sources(init, out);
            collect_sync_sources(e, out);
        }
        Expr::Lam { body, .. } => collect_sync_sources(body, out),
        Expr::App { f, a } => {
            collect_sync_sources(f, out);
            collect_sync_sources(a, out);
        }
        Expr::Rep { e } | Expr::Mk { e, .. } => collect_sync_sources(e, out),
        _ => {}
    }
}

/// Every temporal site of a realization, keyed by its cell identity —
/// the same walk as the reference evaluator's `temporal_sites`.
fn collect_sites(
    owner: DeclId,
    e: &Expr,
    path: &mut ExprPath,
    out: &mut BTreeMap<StateCellId, ()>,
) {
    match e {
        Expr::Sync { init, e, .. } | Expr::Delay { init, e } => {
            out.insert(
                StateCellId {
                    decl: owner,
                    path: path.clone(),
                },
                (),
            );
            path.push(0);
            collect_sites(owner, init, path, out);
            path.pop();
            path.push(1);
            collect_sites(owner, e, path, out);
            path.pop();
        }
        Expr::Lam { body, .. } => {
            path.push(0);
            collect_sites(owner, body, path, out);
            path.pop();
        }
        Expr::App { f, a } => {
            path.push(0);
            collect_sites(owner, f, path, out);
            path.pop();
            path.push(1);
            collect_sites(owner, a, path, out);
            path.pop();
        }
        Expr::Rep { e } | Expr::Mk { e, .. } => {
            path.push(0);
            collect_sites(owner, e, path, out);
            path.pop();
        }
        _ => {}
    }
}

/// The reference evaluator's traversal order over value declarations:
/// roots in `DeclId` order, instantaneous references depth-first in
/// expression order (through inlined functions), post-order.
fn evaluation_order(ir: &DesignIr, values: &[DeclId]) -> Vec<DeclId> {
    fn visit(ir: &DesignIr, d: DeclId, seen: &mut BTreeSet<DeclId>, out: &mut Vec<DeclId>) {
        if !seen.insert(d) {
            return;
        }
        if let Some(b) = ir.decls.get(&d).and_then(|x| x.realization.as_ref()) {
            let mut refs = Vec::new();
            inst_refs_through_functions(ir, b, &mut refs, &mut BTreeSet::new());
            for r in refs {
                if ir
                    .decls
                    .get(&r)
                    .is_some_and(|x| !matches!(x.interface.expected_type, Ty::Arr { .. }))
                {
                    visit(ir, r, seen, out);
                }
            }
        }
        out.push(d);
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for d in values {
        visit(ir, *d, &mut seen, &mut out);
    }
    out
}

/// Instantaneous references in evaluation order; a reference to a
/// function declaration contributes the references of its body (it will
/// be inlined there).
fn inst_refs_through_functions(
    ir: &DesignIr,
    e: &Expr,
    out: &mut Vec<DeclId>,
    expanding: &mut BTreeSet<DeclId>,
) {
    match e {
        Expr::Var { .. } | Expr::BoolLit { .. } | Expr::NatLit { .. } | Expr::Prim { .. } => {}
        Expr::Lam { body, .. } => inst_refs_through_functions(ir, body, out, expanding),
        Expr::App { f, a } => {
            inst_refs_through_functions(ir, f, out, expanding);
            inst_refs_through_functions(ir, a, out, expanding);
        }
        Expr::DeclRef { id } => {
            let is_fn = ir
                .decls
                .get(id)
                .is_some_and(|x| matches!(x.interface.expected_type, Ty::Arr { .. }));
            if is_fn {
                if expanding.insert(*id) {
                    if let Some(b) = resolve_function(ir, *id) {
                        inst_refs_through_functions(ir, b, out, expanding);
                    }
                    expanding.remove(id);
                }
            } else {
                out.push(*id);
            }
        }
        Expr::Rep { e } | Expr::Mk { e, .. } => inst_refs_through_functions(ir, e, out, expanding),
        Expr::Delay { init, .. } | Expr::Sync { init, .. } => {
            inst_refs_through_functions(ir, init, out, expanding)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_exec_ir::interp::{self, CellState};
    use bdl_ir::{Declaration, Interface, Scalar};
    use bdl_model::Dim;
    use bdl_reactive::eval::{self, State, TickInput};
    use bdl_reactive::Value;

    fn d(n: u64) -> DeclId {
        DeclId::from_raw(n)
    }
    fn c(n: u64) -> ClockId {
        ClockId::from_raw(n)
    }
    fn lit(v: f64) -> Expr {
        Expr::prim(Prim::Lit {
            dim: Dim::ZERO,
            value: Scalar(v),
        })
    }
    fn add(a: Expr, b: Expr) -> Expr {
        Expr::apps(Expr::prim(Prim::Add { dim: Dim::ZERO }), [a, b])
    }
    fn ir_with(decls: &[(u64, Option<Expr>, Option<u64>)]) -> DesignIr {
        let mut ir = DesignIr::default();
        for (n, r, clock) in decls {
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
            if let Some(cl) = clock {
                ir.clocks.insert(d(*n), c(*cl));
            }
        }
        ir
    }

    /// Run both engines over `ticks` with every domain active and the
    /// given per-tick inputs (by DeclId); assert identical traces.
    fn agree(
        ir: &DesignIr,
        ticks: usize,
        inputs: &dyn Fn(u64, DeclId) -> Option<Value>,
    ) -> Vec<interp::TickResult> {
        let exec = lower(ir, "t", &BTreeMap::new()).unwrap_or_else(|d| panic!("{d:?}"));
        let all: BTreeSet<ClockId> = ir.clocks.values().copied().collect();
        let slots: Vec<ClockSlot> = exec.clocks.iter().map(|c| c.slot).collect();
        let mut rs = State::default();
        let mut es = CellState::init(&exec);
        let mut out = Vec::new();
        for t in 0..ticks as u64 {
            let input = TickInput {
                values: ir
                    .decls
                    .keys()
                    .filter_map(|id| inputs(t, *id).map(|v| (*id, v)))
                    .collect(),
            };
            let in_slots: Vec<Option<Value>> = exec
                .inputs
                .iter()
                .map(|i| inputs(t, exec.decls[i.decl.0 as usize].id))
                .collect();
            let r = eval::step(ir, t, &all, &rs, &input);
            let e = interp::step(&exec, t, &slots, &es, &in_slots);
            match (r, e) {
                (Ok(r), Ok(e)) => {
                    for dp in &exec.decls {
                        // The reference memo holds computed declarations
                        // only; inputs are read through, never recorded.
                        let expected = match dp.kind {
                            DeclKind::Input { .. } => e.values[dp.index.0 as usize]
                                .as_ref()
                                .and_then(|_| inputs(t, dp.id)),
                            DeclKind::Computed { .. } => r.values.get(&dp.id).cloned(),
                        };
                        assert_eq!(
                            expected.as_ref(),
                            e.values[dp.index.0 as usize].as_ref(),
                            "tick {t} decl {}",
                            dp.id
                        );
                    }
                    for (cell, slot) in state_slots(&exec) {
                        assert_eq!(
                            r.next.cells.get(&cell),
                            e.next.cells[slot.0 as usize].as_ref(),
                            "tick {t} cell {cell:?}"
                        );
                    }
                    rs = r.next;
                    es = e.next.clone();
                    out.push(e);
                }
                (Err(r), Err(e)) => {
                    assert_eq!(r, e, "tick {t}");
                    return out;
                }
                (r, e) => panic!("tick {t}: reference {r:?} vs exec {e:?}"),
            }
        }
        out
    }

    #[test]
    fn arithmetic_inputs_and_delay_agree_with_the_reference() {
        // x := input; acc := delay 0 (acc + x); y := acc * 2
        let ir = ir_with(&[
            (0, None, Some(0)),
            (
                1,
                Some(Expr::delay(
                    lit(0.0),
                    add(Expr::decl(d(1)), Expr::decl(d(0))),
                )),
                Some(0),
            ),
            (
                2,
                Some(Expr::apps(
                    Expr::prim(Prim::Mul {
                        d1: Dim::ZERO,
                        d2: Dim::ZERO,
                    }),
                    [Expr::decl(d(1)), lit(2.0)],
                )),
                Some(0),
            ),
        ]);
        let out = agree(&ir, 5, &|t, id| {
            (id == d(0)).then(|| Value::scalar(t as f64 + 1.0))
        });
        let ys: Vec<f64> = out
            .iter()
            .map(|r| r.values[2].as_ref().unwrap().as_quantity().unwrap().1)
            .collect();
        assert_eq!(ys, vec![0.0, 2.0, 6.0, 12.0, 20.0]);
    }

    #[test]
    fn inlined_lambdas_keep_argument_paths_and_agree() {
        // f := λa. λb. a - b ;  y := f (delay 1 y) 10   (the delay is an argument)
        let f = Expr::Lam {
            dom: Ty::q(Dim::ZERO),
            body: Box::new(Expr::Lam {
                dom: Ty::q(Dim::ZERO),
                body: Box::new(Expr::apps(
                    Expr::prim(Prim::Sub { dim: Dim::ZERO }),
                    [Expr::Var { index: 1 }, Expr::Var { index: 0 }],
                )),
            }),
        };
        let mut ir = ir_with(&[(
            1,
            Some(Expr::apps(
                Expr::decl(d(0)),
                [Expr::delay(lit(1.0), Expr::decl(d(1))), lit(10.0)],
            )),
            Some(0),
        )]);
        ir.decls.insert(
            d(0),
            Declaration {
                id: d(0),
                name: "f".into(),
                interface: Interface {
                    expected_type: Ty::arrows(
                        [Ty::q(Dim::ZERO), Ty::q(Dim::ZERO)],
                        Ty::q(Dim::ZERO),
                    ),
                    commitments: vec![],
                },
                realization: Some(f),
            },
        );
        let exec = lower(&ir, "t", &BTreeMap::new()).unwrap();
        assert_eq!(exec.functions.len(), 1);
        assert_eq!(exec.cells.len(), 1);
        assert_eq!(
            exec.cells[0].cell,
            StateCellId {
                decl: d(1),
                path: vec![0, 1]
            }
        );
        let out = agree(&ir, 4, &|_, _| None);
        let ys: Vec<f64> = out
            .iter()
            .map(|r| r.values[0].as_ref().unwrap().as_quantity().unwrap().1)
            .collect();
        assert_eq!(ys, vec![-9.0, -19.0, -29.0, -39.0]);
        let json = serde_json::to_string(&exec).unwrap();
        assert_eq!(serde_json::from_str::<ExecIr>(&json).unwrap(), exec);
    }

    #[test]
    fn sync_between_two_domains_agrees() {
        // c0: x := input ; c1: y := sync c0 (-1) x ; c0: z := sync c1 0 y
        let mut ir = ir_with(&[
            (0, None, Some(0)),
            (
                1,
                Some(Expr::sync(c(0), lit(-1.0), Expr::decl(d(0)))),
                Some(1),
            ),
            (
                2,
                Some(Expr::sync(c(1), lit(0.0), Expr::decl(d(1)))),
                Some(0),
            ),
        ]);
        ir.clock_names.insert(c(0), "fast".into());
        let exec = lower(&ir, "t", &BTreeMap::new()).unwrap();
        assert_eq!(exec.clocks.len(), 2);
        assert_eq!(exec.cells[0].writer, ClockSlot(0));
        assert_eq!(exec.cells[1].writer, ClockSlot(1));
        agree(&ir, 5, &|t, id| {
            (id == d(0)).then(|| Value::scalar(t as f64))
        });
    }

    #[test]
    fn errors_agree_and_higher_order_is_refused_structurally() {
        let ir = ir_with(&[
            (0, None, Some(0)),
            (
                1,
                Some(Expr::apps(
                    Expr::prim(Prim::Div {
                        d1: Dim::ZERO,
                        d2: Dim::ZERO,
                    }),
                    [lit(1.0), Expr::decl(d(0))],
                )),
                Some(0),
            ),
        ]);
        agree(&ir, 2, &|t, _| {
            Some(Value::scalar(if t == 1 { 0.0 } else { 2.0 }))
        });
        agree(&ir, 1, &|_, _| None);
        // a lambda as a value
        let ir = ir_with(&[(
            0,
            Some(Expr::apps(
                Expr::prim(Prim::GetD {
                    ty: Ty::q(Dim::ZERO),
                }),
                [
                    Expr::prim(Prim::None {
                        ty: Ty::q(Dim::ZERO),
                    }),
                    Expr::Lam {
                        dom: Ty::Bool,
                        body: Box::new(lit(1.0)),
                    },
                ],
            )),
            Some(0),
        )]);
        let ds = lower(&ir, "t", &BTreeMap::new()).unwrap_err();
        assert_eq!(ds[0].code.as_str(), "backend.unsupported_higher_order");
        assert_eq!(ds[0].message, "This relationship cannot be generated yet.");
        // a partially applied primitive
        let ir = ir_with(&[(
            0,
            Some(Expr::app(
                Expr::prim(Prim::Add { dim: Dim::ZERO }),
                lit(1.0),
            )),
            Some(0),
        )]);
        assert_eq!(
            lower(&ir, "t", &BTreeMap::new()).unwrap_err()[0]
                .code
                .as_str(),
            "backend.unsupported_higher_order"
        );
    }

    #[test]
    fn evaluation_order_is_the_reference_traversal_and_agnostic_decls_run_when_anything_is_active()
    {
        // 0 := 1 + 2 (agnostic) ; 1 := 0 (c0) ; 2 := 3 (c0) ; 3 := 5 (agnostic)
        let ir = ir_with(&[
            (0, Some(add(Expr::decl(d(1)), Expr::decl(d(2)))), None),
            (1, Some(Expr::decl(d(0))), Some(0)),
            (2, Some(Expr::decl(d(3))), Some(0)),
            (3, Some(lit(5.0)), None),
        ]);
        // 0 reads 1 reads 0: an instantaneous cycle — lowering does not care, causality does.
        let ir2 = ir_with(&[
            (0, Some(add(Expr::decl(d(2)), lit(1.0))), None),
            (1, Some(Expr::decl(d(0))), Some(0)),
            (2, Some(Expr::decl(d(3))), Some(0)),
            (3, Some(lit(5.0)), None),
        ]);
        let _ = ir;
        let exec = lower(&ir2, "t", &BTreeMap::new()).unwrap();
        let order: Vec<DeclId> = exec.decls.iter().map(|x| x.id).collect();
        assert_eq!(order, vec![d(3), d(2), d(0), d(1)]);
        assert_eq!(exec.decls[2].activation, Activation::Agnostic);
        assert!(exec.has_domains);
        agree(&ir2, 2, &|_, _| None);
        // no domain at all: everything runs every tick
        let ir3 = ir_with(&[(0, Some(lit(1.0)), None)]);
        let exec = lower(&ir3, "t", &BTreeMap::new()).unwrap();
        assert!(!exec.has_domains);
        let r = interp::step(&exec, 0, &[], &CellState::init(&exec), &[]).unwrap();
        assert_eq!(r.values[0], Some(Value::scalar(1.0)));
    }
}
