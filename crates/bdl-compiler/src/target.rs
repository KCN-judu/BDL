//! Compiling for one embedded target: the solved deployment becomes the
//! platform adapter's plan, and the generated crate gains the firmware
//! beside its target-independent core (docs/architecture/embedded-adapter.md).
//!
//! ```text
//! compile_for_target(snapshot, target, options)
//!   = analyze → analyze_deployment(target) → readiness → lower
//!     → adapter_plan (sink ↔ assigned resource, schedule, arena)
//!     → generate_with_adapter
//! ```
//!
//! Every judgment stays where it was: behavior in `analyze`, admissibility
//! and placement in `analyze_deployment`, the machine sinks in the
//! lowering.  This pass only *reads* the assignment and refuses what it
//! cannot bind — a sink without a placed resource, a resource without the
//! capability, a profile the adapter has no sink or reader for, a Source
//! no device provides or whose provider is not admissible, a collection
//! the design does not bound — with `adapter.*` diagnostics.  It never
//! chooses a pin.

use crate::backend::{compile_with_target, CompileArtifact, CompileOptions};
use crate::collections::{CollectionsReadiness, CollectionsReport};
use crate::{analyze_deployment, DeploymentAnalysis, DeploymentStatus};
use bdl_codegen_rust::adapter::{AdapterPlan, ProviderBinding, SinkBinding, SinkKind, SourceKind};
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_exec_ir::ExecIr;
use bdl_hardware::{Capability, Hardware};
use bdl_model::surface::ProjectSnapshot;
use bdl_model::InputProfileId;
use bdl_reactive::Schedule;

/// The reader a provider profile's raw reading is taken through, with
/// the peripheral configuration the profile prescribes.  Profile-level
/// knowledge, not a family's; a family says whether it *reads* the kind
/// (`Entry::reads`).
pub fn source_kind(profile: &InputProfileId) -> Option<SourceKind> {
    match profile.as_str() {
        "gpio_level_in" => Some(SourceKind::LevelPullDown),
        "gpio_level_in_low" => Some(SourceKind::LevelPullUp),
        _ => None,
    }
}

/// What compiling for a target adds to [`CompileOptions`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetOptions {
    /// The base tick the firmware's timer fires at (`--tick-micros`).
    pub tick_micros: u64,
}

impl Default for TargetOptions {
    fn default() -> Self {
        TargetOptions {
            tick_micros: 10_000,
        }
    }
}

/// The arena granularity: bounds are rounded up to whole KiB.
pub const ARENA_GRANULE: u64 = 1024;

/// [`crate::compile`] for one target: the deployment must be feasible on
/// it and every chosen realization admissible; the artefact then carries
/// the adapter (`src/adapter.rs`, `src/bin/rp2040.rs`, …) and the
/// manifest its `adapter` entry.
pub fn compile_for_target(
    snapshot: &ProjectSnapshot,
    target: &Hardware,
    target_options: &TargetOptions,
    options: &CompileOptions,
) -> CompileArtifact {
    let deployment = analyze_deployment(snapshot, target);
    compile_with_target(snapshot, options, &deployment, target, target_options)
}

