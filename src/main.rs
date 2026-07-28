//! Portable WiFi HaLow Station PCBs

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use copperleaf::Backend;
use copperleaf_backend_kicad::KiCad;
use copperleaf_compile::CompileOptions;
use copperleaf_layout::{SolveOptions, solve};
use copperleaf_parts_passives::footprint::Package;

mod ethernet_board;
mod minimal_board;
mod minimal_lipo_board;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
enum BoardName {
    Minimal,
    MinimalLipo,
    Ethernet,
}

/// Portable WiFi HaLow Station PCB generator
#[derive(Parser)]
struct Cli {
    /// Name of the board you are building
    #[arg(short, long, value_enum)]
    board: BoardName,
    /// Project name for your new board
    #[arg(short, long)]
    name: Option<String>,
    /// Path to store projects in (tree: <dir>/<name>/...)
    #[arg(short, long, default_value = "boards/")]
    dir: PathBuf,
    /// Auto-layout the PCB
    #[arg(short, long, default_value_t = false)]
    layout: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let board = match args.board {
        BoardName::Ethernet => ethernet_board::create()?,
        BoardName::Minimal => minimal_board::create(args.name.as_deref())?,
        BoardName::MinimalLipo => minimal_lipo_board::create()?,
    };

    let mut emit_path = args.dir;
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

    if args.layout {
        let solved = solve(&report.board, &SolveOptions::default())?;
        backend.emit_with_layout(&emit_path, &report.board, &solved.layout)?;
    } else {
        backend.emit_update(&emit_path, &report.board)?;
    }

    Ok(())
}
