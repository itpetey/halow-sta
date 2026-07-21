//! Portable Wi-Fi HaLow station

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use copperleaf::Backend;
use copperleaf_backend_kicad::KiCad;
use copperleaf_compile::CompileOptions;
use copperleaf_parts_passives::footprint::Package;

mod ethernet_board;
mod minimal_board;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
enum BoardName {
    Minimal,
    Ethernet,
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_enum)]
    board: BoardName,
    #[arg(short, long, default_value = "boards/")]
    project_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let board = match args.board {
        BoardName::Ethernet => ethernet_board::create()?,
        BoardName::Minimal => minimal_board::create()?,
    };

    let mut emit_path = args.project_dir;
    emit_path.push(board.name());

    let backend = KiCad::new().with_project_name(board.name());

    let report = copperleaf_compile::run(
        board,
        &CompileOptions {
            decoupling_footprint: Package::M0603,
        },
    )
    .context("board compilation failed — check diagnostics")?;

    println!(
        "Compiled {} nets, {} pins, {} components",
        report.summary.nets.len(),
        report.summary.pin_count,
        report.summary.component_count,
    );
    for warning in &report.warnings {
        println!("{:?} - {}", warning.severity, warning.message);
    }

    backend.emit(&emit_path, &report.board)?;

    Ok(())
}
