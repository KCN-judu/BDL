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
//!   realization is a lambda is inlined at each saturated application; a
//!   function passed as an argument (a predicate to `any`, a step to
//!   `fold`) is carried as a *binding* and inlined wherever the receiver
//!   applies it, so the equation library's combinators lower to first-order
//!   code.  The kernel's `fold` becomes [`ExecExpr::Fold`] with its step
//!   inlined over two locals.  Anything else that would need a closure at
//!   runtime is refused with `backend.unsupported_higher_order` (option A
//!   of the brief; DI-24).
//!
//! * **Machine sinks.** Every admissible [`Realization`] the compiler hands
//!   over becomes a [`SinkPlan`] below the outputs: its command is the
//!   encoder applied to the driver's value, lowered in the driver's own
//!   context under no grant.  Sinks introduce no declaration, no cell and
//!   no clock: the behavior plan is the same with or without them
//!   (`lower_transparent`, docs/architecture/output-realization.md).
//!
//! Lowering trusts the analysis that ran before it: types, causality,
//! clock consistency, `DriveWF`/`SingleDriver` and realization
//! admissibility are inputs, not re-derived.
//! Where an invariant it relies on is nonetheless missing it returns
//! `backend.internal_lowering`, never a panic.

#![forbid(unsafe_code)]

use bdl_check::{infer, ExprPath, Grant};
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_exec_ir::{
    Activation, CellPlan, ClockPlan, ClockSlot, ConceptPlan, DeclIndex, DeclKind, DeclPlan,
    ExecExpr, ExecIr, FunctionPlan, InputPlan, InputSlot, LocalId, OutputPlan, OutputSlot, PrimOp,
    ProviderPlan, SinkPlan, SinkSlot, StateSlot, EXEC_IR_VERSION,
};
use bdl_ir::{DesignIr, Expr, Prim, Ty};
use bdl_model::{ClockId, ConceptId, DeclId, DeviceId, InputProfileId, OutputId, OutputProfileId};
use bdl_reactive::StateCellId;
use std::collections::{BTreeMap, BTreeSet};

/// The validated drive edges (`OutputAnalysis::valid_bindings`): lowering
/// projects exactly these, never `β` itself.
pub type ValidBindings = BTreeMap<DeclId, OutputId>;

/// One admissible realization to lower into a machine sink: the output,
/// the profile chosen for it, its raw command type and the encoder body
/// (`encode (rep d)` for the output's driver `d`, built by
/// `bdl_output::realization::encoder_body`).  Keyed by device binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Realization {
    pub device: DeviceId,
    pub device_name: String,
    pub output: OutputId,
    pub profile: OutputProfileId,
    pub raw: Ty,
    pub body: Expr,
}

pub type Realizations = BTreeMap<DeviceId, Realization>;

/// One admissible provision to lower into a provider: the Source, the
/// profile chosen for it, its raw reading type and the provision body —
/// `mk c (transduce r)` with `r` the raw reading as de Bruijn variable 0,
/// built by `bdl_output::provision::provision_body` over `Expr::var(0)`.
/// Keyed by the Source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provision {
    pub source: DeclId,
    pub device: DeviceId,
    pub device_name: String,
    pub profile: InputProfileId,
    pub raw: Ty,
    pub body: Expr,
}

pub type Provisions = BTreeMap<DeclId, Provision>;

/// Lower a checked design.  On failure every diagnostic is returned (in
/// the documented order); nothing partial is produced.
pub fn lower(
    ir: &DesignIr,
    name: &str,
    bindings: &ValidBindings,
    realizations: &Realizations,
) -> Result<ExecIr, Vec<Diagnostic>> {
    lower_with_provisions(ir, name, bindings, realizations, &Provisions::new())
}

/// [`lower`] with the Sources' providers as well (the input half of the
/// adapter).  A Source without a provision stays a plain input slot.
pub fn lower_with_provisions(
    ir: &DesignIr,
    name: &str,
    bindings: &ValidBindings,
    realizations: &Realizations,
    provisions: &Provisions,
) -> Result<ExecIr, Vec<Diagnostic>> {
    let mut lw = Lowerer::new(ir, name, bindings, realizations, provisions);
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
    realizations: &'a Realizations,
    provisions: &'a Provisions,
    diagnostics: Vec<Diagnostic>,
    clocks: BTreeMap<ClockId, ClockSlot>,
    cells: BTreeMap<StateCellId, StateSlot>,
    cell_plans: BTreeMap<StateSlot, CellPlan>,
    next_local: u32,
    /// Declarations that are values (not functions), by plan position.
    index: BTreeMap<DeclId, DeclIndex>,
}

