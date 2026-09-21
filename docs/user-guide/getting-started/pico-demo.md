# Your first board: Button → Lamp on a Raspberry Pi Pico

**Goal.** Make a push button control a lamp — and run it on a real board. You
will open a small product, tell Studio which pins the button and the lamp are
on, build the firmware, put it on the board, and press the button.

**Time.** Fifteen minutes, plus the first build's download of the board's
libraries.

**Needs.** A Raspberry Pi Pico (the RP2040 board), a USB cable that carries
data, one push button, two jumper wires. No LED: the demo drives the Pico's own
on-board LED. Nothing to install beyond Studio and the Rust toolchain the
installation page asks for ([Install and launch](install-and-launch.md)).

## 1. Wire the button

The button goes between **GP2** and **GND**: one leg to GP2, the other to any
GND pin. Nothing else. The Pico pulls the line up on its own, so the line reads
low while the button is held — the demo says as much when you look at the
device.

| Pico pin | Wire                             |
| -------- | -------------------------------- |
| GP2      | one leg of the button            |
| GND      | the other leg                    |
| GP25     | nothing — it is the on-board LED |

## 2. Open the demo

On the Welcome page, under **Demos**, choose **Button → Lamp, wired**. Studio
asks where to put the project (it is an ordinary project: a folder with
`src/main.bdl` in it) and opens it on the Design page.

![The canvas arranged left to right: the Source node pressed with its entry arrow and the word Source at the left, the concept row Pressed and the relationship node lit with the formula pressed in the middle, the concept row Lit and the lamp sink at the right, joined by links; each carries the domain name main.](../assets/getting-started/pico-design.png)

_The demo's design: the Source pressed, the value lit that follows it, and the
lamp it drives._

Three objects. **pressed** is a _Source_: a value the environment provides — the
design does not say how, and the canvas shows no pin. **lit** is a relationship
whose formula is simply `pressed`. **lamp** is the physical output, driven by
`lit`. Nothing here is about a board; that is the next page.

The other demo, **Button → Lamp**, is the same design without its deployment:
take it when you want to make the device choices yourself (step 3 explains
them); the wired one has them made.

## 3. Deploy

Click **Deploy** (⌘3) and, in **Target**, choose **Raspberry Pi Pico (RP2040)**.

![The Deploy page: the Target pop-up showing Raspberry Pi Pico (RP2040); the green verdict Feasible on Raspberry Pi Pico (RP2040); two device cards — button, a Digital input for pressed — Source with the provider GPIO input, active low and pin GP2, and led, a Digital output for lamp with the realization GPIO, on/off and pin GP25 — each with its judgments checked; the placement table with button and led on GP2 and GP25; and at the bottom the Firmware section with the steps Deployment done, Build current, Flash and Observe to come, a green dot with Ready to build for Raspberry Pi Pico (RP2040), and one primary button, Build for Raspberry Pi Pico (RP2040).](../assets/getting-started/pico-firmware-ready.png)

_The wired demo on the Deploy page: the Pico chosen, both devices admissible and
placed, and the one thing left to do — Build._

Read the page top to bottom:

- **Feasible on Raspberry Pi Pico (RP2040).** — the devices fit the board.
- **Devices.** A _device_ is the piece of hardware that stands for one thing in
  the design. `button` is a **Digital input** for `pressed — Source`, with the
  **Provider** _GPIO input, active low_ (low is true: a button to ground) on pin
  **GP2**. `led` is a **Digital output** for `lamp`, with the **Realization**
  _GPIO, on/off_ on pin **GP25**. The ticks under each — for a Source
  _transducer · fits · placed · readable_, for an output _encoder · fits ·
  placed_ — are the compiler's judgments; every one holds.
- **Placement.** Which pin carries what.
- **Firmware.** Where you are on the way to a running board — _✓ Deployment ·
  **Build** · Flash · Observe_ — and the one thing to do next.

With the guided demo the page instead says **Not ready to build** and names the
first missing thing — _pressed has no device on Raspberry Pi Pico._ — and you
make the two devices: **Add device**, name it, choose its kind, choose what it
is for in the third pop-up (`pressed — Source` or `lamp`), choose the Provider
or Realization, type the pin. The sentence changes as you go, until it reads
_Ready to build_.

## 4. Build

Click **Build for Raspberry Pi Pico (RP2040)**. The card shows what is happening
— _Checking the deployment_, _Generating the crate_, _Preparing the toolchain_,
_Compiling_ with the count of pieces compiled, _Writing the image_ — and ends
with **Firmware built at _time_** and the image's size. The first build fetches
the board's libraries and takes a few minutes; later ones take seconds. **Stop**
ends a build early.

If it ends with **The build did not complete**, the card says which step and why
— _The Rust target for Raspberry Pi Pico is not installed._ with the one command
to run, for instance — and **Details** holds the compiler's own words. See
[Troubleshooting: deployment](../troubleshooting/deployment-errors.md).

## 5. Flash

Under the built firmware the card says **No board is reachable.** and how to
make one reachable: **hold the BOOTSEL button on the Pico while plugging it in**
over USB. The Pico appears to the computer as a small drive named `RPI-RP2`;
click **Look again** and the card names it — _Raspberry Pi Pico in BOOTSEL mode
(RPI-RP2)_ — with a **Flash** button.

![The Firmware card after a build: a green dot and Firmware built at a time with the image's size in bytes, a Build again link; below, an orange dot with No board is reachable and the line Hold BOOTSEL while plugging the board in over USB; it appears as a drive named RPI-RP2, and a Look again link.](../assets/getting-started/pico-firmware-built.png)

_Built: the image is current, no board is plugged in yet, and the page says what
to do._

Click **Flash**. The image is copied onto the drive; the Pico restarts into it
and the drive disappears. The card says **Flashed to Raspberry Pi Pico in
BOOTSEL mode (RPI-RP2) at _time_** and then:

> Try it: act on pressed; lamp should follow the design.

If two Picos are plugged in, the card asks which one; it never picks.

## 6. Try it

Press the button. The on-board LED lights while the button is held and goes out
when you let go: `lit` is `pressed`, and `lamp` is `lit`.

That the flash succeeded is all Studio can tell you; whether the board behaves
is yours to see. If the LED does not follow the button, check the wiring first
(GP2 and GND, a button that actually closes), then
[Troubleshooting: deployment](../troubleshooting/deployment-errors.md).

## 7. Change it, and again

Change the design — make `lit` be `!pressed`, say, so the lamp is lit until you
press — or the deployment — the button on GP3. The Firmware card says at once:
**The firmware is from an earlier design or deployment.** and, if you had
flashed, _The board runs an earlier design — build and flash again to update
it._ Only **Build again** is offered; Flash comes back once the image is
current. Studio never lets an old image pass for the design on screen.

## What you did, in the design's terms

The design never learned about GP2 or GP25. The **Source** is what the
environment provides; the **provider** — a catalogue profile, chosen on the
Deploy page — is how a device reads it for the board, and the **realization** is
how a device carries the output's value out. Change the board and the design
stays; change the provider and the design stays. That separation is the point of
the two pages ([Deploy](../studio/deploy.md)).

## Without Studio

The same path from a terminal, for scripts and for checking a build without the
app ([Command line](../reference/cli.md)):

```bash
bdld init my-lamp --template button-lamp-configured
bdld build my-lamp --target rp2040_pico
bdld flash my-lamp --target rp2040_pico
```

## Next

[From sensor to output](../workflows/sensor-to-output.md) extends a design with
a second Source; [Deploy](../studio/deploy.md) has every state of the Firmware
card.
