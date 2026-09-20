---
kind: evidence
area: deployment
status: current
---

# Physical smoke test: Button → Lamp on a Raspberry Pi Pico

**Status: not yet run.** The build and the flash are tested end to end against a
stand-in toolchain and a pretend bootloader volume, the real cross-build
produces a UF2 whose first block is the boot block, and the converter is checked
page by page — but no change so far had a Pico on the desk. This page is the
checklist for the first person who does; its result goes in the table at the
end, and until a row says _passed_ nothing in the records claims the board
behaves.

## What is being checked

That the firmware `bdld build` writes and `bdld flash` (or Studio's Flash) puts
on the board runs the demo's design: the on-board LED lit exactly while the
button is held. Two provider polarities and one deployment change are exercised
so that a wrong pull or an inverted line would show.

## Hardware

- One Raspberry Pi Pico (RP2040; a Pico W is fine, its LED is not on GP25 — use
  an external LED on GP25 then, see the variant below).
- One momentary push button.
- Two jumper wires; a USB cable that carries data.
- A computer with Studio built from this repository (`just studio`) and Rust
  installed (`rustup`; the target `thumbv6m-none-eabi` installs itself on the
  first build).

## Wiring

| Pico pin | Wire                                                                   |
| -------- | ---------------------------------------------------------------------- |
| GP2      | one leg of the button                                                  |
| GND      | the other leg (any GND pin)                                            |
| GP25     | nothing — the on-board LED; an external LED + 330 Ω to GND on a Pico W |

No pull-up resistor: the provider `gpio_level_in_low` configures the internal
pull-up, and a pressed button reads low.

## Steps

Tick each; write what you saw where it differs.

1. **Build.** Welcome → _Demos_ → _Button → Lamp, wired_ → a folder → Deploy →
   Target _Raspberry Pi Pico (RP2040)_ → _Build for Raspberry Pi Pico (RP2040)_.
   - [ ] The stages run through _Checking_, _Generating_, _Preparing_,
         _Compiling_ (crates counted), _Writing the image_.
   - [ ] _Firmware built at hh:mm_ with a size of a few thousand bytes.
   - [ ] `build/rp2040_pico/<package>-rp2040.uf2` exists in the project folder.
   - Time of the first build: ____ (expected: minutes, the HAL is fetched).
2. **Reach the board.** Hold BOOTSEL, plug the Pico in, release.
   - [ ] A drive `RPI-RP2` mounts.
   - [ ] _Look again_ → _Raspberry Pi Pico in BOOTSEL mode (RPI-RP2)_ with its
         path, and a _Flash_ button.
3. **Flash.**
   - [ ] _Flashing…_ → _Writing the image_ → _Restarting the board_.
   - [ ] The `RPI-RP2` drive disappears by itself within ten seconds.
   - [ ] _Flashed to … at hh:mm_ and _The board restarted into the new
         firmware._ (not the _did not leave bootloader mode_ sentence).
   - [ ] _Try it: act on pressed; lamp should follow the design._
4. **The button.**
   - [ ] With the button released the LED is **off**.
   - [ ] While the button is held the LED is **on**; it goes off on release.
   - [ ] No flicker at rest; no visible lag (the tick is 10 ms).
   - [ ] Holding for ten seconds keeps it on (no watchdog, no halt).
5. **The other polarity.** On the Deploy page change `button`'s Provider to
   _GPIO input, active high_ (pulled down; wire the button between GP2 and
   **3V3** instead). The card says the firmware is stale.
   - [ ] _Build again_ → _Flash_ (BOOTSEL again) → the LED follows the button as
         before.
6. **A deployment change.** Move `led` to another pad: an external LED + 330 Ω
   between GP15 and GND; pin `GP15`.
   - [ ] Stale → Build again → Flash → the external LED follows the button; the
         on-board LED stays off.
7. **A design change.** On the Design page make `lit`'s formula `!pressed`.
   - [ ] Stale, _The board runs an earlier design_ → Build again → Flash → the
         LED is on at rest and off while the button is held.
8. **Power cycle.** Unplug and replug without BOOTSEL.
   - [ ] The firmware runs from flash: the LED follows the button at once.

## What a failure means

| Seen                                   | Likely cause, where to look                                                                                                                         |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| the drive never mounts                 | a power-only cable; BOOTSEL released too early                                                                                                      |
| the drive stays mounted after _Flash_  | the UF2 was rejected (a wrong family id or a page outside flash): `firmware/uf2.rs`, compare with `elf2uf2` on the same ELF                         |
| the LED never lights                   | the wrong pad in `targets/rp2040.rs` (GP25 ↔ the LED), `gpio_level` not applied, the tick loop halted at the first tick (`adapter.rs`, `rp2040.rs`) |
| the LED is inverted (on at rest)       | the provider's polarity (`gpio_level_in_low` reads low as true; the pull-up) — `bdl-catalogue`, `Sense` in `bdl-runtime-embassy-rp`                 |
| the LED flickers at rest               | the pull not configured (a floating line): the provider's `SourceKind` → `Pull` in the firmware module                                              |
| the LED is on only while held but lags | the tick period, or a debounce the design does not have (none is expected)                                                                          |
| step 8 fails, steps 1–7 pass           | the boot block / flash layout (`memory.x`, boot2) — the UF2's first page                                                                            |

## Record

| Date | Tester | Pico | Steps passed | Notes |
| ---- | ------ | ---- | ------------ | ----- |
|      |        |      |              |       |
