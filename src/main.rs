//! MM8108-MF15457-based portable Wi-Fi HaLow station
//!
//! This prototype intentionally leaves the power net without an explicit voltage
//! override so that `Board::compile()` returns a `CompileError` demonstrating the
//! new diagnostic pipeline.

use anyhow::{Context, Result};
use copperleaf_backend_kicad::KiCad;
use copperleaf_model::{Backend, Board};

use crate::parts::{mm8108_mf15457::Mm8108Mf15457, rp2354a::Rp2354a};

mod parts;

fn main() -> Result<()> {
    let backend = KiCad::new().with_project_name("halow-sta");

    let mut board = Board::new();
    let rpi = board.add("rpi", Rp2354a::new());
    let radio = board.add("radio", Mm8108Mf15457::new());

    // Intentionally incomplete: no NetHandle::set_voltage() call, so the
    // power net formed by IOVDD+VBAT has no voltage source.
    board.connect(rpi.pin(Rp2354a::IOVDD), radio.pin(Mm8108Mf15457::VBAT))?;

    let report = board
        .compile()
        .context("board compilation failed — check diagnostics above")?;

    println!(
        "Compiled {} nets, {} pins, {} components",
        report.summary.nets.len(),
        report.summary.pin_count,
        report.summary.component_count,
    );
    for warning in &report.warnings {
        println!("warning: {:?} - {}", warning.severity, warning.message);
    }

    backend.emit("path/to/kicad/proj/", &report.board)?;

    Ok(())
}
