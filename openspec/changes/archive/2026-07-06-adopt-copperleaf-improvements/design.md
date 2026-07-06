## Context

The halow-sta project is the first real consumer of copperleaf outside the examples. It exposes significant boilerplate that the planned copperleaf improvements eliminate. This change refactors halow-sta to adopt those improvements once they land.

Current line counts (approximate):
- `passive.rs`: 139 lines (entirely replaceable by `copperleaf::parts`)
- `ht_hc01.rs`: 190 lines (can shrink to ~50 with `part!` macro and `SigSpec` presets)
- `rp2354a.rs`: 123 lines (can shrink to ~40)
- `w5500.rs`: 181 lines (can shrink to ~50)
- `reference_design.rs`: 919 lines (can shrink to ~250 with connection helpers, built-in ERC, built-in reporting, and SigSpec presets)
- **Total: ~1,552 lines → target ~400 lines**

## Goals / Non-Goals

**Goals:**
- Delete `passive.rs` — use `copperleaf::parts::{Capacitor, Resistor, Crystal}`.
- Delete hand-rolled ERC — use `copperleaf::run_erc()`.
- Delete hand-rolled reporting — use `copperleaf::report()`, keep only GPIO allocation as a custom extension.
- Replace custom `SigSpec` helpers with presets.
- Replace passive wiring with `add_cap()`/`add_res()`.
- Replace `connect()` with `wire()`/`connect_net()`.
- Remove `id` fields from part structs.
- Emit design JSON for CLI consumption.
- Keep all existing tests passing (adapted to new APIs).

**Non-Goals:**
- Adopting the `part!` macro (separate follow-up; the manual `Block` impls work fine once `id()` is removed).
- Adding new design features or changing the circuit.
- Restructuring the project layout.

## Decisions

### D1: Use copperleaf passives, delete passive.rs

**Decision:** Delete `src/parts/passive.rs`. Import `Capacitor`, `Resistor`, `Crystal` from `copperleaf::parts`. Update `src/parts/mod.rs` to remove the `passive` module.

**Rationale:** The 139-line `passive.rs` is a carbon copy of what `analysis-stdlib` adds to `copperleaf-parts`. No project-specific customization justifies keeping it.

### D2: Use run_erc(), delete erc_nc_pin_connected()

**Decision:** Replace all calls to the project-local `erc_nc_pin_connected()` and the hand-rolled overvoltage loop with `copperleaf::run_erc(&design)`.

**Rationale:** `run_erc()` includes NC-pin checks, overvoltage checks, and floating-input checks. The project's `erc_nc_pin_connected()` is literally the same logic moving to copperleaf.

### D3: Use report(), reduce run_analysis()

**Decision:** Replace `run_analysis()` with a thin wrapper that calls `copperleaf::report(&design)` and appends the project-specific GPIO allocation table.

**Rationale:** Of the 166 lines in `run_analysis()`, ~140 are generic (component list, net summary, ERC, decoupling). Only the GPIO allocation table (lines 653-674) is project-specific. `report()` covers the generic parts.

### D4: SigSpec presets replace all custom helpers

**Decision:** Delete `sig_net()`, `spi_spec()`, `spi_clk_spec()`, `spi1_spec()`, `spi1_clk_spec()`, `ctrl_spec()`, `rf_spec()` from `reference_design.rs`. Delete `spi_sig()`, `spi_clk_sig()` from `ht_hc01.rs` and `w5500.rs`. Use `SigSpec::spi(50.0)`, `::spi_clk(50.0)`, `::control()`, `::rf_50ohm()` instead.

**Mapping:**
| Old helper | New preset |
|---|---|
| `spi_spec()` / `spi_sig()` | `SigSpec::spi(50.0)` |
| `spi_clk_spec()` / `spi_clk_sig()` | `SigSpec::spi_clk(50.0)` |
| `spi1_spec()` | `SigSpec::spi(33.0)` |
| `spi1_clk_spec()` | `SigSpec::spi_clk(33.0)` |
| `ctrl_spec()` | `SigSpec::control()` |
| `rf_spec()` | `SigSpec::rf_50ohm()` |

### D5: Connection helpers replace connect() calls

**Decision:** Replace passive wiring blocks with `d.add_cap(refdes, value, net_pos, net_neg)` and `d.add_res(refdes, value, net_a, net_b)`. Replace IC-to-IC `connect()` pairs with `d.connect_net(net, &["U1.pin", "U2.pin"])`.

**Example transformation:**
```rust
// Before (5 lines per cap):
let c10 = Capacitor::new("C10", 4.7.uf());
let c10_inst = ComponentInst::new("C10", c10);
d.add_component(&c10_inst);
d.connect("C10", "1", "W5500_TOCAP");
d.connect("C10", "2", "GND");

// After (1 line):
d.add_cap("C10", 4.7.uf(), "W5500_TOCAP", "GND");
```

### D6: Emit design JSON

**Decision:** `main.rs` writes the design to `halow-design.json` using `serde_json::to_string_pretty`. This enables `cl report halow-design.json` and `cl verify halow-design.json`.

**Rationale:** Makes the project CLI-compatible. Users can run any copperleaf CLI command against the emitted design without recompiling.

### D7: Remove id fields from parts

**Decision:** Remove `id: String` field and `id()` method from `HtHc01`, `Rp2354a`, `W5500`. The `new(id: &str)` constructors keep the parameter for backwards compat but drop the field (or repurpose it if needed for MPN metadata later).

Actually, since `id()` is removed from `Block`, the `new()` constructors can drop the `id` parameter entirely. The refdes is assigned at `ComponentInst::new()` time.

## Risks / Trade-offs

- **[Dependency on copperleaf changes]** This change cannot be implemented until all four copperleaf changes land. → Order: implement copperleaf changes first, then this one.
- **[Test churn]** All tests in `reference_design.rs` use the old APIs and must be rewritten. → Mechanical migration; test logic stays the same, just API calls change.
- **[GPIO allocation reporting]** The GPIO allocation table is project-specific and stays in halow-sta. → Acceptable: `report()` handles the generic parts, projects append their own sections.
