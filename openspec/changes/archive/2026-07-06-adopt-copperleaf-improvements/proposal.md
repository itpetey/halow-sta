## Why

This project currently reimplements ~500 lines of boilerplate that the copperleaf improvements (see copperleaf changes `serialize-ir-connectivity`, `cli-ingest-external-designs`, `analysis-stdlib`, `edsl-ergonomics`) eliminate: a custom `passive.rs` (139 lines), a hand-rolled `erc_nc_pin_connected()` function, ~166 lines of `println!` reporting in `run_analysis()`, 6 duplicate `SigSpec` helper functions, ~150 lines of passive wiring ceremony, and `id` fields on every part struct. Adopting the new copperleaf APIs will cut this project from ~1,300 lines to ~400 while gaining CLI-driven analysis via `cl report design.json`.

## What Changes

- Delete `src/parts/passive.rs` entirely — use `copperleaf::parts::{Capacitor, Resistor, Crystal}` from the stdlib.
- Delete `erc_nc_pin_connected()` from `reference_design.rs` — use `copperleaf::run_erc()`.
- Replace `run_analysis()` with `copperleaf::report(&design)` — delete the ~166-line hand-rolled reporting function. Keep only the project-specific GPIO allocation section as a small extension.
- Replace all custom `SigSpec` helper functions (`spi_spec()`, `spi_clk_spec()`, `ctrl_spec()`, `rf_spec()`, etc.) with `SigSpec::spi()`, `::spi_clk()`, `::control()`, `::rf_50ohm()` presets.
- Replace all 5-line passive instantiation patterns with `d.add_cap()` / `d.add_res()` one-liners.
- Replace `d.connect("U1", "SDIO_CLK", "SDIO_CLK")` patterns with `d.wire("U1.SDIO_CLK", "SDIO_CLK")` and `d.connect_net()`.
- Remove `id` field and `id()` method from `HtHc01`, `Rp2354a`, `W5500` structs (required by `Block` trait change).
- Change `d.add_component(&u1)` to `d.add_component(u1)` (consume instead of borrow).
- Update `main.rs` to emit design JSON to a file (`halow-design.json`) in addition to running analysis inline, enabling `cl report halow-design.json` workflow.
- Update all tests to use the new APIs.

## Capabilities

### New Capabilities
- `cli-integration`: The project emits a design JSON file that can be consumed by the copperleaf CLI (`cl report`, `cl verify`, `cl decouple`).

### Modified Capabilities

## Impact

- **`src/parts/mod.rs`**: Remove `passive` module and its re-exports. Import passives from `copperleaf::parts` instead.
- **`src/parts/ht_hc01.rs`**: Remove `id` field, `id()` method, custom `spi_sig()`/`spi_clk_sig()` helpers → use `SigSpec::spi()`/`::spi_clk()`.
- **`src/parts/rp2354a.rs`**: Remove `id` field, `id()` method, inline `SigSpec` construction → use presets.
- **`src/parts/w5500.rs`**: Same cleanup as above.
- **`src/reference_design.rs`**: Delete `sig_net()`, all `*_spec()` helpers, `erc_nc_pin_connected()`, most of `run_analysis()`. Replace passive wiring with `add_cap`/`add_res`. Replace `connect()` calls with `wire()`/`connect_net()`. Add JSON emission.
- **`src/main.rs`**: Add `serde_json::to_string_pretty` write to file.
- **`Cargo.toml`**: Already depends on `copperleaf` and `serde_json` — no new deps.
- Depends on all four copperleaf changes being implemented first.