/// What a de Bruijn variable stands for while lowering: a runtime local,
/// or a function known syntactically that is inlined where it is applied.
#[derive(Clone)]
enum Bind<'e> {
    Val(LocalId),
    Fun(Fun<'e>),
}

/// A function known at lowering time.
#[derive(Clone)]
enum Fun<'e> {
    /// A lambda with the bindings it closes over, and the locals that its
    /// partially applied arguments were bound to (re-bound around every
    /// inlined use: the values are pure, so the result is the same).
    Lam {
        lam: &'e Expr,
        env: Vec<Bind<'e>>,
        lets: Vec<(LocalId, ExecExpr)>,
    },
    /// A primitive with the arguments already given.
    Prim { p: &'e Prim, args: Vec<ExecExpr> },
}

/// An argument at an application site.
enum Arg<'e> {
    Val(ExecExpr),
    Fun(Fun<'e>),
}

impl<'a> Lowerer<'a> {
    fn new(
        ir: &'a DesignIr,
        name: &str,
        bindings: &'a ValidBindings,
        realizations: &'a Realizations,
        provisions: &'a Provisions,
    ) -> Self {
        Lowerer {
            ir,
            name: name.to_owned(),
            bindings,
            realizations,
            provisions,
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

        // Machine sinks: one per admissible realization, in DeviceId order,
        // below the outputs.  The command is lowered in the driver's
        // context under no grant, so it can construct nothing and read
        // only what the driver already is.
        let mut sinks = Vec::new();
        for (i, r) in self.realizations.values().enumerate() {
            let Some(plan) = outputs.iter().find(|o| o.id == r.output) else {
                self.internal(
                    None,
                    format!(
                        "realization of {} for an output that is not driven",
                        r.output
                    ),
                );
                continue;
            };
            let Some(driver) = ir.decls.get(&plan_driver_id(&decls, plan.driver)) else {
                self.internal(None, format!("driver of {} missing", r.output));
                continue;
            };
            match infer(ir, &Grant::None, &[], &r.body) {
                Ok(t) if t == r.raw => {}
                Ok(t) => {
                    self.internal(
                        Some(driver.id),
                        format!(
                            "raw command of {} has type {} but the profile says {}",
                            r.output,
                            bdl_check::pretty::kernel(&t),
                            bdl_check::pretty::kernel(&r.raw)
                        ),
                    );
                    continue;
                }
                Err(e) => {
                    self.internal(
                        Some(driver.id),
                        format!("raw command of {} does not type: {:?}", r.output, e.kind),
                    );
                    continue;
                }
            }
            let mut cx = ExprCx {
                owner: driver.id,
                grant: Grant::None,
                env: Vec::new(),
            };
            let mut path = ExprPath::new();
            let Some(command) = self.expr(&mut cx, &r.body, &mut path) else {
                continue;
            };
            sinks.push(SinkPlan {
                slot: SinkSlot(i as u32),
                device: r.device,
                device_name: r.device_name.clone(),
                output: r.output,
                driver: plan.driver,
                profile: r.profile.clone(),
                raw: r.raw.clone(),
                command,
            });
        }

        // Providers: one per admissible provision, in input-slot order.
        // The body is lowered in the Source's own context under its own
        // grant — it constructs exactly the Source's concept — with the raw
        // reading bound to a fresh local and nothing else in scope.
        let mut providers = Vec::new();
        for p in self.provisions.values() {
            let Some(slot) = input_slot.get(&p.source).copied() else {
                self.internal(
                    Some(p.source),
                    format!("provision of {} which is not an input", p.source),
                );
                continue;
            };
            let Some(source) = ir.decls.get(&p.source) else {
                self.internal(None, format!("provided Source {} missing", p.source));
                continue;
            };
            let grant = Grant::of(&source.interface.expected_type);
            match infer(ir, &grant, std::slice::from_ref(&p.raw), &p.body) {
                Ok(t) if t == source.interface.expected_type => {}
                Ok(t) => {
                    self.internal(
                        Some(p.source),
                        format!(
                            "provision of {} has type {} but the Source expects {}",
                            p.source,
                            bdl_check::pretty::kernel(&t),
                            bdl_check::pretty::kernel(&source.interface.expected_type)
                        ),
                    );
                    continue;
                }
                Err(e) => {
                    self.internal(
                        Some(p.source),
                        format!("provision of {} does not type: {:?}", p.source, e.kind),
                    );
                    continue;
                }
            }
            let local = self.fresh_local();
            let mut cx = ExprCx {
                owner: p.source,
                grant,
                env: vec![Bind::Val(local)],
            };
            let mut path = ExprPath::new();
            let Some(provide) = self.expr(&mut cx, &p.body, &mut path) else {
                continue;
            };
            providers.push(ProviderPlan {
                slot,
                decl: self.index[&p.source],
                device: p.device,
                device_name: p.device_name.clone(),
                profile: p.profile.clone(),
                raw: p.raw.clone(),
                local,
                provide,
            });
        }
        providers.sort_by_key(|p| p.slot);

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
        let mut mentioned: BTreeSet<ConceptId> = BTreeSet::new();
        for t in decls
            .iter()
            .map(|d| &d.ty)
            .chain(cells.iter().map(|c| &c.ty))
            .chain(outputs.iter().map(|o| &o.ty))
        {
            collect_sems(t, &mut mentioned);
        }
        let mut concepts = Vec::new();
        let mut queue: Vec<ConceptId> = mentioned.iter().copied().collect();
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
            sinks,
            providers,
        })
    }

    /// Lower one expression of `cx.owner` at `path`.  `None` after a
    /// diagnostic was recorded.
    fn expr(&mut self, cx: &mut ExprCx<'a>, e: &'a Expr, path: &mut ExprPath) -> Option<ExecExpr> {
        match e {
            Expr::BoolLit { value } => Some(ExecExpr::Bool { value: *value }),
            Expr::NatLit { value } => Some(ExecExpr::Nat { value: *value }),
            Expr::Var { index } => match cx.lookup(*index) {
                Some(Bind::Val(l)) => Some(ExecExpr::Local { id: *l }),
                Some(Bind::Fun(_)) => {
                    self.unsupported(cx.owner, path, "a rule used as a value");
                    None
                }
                None => {
                    self.internal(
                        Some(cx.owner),
                        format!("unbound variable {index} at {path:?}"),
                    );
                    None
                }
            },
            Expr::Lam { .. } => {
                self.unsupported(cx.owner, path, "a lambda used as a value");
                None
            }
            Expr::Fold { f, z, l } => {
                path.push(0);
                let fun = self.fun(cx, f, path);
                path.pop();
                path.push(1);
                let init = self.expr(cx, z, path);
                path.pop();
                path.push(2);
                let list = self.expr(cx, l, path);
                path.pop();
                let (fun, init, list) = (fun?, init?, list?);
                let elem = self.fresh_local();
                let acc = self.fresh_local();
                let mut step_path = path.clone();
                step_path.push(0);
                let step = self.apply(
                    cx,
                    fun,
                    vec![
                        Arg::Val(ExecExpr::Local { id: elem }),
                        Arg::Val(ExecExpr::Local { id: acc }),
                    ],
                    &step_path,
                )?;
                Some(ExecExpr::Fold {
                    elem,
                    acc,
                    step: Box::new(step),
                    init: Box::new(init),
                    list: Box::new(list),
                })
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

    /// Flatten a (possibly nested) application into its head and its
    /// arguments, each with its own path (for temporal identity).
    fn flatten<'e>(
        e: &'e Expr,
        path: &ExprPath,
    ) -> (&'e Expr, ExprPath, Vec<(&'e Expr, ExprPath)>) {
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
        (head, hp, args)
    }

    /// The function an expression stands for, when it is known
    /// syntactically: a lambda, a primitive, a function declaration, a
    /// variable bound to one of those, or a partial application of one.
    /// `None` after a diagnostic.
    fn fun(&mut self, cx: &mut ExprCx<'a>, e: &'a Expr, path: &ExprPath) -> Option<Fun<'a>> {
        match e {
            lam @ Expr::Lam { .. } => Some(Fun::Lam {
                lam,
                env: cx.env.clone(),
                lets: Vec::new(),
            }),
            Expr::Prim { p } if p.arity() > 0 => Some(Fun::Prim {
                p,
                args: Vec::new(),
            }),
            Expr::Var { index } => match cx.lookup(*index) {
                Some(Bind::Fun(f)) => Some(f.clone()),
                Some(Bind::Val(_)) => {
                    self.unsupported(cx.owner, path, "a value applied as a relationship");
                    None
                }
                None => {
                    self.internal(
                        Some(cx.owner),
                        format!("unbound variable {index} at {path:?}"),
                    );
                    None
                }
            },
            Expr::DeclRef { id } if !self.index.contains_key(id) => {
                match resolve_function(self.ir, *id) {
                    Some(lam @ Expr::Lam { .. }) => Some(Fun::Lam {
                        lam,
                        env: Vec::new(),
                        lets: Vec::new(),
                    }),
                    _ => {
                        self.unsupported(
                            cx.owner,
                            path,
                            "an application of something that is not a lambda or a primitive",
                        );
                        None
                    }
                }
            }
            Expr::App { .. } => {
                let (head, hp, args) = Self::flatten(e, path);
                let fun = self.fun(cx, head, &hp)?;
                let args = self.args(cx, args)?;
                self.partial(cx, fun, args, &hp)
            }
            _ => {
                self.unsupported(cx.owner, path, "an application of a non-function");
                None
            }
        }
    }

    /// Whether an expression is a function syntactically — what decides
    /// whether an argument is carried as a binding or lowered to a value.
    fn is_function(&self, cx: &ExprCx<'a>, e: &Expr) -> bool {
        match e {
            Expr::Lam { .. } => true,
            Expr::Prim { p } => p.arity() > 0,
            Expr::Var { index } => matches!(cx.lookup(*index), Some(Bind::Fun(_))),
            Expr::DeclRef { id } => {
                !self.index.contains_key(id) && resolve_function(self.ir, *id).is_some()
            }
            Expr::App { .. } => {
                let (head, _, args) = Self::flatten(e, &Vec::new());
                if !self.is_function(cx, head) {
                    return false;
                }
                match self.params_of(cx, head) {
                    Some(n) => args.len() < n,
                    None => false,
                }
            }
            _ => false,
        }
    }

    /// How many arguments a syntactic function takes before it yields a
    /// value: a primitive's arity, a lambda's nesting depth.
    fn params_of(&self, cx: &ExprCx<'a>, head: &Expr) -> Option<usize> {
        fn depth(mut e: &Expr) -> usize {
            let mut n = 0;
            while let Expr::Lam { body, .. } = e {
                n += 1;
                e = body;
            }
            n
        }
        match head {
            Expr::Lam { .. } => Some(depth(head)),
            Expr::Prim { p } => Some(p.arity()),
            Expr::Var { index } => match cx.lookup(*index) {
                Some(Bind::Fun(Fun::Lam { lam, .. })) => Some(depth(lam)),
                Some(Bind::Fun(Fun::Prim { p, args })) => {
                    Some(p.arity().saturating_sub(args.len()))
                }
                _ => None,
            },
            Expr::DeclRef { id } => resolve_function(self.ir, *id).map(depth),
            _ => None,
        }
    }

    fn args(
        &mut self,
        cx: &mut ExprCx<'a>,
        args: Vec<(&'a Expr, ExprPath)>,
    ) -> Option<Vec<Arg<'a>>> {
        let mut out = Vec::with_capacity(args.len());
        let mut failed = false;
        for (a, mut ap) in args {
            if self.is_function(cx, a) {
                match self.fun(cx, a, &ap) {
                    Some(f) => out.push(Arg::Fun(f)),
                    None => failed = true,
                }
            } else {
                match self.expr(cx, a, &mut ap) {
                    Some(v) => out.push(Arg::Val(v)),
                    None => failed = true,
                }
            }
        }
        if failed {
            None
        } else {
            Some(out)
        }
    }

    /// A (possibly nested) application in value position: saturate a
    /// primitive, or inline a lambda / function declaration.
    fn app(&mut self, cx: &mut ExprCx<'a>, e: &'a Expr, path: &mut ExprPath) -> Option<ExecExpr> {
        let (head, hp, args) = Self::flatten(e, path);
        let fun = self.fun(cx, head, &hp)?;
        let args = self.args(cx, args)?;
        self.apply(cx, fun, args, &hp)
    }

    /// Give a function fewer arguments than it takes: the result is still a
    /// function.  (Exactly as many is [`Self::apply`].)
    fn partial(
        &mut self,
        cx: &mut ExprCx<'a>,
        fun: Fun<'a>,
        args: Vec<Arg<'a>>,
        path: &ExprPath,
    ) -> Option<Fun<'a>> {
        match fun {
            Fun::Prim { p, args: mut pre } => {
                for a in args {
                    match a {
                        Arg::Val(v) => pre.push(v),
                        Arg::Fun(_) => {
                            self.unsupported(cx.owner, path, "a primitive over relationships");
                            return None;
                        }
                    }
                }
                if pre.len() >= p.arity() {
                    self.unsupported(cx.owner, path, "a primitive applied to too many arguments");
                    return None;
                }
                Some(Fun::Prim { p, args: pre })
            }
            Fun::Lam {
                mut lam,
                mut env,
                mut lets,
            } => {
                for a in args {
                    let Expr::Lam { body, .. } = lam else {
                        self.unsupported(cx.owner, path, "more arguments than lambda parameters");
                        return None;
                    };
                    match a {
                        Arg::Val(v) => {
                            let l = self.fresh_local();
                            lets.push((l, v));
                            env.push(Bind::Val(l));
                        }
                        Arg::Fun(f) => env.push(Bind::Fun(f)),
                    }
                    lam = body;
                }
                if !matches!(lam, Expr::Lam { .. }) {
                    self.unsupported(
                        cx.owner,
                        path,
                        "a fully applied rule where a rule was expected",
                    );
                    return None;
                }
                Some(Fun::Lam { lam, env, lets })
            }
        }
    }

    /// Apply a known function to exactly the arguments that make it a
    /// value: a saturated primitive, or a lambda body lowered in an
    /// environment of its parameters (values bound by `Let`, functions
    /// carried as bindings).
    fn apply(
        &mut self,
        cx: &mut ExprCx<'a>,
        fun: Fun<'a>,
        args: Vec<Arg<'a>>,
        path: &ExprPath,
    ) -> Option<ExecExpr> {
        match fun {
            Fun::Prim { p, args: mut pre } => {
                for a in args {
                    match a {
                        Arg::Val(v) => pre.push(v),
                        Arg::Fun(_) => {
                            self.unsupported(cx.owner, path, "a primitive over relationships");
                            return None;
                        }
                    }
                }
                if pre.len() != p.arity() {
                    self.unsupported(cx.owner, path, "a partially applied primitive");
                    return None;
                }
                self.prim(cx, p, pre, path)
            }
            Fun::Lam {
                mut lam,
                mut env,
                mut lets,
            } => {
                // Arguments are bound outermost first; the body runs in an
                // environment of its parameters over the lambda's own
                // (a literal lambda's is the caller's; an inlined
                // declaration's is empty).
                for a in args {
                    let Expr::Lam { body, .. } = lam else {
                        self.unsupported(cx.owner, path, "more arguments than lambda parameters");
                        return None;
                    };
                    match a {
                        Arg::Val(v) => {
                            let l = self.fresh_local();
                            lets.push((l, v));
                            env.push(Bind::Val(l));
                        }
                        Arg::Fun(f) => env.push(Bind::Fun(f)),
                    }
                    lam = body;
                }
                if matches!(lam, Expr::Lam { .. }) {
                    self.unsupported(cx.owner, path, "a lambda used as a value");
                    return None;
                }
                let mut inner = ExprCx {
                    owner: cx.owner,
                    grant: cx.grant.clone(),
                    env,
                };
                // Paths inside an inlined body carry no temporal identity
                // (typing forbids temporal forms under binders).
                let mut body_path = path.clone();
                body_path.push(0);
                let mut result = self.expr(&mut inner, lam, &mut body_path)?;
                for (l, v) in lets.into_iter().rev() {
                    result = ExecExpr::let_(l, v, result);
                }
                Some(result)
            }
        }
    }

    fn prim(
        &mut self,
        cx: &mut ExprCx<'a>,
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
            Prim::Eq { ty } => {
                if !ty.is_data() {
                    self.unsupported(cx.owner, path, "equality over relationships");
                    return None;
                }
                PrimOp::Eq
            }
            Prim::Not => PrimOp::Not,
            Prim::And => PrimOp::And,
            Prim::Or => PrimOp::Or,
            Prim::Ite { ty } => PrimOp::Ite { ty: ty.clone() },
            Prim::None { ty } => PrimOp::None { ty: ty.clone() },
            Prim::Some { ty } => PrimOp::Some { ty: ty.clone() },
            Prim::IsSome { ty } => PrimOp::IsSome { ty: ty.clone() },
            Prim::GetD { ty } => PrimOp::GetD { ty: ty.clone() },
            Prim::Nil { ty } => PrimOp::Nil { ty: ty.clone() },
            Prim::Cons { ty } => PrimOp::Cons { ty: ty.clone() },
            Prim::Length { ty } => PrimOp::Length { ty: ty.clone() },
            Prim::Take { ty } => PrimOp::Take { ty: ty.clone() },
            Prim::Drop { ty } => PrimOp::Drop { ty: ty.clone() },
            Prim::Reverse { ty } => PrimOp::Reverse { ty: ty.clone() },
            Prim::Head { ty } => PrimOp::Head { ty: ty.clone() },
            Prim::ToList { ty } => PrimOp::ToList { ty: ty.clone() },
            Prim::Pair { fst, snd } => PrimOp::Pair {
                fst: fst.clone(),
                snd: snd.clone(),
            },
            Prim::Fst { fst, snd } => PrimOp::Fst {
                fst: fst.clone(),
                snd: snd.clone(),
            },
            Prim::Snd { fst, snd } => PrimOp::Snd {
                fst: fst.clone(),
                snd: snd.clone(),
            },
        };
        if prim_result_ty(&op).iter().any(mentions_arrow) {
            self.unsupported(cx.owner, path, "a primitive over relationships");
            return None;
        }
        Some(ExecExpr::Prim { op, args })
    }
}

