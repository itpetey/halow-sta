## ADDED Requirements

### Requirement: Project emits design JSON for CLI consumption
The project's `main.rs` SHALL serialize the built design to `halow-design.json` using `serde_json::to_string_pretty`. The emitted JSON SHALL include all nets, components, connections, and constraints, and SHALL be loadable by the copperleaf CLI (`cl report`, `cl verify`, `cl decouple`).

#### Scenario: Running main produces a design JSON file
- **WHEN** `cargo run` is executed
- **THEN** a file `halow-design.json` is written to the project root
- **AND** the file contains valid JSON with `"connections"`, `"nets"`, `"components"`, and `"constraints"` arrays

#### Scenario: Emitted JSON is CLI-compatible
- **WHEN** `cl report halow-design.json` is run
- **THEN** the CLI loads the design and prints a report with components, nets, and ERC results

### Requirement: Project uses copperleaf standard passives
The project SHALL NOT define its own passive component types. `Capacitor`, `Resistor`, and `Crystal` SHALL be sourced from `copperleaf::parts` whenever they are used (directly or via `Design::add_cap()` / `Design::add_res()`). The file `src/parts/passive.rs` SHALL be deleted.

#### Scenario: No local passive definitions
- **WHEN** the project source is inspected
- **THEN** `src/parts/passive.rs` does not exist
- **AND** `src/parts/mod.rs` does not define or re-export `Capacitor`, `Resistor`, or `Crystal`
- **AND** any use of those types resolves to `copperleaf::parts`

### Requirement: Project uses copperleaf built-in ERC
The project SHALL NOT define its own ERC rule functions. ERC SHALL be run via `copperleaf::run_erc(&design)`. The function `erc_nc_pin_connected()` SHALL be deleted from `reference_design.rs`.

#### Scenario: ERC runs via copperleaf
- **WHEN** the design is analyzed
- **THEN** `copperleaf::run_erc()` is called
- **AND** no project-local ERC function exists

### Requirement: Project uses copperleaf report function
The project SHALL use `copperleaf::report(&design)` for the generic design summary. The project MAY append project-specific sections (e.g., GPIO allocation) after the report output. The hand-rolled reporting in `run_analysis()` (component list, net summary, ERC printing, decoupling printing) SHALL be replaced.

#### Scenario: Report uses copperleaf
- **WHEN** `run_analysis()` is called
- **THEN** `copperleaf::report()` is called for the generic summary
- **AND** only project-specific sections (GPIO allocation) are printed separately

### Requirement: Project uses SigSpec presets
The project SHALL NOT define custom signal spec helper functions. `SigSpec` values SHALL be constructed using `SigSpec::spi()`, `::spi_clk()`, `::control()`, and `::rf_50ohm()` presets.

#### Scenario: No custom sig spec helpers
- **WHEN** the project source is inspected
- **THEN** functions `spi_spec()`, `spi_clk_spec()`, `ctrl_spec()`, `rf_spec()`, `spi1_spec()`, `spi1_clk_spec()`, `sig_net()` do not exist
- **AND** signal specs use `SigSpec::spi(50.0)`, `SigSpec::spi_clk(50.0)`, `SigSpec::control()`, `SigSpec::rf_50ohm()`, etc.

### Requirement: Project uses connection helper methods
The project SHALL use `Design::add_cap()`, `Design::add_res()`, `Design::wire()`, and `Design::connect_net()` for wiring instead of the 5-line `ComponentInst::new()` + `add_component()` + `connect()` pattern.

#### Scenario: Passive wiring uses add_cap/add_res
- **WHEN** a decoupling capacitor is added to the design
- **THEN** `d.add_cap(refdes, value, net_pos, net_neg)` is used instead of manual construction and connection

### Requirement: Part structs have no id field
Part structs (`HtHc01`, `Rp2354a`, `W5500`) SHALL NOT store an `id: String` field and SHALL NOT implement `id()`. The `Block` trait impls SHALL include only `pins()` and optionally `constraints()`.

#### Scenario: Part struct without id field
- **WHEN** `HtHc01::new()` is called
- **THEN** the returned struct has a `pins` field but no `id` field
- **AND** the `impl Block for HtHc01` does not include an `id()` method
