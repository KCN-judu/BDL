//! The platform adapter's target-independent half
//! (docs/architecture/embedded-adapter.md).
//!
//! A generated core ends at the raw command boundary: `Tick.commands`
//! carries, per realised output, the value the profile's pure encoder made
//! of the output's value (docs/architecture/output-realization.md).  This
//! crate is what turns such a value into a peripheral operation without
//! deciding anything the behavior or the encoder decided:
//!
//! * [`duty8`] — the one **numeric policy** at the boundary: a raw
//!   `f64` duty command becomes the 8-bit duty a PWM peripheral takes, or
//!   a [`CommandFault`] — explicit, deterministic, and never a Rust `as`
//!   cast deciding BDL semantics (ISS-0006 stays open for the core's own
//!   arithmetic; this policy concerns the boundary only).
//! * [`PwmDuty8`] and [`Level`] — the **sink traits** a target implements
//!   over its peripherals and a host implements over recorders.
//! * [`apply_duty8`] and [`apply_level`] — one command, one sink: a
//!   refused command writes nothing, so the peripheral **holds** its last
//!   applied value; nothing here retries, filters or remembers.
//!
//! * [`schedule::active`] — the compiled activation schedule (a period in
//!   ticks per clock slot, `bdld compile --period`) as the `ActiveDomains`
//!   of one global tick: the same rule the host simulator applies, so the
//!   firmware invents no second clock semantics (ADR-0004).
//!
//! `no_std`, no allocation, one dependency (the core's vocabulary): the
//! generated `adapter` module of a core is written against this crate
//! alone, and the target's HAL binding (`bdl-runtime-embassy-rp`)
//! implements the traits.

#![no_std]
#![forbid(unsafe_code)]

pub mod schedule {
    use bdl_runtime_core::{ActiveDomains, ClockSlot};

    /// Which clock slots are active at global tick `tick` under `periods`
    /// (`periods[slot]` in ticks; `0` never activates): slot `s` is active
    /// when `tick % periods[s] == 0` — `bdl_reactive::Schedule::active_at`
    /// verbatim.  Tick 0 activates every domain.
    pub fn active(tick: u64, periods: &[u64]) -> ActiveDomains {
        let mut a = ActiveDomains::none();
        for (slot, p) in periods.iter().enumerate() {
            if *p > 0 && tick % *p == 0 {
                a = a.with(ClockSlot(slot as u16));
            }
        }
        a
    }
}

/// Why a raw command was not applied.  The sink keeps its last value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandFault {
    /// NaN or ±∞ reached the boundary (the core's arithmetic refuses
    /// non-finite results, so this names a value that was fed, not
    /// computed).
    NotFinite,
    /// A finite value outside the peripheral's range; `min`/`max` are the
    /// bounds of the raw command, not of the peripheral register.
    OutOfRange { min: i32, max: i32 },
}

/// The 8-bit duty policy: a raw command in `0 ..= 255` rounds to the
/// nearest whole duty, halves rounding up (`127.5 → 128`); anything
/// outside that closed range, and any non-finite value, is refused.  No
/// clamping: a design that commands 105 % has said something the profile
/// did not promise to carry, and the peripheral holds instead of guessing.
pub fn duty8(raw: f64) -> Result<u8, CommandFault> {
    if !raw.is_finite() {
        return Err(CommandFault::NotFinite);
    }
    if !(0.0..=255.0).contains(&raw) {
        return Err(CommandFault::OutOfRange { min: 0, max: 255 });
    }
    // `raw` is finite and within [0, 255]: the truncation is exact.
    let whole = raw as u32;
    let fraction = raw - whole as f64;
    let rounded = if fraction >= 0.5 { whole + 1 } else { whole };
    // 255.x with x ≥ 0.5 cannot occur (raw ≤ 255.0), so this never exceeds 255.
    Ok(rounded.min(255) as u8)
}

/// One PWM line whose duty is an 8-bit fraction: `0` is always low,
/// `255` always high, `d` high for `d / 255` of the carrier period.  The
/// carrier itself is the target's configuration, never a command.
pub trait PwmDuty8 {
    fn set_duty8(&mut self, duty: u8);
}

/// One digital line: `true` is high.
pub trait Level {
    fn set_level(&mut self, high: bool);
}

/// One digital line read: `true` is high.  The input half of the adapter
/// (docs/architecture/embedded-adapter.md § The input half): the firmware
/// observes every source once per tick, before the core steps, and the
/// generated `adapter::provide` turns the readings into the core's
/// `Inputs` through each provider's transducer.  A polarity (active low)
/// is the transducer's, never the reader's; a pull is the peripheral's
/// configuration, never a value.
pub trait LevelSource {
    fn level(&mut self) -> bool;
}

/// Read one line; nothing can be refused.
pub fn read_level<S: LevelSource + ?Sized>(source: &mut S) -> bool {
    source.level()
}

