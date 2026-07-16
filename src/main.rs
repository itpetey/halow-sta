//! Portable Wi-Fi HaLow station

use anyhow::{Context, Result};
use clap::Parser;
use copperleaf::Backend;
use copperleaf_backend_kicad::KiCad;

mod ethernet_board;
mod minimal_board;

#[derive(Parser)]
enum BoardArg {
    Minimal,
    Ethernet,
}

fn main() -> Result<()> {
    let backend = KiCad::new().with_project_name("halow-sta");
    let arg = BoardArg::parse();
    let board = match arg {
        BoardArg::Ethernet => ethernet_board::create(),
        BoardArg::Minimal => minimal_board::create(),
    };
    let report = board?
        .compile()
        .context("board compilation failed — check diagnostics")?;

    println!(
        "Compiled {} nets, {} pins, {} components",
        report.summary.nets.len(),
        report.summary.pin_count,
        report.summary.component_count,
    );
    for warning in &report.warnings {
        println!("warning: {:?} - {}", warning.severity, warning.message);
    }

    backend.emit("kicad/", &report.board)?;

    Ok(())
}
