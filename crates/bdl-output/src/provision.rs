//! Source provision (`BDL_FV/BDL/Surface/Provision.lean`, Phase 13;
//! `Assignment.lean`, Phase 16).
//!
//! A Source stays a declaration of the design: `() -> B`, no realization,
//! a value the environment supplies once per activation.  *What* supplies
//! it on a board is deployment data — a **provider profile** chosen per
//! device binding — and the bridge is a **transducer**: a closed, pure,
//! typed Core term `raw → rep` that lifts a device's raw reading into the
//! representation the Source's concept carries.  The transducer is the
//! mirror image of the encoder (`realization`): typed under
//! [`Grant::None`] it constructs nothing; the concept itself is constructed
//! by the provision, under the Source's own grant (`Provision.one`:
//! `s := mk c (tr r)`), so the design still owns every Sem value.
//!
//! The judgments are the encoder's, mirrored, and kept apart for the same
//! reason (each can fail alone):
//!
//! * [`well_formed`] — the transducer is a typed pure function between data
//!   (`Channel.WF`);
//! * [`fits`] — its codomain is the representation the Source's concept
//!   carries (`Fits Θ τ ch`);
//! * hardware solvable — the profile's requirement template places on the
//!   target (`bdl-hardware`, composed by the compiler);
//! * backend supported — the chosen target's adapter has a reader for the
//!   profile's raw shape (the target entry's, composed by the compiler).
//!
//! The first two are the **semantic contract** (`InputContract`); the
//! last two are feasibility.  They are never folded into one verdict.
//!
//! What this module does not do: decide when a reading is taken.  A
//! provider observes once per activation of the Source's domain, in the
//! adapter's order; anything finer — a batch of occurrences inside one
//! tick, a merged order across providers — is FVI-0029, open.

use crate::realization::Impurity;
use bdl_catalogue::{Catalogue, InputProfile, Transducer};
use bdl_check::{infer, Grant};
use bdl_ir::{DesignIr, Expr, Ty};
use bdl_model::surface::{DeviceBinding, DeviceKind};
use bdl_model::ConceptId;
use serde::{Deserialize, Serialize};

/// Why a transducer is not well formed (`¬ Channel.WF`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TransducerFault {
    /// `raw` or `rep` is not sem-free data.
    NotData {
        which: String,
        ty: Ty,
    },
    Impure(Impurity),
    IllTyped {
        technical: String,
    },
    /// Typed, but not `raw → rep`.
    WrongType {
        expected: Ty,
        found: Ty,
    },
}

/// Why a transducer does not fit a Source (`¬ Fits`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProvisionFitFault {
    /// The Source produces a concept with no representation yet.
    NoRepresentation { concept: ConceptId },
    /// The concept's representation differs from the transducer's codomain.
    Representation { carried: Ty, produced: Ty },
}

/// Where a device binding's provision stands, target-independently.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProvisionStatus {
    /// No profile chosen: the device places by kind; nothing is read.
    NotChosen,
    /// The persisted id is not in the catalogue.
    UnknownProfile,
    /// The profile's requirement template is not the device's kind.
    KindMismatch { profile_kind: DeviceKind },
    /// A catalogue defect: the transducer fails `Channel.WF`.
    TransducerInvalid(TransducerFault),
    /// `¬ Fits`: the transducer produces another representation.
    Incompatible(ProvisionFitFault),
    /// `Channel.WF ∧ Fits`: the semantic contract holds.  Admissible once
    /// the hardware places and the backend reads.
    Valid,
}

/// The provision judgment of one device binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvisionCheck {
    pub profile: Option<InputProfile>,
    pub status: ProvisionStatus,
}

