//! Output realization (`BDL_FV/BDL/Surface/OutputRealization.lean`, Phase 14).
//!
//! A physical output stays logical: it accepts a semantic value in a
//! domain and nothing more.  *How* that value reaches a board is
//! deployment data — a **realization profile** chosen per device binding —
//! and the bridge between the two is an **encoder**: a closed, pure, typed
//! Core term `rep → raw` that lowers the output value's representation
//! into a machine-facing raw command.  The encoder is data (`Encoder.WF`),
//! constructs no semantic value (`encoder_constructs_nothing`: it is typed
//! under [`Grant::None`]), and refers to no declaration and no domain, so
//! it can only be evaluated where its argument is — in the output's own
//! domain.
//!
//! Three judgments make a realization admissible, and they are kept apart
//! because each can fail on its own (`FVD-0139`; the earlier "fits and
//! allocates" pair was wrong on the same evidence):
//!
//! * [`well_formed`] — the encoder is a typed pure function between data;
//! * [`fits`] — its domain is the representation the output's concept
//!   carries (`EFits`);
//! * hardware solvable — the profile's requirement template places on the
//!   target; that judgment lives in `bdl-hardware` and is composed by the
//!   compiler's deployment analysis, never here.
//!
//! Nothing here evaluates anything or changes a design: [`encoder_body`]
//! is the term a lowering adds *below* the behavior IR, and behavior does
//! not see it (`behavior_unchanged`, `lower_transparent`).

use bdl_check::{infer, Grant};
use bdl_ir::{DesignIr, Expr, Prim, Scalar, Ty};
use bdl_model::surface::{DeviceBinding, DeviceKind};
use bdl_model::{DeclId, Dim, OutputProfileId, SemanticId};
use serde::{Deserialize, Serialize};

/// `Encoder`: a closed term from a representation to a raw command type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encoder {
    pub rep: Ty,
    pub raw: Ty,
    pub encode: Expr,
}

/// `DeviceOutputProfile`: an encoder plus the hardware it needs.  A profile
/// is not a device kind — one output may be realised by several profiles,
/// and several profiles may share a requirement template.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputProfile {
    pub id: OutputProfileId,
    pub display_name: String,
    pub description: String,
    pub encoder: Encoder,
    /// The hardware requirement template (`bdl-hardware::devices::needs`).
    pub kind: DeviceKind,
}

/// Why an encoder is not pure: it reaches outside its argument.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Impurity {
    /// Refers to a declaration of the design.
    DeclRef { id: DeclId },
    /// Remembers a previous activation.
    Delay,
    /// Reads another domain — an implicit clock crossing.
    Sync { src: bdl_model::ClockId },
    /// Constructs a semantic value.
    Constructs { concept: SemanticId },
}

/// Why an encoder is not well formed (`¬ Encoder.WF`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncoderFault {
    /// `rep` or `raw` is not sem-free data.
    NotData {
        which: String,
        ty: Ty,
    },
    Impure(Impurity),
    /// The checker's error, rendered.
    IllTyped {
        technical: String,
    },
    /// Typed, but not `rep → raw`.
    WrongType {
        expected: Ty,
        found: Ty,
    },
}

/// Why an encoder does not fit an output (`¬ EFits`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FitFault {
    /// The output accepts a concept with no representation yet.
    NoRepresentation { concept: SemanticId },
    /// The concept's representation (or the accepted data type) differs
    /// from the encoder's domain.
    Representation { carried: Ty, expected: Ty },
}

/// Where a device binding's realization stands, target-independently.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RealizationStatus {
    /// No profile chosen: the device places by kind; no command is lowered.
    NotChosen,
    /// The persisted id is not in the registry.
    UnknownProfile,
    /// The profile's requirement template is not the device's kind.
    KindMismatch { profile_kind: DeviceKind },
    /// A registry defect: the encoder fails `Encoder.WF`.
    EncoderInvalid(EncoderFault),
    /// `¬ EFits`: the encoder expects another representation.
    Incompatible(FitFault),
    /// `Encoder.WF ∧ EFits`.  Admissible once the hardware also places.
    Valid,
}

/// The realization judgment of one device binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizationCheck {
    pub profile: Option<OutputProfile>,
    pub status: RealizationStatus,
}

