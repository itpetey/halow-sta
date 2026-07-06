//! HT-HC01 V2 WiFi HaLow — SPI reference design in copperleaf.
//!
//! This binary builds the complete SPI reference design from the
//! HT-HC01 V2 datasheet (§4.1) using the `copperleaf` EDA library,
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