impl ProvisionCheck {
    pub fn profile_known(&self) -> bool {
        self.profile.is_some()
    }
    pub fn transducer_well_formed(&self) -> bool {
        matches!(
            self.status,
            ProvisionStatus::Valid | ProvisionStatus::Incompatible(_)
        )
    }
    pub fn representation_fits(&self) -> bool {
        matches!(self.status, ProvisionStatus::Valid)
    }
    pub fn is_valid(&self) -> bool {
        matches!(self.status, ProvisionStatus::Valid)
    }
    /// The binding blocks an artefact: a profile is chosen and is not valid.
    pub fn is_blocking(&self) -> bool {
        !matches!(
            self.status,
            ProvisionStatus::Valid | ProvisionStatus::NotChosen
        )
    }
}

/// `Channel.WF Θ` in the empty declaration environment under no grant:
/// `raw` and `rep` are sem-free data, the term is pure, and it has type
/// `raw → rep`.
pub fn well_formed(t: &Transducer) -> Result<(), TransducerFault> {
    for (which, ty) in [("raw reading", &t.raw), ("representation", &t.rep)] {
        if !(ty.is_sem_free() && ty.is_data()) {
            return Err(TransducerFault::NotData {
                which: which.into(),
                ty: ty.clone(),
            });
        }
    }
    crate::realization::purity(&t.transduce).map_err(TransducerFault::Impure)?;
    let empty = DesignIr::default();
    let found = infer(&empty, &Grant::None, &[], &t.transduce).map_err(|err| {
        TransducerFault::IllTyped {
            technical: format!("{:?} at {:?}", err.kind, err.path),
        }
    })?;
    let expected = Ty::arr(t.raw.clone(), t.rep.clone());
    if found != expected {
        return Err(TransducerFault::WrongType { expected, found });
    }
    Ok(())
}

/// `Fits Θ τ ch`: the Source's concept carries the transducer's codomain.
pub fn fits(
    produces: ConceptId,
    representation_of: impl Fn(ConceptId) -> Option<Ty>,
    t: &Transducer,
) -> Result<(), ProvisionFitFault> {
    let carried = representation_of(produces)
        .ok_or(ProvisionFitFault::NoRepresentation { concept: produces })?;
    if carried == t.rep {
        Ok(())
    } else {
        Err(ProvisionFitFault::Representation {
            carried,
            produced: t.rep.clone(),
        })
    }
}

/// `Provision.one`'s realization body: `mk c (tr r)` — the concept the
/// Source produces, constructed from the transduced raw reading `r`.  Only
/// this term constructs; it lives in the Source's domain under the
/// Source's grant, so behaviour still owns the value.
pub fn provision_body(produces: ConceptId, t: &Transducer, raw: Expr) -> Expr {
    Expr::mk(produces, Expr::app(t.transduce.clone(), raw))
}

