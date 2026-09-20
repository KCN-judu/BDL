---
kind: project
area: studio
status: current
---

# The first external UX study: the Pico demo

The protocol for the first test of Behavior Designer with a person who has not
read this repository. One task, one board, measured; the findings are routed
into issues, and the roadmap's order is reconsidered on them (`roadmap.md`).

## Question

Can a designer who knows what a button and a lamp are, and has never seen BDL,
get from an opened design to a board that runs it — understanding the difference
between the design and its deployment on the way — without a terminal, Rust,
Cargo, a board file or textual BDL?

## Setup

- A machine with Studio built (`just studio`) and Rust installed; the target and
  the HAL fetched once beforehand so the first build is seconds, not minutes
  (run the smoke test's step 1 on that machine first —
  `docs/evidence/pico-smoke-test.md` — and delete the project).
- A Raspberry Pi Pico, a button, two jumpers, the USB cable; the wiring **not**
  made — a printed copy of the wiring table from the guide's
  [Your first board](../user-guide/getting-started/pico-demo.md) is on the desk,
  nothing else from the guide.
- Studio open on the Welcome page. The observer sits beside, screen recorder on,
  a stopwatch, the sheet below.

Which demo the tester starts from is the study's first choice. The **guided**
demo (_Button → Lamp_) tests deployment understanding — the tester has to make
the two devices, choose what each is for, choose a provider and a realization,
place them; the **wired** demo tests the build-and-flash path alone. The first
session is the guided one; a second tester takes the wired one if the first
never reached a build.

## Task, as given to the tester

> This design is a lamp that follows a push button. Make the button control the
> lamp and run it on this Pico. The button is on the desk; the wiring sheet says
> which pins.

No further coaching. The observer answers only "what does this word mean?"
questions about English, not about BDL, and notes each one. When the tester is
stuck for three minutes the observer gives the smallest hint that unblocks
(recorded as a hint, and what it was).

## What the observer records

For each of the nine sub-goals, the time reached, whether it was reached without
a hint, and the wrong actions on the way:

| #   | Sub-goal                                                                                            | Reached at | Hints | Wrong actions |
| --- | --------------------------------------------------------------------------------------------------- | ---------- | ----- | ------------- |
| 1   | identifies `pressed` as the Source (says it is "the button", "the input", or reads the header word) |            |       |               |
| 2   | identifies `lit` as the relationship between them                                                   |            |       |               |
| 3   | identifies `lamp` as the output                                                                     |            |       |               |
| 4   | opens Deploy and chooses the Pico                                                                   |            |       |               |
| 5   | makes a device for the button and chooses _GPIO input, active low_ (guided only)                    |            |       |               |
| 6   | makes a device for the lamp and chooses _GPIO, on/off_ (guided only)                                |            |       |               |
| 7   | reads the placement (says which pin the button is on)                                               |            |       |               |
| 8   | builds                                                                                              |            |       |               |
| 9   | flashes (BOOTSEL) and presses the button                                                            |            |       |               |

And, across the session:

| Metric                                                                                                                                                    | Value                                  |
| --------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
| task completed (LED follows the button)                                                                                                                   | yes / no / partial (to which sub-goal) |
| time to completion                                                                                                                                        | mm:ss                                  |
| wrong actions (an action that had to be undone or led nowhere)                                                                                            | count, listed                          |
| blocked states (the page said _Not ready_ / a failure and the tester had to act)                                                                          | count, which                           |
| hints given                                                                                                                                               | count, which                           |
| help sought (asked the observer; looked for documentation)                                                                                                | count                                  |
| misunderstood Design vs Deploy (tried to set a pin on the Design page; looked for the button on the canvas; expected the design to change with the board) | count, quotes                          |
| misunderstood Source vs device (called the device "the button" in the design; expected the Source to be the pin; asked why the canvas shows no pin)       | count, quotes                          |
| misunderstood concept vs relationship (edited `Pressed` when meaning `pressed`; asked what `Lit` is for)                                                  | count, quotes                          |
| recovery from a diagnostic (a blocker or a failure read, acted on, cleared without a hint)                                                                | count, which                           |
| the status line vs the Deploy verdict (looked at one, believed the other)                                                                                 | notes                                  |

Subjective liking is not recorded. A closing question, verbatim: "In your words,
what did the Deploy page add to the design?"

## The audit questions, answered afterwards

From the observer's notes, one paragraph each (`studio-ui.md` § 15 lists what
the design assumed):

- Is the path from Design to Deploy discoverable?
- Does the tester understand why the button's pin is absent from Design?
- Is provider selection understandable? Is _active low_ read correctly?
- Is realization selection symmetric enough — the same row read the same way —
  without the tester thinking a Source and an output are the same?
- Is the deployment verdict actionable?
- Is Build readiness obvious — did the tester know when Build would be offered,
  and why not before?
- Is Flash discoverable — BOOTSEL, _Look again_?
- Can the tester recover from an incomplete assignment?
- Does the status line conflict with Deploy's verdict?

## After the session

Each misunderstanding or unrecovered block becomes an issue in `docs/issues/`
with the quote and the sub-goal; the metrics table is appended to this page
under _Sessions_; `roadmap.md` is reconsidered — if the first-run workflow
blocked the tester before the board, it stays first; if the tester reached the
board and the blocks were the adapter's breadth (a sensor the catalogue lacks),
the adapter moves back up.

## Sessions

| Date | Tester (role) | Demo | Completed | Time | Wrong actions | Hints | Findings |
| ---- | ------------- | ---- | --------- | ---- | ------------- | ----- | -------- |
|      |               |      |           |      |               |       |          |
