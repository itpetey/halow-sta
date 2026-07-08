//! MM8108-MF15457 Wi-Fi HaLow — SPI reference design in copperleaf.
//!
//! This binary builds the complete SPI reference design from the
//! MM8108-MF15457 datasheet using the `copperleaf` EDA library,
//! then runs ERC and decoupling synthesis.

use reference_design::{build_spi_reference_design, run_analysis};

mod parts;
mod reference_design;

fn main() {
    let design = build_spi_reference_design();

    let json = serde_json::to_string_pretty(&design).expect("serialize design to JSON");
    std::fs::write("halow-sta.json", json).expect("write halow-sta.json");
    println!("Wrote halow-sta.json");

    run_analysis(&design);
}