/// Judge one device binding against the Source it provides.  `produces` is
/// the Source's concept when the binding names a Source the design knows;
/// `None` leaves the fit undecided.  The judgment takes the catalogue
/// entry's *profile* and never its origin (`assignSource_origin_irrelevant`).
pub fn check_provider(
    catalogue: &Catalogue,
    binding: &DeviceBinding,
    produces: Option<ConceptId>,
    representation_of: impl Fn(ConceptId) -> Option<Ty>,
) -> ProvisionCheck {
    let Some(id) = &binding.provider else {
        return ProvisionCheck {
            profile: None,
            status: ProvisionStatus::NotChosen,
        };
    };
    let Some(profile) = catalogue.input(id).map(|e| e.profile.clone()) else {
        return ProvisionCheck {
            profile: None,
            status: ProvisionStatus::UnknownProfile,
        };
    };
    let status = if profile.kind != binding.kind {
        ProvisionStatus::KindMismatch {
            profile_kind: profile.kind,
        }
    } else if let Err(fault) = well_formed(&profile.transducer) {
        ProvisionStatus::TransducerInvalid(fault)
    } else {
        match produces {
            Some(c) => match fits(c, representation_of, &profile.transducer) {
                Ok(()) => ProvisionStatus::Valid,
                Err(fault) => ProvisionStatus::Incompatible(fault),
            },
            None => ProvisionStatus::Valid,
        }
    };
    ProvisionCheck {
        profile: Some(profile),
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_catalogue::{InputEntry, Origin, PackageId};
    use bdl_model::{DeclId, DeviceId, Dim, InputProfileId};
    use std::collections::BTreeMap;

    fn button() -> ConceptId {
        ConceptId::from_raw(3)
    }
    fn theta(s: ConceptId) -> Option<Ty> {
        (s == button()).then_some(Ty::Bool)
    }
    fn binding(profile: &str, kind: DeviceKind) -> DeviceBinding {
        DeviceBinding {
            id: DeviceId::from_raw(2),
            name: "btn".into(),
            kind,
            output: None,
            realization: None,
            source: Some(DeclId::from_raw(9)),
            provider: Some(InputProfileId(profile.into())),
            fixed_pins: BTreeMap::new(),
        }
    }

    #[test]
    fn every_builtin_input_profile_is_well_formed() {
        for p in bdl_catalogue::builtin_inputs() {
            assert_eq!(well_formed(&p.transducer), Ok(()), "{}", p.id);
        }
    }

    #[test]
    fn both_polarities_fit_a_truth_valued_source() {
        let cat = Catalogue::builtin();
        for id in ["gpio_level_in", "gpio_level_in_low"] {
            let c = check_provider(
                &cat,
                &binding(id, DeviceKind::DigitalInput),
                Some(button()),
                theta,
            );
            assert!(c.is_valid(), "{id}: {:?}", c.status);
        }
    }

    #[test]
    fn a_level_valued_source_does_not_fit_a_line() {
        let cat = Catalogue::builtin();
        let level = |s: ConceptId| (s == button()).then_some(Ty::q(Dim::ZERO));
        let c = check_provider(
            &cat,
            &binding("gpio_level_in", DeviceKind::DigitalInput),
            Some(button()),
            level,
        );
        assert!(matches!(c.status, ProvisionStatus::Incompatible(_)));
        assert!(c.transducer_well_formed());
        assert!(!c.representation_fits());
    }

    #[test]
    fn unknown_kind_mismatch_and_not_chosen() {
        let cat = Catalogue::builtin();
        let c = check_provider(
            &cat,
            &binding("nope", DeviceKind::DigitalInput),
            Some(button()),
            theta,
        );
        assert_eq!(c.status, ProvisionStatus::UnknownProfile);
        assert!(c.is_blocking());
        let c = check_provider(
            &cat,
            &binding("gpio_level_in", DeviceKind::PwmChannel),
            Some(button()),
            theta,
        );
        assert!(matches!(c.status, ProvisionStatus::KindMismatch { .. }));
        let mut b = binding("gpio_level_in", DeviceKind::DigitalInput);
        b.provider = None;
        let c = check_provider(&cat, &b, Some(button()), theta);
        assert_eq!(c.status, ProvisionStatus::NotChosen);
        assert!(!c.is_blocking());
    }

    #[test]
    fn origin_never_reaches_the_judgment() {
        // The same profile, once builtin and once from a package: the
        // judgment is the same value (`assignSource_origin_irrelevant`).
        let builtin = Catalogue::builtin();
        let mut packaged = Catalogue::default();
        for e in &builtin.inputs {
            packaged
                .add_input(InputEntry {
                    origin: Origin::Package {
                        id: PackageId("vendor.io".into()),
                    },
                    profile: e.profile.clone(),
                })
                .expect("fresh id");
        }
        let b = binding("gpio_level_in_low", DeviceKind::DigitalInput);
        assert_eq!(
            check_provider(&builtin, &b, Some(button()), theta),
            check_provider(&packaged, &b, Some(button()), theta)
        );
        // and a package cannot shadow a builtin id
        let mut both = Catalogue::builtin();
        assert!(both
            .add_input(InputEntry {
                origin: Origin::Package {
                    id: PackageId("vendor.io".into()),
                },
                profile: builtin.inputs[0].profile.clone(),
            })
            .is_err());
    }
}
