//! The reactive semantics of BDL, executable.
//!
//! * [`graph`]     — dependency graphs from realizations (`refs` / `inst_refs`).
//! * [`causality`] — the kernel's `Causal`: the instantaneous graph is acyclic;
//!   yields a deterministic evaluation order.
//! * [`clocks`]    — the kernel's `Clocked` judgment: references stay in their
//!   domain or are agnostic; `sync` is the only crossing.
//! * [`value`]     — runtime values, keeping concept identity and dimension.
//! * [`eval`]      — the reference evaluator (`MEv`) with explicit delay/sync
//!   state cells, previous/next state, and per-domain published snapshots.
//! * [`simulate`]  — schedules, input traces, ticks, and the resulting trace.
//! * [`capacity`]  — cross-domain window sizes under a schedule: the
//!   production counterpart of FV `Validation/Capacity.lean` (Phase 9a).
//!
//! Everything is pure and deterministic: the same design, schedule and
//! inputs yield the same trace, whatever order a host processes
//! simultaneously active domains in.  This evaluator is the executable
//! definition of BDL runtime behaviour (`docs/spec/runtime-semantics.md`).

#![forbid(unsafe_code)]

pub mod capacity;
pub mod causality;
pub mod clocks;
pub mod eval;
pub mod graph;
pub mod simulate;
pub mod value;

#[cfg(test)]
pub(crate) mod test_designs;

pub use causality::{check_causality, CausalityAnalysis};
pub use clocks::{check_clocks, ClockAnalysis};
pub use eval::{StateCellId, TickInput};
pub use graph::{analyze_dependencies, DependencyGraph};
pub use simulate::{InputTrace, Schedule, Simulation, SimulationTrace, TickSample};
pub use value::Value;