/// The plan for one lowered program on one solved deployment.  Pure: the
/// same inputs give the same plan or the same diagnostics.
pub fn adapter_plan(
    exec: &ExecIr,
    target: &Hardware,
    deployment: &DeploymentAnalysis,
    collections: &CollectionsReport,
    schedule: Option<&Schedule>,
    target_options: &TargetOptions,
) -> Result<AdapterPlan, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    let entry = bdl_codegen_rust::targets::Entry::for_board(&target.family, &target.name);
    if entry.is_none() {
        diagnostics.push(refuse(
            "adapter.target_unsupported",
            format!("No firmware can be generated for {} yet.", target.display()),
            "The board is known to the placement, but no target entry drives its peripherals; the Raspberry Pi Pico and the Arduino Nano have one.",
            format!("no target entry for family `{}`, board `{}`", target.family, target.name),
        ));
    }
    // An incomplete deployment whose only gap is an unprovided Source is
    // named by the Source below, not by the placement.
    let placement_incomplete = deployment.status == DeploymentStatus::Incomplete
        && (!deployment.unbound_devices.is_empty() || !deployment.unrealised_outputs.is_empty());
    if deployment.status == DeploymentStatus::Infeasible || placement_incomplete {
        diagnostics.push(refuse(
            "adapter.deployment_not_feasible",
            format!(
                "The design is not placed on {} yet.",
                target.display()
            ),
            "Firmware is generated only from a placement every device fits into; the Deploy page names what is missing or blocked.",
            format!("deployment status {:?}", deployment.status),
        ));
    }
    if deployment.realization_blocked() {
        diagnostics.push(refuse(
            "adapter.realization_invalid",
            "A device's realization cannot be used.".into(),
            "Choose a realization profile the Deploy page accepts for every device before generating firmware.",
            "DeploymentAnalysis::realization_blocked".into(),
        ));
    }
    if deployment.provision_blocked() {
        diagnostics.push(refuse(
            "adapter.provider_invalid",
            "A device's provider cannot be used.".into(),
            "Choose a provider profile the Deploy page accepts for every Source before generating firmware.",
            "DeploymentAnalysis::provision_blocked".into(),
        ));
    }
    // Every input slot of the core is a Source; each needs an admissible
    // provider on this board, or the firmware could not take a reading.
    // Whether a provider exists and whether this family reads it need no
    // placement; the resource it reads from does.
    let entry_ref = entry.as_ref();
    let mut provided = Vec::new();
    for i in &exec.inputs {
        let Some(decl) = exec.decl(i.decl) else {
            continue;
        };
        let Some(p) = exec.providers.iter().find(|p| p.slot == i.slot) else {
            let unprovided = deployment
                .provisions
                .get(&decl.id)
                .is_none_or(|p| p.device.is_none());
            if unprovided {
                diagnostics.push(refuse(
                    "adapter.source_unprovided",
                    format!("No device provides {} on {}.", decl.name, target.display()),
                    "A Source is a value the environment supplies; on a board a device must provide it. Connect a device to it on the Deploy page.",
                    format!("input slot {:?} has no provider", i.slot),
                ));
            } else {
                diagnostics.push(refuse(
                    "adapter.provider_unspecified",
                    format!("The device for {} has no provider chosen.", decl.name),
                    "Choose a provider profile for the device on the Deploy page.",
                    format!("input slot {:?}: device bound, no profile", i.slot),
                ));
            }
            continue;
        };
        let Some(kind) = source_kind(&p.profile).filter(|k| entry_ref.is_some_and(|e| e.reads(*k)))
        else {
            diagnostics.push(refuse(
                "adapter.provider_unsupported",
                format!(
                    "{} provides {} as `{}`, which {} cannot read yet.",
                    p.device_name,
                    decl.name,
                    p.profile,
                    target.display()
                ),
                "This board's firmware has no reader for the profile; choose a board that reads it, or wait for the adapter that does.",
                format!("input slot {:?} profile {}: no reader in the target entry", i.slot, p.profile),
            ));
            continue;
        };
        provided.push((i.slot, p, decl, kind));
    }
    if !diagnostics.is_empty() {
        // Without a placement there is nothing to bind; the sinks would
        // only repeat the refusal.
        sort_diagnostics(&mut diagnostics);
        return Err(diagnostics);
    }

    let mut providers = Vec::new();
    for (slot, p, _decl, kind) in provided {
        let i_slot = slot;
        let capability = Capability::DigitalIn;
        let requirement = deployment
            .requirements
            .iter()
            .find(|r| r.id.device == p.device && r.capability == capability);
        let resource = requirement.and_then(|r| {
            deployment
                .assignment
                .as_ref()
                .and_then(|a| a.get(&r.id))
                .cloned()
        });
        let Some(resource) = resource else {
            diagnostics.push(refuse(
                "adapter.source_unbound",
                format!(
                    "{} has no {} line on {}.",
                    p.device_name,
                    capability.label(),
                    target.display()
                ),
                "Every provided Source needs the board resource its profile reads; the placement did not give this device one.",
                format!("input slot {:?}: no assigned requirement with {:?}", i_slot, capability),
            ));
            continue;
        };
        if !target.supports(&resource, capability) {
            diagnostics.push(refuse(
                "adapter.resource_incompatible",
                format!(
                    "{} was placed on {}, which cannot carry {} on {}.",
                    p.device_name,
                    resource.0,
                    capability.label(),
                    target.display()
                ),
                "The placement and the profile disagree about this line; this is a compiler inconsistency, and no other pin is substituted.",
                format!("{} lacks {:?}", resource.0, capability),
            ));
            continue;
        }
        providers.push(ProviderBinding {
            slot: i_slot,
            device: p.device,
            profile: p.profile.clone(),
            kind,
            resource: resource.0.clone(),
        });
    }
    let supports_collections = entry.as_ref().is_some_and(|e| e.supports_collections());
    let arena_bytes = match collections.readiness {
        CollectionsReadiness::ScalarOnly => None,
        CollectionsReadiness::Bounded if !supports_collections => {
            diagnostics.push(refuse(
                "adapter.collections_unsupported",
                format!(
                    "{} cannot carry the collections this design keeps.",
                    target.display()
                ),
                "This board has no memory for a collection arena; a design that carries lists needs a board that does (docs/spec/deployment-capacity.md).",
                format!("collections readiness {:?} on `{}`", collections.readiness, target.name),
            ));
            None
        }
        CollectionsReadiness::Bounded => {
            let floor =
                collections.state_bytes_max.unwrap_or(0) + collections.tick_bytes_max.unwrap_or(0);
            Some(floor.div_ceil(ARENA_GRANULE).max(1) * ARENA_GRANULE)
        }
        CollectionsReadiness::InputBounded | CollectionsReadiness::Unbounded => {
            diagnostics.push(refuse(
                "adapter.collections_unbounded",
                "A collection in this design has no bound the board can be sized for.".into(),
                "Firmware sizes its memory from the design's bounds; a collection as large as its inputs, or one that grows without bound, cannot be placed (docs/spec/deployment-capacity.md).",
                format!("collections readiness {:?}", collections.readiness),
            ));
            None
        }
    };

    let mut sinks = Vec::new();
    for s in &exec.sinks {
        let kind = match s.profile.as_str() {
            "pwm_duty8" | "pwm_duty4" => SinkKind::PwmDuty8,
            "gpio_level" => SinkKind::Level,
            other => {
                diagnostics.push(refuse(
                    "adapter.profile_unsupported",
                    format!(
                        "{} realises its output as `{other}`, which this board's adapter cannot drive yet.",
                        s.device_name
                    ),
                    "The first adapter drives PWM duties and digital lines; choose one of those profiles, or wait for the adapter that carries this one.",
                    format!("sink {:?} profile {other}", s.slot),
                ));
                continue;
            }
        };
        let capability = match kind {
            SinkKind::PwmDuty8 => Capability::Pwm,
            SinkKind::Level => Capability::DigitalOut,
        };
        let requirement = deployment
            .requirements
            .iter()
            .find(|r| r.id.device == s.device && r.capability == capability);
        let resource = requirement.and_then(|r| {
            deployment
                .assignment
                .as_ref()
                .and_then(|a| a.get(&r.id))
                .cloned()
        });
        let Some(resource) = resource else {
            diagnostics.push(refuse(
                "adapter.sink_unbound",
                format!(
                    "{} has no {} line on {}.",
                    s.device_name,
                    capability.label(),
                    target.display()
                ),
                "Every realised output needs the board resource its profile drives; the placement did not give this device one.",
                format!("sink {:?}: no assigned requirement with {:?}", s.slot, capability),
            ));
            continue;
        };
        if !target.supports(&resource, capability) {
            diagnostics.push(refuse(
                "adapter.resource_incompatible",
                format!(
                    "{} was placed on {}, which cannot carry {} on {}.",
                    s.device_name,
                    resource.0,
                    capability.label(),
                    target.display()
                ),
                "The placement and the profile disagree about this line; this is a compiler inconsistency, and no other pin is substituted.",
                format!("{} lacks {:?}", resource.0, capability),
            ));
            continue;
        }
        sinks.push(SinkBinding {
            slot: s.slot,
            device: s.device,
            profile: s.profile.clone(),
            kind,
            resource: resource.0.clone(),
            unit: target.unit_of(&resource, capability).map(|u| u.0),
        });
    }

    let periods: Vec<u64> = exec
        .clocks
        .iter()
        .map(|c| {
            schedule
                .and_then(|s| s.periods.get(&c.id).copied())
                .unwrap_or(1)
        })
        .collect();
    if target_options.tick_micros == 0 {
        diagnostics.push(refuse(
            "adapter.tick_invalid",
            "The base tick must be a positive number of microseconds.".into(),
            "Give `--tick-micros` a value above zero.",
            "tick_micros = 0".into(),
        ));
    }

    if diagnostics.is_empty() {
        Ok(AdapterPlan {
            board: target.name.clone(),
            family: target.family.clone(),
            tick_micros: target_options.tick_micros,
            periods,
            sinks,
            providers,
            arena_bytes,
        })
    } else {
        sort_diagnostics(&mut diagnostics);
        Err(diagnostics)
    }
}

fn refuse(code: &'static str, message: String, explanation: &str, technical: String) -> Diagnostic {
    Diagnostic::error(code, Entity::Project, message)
        .explain(explanation)
        .technical(technical)
}
