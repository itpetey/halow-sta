//! Portable Wi-Fi HaLow station

use anyhow::{Context, Result};
use copperleaf_analysis::analyse;
use copperleaf_backend_kicad::KiCad;
use copperleaf_model::{Backend, Board};
use copperleaf_parts_morsemicro::Mm8108Mf15457;
use copperleaf_parts_raspberrypi::Rp2354a;

fn main() -> Result<()> {
    let backend = KiCad::new().with_project_name("halow-sta");

    let mut board = Board::new();
    let rpi = board.add("rpi", Rp2354a::new());
    let radio = board.add("radio", Mm8108Mf15457::new());

    // Intentionally incomplete: no NetHandle::set_voltage() call, so the
    // power net formed by IOVDD+VBAT has no voltage source.
    board.connect(rpi.pin(Rp2354a::IOVDD), radio.pin(Mm8108Mf15457::VBAT))?;

    let compiled = board
        .compile()
        .context("board compilation failed — check diagnostics")?;

    let report = analyse(compiled)?;

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
