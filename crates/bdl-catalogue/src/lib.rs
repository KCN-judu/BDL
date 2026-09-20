//! The catalogue (`BDL_FV/BDL/Surface/Assignment.lean`, Phase 16): what a
//! deployment may assign to a logical output or a Source.
//!
//! An entry is a **profile with an origin**.  An output profile is a pure
//! encoder `Rep(C) → raw` and the hardware requirement template it needs
//! (Phase 14); an input profile is a pure transducer `raw → Rep(C)` and its
//! requirement template (Phase 13).  The origin — this toolchain's own
//! registry, or a package — is display data: every judgment the compiler
//! makes takes the *profile* (`assign_indistinguishable`,
//! `assignSource_origin_irrelevant`), so a builtin and a packaged profile
//! are told apart by nothing downstream of the contract.  The judgments
//! themselves stay in `bdl-output` (`realization`, `provision`); this crate
//! holds the values and their ids.
//!
//! What a catalogue entry may contribute: a profile, its representation and
//! raw shape, its encoder or transducer, its requirement template.  What it
//! never contributes: a type rule, a primitive, an evaluation rule, a
//! causality or clock judgment — a larger catalogue realizes and provisions
//! more and nothing else (`realizable_mono`, `provisionable_mono`).
//!
//! Today every entry is compiled in ([`Catalogue::builtin`]); a package
//! manager that resolves and validates a package would add entries with
//! `Origin::Package` and change nothing here
//! (docs/architecture/embedded-adapter.md § The package boundary).

#![forbid(unsafe_code)]

use bdl_ir::{Expr, Prim, Scalar, Ty};
use bdl_model::surface::DeviceKind;
use bdl_model::{Dim, InputProfileId, OutputProfileId};
use serde::{Deserialize, Serialize};

/// A package's identity — display data, never read by a judgment.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PackageId(pub String);

/// Where a profile comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "origin", rename_all = "snake_case")]
pub enum Origin {
    /// This toolchain's own registry.
    Builtin,
    /// A package, by id.
    Package { id: PackageId },
}

impl Origin {
    /// The origin as a tool shows it.
    pub fn label(&self) -> String {
        match self {
            Origin::Builtin => "builtin".into(),
            Origin::Package { id } => format!("package {}", id.0),
        }
    }
}

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

/// `Channel`: a closed term from a raw physical reading to a Source's
/// representation (Phase 13's transducer).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transducer {
    pub raw: Ty,
    pub rep: Ty,
    pub transduce: Expr,
}

/// `DeviceProfile` (input side): a transducer plus the hardware it needs.
/// A profile is not a device kind — one Source may be provided by several
/// profiles; the profile prescribes the kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputProfile {
    pub id: InputProfileId,
    pub display_name: String,
    pub description: String,
    pub transducer: Transducer,
    /// The hardware requirement template (`bdl-hardware::devices::needs`).
    pub kind: DeviceKind,
}

/// A catalogue entry that consumes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputEntry {
    pub origin: Origin,
    pub profile: OutputProfile,
}

/// A catalogue entry that provides.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEntry {
    pub origin: Origin,
    pub profile: InputProfile,
}

/// What a deployment may assign: builtin and packaged entries in one list,
/// in a stable order (the compiler iterates it for candidates).
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Catalogue {
    pub inputs: Vec<InputEntry>,
    pub outputs: Vec<OutputEntry>,
}

impl Catalogue {
    /// This toolchain's own entries.
    pub fn builtin() -> Catalogue {
        Catalogue {
            inputs: builtin_inputs()
                .into_iter()
                .map(|profile| InputEntry {
                    origin: Origin::Builtin,
                    profile,
                })
                .collect(),
            outputs: builtin_outputs()
                .into_iter()
                .map(|profile| OutputEntry {
                    origin: Origin::Builtin,
                    profile,
                })
                .collect(),
        }
    }

    pub fn output(&self, id: &OutputProfileId) -> Option<&OutputEntry> {
        self.outputs.iter().find(|e| &e.profile.id == id)
    }

    pub fn input(&self, id: &InputProfileId) -> Option<&InputEntry> {
        self.inputs.iter().find(|e| &e.profile.id == id)
    }

    /// Add an entry (a package's, once resolved and validated); an id
    /// already present is refused so a package cannot shadow a builtin.
    pub fn add_output(&mut self, entry: OutputEntry) -> Result<(), OutputProfileId> {
        if self.output(&entry.profile.id).is_some() {
            return Err(entry.profile.id);
        }
        self.outputs.push(entry);
        Ok(())
    }

    pub fn add_input(&mut self, entry: InputEntry) -> Result<(), InputProfileId> {
        if self.input(&entry.profile.id).is_some() {
            return Err(entry.profile.id);
        }
        self.inputs.push(entry);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// The builtin registry

/// The output witnesses (Phase 14).  Raw command types are drawn from the
/// existing data types only (a level, a truth value, a pair); no raw type
/// exists for its own sake.  Ids are stable and persisted in project files.
pub fn builtin_outputs() -> Vec<OutputProfile> {
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

/// The input witnesses (Phase 13, the first providers): a digital line
/// read as a truth value, in either polarity.  The transducer is the
/// representation's, never a concept's: the Source's concept is
/// constructed by the provision, under the Source's own grant.
pub fn builtin_inputs() -> Vec<InputProfile> {
    vec![
        InputProfile {
            id: InputProfileId("gpio_level_in".into()),
            display_name: "GPIO input, active high".into(),
            description: "A digital line read as a truth value: high is true (the line is pulled down).".into(),
            transducer: Transducer {
                raw: Ty::Bool,
                rep: Ty::Bool,
                transduce: Expr::Lam {
                    dom: Ty::Bool,
                    body: Box::new(var0()),
                },
            },
            kind: DeviceKind::DigitalInput,
        },
        InputProfile {
            id: InputProfileId("gpio_level_in_low".into()),
            display_name: "GPIO input, active low".into(),
            description: "A digital line read as a truth value: low is true (the line is pulled up; a button to ground).".into(),
            transducer: Transducer {
                raw: Ty::Bool,
                rep: Ty::Bool,
                transduce: Expr::Lam {
                    dom: Ty::Bool,
                    body: Box::new(Expr::app(Expr::prim(Prim::Not), var0())),
                },
            },
            kind: DeviceKind::DigitalInput,
        },
    ]
}

// Term helpers, public so a test can build an encoder the way the
// registry does.
pub fn var0() -> Expr {
    Expr::Var { index: 0 }
}

pub fn lit(value: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim: Dim::ZERO,
        value: Scalar(value),
    })
}

pub fn lam_level(body: Expr) -> Expr {
    Expr::Lam {
        dom: Ty::q(Dim::ZERO),
        body: Box::new(body),
    }
}

/// `n · num / den` on dimensionless levels.
pub fn scale(n: Expr, num: f64, den: f64) -> Expr {
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
pub fn quantize4(n: Expr) -> Expr {
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
