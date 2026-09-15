//! The finite hardware model.

use bdl_model::DeviceId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A shared vocabulary between board descriptions and device descriptions.
/// The solver treats it as an opaque decidable type.  Semantic hardware
/// capabilities, never HAL types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    DigitalIn,
    DigitalOut,
    Pwm,
    AnalogIn,
    Interrupt,
    I2cSda,
    I2cScl,
    SpiMosi,
    SpiMiso,
    SpiSck,
    SpiSs,
    UartTx,
    UartRx,
}

impl Capability {
    /// Designer-facing wording.
    pub fn label(self) -> &'static str {
        match self {
            Capability::DigitalIn => "digital in",
            Capability::DigitalOut => "digital out",
            Capability::Pwm => "PWM",
            Capability::AnalogIn => "analog in",
            Capability::Interrupt => "interrupt",
            Capability::I2cSda => "I²C SDA",
            Capability::I2cScl => "I²C SCL",
            Capability::SpiMosi => "SPI MOSI",
            Capability::SpiMiso => "SPI MISO",
            Capability::SpiSck => "SPI SCK",
            Capability::SpiSs => "SPI SS",
            Capability::UartTx => "UART TX",
            Capability::UartRx => "UART RX",
        }
    }
    /// What the unit behind this capability is called on a board.
    pub fn unit_noun(self) -> &'static str {
        match self {
            Capability::Pwm => "timer",
            Capability::Interrupt => "INT",
            Capability::I2cSda | Capability::I2cScl => "I²C unit",
            Capability::SpiMosi | Capability::SpiMiso | Capability::SpiSck | Capability::SpiSs => {
                "SPI unit"
            }
            Capability::UartTx | Capability::UartRx => "UART unit",
            Capability::DigitalIn | Capability::DigitalOut | Capability::AnalogIn => "unit",
        }
    }
    /// The stable snake_case name (the serde form), for wire messages and
    /// board files.
    pub fn as_str(self) -> &'static str {
        match self {
            Capability::DigitalIn => "digital_in",
            Capability::DigitalOut => "digital_out",
            Capability::Pwm => "pwm",
            Capability::AnalogIn => "analog_in",
            Capability::Interrupt => "interrupt",
            Capability::I2cSda => "i2c_sda",
            Capability::I2cScl => "i2c_scl",
            Capability::SpiMosi => "spi_mosi",
            Capability::SpiMiso => "spi_miso",
            Capability::SpiSck => "spi_sck",
            Capability::SpiSs => "spi_ss",
            Capability::UartTx => "uart_tx",
            Capability::UartRx => "uart_rx",
        }
    }
}

/// A board resource, named as the board names it (`D3`, `A4`, `GP15`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new(s: impl Into<String>) -> ResourceId {
        ResourceId(s.into())
    }
}

/// The backing unit of a capability on a resource: a timer for PWM, a
/// peripheral controller for a bus line.  Two resources may share a unit;
/// one resource's capabilities may be backed by different units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub id: ResourceId,
    pub capabilities: BTreeSet<Capability>,
    /// Per-capability backing unit.  `Resource.unitOf`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub units: BTreeMap<Capability, UnitId>,
}

impl Resource {
    pub fn unit_of(&self, cap: Capability) -> Option<UnitId> {
        self.units.get(&cap).copied()
    }
}

/// A target: its resources and which capabilities several requirements may
/// share on one resource (bus lines); everything else is exclusive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hardware {
    /// Stable target id (`arduino_nano`); never shown as-is.
    pub name: String,
    /// Descriptive metadata for a target chooser; never consulted by the
    /// solver.
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    /// Board/platform family (`avr`, `rp2040`, …).
    #[serde(default)]
    pub family: String,
    pub resources: Vec<Resource>,
    #[serde(default)]
    pub shareable: BTreeSet<Capability>,
}

/// How many resources of a target offer one capability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySummary {
    pub capability: Capability,
    pub resource_count: usize,
    pub shareable: bool,
}

impl Hardware {
    /// The name to show designers: `display_name`, else the id.
    pub fn display(&self) -> &str {
        if self.display_name.is_empty() {
            &self.name
        } else {
            &self.display_name
        }
    }
    /// Per capability, how many resources offer it; in `Capability` order,
    /// absent capabilities omitted.
    pub fn capability_summary(&self) -> Vec<CapabilitySummary> {
        let mut counts: BTreeMap<Capability, usize> = BTreeMap::new();
        for r in &self.resources {
            for c in &r.capabilities {
                *counts.entry(*c).or_default() += 1;
            }
        }
        counts
            .into_iter()
            .map(|(capability, resource_count)| CapabilitySummary {
                capability,
                resource_count,
                shareable: self.is_shareable(capability),
            })
            .collect()
    }
    /// A designer-readable description of one resource: its id and what
    /// it can do, with the unit behind each unit-bearing capability
    /// (`D3: PWM (timer 2), interrupt (unit 1), digital in, digital out`).
    pub fn describe_resource(&self, r: &ResourceId) -> Option<String> {
        let res = self.find(r)?;
        let parts: Vec<String> = res
            .capabilities
            .iter()
            .map(|c| match res.units.get(c) {
                Some(u) => format!("{} ({} {})", c.label(), c.unit_noun(), u.0),
                None => c.label().to_string(),
            })
            .collect();
        Some(format!("{}: {}", r.0, parts.join(", ")))
    }
    pub fn find(&self, r: &ResourceId) -> Option<&Resource> {
        self.resources.iter().find(|x| &x.id == r)
    }
    /// `Supports H r c`.
    pub fn supports(&self, r: &ResourceId, cap: Capability) -> bool {
        self.find(r).is_some_and(|x| x.capabilities.contains(&cap))
    }
    /// `Hardware.unitOf`.
    pub fn unit_of(&self, r: &ResourceId, cap: Capability) -> Option<UnitId> {
        self.find(r).and_then(|x| x.unit_of(cap))
    }
    pub fn is_shareable(&self, cap: Capability) -> bool {
        self.shareable.contains(&cap)
    }
    /// Resource ids in the board's declared order.
    pub fn resource_ids(&self) -> impl Iterator<Item = &ResourceId> {
        self.resources.iter().map(|r| &r.id)
    }
}

/// Stable identity of a requirement: the device binding it comes from and
/// its index within that device's requirement list.  Independent of solver
/// traversal; stable under unrelated edits.
///
/// Serialized as the string `"<device raw id>/<index>"` so it can key a JSON
/// object (an `Assignment`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequirementId {
    pub device: DeviceId,
    pub index: u16,
}

impl std::fmt::Display for RequirementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.device.raw(), self.index)
    }
}

impl std::str::FromStr for RequirementId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        let (d, i) = s
            .split_once('/')
            .ok_or_else(|| format!("requirement id `{s}`: expected `<device>/<index>`"))?;
        let device = d
            .parse::<u64>()
            .map_err(|e| format!("requirement id `{s}`: {e}"))?;
        let index = i
            .parse::<u16>()
            .map_err(|e| format!("requirement id `{s}`: {e}"))?;
        Ok(RequirementId {
            device: DeviceId::from_raw(device),
            index,
        })
    }
}

impl Serialize for RequirementId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for RequirementId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A unit-relation group: requirements in the same group must be backed by
/// the same unit (`Same`) or by pairwise different units (`Distinct`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GroupId {
    pub device: DeviceId,
    pub index: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitRel {
    Same,
    Distinct,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub id: RequirementId,
    pub capability: Capability,
    /// A manual pin choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed: Option<ResourceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<(GroupId, UnitRel)>,
    /// What this requirement is for, in the designer's words ("M1 PWM").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
}