impl RealizationCheck {
    pub fn encoder_well_formed(&self) -> bool {
        matches!(
            self.status,
            RealizationStatus::Valid | RealizationStatus::Incompatible(_)
        )
    }
    pub fn representation_fits(&self) -> bool {
        matches!(self.status, RealizationStatus::Valid)
    }
    pub fn is_valid(&self) -> bool {
        matches!(self.status, RealizationStatus::Valid)
    }
    /// The binding blocks an artefact: a profile is chosen and is not valid.
    pub fn is_blocking(&self) -> bool {
        !matches!(
            self.status,
            RealizationStatus::Valid | RealizationStatus::NotChosen
        )
    }
}

// ---------------------------------------------------------------------------
// Judgments

/// `Encoder.pure`: the term mentions no declaration, no memory, no other
/// domain and constructs nothing.  Structural; the checker separately
/// refuses construction under [`Grant::None`], this names it.
pub fn purity(e: &Expr) -> Result<(), Impurity> {
    match e {
        Expr::Var { .. } | Expr::BoolLit { .. } | Expr::NatLit { .. } | Expr::Prim { .. } => Ok(()),
        Expr::Lam { body, .. } => purity(body),
        Expr::App { f, a } => purity(f).and_then(|()| purity(a)),
        Expr::Rep { e } => purity(e),
        Expr::Fold { f, z, l } => purity(f).and_then(|()| purity(z)).and_then(|()| purity(l)),
        Expr::DeclRef { id } => Err(Impurity::DeclRef { id: *id }),
        Expr::Delay { .. } => Err(Impurity::Delay),
        Expr::Sync { src, .. } => Err(Impurity::Sync { src: *src }),
        Expr::Mk { s, .. } => Err(Impurity::Constructs { concept: *s }),
    }
}

/// `Encoder.WF Θ` in the empty declaration environment under no grant:
/// `rep` and `raw` are sem-free data, `encode` is pure, and it has type
/// `rep → raw`.  Concept representations are irrelevant to a closed
/// sem-free term, so an empty `Θ` suffices.
pub fn well_formed(e: &Encoder) -> Result<(), EncoderFault> {
    for (which, ty) in [("representation", &e.rep), ("raw command", &e.raw)] {
        if !(ty.is_sem_free() && ty.is_data()) {
            return Err(EncoderFault::NotData {
                which: which.into(),
                ty: ty.clone(),
            });
        }
    }
    purity(&e.encode).map_err(EncoderFault::Impure)?;
    let empty = DesignIr::default();
    let found =
        infer(&empty, &Grant::None, &[], &e.encode).map_err(|err| EncoderFault::IllTyped {
            technical: format!("{:?} at {:?}", err.kind, err.path),
        })?;
    let expected = Ty::arr(e.rep.clone(), e.raw.clone());
    if found != expected {
        return Err(EncoderFault::WrongType { expected, found });
    }
    Ok(())
}

/// `EFits Θ accepts E`: at a semantic output the concept's representation
/// is the encoder's domain; at a data output the type itself is.
pub fn fits(
    accepts: &Ty,
    representation_of: impl Fn(SemanticId) -> Option<Ty>,
    e: &Encoder,
) -> Result<(), FitFault> {
    let carried = match accepts {
        Ty::Sem { id } => {
            representation_of(*id).ok_or(FitFault::NoRepresentation { concept: *id })?
        }
        other => other.clone(),
    };
    if carried == e.rep {
        Ok(())
    } else {
        Err(FitFault::Representation {
            carried,
            expected: e.rep.clone(),
        })
    }
}

/// `encoderBody`: the term of the machine sink — `encode (rep d)` at a
/// semantic output, `encode d` at a data output.  Closed except for the
/// driver, so it lives in the driver's (= the output's) domain.
pub fn encoder_body(accepts: &Ty, e: &Encoder, driver: DeclId) -> Expr {
    let arg = match accepts {
        Ty::Sem { .. } => Expr::rep(Expr::decl(driver)),
        _ => Expr::decl(driver),
    };
    Expr::app(e.encode.clone(), arg)
}

/// Judge one device binding against the output it realises.  `accepts` is
/// the output's accepted type when the device is bound to an output the
/// design knows; `None` leaves the fit undecided (reported as not chosen
/// or as the registry's own faults only).
pub fn check_binding(
    binding: &DeviceBinding,
    accepts: Option<&Ty>,
    representation_of: impl Fn(SemanticId) -> Option<Ty>,
) -> RealizationCheck {
    let Some(id) = &binding.realization else {
        return RealizationCheck {
            profile: None,
            status: RealizationStatus::NotChosen,
        };
    };
    let Some(profile) = profile(id) else {
        return RealizationCheck {
            profile: None,
            status: RealizationStatus::UnknownProfile,
        };
    };
    let status = if profile.kind != binding.kind {
        RealizationStatus::KindMismatch {
            profile_kind: profile.kind,
        }
    } else if let Err(fault) = well_formed(&profile.encoder) {
        RealizationStatus::EncoderInvalid(fault)
    } else {
        match accepts {
            Some(accepts) => match fits(accepts, representation_of, &profile.encoder) {
                Ok(()) => RealizationStatus::Valid,
                Err(fault) => RealizationStatus::Incompatible(fault),
            },
            None => RealizationStatus::Valid,
        }
    };
    RealizationCheck {
        profile: Some(profile),
        status,
    }
}