/// The `DeclId` at a plan position.
fn plan_driver_id(decls: &[DeclPlan], i: DeclIndex) -> DeclId {
    decls
        .get(i.0 as usize)
        .map(|d| d.id)
        .unwrap_or(DeclId::from_raw(u64::MAX))
}

#[derive(Clone)]
struct ExprCx<'e> {
    owner: DeclId,
    grant: Grant,
    /// De Bruijn environment: what each enclosing lambda parameter stands
    /// for, outermost first.
    env: Vec<Bind<'e>>,
}

impl<'e> ExprCx<'e> {
    fn lookup(&self, index: u32) -> Option<&Bind<'e>> {
        self.env
            .len()
            .checked_sub(1 + index as usize)
            .and_then(|i| self.env.get(i))
    }
}

/// The types a primitive's arguments or result range over, for the
/// "no relationship as a value" check.
fn prim_result_ty(op: &PrimOp) -> Vec<Ty> {
    match op {
        PrimOp::Ite { ty }
        | PrimOp::GetD { ty }
        | PrimOp::Some { ty }
        | PrimOp::None { ty }
        | PrimOp::IsSome { ty }
        | PrimOp::Nil { ty }
        | PrimOp::Cons { ty }
        | PrimOp::Length { ty }
        | PrimOp::Take { ty }
        | PrimOp::Drop { ty }
        | PrimOp::Reverse { ty }
        | PrimOp::Head { ty }
        | PrimOp::ToList { ty } => vec![ty.clone()],
        PrimOp::Pair { fst, snd } | PrimOp::Fst { fst, snd } | PrimOp::Snd { fst, snd } => {
            vec![fst.clone(), snd.clone()]
        }
        _ => Vec::new(),
    }
}

