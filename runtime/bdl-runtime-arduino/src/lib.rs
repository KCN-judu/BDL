//! The Arduino half of the platform adapter (docs/architecture/embedded-adapter.md § Arduino).
//!
//! Implements the sink traits of `bdl-runtime-adapter` over `avr-hal`'s
//! pins — a PWM pin on one of the ATmega's timers, an output line — and
//! owns what a raw command never carries: the initial level, the blocking
//! tick wait and the latched fault halt.  Board-neutral: `arduino-hal`'s
//! board feature (`arduino-nano`, …) decides which pins exist; the
//! generated firmware constructs the sinks it needs on the lines the
//! placement assigned and hands each tick's `Commands` to the generated
//! `adapter::apply`.
//!
//! No Embassy here: `avr-hal` is synchronous and the AVR has no Embassy
//! time driver.  The tick is [`tick_wait`] after each step — the same one
//! global `step` as every target (ADR-0004), on a coarser timer.

#![no_std]
#![forbid(unsafe_code)]

use arduino_hal::port::mode::{Output, PwmOutput};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::PwmPinOps;
use bdl_runtime_adapter::{Level, PwmDuty8};

/// One PWM pin on its timer, driving an 8-bit duty: `avr-hal`'s PWM is
/// 8-bit natively (`set_duty(u8)`), so the boundary's `duty8` value is
/// the register value — `0` always low, `255` always high.  Enabled at
/// construction with the duty `0` (line low): the startup policy.
pub struct PwmLine<TC, PIN: PwmPinOps<TC>> {
    pin: Pin<PwmOutput<TC>, PIN>,
}

impl<TC, PIN: PwmPinOps<TC>> PwmLine<TC, PIN> {
    pub fn new(mut pin: Pin<PwmOutput<TC>, PIN>) -> Self {
        pin.set_duty(0);
        pin.enable();
        PwmLine { pin }
    }
}

impl<TC, PIN: PwmPinOps<TC>> PwmDuty8 for PwmLine<TC, PIN> {
    fn set_duty8(&mut self, duty: u8) {
        self.pin.set_duty(duty);
    }
}

/// One digital output line, low at startup.
pub struct Line<PIN: arduino_hal::port::PinOps> {
    pin: Pin<Output, PIN>,
}

impl<PIN: arduino_hal::port::PinOps> Line<PIN> {
    pub fn new(mut pin: Pin<Output, PIN>) -> Self {
        pin.set_low();
        Line { pin }
    }
}

impl<PIN: arduino_hal::port::PinOps> Level for Line<PIN> {
    fn set_level(&mut self, high: bool) {
        if high {
            self.pin.set_high();
        } else {
            self.pin.set_low();
        }
    }
}

/// The base tick: a blocking wait of `micros` after each step.  The wait
/// does not account for the step's own duration, so a tick lasts at least
/// `micros` — the coarse timer of this family, recorded as its policy
/// (ISS-0017 for a timer-driven tick).
pub fn tick_wait(micros: u64) {
    let clamped = u32::try_from(micros).unwrap_or(u32::MAX);
    arduino_hal::delay_us(clamped);
}

/// The latched fault: a tick failed (`RuntimeError`), so the core's state
/// did not advance and nothing is applied again.  Every line holds the
/// last value it was given; the MCU sleeps until reset
/// (docs/architecture/embedded-adapter.md § Faults).
pub fn halt() -> ! {
    loop {
        avr_device::asm::sleep();
    }
}
