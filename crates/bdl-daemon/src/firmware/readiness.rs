//! Whether a board's firmware can be built from the project as it is,
//! and what stands in the way — composed at the read-model edge from the
//! deployment report and the compiler's own dry run, so the page that
//! offers *Build* offers it exactly when the build's first stage would
//! pass.  Nothing here judges; it orders what the judges said.

use super::Plan;
use bdl_compiler::{DeploymentReport, MissingKind};
use bdl_diagnostics::Severity;
use bdl_model::surface::ProjectSnapshot;

/// One thing that stops a build, in product language, with the object it
/// is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blocker {
    pub code: String,
    pub message: String,
    pub explanation: String,
    pub output: Option<u64>,
    pub device: Option<u64>,
    pub mapping: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Readiness {
    pub ready: bool,
    /// The smallest actionable thing first: the report's missing items in
    /// its order (semantic, then deployment), the placement's dead end,
    /// then what the firmware pass alone refuses.
    pub blockers: Vec<Blocker>,
}

fn kind_code(k: MissingKind) -> &'static str {
    match k {
        MissingKind::RelationshipNotChecking => "relationship_not_checking",
        MissingKind::NotCausal => "not_causal",
        MissingKind::NotClockConsistent => "not_clock_consistent",
        MissingKind::OutputNoDomain => "output_no_domain",
        MissingKind::OutputNoDriver => "output_no_driver",
        MissingKind::OutputConnectionInvalid => "output_connection_invalid",
        MissingKind::OutputNoDevice => "output_no_device",
        MissingKind::DeviceNoOutput => "device_no_output",
        MissingKind::RealizationInvalid => "realization_invalid",
        MissingKind::SourceNoDevice => "source_no_device",
        MissingKind::ProviderInvalid => "provider_invalid",
    }
}

/// The readiness of `snapshot` for the plan's board, given the report the
/// Deploy page already renders.  `None` plan (no target entry) is never
/// ready: the board has no firmware.
pub fn readiness(
    plan: Option<&Plan>,
    snapshot: &ProjectSnapshot,
    report: &DeploymentReport,
) -> Readiness {
    let mut blockers: Vec<Blocker> = report
        .missing
        .iter()
        .map(|m| Blocker {
            code: kind_code(m.kind).to_owned(),
            message: m.message.clone(),
            explanation: m.explanation.clone(),
            output: m.output.map(|o| o.raw()),
            device: m.device.map(|d| d.raw()),
            mapping: m.mapping.map(|d| d.raw()),
        })
        .collect();
    if let Some(b) = &report.blocker {
        blockers.push(Blocker {
            code: "placement_blocked".to_owned(),
            message: b.message.clone(),
            explanation: b.explanation.clone(),
            output: None,
            device: Some(b.device.raw()),
            mapping: None,
        });
    }
    let Some(plan) = plan else {
        blockers.push(Blocker {
            code: "adapter.target_unsupported".to_owned(),
            message: format!(
                "No firmware can be built for {} yet.",
                report.target.display_name
            ),
            explanation:
                "The board is known to the placement, but nothing generates firmware for it."
                    .to_owned(),
            output: None,
            device: None,
            mapping: None,
        });
        return Readiness {
            ready: false,
            blockers,
        };
    };
    // The firmware pass's own refusals — what no earlier judgment covers
    // (a provider this board's adapter cannot read, an unbounded
    // collection, a profile with no sink here) — after the report's, and
    // only those the report did not already say.
    let art = plan.compile(snapshot);
    let ready = art.generated.is_some() && blockers.is_empty();
    if !ready && blockers.is_empty() {
        for d in art
            .diagnostics
            .iter()
            .chain(art.analysis.diagnostics.iter())
            .filter(|d| d.severity == Severity::Error)
        {
            let code = d.code.as_str().to_owned();
            if blockers.iter().any(|b| b.code == code) {
                continue;
            }
            // A provider refusal is about the device that provides the
            // Source: the one without a provider, else the first with one.
            let device = if code.starts_with("adapter.provider") {
                let providing: Vec<_> = snapshot
                    .design
                    .devices
                    .values()
                    .filter(|dv| dv.source.is_some())
                    .collect();
                providing
                    .iter()
                    .find(|dv| dv.provider.is_none())
                    .or(providing.first())
                    .map(|dv| dv.id.raw())
            } else {
                None
            };
            blockers.push(Blocker {
                code,
                message: d.message.clone(),
                explanation: d.explanation.clone(),
                output: None,
                device,
                mapping: match &d.entity {
                    bdl_diagnostics::Entity::Mapping { id } => Some(id.raw()),
                    _ => None,
                },
            });
        }
    }
    if !ready && blockers.is_empty() {
        blockers.push(Blocker {
            code: "build.not_ready".to_owned(),
            message: format!("The design cannot be built for {} yet.", report.target.display_name),
            explanation: "The compiler refused the firmware without naming a reason the page knows; the build's own report will say more.".to_owned(),
            output: None,
            device: None,
            mapping: None,
        });
    }
    Readiness { ready, blockers }
}