fn mentions_arrow(t: &Ty) -> bool {
    match t {
        Ty::Arr { .. } => true,
        Ty::Opt { inner } | Ty::List { elem: inner } => mentions_arrow(inner),
        Ty::Prod { fst, snd } => mentions_arrow(fst) || mentions_arrow(snd),
        Ty::Bool | Ty::Nat | Ty::Unit | Ty::Q { .. } | Ty::Sem { .. } => false,
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

fn collect_sems(t: &Ty, out: &mut BTreeSet<ConceptId>) {
    match t {
        Ty::Sem { id } => {
            out.insert(*id);
        }
        Ty::Arr { dom, cod } => {
            collect_sems(dom, out);
            collect_sems(cod, out);
        }
        Ty::Opt { inner } | Ty::List { elem: inner } => collect_sems(inner, out),
        Ty::Prod { fst, snd } => {
            collect_sems(fst, out);
            collect_sems(snd, out);
        }
        Ty::Bool | Ty::Nat | Ty::Unit | Ty::Q { .. } => {}
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
        Expr::Fold { f, z, l } => {
            collect_sync_sources(f, out);
            collect_sync_sources(z, out);
            collect_sync_sources(l, out);
        }
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
        Expr::Fold { f, z, l } => {
            for (i, sub) in [(0u8, f), (1, z), (2, l)] {
                path.push(i);
                collect_sites(owner, sub, path, out);
                path.pop();
            }
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
        Expr::Fold { f, z, l } => {
            inst_refs_through_functions(ir, f, out, expanding);
            inst_refs_through_functions(ir, z, out, expanding);
            inst_refs_through_functions(ir, l, out, expanding);
        }
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
        let exec =
            lower(ir, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap_or_else(|d| panic!("{d:?}"));
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
        let exec = lower(&ir, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap();
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
        let exec = lower(&ir, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap();
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
        let ds = lower(&ir, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap_err();
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
            lower(&ir, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap_err()[0]
                .code
                .as_str(),
            "backend.unsupported_higher_order"
        );
    }

    #[test]
    fn folds_lists_pairs_and_function_arguments_lower_first_order_and_agree() {
        let q0 = Ty::q(Dim::ZERO);
        let list = |items: Vec<Expr>| {
            items
                .into_iter()
                .rev()
                .fold(Expr::prim(Prim::Nil { ty: q0.clone() }), |tail, x| {
                    Expr::apps(Expr::prim(Prim::Cons { ty: q0.clone() }), [x, tail])
                })
        };
        // any := λp. λxs. fold (λx. λacc. or (p x) acc) false xs  — the library's `any`
        let any = Expr::lam(
            Ty::arr(q0.clone(), Ty::Bool),
            Expr::lam(
                Ty::list(q0.clone()),
                Expr::fold(
                    Expr::lam(
                        q0.clone(),
                        Expr::lam(
                            Ty::Bool,
                            Expr::apps(
                                Expr::prim(Prim::Or),
                                [Expr::app(Expr::var(3), Expr::var(1)), Expr::var(0)],
                            ),
                        ),
                    ),
                    Expr::BoolLit { value: false },
                    Expr::var(0),
                ),
            ),
        );
        // above := any (λx. 2 < x) xs   — a lambda passed as an argument
        let above = Expr::apps(
            any,
            [
                Expr::lam(
                    q0.clone(),
                    Expr::apps(
                        Expr::prim(Prim::Lt { dim: Dim::ZERO }),
                        [lit(2.0), Expr::var(0)],
                    ),
                ),
                Expr::decl(d(0)),
            ],
        );
        // total := fold add 0 xs  — a primitive as the step
        let total = Expr::fold(
            Expr::prim(Prim::Add { dim: Dim::ZERO }),
            lit(0.0),
            Expr::decl(d(0)),
        );
        // shifted := fold (λx. λacc. cons (add 5 x) acc) [] xs  — `map (+5)` by hand,
        // with `add 5` a partially applied primitive
        let shifted = Expr::fold(
            Expr::lam(
                q0.clone(),
                Expr::lam(
                    Ty::list(q0.clone()),
                    Expr::apps(
                        Expr::prim(Prim::Cons { ty: q0.clone() }),
                        [
                            Expr::app(
                                Expr::app(Expr::prim(Prim::Add { dim: Dim::ZERO }), lit(5.0)),
                                Expr::var(1),
                            ),
                            Expr::var(0),
                        ],
                    ),
                ),
            ),
            Expr::prim(Prim::Nil { ty: q0.clone() }),
            Expr::decl(d(0)),
        );
        // pairs := (head xs or 0, length xs)  and its first part
        let pair = Expr::apps(
            Expr::prim(Prim::Pair {
                fst: q0.clone(),
                snd: q0.clone(),
            }),
            [
                Expr::apps(
                    Expr::prim(Prim::GetD { ty: q0.clone() }),
                    [
                        Expr::app(Expr::prim(Prim::Head { ty: q0.clone() }), Expr::decl(d(0))),
                        lit(0.0),
                    ],
                ),
                Expr::app(
                    Expr::prim(Prim::Length { ty: q0.clone() }),
                    Expr::decl(d(0)),
                ),
            ],
        );
        let first = Expr::app(
            Expr::prim(Prim::Fst {
                fst: q0.clone(),
                snd: q0.clone(),
            }),
            Expr::decl(d(4)),
        );
        // same := xs == reverse (reverse xs)  — structural equality on lists
        let same = Expr::apps(
            Expr::prim(Prim::Eq {
                ty: Ty::list(q0.clone()),
            }),
            [
                Expr::decl(d(0)),
                Expr::app(
                    Expr::prim(Prim::Reverse { ty: q0.clone() }),
                    Expr::app(
                        Expr::prim(Prim::Reverse { ty: q0.clone() }),
                        Expr::decl(d(0)),
                    ),
                ),
            ],
        );
        // remembered := delay [] xs — a list in a state cell
        let remembered = Expr::delay(list(vec![]), Expr::decl(d(0)));
        let mut ir = ir_with(&[
            (0, None, Some(0)),
            (1, Some(above), Some(0)),
            (2, Some(total), Some(0)),
            (3, Some(shifted), Some(0)),
            (4, Some(pair), Some(0)),
            (5, Some(first), Some(0)),
            (6, Some(same), Some(0)),
            (7, Some(remembered), Some(0)),
        ]);
        let set_ty = |ir: &mut DesignIr, n: u64, ty: Ty| {
            ir.decls.get_mut(&d(n)).unwrap().interface.expected_type = ty;
        };
        set_ty(&mut ir, 0, Ty::list(q0.clone()));
        set_ty(&mut ir, 1, Ty::Bool);
        set_ty(&mut ir, 3, Ty::list(q0.clone()));
        set_ty(&mut ir, 4, Ty::prod(q0.clone(), q0.clone()));
        set_ty(&mut ir, 6, Ty::Bool);
        set_ty(&mut ir, 7, Ty::list(q0.clone()));
        let xs = |t: u64| Value::list((0..t + 1).map(|i| Value::scalar(i as f64)));
        let out = agree(&ir, 4, &|t, id| (id == d(0)).then(|| xs(t)));
        let q = |x: f64| Value::scalar(x);
        assert_eq!(out[0].values[1], Some(Value::boolean(false)));
        assert_eq!(out[3].values[1], Some(Value::boolean(true)));
        assert_eq!(out[3].values[2], Some(q(6.0)));
        assert_eq!(out[1].values[3], Some(Value::list([q(5.0), q(6.0)])));
        assert_eq!(out[2].values[4], Some(Value::pair(q(0.0), q(3.0))));
        assert_eq!(out[2].values[5], Some(q(0.0)));
        assert_eq!(out[2].values[6], Some(Value::boolean(true)));
        assert_eq!(out[1].values[7], Some(xs(0)));
        // the plan is first-order: no closure anywhere, one Fold per recursor
        let exec = lower(&ir, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap();
        assert!(exec.uses_lists());
        let folds = exec
            .decls
            .iter()
            .filter(|p| {
                matches!(
                    &p.kind,
                    DeclKind::Computed {
                        body: ExecExpr::Fold { .. }
                    }
                )
            })
            .count();
        assert_eq!(folds, 2);
        assert!(matches!(
            &exec.decls[1].kind,
            DeclKind::Computed { body: ExecExpr::Let { body, .. } }
                if matches!(**body, ExecExpr::Fold { .. })
        ));
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
        let exec = lower(&ir2, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap();
        let order: Vec<DeclId> = exec.decls.iter().map(|x| x.id).collect();
        assert_eq!(order, vec![d(3), d(2), d(0), d(1)]);
        assert_eq!(exec.decls[2].activation, Activation::Agnostic);
        assert!(exec.has_domains);
        agree(&ir2, 2, &|_, _| None);
        // no domain at all: everything runs every tick
        let ir3 = ir_with(&[(0, Some(lit(1.0)), None)]);
        let exec = lower(&ir3, "t", &BTreeMap::new(), &BTreeMap::new()).unwrap();
        assert!(!exec.has_domains);
        let r = interp::step(&exec, 0, &[], &CellState::init(&exec), &[]).unwrap();
        assert_eq!(r.values[0], Some(Value::scalar(1.0)));
    }
}
