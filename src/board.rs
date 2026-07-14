use anyhow::Result;
use copperleaf::Board;
use copperleaf_parts_morsemicro::Mm8108Mf15457;
use copperleaf_parts_raspberrypi::Rp2354a;
use copperleaf_parts_wiznet::W5500;

pub fn create() -> Result<Board> {
    let mut board = Board::new();
    let rpi = board.add("rpi", Rp2354a::new());
    let radio = board.add("radio", Mm8108Mf15457::new());
    let eth_ctrl = board.add("eth_ctrl", W5500::new());

    // Intentionally incomplete: no NetHandle::set_voltage() call, so the
    // power net formed by IOVDD+VBAT has no voltage source.
    board.connect(rpi.pin(Rp2354a::IOVDD), radio.pin(Mm8108Mf15457::VBAT))?;

    Ok(board)
}
