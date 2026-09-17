# hardware/boards

Board descriptions as data: one `Hardware` per file (TOML form of
`bdl-hardware::model::Hardware`). Per resource: its capabilities and, per
capability, the unit (timer / peripheral) behind it; board-wide, which
capabilities are shareable (buses). No Rust source fragments live here.

`arduino_nano.toml` and `big_board.toml` are generated from
`bdl-hardware::boards` and checked in; the round-trip test fails when they drift
(regenerate with `BDL_WRITE_BOARDS=1 cargo test -p bdl-hardware`). Schema and
constraints: `docs/spec/hardware-model.md`.