// ---------------------------------------------------------------------------
// Registry

/// The first witnesses.  Raw command types are drawn from the existing
/// data types only (a level, a truth value, a pair); no raw type exists
/// for its own sake.  Ids are stable and persisted in project files.
pub fn profiles() -> Vec<OutputProfile> {
    let level = Ty::q(Dim::ZERO);
    vec![
        OutputProfile {
            id: OutputProfileId("pwm_duty8".into()),
            display_name: "PWM, 8-bit duty".into(),
            description: "A level 0–100 becomes a duty 0–255 on one PWM line.".into(),
            encoder: Encoder {
                rep: level.clone(),
                raw: level.clone(),
                encode: lam_level(scale(var0(), 255.0, 100.0)),
            },
            kind: DeviceKind::PwmChannel,
        },
        OutputProfile {
            id: OutputProfileId("pwm_duty4".into()),
            display_name: "PWM, 4 levels".into(),
            description: "A level 0–100 becomes one of four duties (0, 85, 170, 255) on one PWM line; nearby levels share a duty.".into(),
            encoder: Encoder {
                rep: level.clone(),
                raw: level.clone(),
                encode: lam_level(quantize4(var0())),
            },
            kind: DeviceKind::PwmChannel,
        },
        OutputProfile {
            id: OutputProfileId("i2c_level8".into()),
            display_name: "I2C register, 8-bit".into(),
            description: "A level 0–100 becomes the pair (register 42, value 0–255) written over I2C.".into(),
            encoder: Encoder {
                rep: level.clone(),
                raw: Ty::prod(level.clone(), level.clone()),
                encode: lam_level(Expr::apps(
                    Expr::prim(Prim::Pair {
                        fst: level.clone(),
                        snd: level.clone(),
                    }),
                    [lit(42.0), scale(var0(), 255.0, 100.0)],
                )),
            },
            kind: DeviceKind::I2cSensor,
        },
        OutputProfile {
            id: OutputProfileId("gpio_level".into()),
            display_name: "GPIO, on/off".into(),
            description: "A truth value drives one digital line as written.".into(),
            encoder: Encoder {
                rep: Ty::Bool,
                raw: Ty::Bool,
                encode: Expr::Lam {
                    dom: Ty::Bool,
                    body: Box::new(var0()),
                },
            },
            kind: DeviceKind::DigitalOutput,
        },
        OutputProfile {
            id: OutputProfileId("hbridge_signed".into()),
            display_name: "H-bridge, signed level".into(),
            description: "A signed level −100–100 becomes (forward?, duty 0–255) on an H-bridge channel.".into(),
            encoder: Encoder {
                rep: level.clone(),
                raw: Ty::prod(Ty::Bool, level.clone()),
                encode: lam_level(Expr::apps(
                    Expr::prim(Prim::Pair {
                        fst: Ty::Bool,
                        snd: level.clone(),
                    }),
                    [
                        // forward := ¬ (n < 0)
                        Expr::app(
                            Expr::prim(Prim::Not),
                            Expr::apps(Expr::prim(Prim::Lt { dim: Dim::ZERO }), [var0(), lit(0.0)]),
                        ),
                        // duty := (if n < 0 then 0 − n else n) · 255 / 100
                        scale(
                            Expr::apps(
                                Expr::prim(Prim::Ite { ty: level.clone() }),
                                [
                                    Expr::apps(
                                        Expr::prim(Prim::Lt { dim: Dim::ZERO }),
                                        [var0(), lit(0.0)],
                                    ),
                                    Expr::apps(
                                        Expr::prim(Prim::Sub { dim: Dim::ZERO }),
                                        [lit(0.0), var0()],
                                    ),
                                    var0(),
                                ],
                            ),
                            255.0,
                            100.0,
                        ),
                    ],
                )),
            },
            kind: DeviceKind::HBridgeChannel,
        },
    ]
}

