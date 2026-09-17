# examples

Checked-in projects that open in Studio (Open Project…) and are kept
faithful by tests.

| Project | Demonstrates | Kept by |
|---|---|---|
| `smart_lamp/` | `Tilt` and `AmbientLight` in (two unresolved inputs on the `interaction` domain), `Brightness` out; `dimByTilt`, `adaptBrightness` and the driver `brightness = adaptBrightness(dimByTilt(tilt), ambient)`; one required output *Light Output* realised by a `pwm_channel` device — Design → Simulate → Deploy | `crates/bdl-compiler/tests/examples.rs` (authored from ops, analysed, simulated, placed, byte-identical to the files); `apps/studio/test/smart_lamp_e2e_test.dart` (the same walk through Studio's reducer against `bdld`) |

Format: docs/spec/project-format.md. Walk-through: docs/guides/getting-started.md §4.
