//! MM8108-MF15457-based portable Wi-Fi HaLow station

use copperleaf_model::Board;

use crate::parts::{mm8108_mf15457::Mm8108Mf15457, rp2354a::Rp2354a};

mod parts;

fn main() -> Result<(), ()> {
    let board = board()?;
    // run_analysis(&board);

    Ok(())
}

fn board() -> Result<Board, ()> {
    let mut board = Board::new();
    board.add("rpi", Rp2354a::new());
    board.add("radio", Mm8108Mf15457::new());
    board.connect("rpi.IOVDD", "radio.VBAT")?;
    Ok(board)
}