/// One profile by id.
pub fn profile(id: &OutputProfileId) -> Option<OutputProfile> {
    profiles().into_iter().find(|p| &p.id == id)
}

fn var0() -> Expr {
    Expr::Var { index: 0 }
}

fn lit(value: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim: Dim::ZERO,
        value: Scalar(value),
    })
}

fn lam_level(body: Expr) -> Expr {
    Expr::Lam {
        dom: Ty::q(Dim::ZERO),
        body: Box::new(body),
    }
}

/// `n · num / den` on dimensionless levels.
fn scale(n: Expr, num: f64, den: f64) -> Expr {
    Expr::apps(
        Expr::prim(Prim::Div {
            d1: Dim::ZERO,
            d2: Dim::ZERO,
        }),
        [
            Expr::apps(
                Expr::prim(Prim::Mul {
                    d1: Dim::ZERO,
                    d2: Dim::ZERO,
                }),
                [n, lit(num)],
            ),
            lit(den),
        ],
    )
}

/// Four duties by thresholds at 25, 50 and 75: many levels, one duty.
fn quantize4(n: Expr) -> Expr {
    let below = |k: f64| Expr::apps(Expr::prim(Prim::Lt { dim: Dim::ZERO }), [n.clone(), lit(k)]);
    let ite = |c: Expr, t: Expr, e: Expr| {
        Expr::apps(
            Expr::prim(Prim::Ite {
                ty: Ty::q(Dim::ZERO),
            }),
            [c, t, e],
        )
    };
    ite(
        below(25.0),
        lit(0.0),
        ite(
            below(50.0),
            lit(85.0),
            ite(below(75.0), lit(170.0), lit(255.0)),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::{ClockId, DeviceId, OutputId};
    use std::collections::BTreeMap;

    fn level() -> Ty {
        Ty::q(Dim::ZERO)
    }
    fn brightness() -> SemanticId {
        SemanticId::from_raw(1)
    }
    fn theta(s: SemanticId) -> Option<Ty> {
        (s == brightness()).then(level)
    }
    fn binding(profile: &str, kind: DeviceKind) -> DeviceBinding {
        DeviceBinding {
            id: DeviceId::from_raw(7),
            name: "lamp".into(),
            kind,
            output: Some(OutputId::from_raw(4)),
            realization: Some(OutputProfileId(profile.into())),
            fixed_pins: BTreeMap::new(),
        }
    }

    /// `exJ` (FV): `λn. true` claims `q0 → q0`, fits the concept and would
    /// allocate a PWM line — and is not admissible, because it is ill typed.
    fn ill_typed() -> Encoder {
        Encoder {
            rep: level(),
            raw: level(),
            encode: lam_level(Expr::BoolLit { value: true }),
        }
    }

    #[test]
    fn every_registry_profile_is_well_formed_and_ids_are_unique() {
        let all = profiles();
        let ids: std::collections::BTreeSet<_> = all.iter().map(|p| p.id.clone()).collect();
        assert_eq!(ids.len(), all.len());
        for p in &all {
            assert_eq!(well_formed(&p.encoder), Ok(()), "{}", p.id);
            assert!(!p.display_name.is_empty() && !p.description.is_empty());
        }
    }

    #[test]
    fn ill_typed_encoder_fits_but_is_not_well_formed() {
        let e = ill_typed();
        // The two judgments that are not enough on their own.
        assert_eq!(fits(&Ty::sem(brightness()), theta, &e), Ok(()));
        assert!(purity(&e.encode).is_ok());
        // The one that refuses it.
        assert!(matches!(
            well_formed(&e),
            Err(EncoderFault::WrongType { .. })
        ));
    }

    #[test]
    fn wrong_representation_does_not_fit() {
        let gpio = profile(&OutputProfileId("gpio_level".into())).expect("gpio");
        assert_eq!(
            fits(&Ty::sem(brightness()), theta, &gpio.encoder),
            Err(FitFault::Representation {
                carried: level(),
                expected: Ty::Bool
            })
        );
        assert_eq!(
            fits(&Ty::sem(SemanticId::from_raw(9)), theta, &gpio.encoder),
            Err(FitFault::NoRepresentation {
                concept: SemanticId::from_raw(9)
            })
        );
        // A data output fits by its own type.
        assert_eq!(fits(&Ty::Bool, theta, &gpio.encoder), Ok(()));
    }

    #[test]
    fn impure_encoders_are_refused_by_purity_and_by_well_formedness() {
        let d = DeclId::from_raw(3);
        let cases: Vec<(Expr, Impurity)> = vec![
            (lam_level(Expr::decl(d)), Impurity::DeclRef { id: d }),
            (lam_level(Expr::delay(lit(0.0), var0())), Impurity::Delay),
            (
                lam_level(Expr::Sync {
                    src: ClockId::from_raw(2),
                    init: Box::new(lit(0.0)),
                    e: Box::new(var0()),
                }),
                Impurity::Sync {
                    src: ClockId::from_raw(2),
                },
            ),
            (
                lam_level(Expr::mk(brightness(), var0())),
                Impurity::Constructs {
                    concept: brightness(),
                },
            ),
        ];
        for (encode, why) in cases {
            assert_eq!(purity(&encode), Err(why.clone()));
            let e = Encoder {
                rep: level(),
                raw: level(),
                encode,
            };
            assert_eq!(well_formed(&e), Err(EncoderFault::Impure(why)));
        }
    }

    #[test]
    fn construction_is_also_refused_by_the_checker_under_no_grant() {
        let empty = DesignIr::default();
        let mk = lam_level(Expr::mk(brightness(), var0()));
        assert!(infer(&empty, &Grant::None, &[], &mk).is_err());
    }

    #[test]
    fn semantic_types_are_not_raw() {
        let e = Encoder {
            rep: level(),
            raw: Ty::sem(brightness()),
            encode: lam_level(var0()),
        };
        assert!(matches!(
            well_formed(&e),
            Err(EncoderFault::NotData { which, .. }) if which == "raw command"
        ));
        let e = Encoder {
            rep: Ty::arr(level(), level()),
            raw: level(),
            encode: Expr::Lam {
                dom: Ty::arr(level(), level()),
                body: Box::new(lit(0.0)),
            },
        };
        assert!(matches!(
            well_formed(&e),
            Err(EncoderFault::NotData { which, .. }) if which == "representation"
        ));
    }

    #[test]
    fn quantization_is_a_valid_encoder() {
        let p = profile(&OutputProfileId("pwm_duty4".into())).expect("pwm4");
        assert_eq!(well_formed(&p.encoder), Ok(()));
        assert_eq!(fits(&Ty::sem(brightness()), theta, &p.encoder), Ok(()));
    }

    #[test]
    fn encoder_body_observes_the_driver_and_constructs_nothing() {
        let p = profile(&OutputProfileId("pwm_duty8".into())).expect("pwm8");
        let d = DeclId::from_raw(2);
        let body = encoder_body(&Ty::sem(brightness()), &p.encoder, d);
        assert_eq!(
            body,
            Expr::app(p.encoder.encode.clone(), Expr::rep(Expr::decl(d)))
        );
        assert_eq!(body.refs().into_iter().collect::<Vec<_>>(), vec![d]);
        assert_eq!(
            encoder_body(&Ty::Bool, &p.encoder, d),
            Expr::app(p.encoder.encode.clone(), Expr::decl(d))
        );
    }

    #[test]
    fn check_binding_distinguishes_every_status() {
        let accepts = Ty::sem(brightness());
        let not_chosen = DeviceBinding {
            realization: None,
            ..binding("pwm_duty8", DeviceKind::PwmChannel)
        };
        assert_eq!(
            check_binding(&not_chosen, Some(&accepts), theta).status,
            RealizationStatus::NotChosen
        );
        assert_eq!(
            check_binding(
                &binding("no_such", DeviceKind::PwmChannel),
                Some(&accepts),
                theta
            )
            .status,
            RealizationStatus::UnknownProfile
        );
        assert_eq!(
            check_binding(
                &binding("pwm_duty8", DeviceKind::DigitalOutput),
                Some(&accepts),
                theta
            )
            .status,
            RealizationStatus::KindMismatch {
                profile_kind: DeviceKind::PwmChannel
            }
        );
        assert!(matches!(
            check_binding(
                &binding("gpio_level", DeviceKind::DigitalOutput),
                Some(&accepts),
                theta
            )
            .status,
            RealizationStatus::Incompatible(FitFault::Representation { .. })
        ));
        let ok = check_binding(
            &binding("pwm_duty8", DeviceKind::PwmChannel),
            Some(&accepts),
            theta,
        );
        assert_eq!(ok.status, RealizationStatus::Valid);
        assert!(ok.encoder_well_formed() && ok.representation_fits() && !ok.is_blocking());
        // Bound to nothing the design knows: the registry's own faults only.
        assert_eq!(
            check_binding(&binding("pwm_duty8", DeviceKind::PwmChannel), None, theta).status,
            RealizationStatus::Valid
        );
    }
}
