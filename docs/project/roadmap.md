---
kind: project
area: process
status: current
---

# Roadmap

What is planned, in priority order, and what each item waits on. This page is
never evidence that something exists: what is implemented is in
[status.md](status.md); what landed and when is in
[changes/history/milestones.md](../changes/history/milestones.md) and
[changes/unreleased/](../changes/unreleased). A row is deleted when it lands;
nothing here has a status column.

## Priorities

| #   | Outcome                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Waits on                                                                                                                                     | Records                                          |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| 1   | The first external UX study of the Pico demo (`docs/project/ux-study-pico.md`): one tester, the task _make the button control the lamp and run it on this Pico_, the metrics recorded, the findings routed into issues — and, first, the physical smoke test (`docs/evidence/pico-smoke-test.md`) on a real Pico, which no change so far has had                                                                                                                                                                                                                                                                                            | a Raspberry Pi Pico, a button, a tester who has not read the repository                                                                      | ADR-0039                                         |
| 2   | The platform adapter beyond one line and one duty: the provider occurrence contract (FV Phase 17: a batch with transport identities, deduplication, an overflow flag — first for a provider whose deliveries carry an identity), analog and bus providers and an Arduino reader (ISS-0018), I²C and H-bridge sinks and the output device clock (FV Phase 15 / 17 `lowerSync` / `lowerWindow`, ISS-0017), the most the adapter supplies per list input; the core's numeric representation on device (ISS-0006). Both halves exist in their first slice — outputs since ADR-0037, Sources since ADR-0038 (`Button → rule → Lamp` on the Pico) | a first identity-carrying provider (a UART frame, a CAN sequence); an FV phase for stateful transducers and a Source device clock (FVI-0020) | ADR-0037, ADR-0038, ADR-0004, ADR-0016, ADR-0027 |
| 3   | Telemetry back into Studio and the Monitor page (today a placeholder)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | a stable identity manifest (`bdl-manifest.json`); the flash path (ADR-0039) for the return channel                                           | ADR-0039                                         |
| 4   | Packaging: the runtime crates and the toolchain with an installed Studio (today a checkout or `BDL_RUNTIME_DIR`), flashing the Arduino Nano from Studio (`avrdude`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | the study's evidence on who installs what                                                                                                    | ADR-0039                                         |
| 5   | Third embedded family (ESP32-S3) — HAL independence is shown by the RP2040 (Embassy) and the Arduino Nano (avr-hal); the ESP32 adds a Wi-Fi-class target                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | priority 2                                                                                                                                   | ADR-0037                                         |

## Language and toolchain

Not ordered against the priorities above; each moves when a concrete need or a
formal result arrives.

- Cross-domain occurrence windows: a surface form for the window the design can
  already write, bounded (ISS-0001) — with it, a compiler-recognised bounded
  transport becomes possible (ADR-0027's rejected ring buffer).
- The unit-domain spelling, stage 2 and 3 (ADR-0029 amendment): the hint on
  `mapping f : A` becomes a warning, the Code view shows it too, and a language
  edition may remove the shorthand with `bdld migrate-unit-domain` applied
  automatically — after a versioning policy for the language exists.
- The equation language's remaining edges: `zip`'s cost in the generated core
  (ISS-0013); record syntax lowering to nested grouped values — only if a case
  asks (FV Phase 11 removes the general quantifier and comprehension, FVD-0113).
- The Formula Composer's next slices: `match`, blocks (`let`), rules (`x => …`),
  collection and grouped literals and `delay` / `sync` as structured components
  (the Composer shows them as text, `NodeKind::Opaque`); a preferred display
  unit per concept and per simulation input (presentation only, FV Phase 10 §9);
  hover cards on components; affine units (°C, °F) once the formal
  point/difference follow-up lands (ISS-0004); drag-and-drop from the palette.
- Studio: the collections report (readiness, byte bounds, window requirements)
  on the Deploy page, and the schedule as deployment data there — today
  `bdld compile --period` only.
- User-defined enums — ISS-0005; affine units — ISS-0004; temporal modifiers and
  contexts — ISS-0010: each starts as a proposal.
- Studio gaps listed in `docs/architecture/studio-compiler-integration.md` §3
  (_What remains_): the Deploy page on the 0.5 read model (`rows` / `missing` /
  `blocker`), an Explain request over the protocol (hover cards exist; Explain
  is the compiler's formal detail and is a different answer), domain regions and
  cycle emphasis on the canvas, entity hover and fixes inside a component's
  source (a body-scoped `EntityRef` for `HoverEntity` / `ListSemanticActions`).

- Cross-surface actions applying model operations from text editors; workspace
  symbols in the LSP.
- The Code view's remaining slice (ADR-0023): inline fault ranges (the list and
  the caret jump stand in); rename from the editor; a second source file created
  from Studio; the unit-domain hint shown in the pane; whole-graph relayout on
  request; a persisted edit history that makes text-only changes undoable
  (ISS-0009).

- Projection deltas and a persisted edit history — ISS-0009.
- Team / project / package libraries of items (LIB-2); a packaged device
  catalogue (LIB-3) — `docs/spec/concept-library.md`. Both realization sides
  exist in their first slice: outputs (ADR-0036: five profiles, a pure encoder
  each, three-judgment admissibility, machine sinks, the generated `Commands`)
  and Sources (ADR-0038: two GPIO line profiles, a pure transducer each,
  four-judgment admissibility, providers below the inputs, the generated
  `provide`), both drawn from `bdl-catalogue`, whose entries carry an origin no
  judgment reads (FV Phase 16). A package manager that resolves, validates and
  signs a package adds entries and changes nothing else; it starts as a
  proposal. What remains on the profiles themselves is ISS-0017 (stateful
  adapters, a device clock, atomic frames, the adapter's correspondence) and
  ISS-0018 (the occurrence contract, stateful transducers, analog and bus
  providers) — each after its FV phase.
- Native menu bar (`PlatformMenuBar` on macOS, in-window on Windows).
- An incremental query engine for the IDE service — only if profiling on real
  projects asks for it (baseline: ~2 ms per full analysis at 400 mappings).
- Supplied Rust computation blocks — designed in
  `docs/architecture/component-boundary.md` under ADR-0005; not started.

## Explicitly out of scope

From the engineering brief (§52): AI assistant · plugin marketplace · animation
polish · cloud sync · collaboration · full StateHandler editor · arbitrary
firmware backends · SMT · full numeric physical validation · code sandbox ·
component marketplace · custom debugger.
