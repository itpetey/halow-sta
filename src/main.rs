//! Portable Wi-Fi HaLow station

use anyhow::{Context, Result};
use copperleaf::Backend;
use copperleaf_backend_kicad::KiCad;

mod board;

fn main() -> Result<()> {
    let backend = KiCad::new().with_project_name("halow-sta");

    let report = board::create()?
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