/// Apply one raw duty command: converted by [`duty8`], written when
/// accepted, held when refused.  Returns what was written.
pub fn apply_duty8<S: PwmDuty8 + ?Sized>(sink: &mut S, raw: f64) -> Result<u8, CommandFault> {
    let duty = duty8(raw)?;
    sink.set_duty8(duty);
    Ok(duty)
}

/// Apply one truth-valued command; nothing can be refused.
pub fn apply_level<S: Level + ?Sized>(sink: &mut S, high: bool) -> bool {
    sink.set_level(high);
    high
}

/// What a generated `adapter` module records about one machine sink, for
/// tools and tests: stable ids and the board resource it was bound to.
/// Never consulted at runtime to find a peripheral — the binding is
/// generated as code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SinkBinding {
    /// `DeviceId` of the device binding (the `command_<id>` field).
    pub device_id: u64,
    pub symbol: &'static str,
    /// The output realization profile id.
    pub profile: &'static str,
    /// The board resource the solver assigned, as the board names it.
    pub resource: &'static str,
    /// The capability the resource carries for this sink (`pwm`,
    /// `digital_out`).
    pub capability: &'static str,
}

/// What a generated `adapter` module records about one provider, the
/// counterpart of [`SinkBinding`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceBinding {
    /// `DeviceId` of the device binding (the `reading_<id>` parameter).
    pub device_id: u64,
    pub symbol: &'static str,
    /// The input provider profile id.
    pub profile: &'static str,
    /// The board resource the solver assigned, as the board names it.
    pub resource: &'static str,
    /// The capability the resource carries for this source (`digital_in`).
    pub capability: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duty8_rounds_halves_up_and_keeps_the_ends() {
        assert_eq!(duty8(0.0), Ok(0));
        assert_eq!(duty8(-0.0), Ok(0));
        assert_eq!(duty8(255.0), Ok(255));
        assert_eq!(duty8(127.5), Ok(128));
        assert_eq!(duty8(127.499_999), Ok(127));
        assert_eq!(duty8(31.875), Ok(32));
        assert_eq!(duty8(254.5), Ok(255));
        assert_eq!(duty8(0.499), Ok(0));
        assert_eq!(duty8(0.5), Ok(1));
        assert_eq!(duty8(85.0), Ok(85));
        assert_eq!(duty8(170.0), Ok(170));
    }

    #[test]
    fn duty8_refuses_out_of_range_and_non_finite() {
        let range = Err(CommandFault::OutOfRange { min: 0, max: 255 });
        assert_eq!(duty8(-1.0), range);
        assert_eq!(duty8(-0.001), range);
        assert_eq!(duty8(255.001), range);
        assert_eq!(duty8(300.0), range);
        assert_eq!(duty8(f64::NAN), Err(CommandFault::NotFinite));
        assert_eq!(duty8(f64::INFINITY), Err(CommandFault::NotFinite));
        assert_eq!(duty8(f64::NEG_INFINITY), Err(CommandFault::NotFinite));
    }

    struct Pwm(Option<u8>);
    impl PwmDuty8 for Pwm {
        fn set_duty8(&mut self, duty: u8) {
            self.0 = Some(duty);
        }
    }
    struct Pin(bool);
    impl Level for Pin {
        fn set_level(&mut self, high: bool) {
            self.0 = high;
        }
    }

    #[test]
    fn a_refused_command_holds_the_last_applied_duty() {
        let mut pwm = Pwm(None);
        assert_eq!(apply_duty8(&mut pwm, 127.5), Ok(128));
        assert_eq!(pwm.0, Some(128));
        assert_eq!(
            apply_duty8(&mut pwm, 300.0),
            Err(CommandFault::OutOfRange { min: 0, max: 255 })
        );
        assert_eq!(pwm.0, Some(128));
        assert_eq!(
            apply_duty8(&mut pwm, f64::NAN),
            Err(CommandFault::NotFinite)
        );
        assert_eq!(pwm.0, Some(128));
        let mut pin = Pin(false);
        assert!(apply_level(&mut pin, true) && pin.0);
        assert!(!apply_level(&mut pin, false) && !pin.0);
    }

    #[test]
    fn the_schedule_is_the_simulators() {
        use bdl_runtime_core::ClockSlot;
        let periods = [1, 4, 0];
        for tick in 0..12 {
            let a = schedule::active(tick, &periods);
            assert!(a.is_active(ClockSlot(0)));
            assert_eq!(a.is_active(ClockSlot(1)), tick % 4 == 0);
            assert!(!a.is_active(ClockSlot(2)));
        }
        assert!(!schedule::active(5, &[]).any());
    }

    #[test]
    fn the_policy_is_deterministic() {
        for raw in [0.0, 1.25, 64.5, 127.5, 200.999, 255.0] {
            assert_eq!(duty8(raw), duty8(raw));
        }
    }
}
