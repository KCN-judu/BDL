//! The RP2040 half of the platform adapter (docs/architecture/embedded-adapter.md).
//!
//! Implements the sink traits of `bdl-runtime-adapter` over Embassy's
//! RP2040 peripherals, owns the peripheral configuration a raw command
//! never carries (the PWM carrier, the initial line level), the collection
//! arena, and the latched fault halt.  Nothing here reads a design: the
//! generated firmware constructs the sinks it needs, on the resources the
//! solver assigned, and hands each tick's `Commands` to the generated
//! `adapter::apply`.

#![no_std]
// `deny`, not `forbid`: the one `unsafe` block hands the static arena to
// the heap (`arena::init`), the discharge of which is written beside it.
#![deny(unsafe_code)]

use bdl_runtime_adapter::{Level, LevelSource, PwmDuty8};
use embassy_rp::gpio::{Input, Output, Pull};
use embassy_rp::pwm::{ChannelAPin, ChannelBPin, Config, Pwm, Slice};
use embassy_rp::Peri;

/// `top` of an 8-bit duty slice: the counter runs `0 ..= 254` (255 steps),
/// so `compare = d` is high for exactly `d / 255` of the period — `0`
/// always low, `255` always high.
pub const DUTY8_TOP: u16 = 254;

/// The default PWM carrier: the 125 MHz system clock divided by 16 over
/// 255 steps ≈ 30.6 kHz — above hearing, fine for LEDs and motor drivers.
/// Peripheral configuration, never a BDL clock domain
/// (docs/architecture/embedded-adapter.md § Clocks).
pub const DEFAULT_PWM_DIVIDER: u8 = 16;

fn duty8_config(divider: u8) -> Config {
    let mut c = Config::default();
    c.top = DUTY8_TOP;
    c.compare_a = 0;
    c.compare_b = 0;
    c.divider = divider.into();
    c.enable = true;
    c
}

/// One PWM line on the A channel of its slice, driving an 8-bit duty.
/// Configured at construction with the duty `0` (line low) — the startup
/// policy — and the carrier divider.
pub struct PwmA<'d> {
    pwm: Pwm<'d>,
    config: Config,
}

impl<'d> PwmA<'d> {
    pub fn new<T: Slice>(
        slice: Peri<'d, T>,
        pin: Peri<'d, impl ChannelAPin<T>>,
        divider: u8,
    ) -> Self {
        let config = duty8_config(divider);
        let pwm = Pwm::new_output_a(slice, pin, config.clone());
        PwmA { pwm, config }
    }
}

impl PwmDuty8 for PwmA<'_> {
    fn set_duty8(&mut self, duty: u8) {
        self.config.compare_a = u16::from(duty);
        self.pwm.set_config(&self.config);
    }
}

/// One PWM line on the B channel of its slice; see [`PwmA`].
pub struct PwmB<'d> {
    pwm: Pwm<'d>,
    config: Config,
}

impl<'d> PwmB<'d> {
    pub fn new<T: Slice>(
        slice: Peri<'d, T>,
        pin: Peri<'d, impl ChannelBPin<T>>,
        divider: u8,
    ) -> Self {
        let config = duty8_config(divider);
        let pwm = Pwm::new_output_b(slice, pin, config.clone());
        PwmB { pwm, config }
    }
}

impl PwmDuty8 for PwmB<'_> {
    fn set_duty8(&mut self, duty: u8) {
        self.config.compare_b = u16::from(duty);
        self.pwm.set_config(&self.config);
    }
}

/// One digital output line, low at startup.
pub struct Line<'d> {
    out: Output<'d>,
}

impl<'d> Line<'d> {
    pub fn new(pin: Peri<'d, impl embassy_rp::gpio::Pin>) -> Self {
        Line {
            out: Output::new(pin, embassy_rp::gpio::Level::Low),
        }
    }
}

impl Level for Line<'_> {
    fn set_level(&mut self, high: bool) {
        self.out.set_level(if high {
            embassy_rp::gpio::Level::High
        } else {
            embassy_rp::gpio::Level::Low
        });
    }
}

/// One digital input line with its pull configured at construction: the
/// pull is peripheral configuration a raw reading never carries (a button
/// to ground reads through a pull-up; its polarity is the provider
/// profile's transducer, not this line's).
pub struct Sense<'d> {
    inp: Input<'d>,
}

impl<'d> Sense<'d> {
    pub fn pull_down(pin: Peri<'d, impl embassy_rp::gpio::Pin>) -> Self {
        Sense {
            inp: Input::new(pin, Pull::Down),
        }
    }
    pub fn pull_up(pin: Peri<'d, impl embassy_rp::gpio::Pin>) -> Self {
        Sense {
            inp: Input::new(pin, Pull::Up),
        }
    }
}

impl LevelSource for Sense<'_> {
    fn level(&mut self) -> bool {
        self.inp.is_high()
    }
}

/// The latched fault: a tick failed (`RuntimeError`), so the core's state
/// did not advance and nothing is applied again.  Every line holds the
/// last value it was given; the firmware stops ticking and waits for
/// interrupts forever — a reset is the only way out, by design
/// (docs/architecture/embedded-adapter.md § Faults).
pub fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

/// The collection arena: a `#[global_allocator]` the generated firmware
/// declares over a static byte array sized from the manifest's bounds
/// (`state_bytes_max + tick_bytes_max`, rounded up; docs/spec/deployment-capacity.md § 6).
/// Exhaustion is a fault (`handle_alloc_error` → the panic handler →
/// halt), never a dropped value; for a `bounded` design it cannot occur.
#[cfg(feature = "collections")]
pub mod arena {
    pub use embedded_alloc::LlffHeap as Heap;

    /// Hand the arena to the heap once, before the first tick.  `arena`
    /// must be a `'static` mutable byte array the firmware owns; the heap
    /// takes it whole.
    pub fn init(heap: &Heap, arena: &'static mut [u8]) {
        let start = arena.as_mut_ptr() as usize;
        let size = arena.len();
        // SAFETY is the heap's contract: the region is exclusively ours and
        // lives forever; `embedded-alloc` documents `init` as unsafe for
        // exactly those two reasons, both discharged by `&'static mut`.
        #[allow(unsafe_code)]
        unsafe {
            heap.init(start, size)
        }
    }
}
