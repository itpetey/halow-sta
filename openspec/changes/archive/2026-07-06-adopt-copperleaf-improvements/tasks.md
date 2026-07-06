## 1. Delete passive.rs and use copperleaf passives

- [x] 1.1 Delete `src/parts/passive.rs`
- [x] 1.2 Update `src/parts/mod.rs` to remove `pub mod passive` and the `pub use passive::{...}` re-export
- [x] 1.3 Update imports in `src/reference_design.rs` to use `copperleaf::parts::{Capacitor, Resistor, Crystal}` instead of `crate::parts::{Capacitor, Resistor, Crystal}`

## 2. Remove id field and id() from part structs

- [x] 2.1 In `src/parts/ht_hc01.rs`: remove `id: String` field, remove `id: id.to_owned()` from `new()`, remove `fn id()` from `impl Block`
- [x] 2.2 In `src/parts/rp2354a.rs`: same removals
- [x] 2.3 In `src/parts/w5500.rs`: same removals
- [x] 2.4 Update `new()` constructors: keep the `id: &str` parameter for now (it may be used as MPN metadata later) or drop it if nothing references it

## 3. Replace SigSpec helpers with presets

- [x] 3.1 In `src/reference_design.rs`: delete `sig_net()`, `spi_spec()`, `spi_clk_spec()`, `spi1_spec()`, `spi1_clk_spec()`, `ctrl_spec()`, `rf_spec()`
- [x] 3.2 Replace all `sig_net("name", spi_spec())` calls with `sig_net("name", SigSpec::spi(50.0))` or inline `Net { ... kind: NetKind::Signal { spec: SigSpec::spi(50.0) } ... }`
- [x] 3.3 In `src/parts/ht_hc01.rs`: delete `spi_sig()` and `spi_clk_sig()`, use `SigSpec::spi(50.0)` and `SigSpec::spi_clk(50.0)` in pin definitions
- [x] 3.4 In `src/parts/w5500.rs`: delete `spi_sig`, `spi_clk_sig`, `analog_sig`, use presets
- [x] 3.5 In `src/parts/rp2354a.rs`: replace inline `SigSpec` construction with presets

## 4. Replace passive wiring with add_cap/add_res

- [x] 4.1 Replace all W5500 external component blocks (C10, C11, R23) with `d.add_cap()` / `d.add_res()` calls
- [x] 4.2 Replace W5500 decoupling caps loop (C12-C15) with `d.add_cap()` calls
- [x] 4.3 Replace PMODE pull-ups (R24-R26) with `d.add_res()` calls
- [x] 4.4 Replace HaLow decoupling caps loop (C1-C9) with `d.add_cap()` calls
- [x] 4.5 Replace HaLow SPI pull-ups (R1-R3, R5) with `d.add_res()` calls
- [x] 4.6 Replace HaLow pull-downs (R7-R22) with `d.add_res()` calls
- [x] 4.7 Replace antenna jumper (R6) with `d.add_res()`
- [x] 4.8 Replace crystal (Y2) with direct `add_component` + `wire` (crystal is not a cap/res)

## 5. Replace connect() with wire()/connect_net()

- [x] 5.1 Replace all `d.connect("U1", "SDIO_CLK", "SDIO_CLK")` with `d.wire("U1.SDIO_CLK", "SDIO_CLK")` or `d.connect_net("SDIO_CLK", &["U1.SDIO_CLK", "U2.GPIO2"])`
- [x] 5.2 Update all HaLow module ↔ RP2354A connections (section 8 of reference_design.rs)
- [x] 5.3 Update all W5500 ↔ RP2354A connections (section 9)
- [x] 5.4 Update power connections for all ICs

## 6. Use run_erc and report, simplify run_analysis

- [x] 6.1 Delete `erc_nc_pin_connected()` from `reference_design.rs`
- [x] 6.2 Replace the overvoltage ERC loop in `run_analysis()` with `copperleaf::run_erc(&d)`
- [x] 6.3 Replace the NC-pin check block with `run_erc()` results (NC checks are included)
- [x] 6.4 Replace component list, net summary, and decoupling print sections with `copperleaf::report(&d)`
- [x] 6.5 Keep the GPIO allocation section as a custom extension after `report()` output
- [x] 6.6 Keep the SPI bus connectivity section if desired (project-specific), or drop it if `report()` covers it

## 7. Emit design JSON

- [x] 7.1 In `src/main.rs`, after building the design, serialize to `halow-design.json` using `serde_json::to_string_pretty` and `std::fs::write`
- [x] 7.2 Print a message indicating the JSON file was written
- [x] 7.3 Verify that `cl report halow-design.json` works (manual test after copperleaf changes land)

## 8. Update add_component calls

- [x] 8.1 Change all `d.add_component(&u1)` to `d.add_component(u1)` (consume by value) throughout `reference_design.rs`

## 9. Update tests

- [x] 9.1 Update `halow_module_has_38_pins` and similar part tests to use new `new()` signature (no `id` param if dropped)
- [x] 9.2 Update design-level tests to use new `add_component()` signature
- [x] 9.3 Replace `erc_nc_pin_connected` test calls with `run_erc()` assertions
- [x] 9.4 Verify all tests pass: `cargo test`

## 10. Final validation

- [x] 10.1 Run `cargo build` — no warnings
- [x] 10.2 Run `cargo test` — all tests pass
- [x] 10.3 Run `cargo run` — design JSON is emitted and report is printed
- [x] 10.4 Manually verify line count reduction (target: ~400 lines total in src/)
