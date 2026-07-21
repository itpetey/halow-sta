//! MM8108-MF15457 + RP2354A + W5500 reference design using the new Copperleaf
//! `Board` API.
//!
//! A WiFi HaLow ↔ Ethernet bridge using:
//!
//! - **U1**: MM8108-MF15457 Wi-Fi HaLow module (SPI0 ↔ RP2354A)
//! - **U2**: RP2354A host MCU
//! - **U3**: W5500 hardwired TCP/IP Ethernet controller (SPI1 ↔ RP2354A)
//!
//! Decoupling capacitors are synthesised automatically from the decoupling
//! constraints declared on each part definition.

use anyhow::{Context, Result};
use copperleaf::{Board, PinRef, UnitExt, helpers::join};
use copperleaf_parts_connectors::Conmhf4SmdGT;
use copperleaf_parts_morsemicro::Mm8108Mf15457;
use copperleaf_parts_passives::{
    B82472p6222m000, Capacitor, Crystal, Resistor, footprint::Package, pulldown, pullup,
};
use copperleaf_parts_raspberrypi::Rp2354a;
use copperleaf_parts_wiznet::W5500;

pub fn create() -> Result<Board> {
    let mut board = Board::new("halow-sta-low-eth");

    let rpi = board.add("U2", Rp2354a::new());
    let radio = board.add("U1", Mm8108Mf15457::new());
    let eth = board.add("U3", W5500::new());
    let ant = board.add("J3", Conmhf4SmdGT::new());

    // ═══ Ground net ═════════════════════════════════════════════════
    let gnd = join(
        &mut board,
        &[
            rpi.pin(Rp2354a::VREG_PGND),
            radio.pin(Mm8108Mf15457::GND_1),
            eth.pin(W5500::GND),
            eth.pin(W5500::AGND),
        ],
    )?;
    board.set_net_name(gnd, "GND");

    // ═══ Power nets ═════════════════════════════════════════════════
    let vdd_io = join(
        &mut board,
        &[
            rpi.pin(Rp2354a::IOVDD),
            eth.pin(W5500::VDD),
            radio.pin(Mm8108Mf15457::VDDIO),
        ],
    )?;
    board.set_net_voltage(vdd_io, 3.3.volt());
    board.set_net_name(vdd_io, "VDD_IO");

    let vbat = board.net(radio.pin(Mm8108Mf15457::VBAT))?;
    board.set_net_voltage(vbat, 3.3.volt());
    board.set_net_name(vbat, "VBAT");

    let vbat_tx = board.net(radio.pin(Mm8108Mf15457::VBAT_TX))?;
    board.set_net_voltage(vbat_tx, 3.3.volt());
    board.set_net_name(vbat_tx, "VBAT_TX");

    // ═══ RP2354A internal voltage regulator (1.1 V core) ═══════════════
    // VREG_VIN ← VDD_IO (3.3 V, external supply).
    board.connect(rpi.pin(Rp2354a::VREG_VIN), rpi.pin(Rp2354a::IOVDD))?;

    // Inductor from VREG_LX to VREG_AVDD (2.2 µH typ.).
    let l_vreg = board.add("L_VREG", B82472p6222m000::new());
    let vreg_lx = board.connect(
        rpi.pin(Rp2354a::VREG_LX),
        l_vreg.pin(B82472p6222m000::PIN_1),
    )?;
    board.set_net_voltage(vreg_lx, 1.1.volt());
    board.set_net_name(vreg_lx, "VREG_SW");

    let vreg_avdd = board.connect(
        rpi.pin(Rp2354a::VREG_AVDD),
        l_vreg.pin(B82472p6222m000::PIN_2),
    )?;
    board.set_net_voltage(vreg_avdd, 1.1.volt());
    board.set_net_name(vreg_avdd, "VREG_1V1");

    // VREG_FB → VREG_1V1 (fixed-output configuration).
    board.connect(rpi.pin(Rp2354a::VREG_FB), rpi.pin(Rp2354a::VREG_AVDD))?;

    // Core supply: DVDD ← VREG_1V1.
    board.connect(rpi.pin(Rp2354a::DVDD), rpi.pin(Rp2354a::VREG_AVDD))?;

    // Remaining supply pins ← VDD_IO (3.3 V).
    board.connect(rpi.pin(Rp2354a::IOVDD), rpi.pin(Rp2354a::ADC_AVDD))?;
    board.connect(rpi.pin(Rp2354a::IOVDD), rpi.pin(Rp2354a::USB_OTP_VDD))?;
    board.connect(rpi.pin(Rp2354a::IOVDD), rpi.pin(Rp2354a::QSPI_IOVDD))?;

    // ═══ SPI0: HaLow module ↔ RP2354A (50 MHz) ══════════════════════
    let sdio_clk = board
        .connect(
            radio.pin(Mm8108Mf15457::SDIO_CLK_SPI_SCK),
            rpi.pin(Rp2354a::GPIO2),
        )
        .context("SDIO_CLK")?;
    board.set_net_name(sdio_clk, "SDIO_CLK");

    let sdio_cmd = board
        .connect(
            radio.pin(Mm8108Mf15457::SDIO_CMD_SPI_MOSI),
            rpi.pin(Rp2354a::GPIO3),
        )
        .context("SDIO_CMD")?;
    board.set_net_name(sdio_cmd, "SDIO_CMD");

    let sdio_d0 = board
        .connect(
            radio.pin(Mm8108Mf15457::SDIO_D0_SPI_MISO),
            rpi.pin(Rp2354a::GPIO0),
        )
        .context("SDIO_D0")?;
    board.set_net_name(sdio_d0, "SDIO_D0");

    let sdio_d3 = board
        .connect(
            radio.pin(Mm8108Mf15457::SDIO_D3_SPI_CS),
            rpi.pin(Rp2354a::GPIO1),
        )
        .context("SDIO_D3")?;
    board.set_net_name(sdio_d3, "SDIO_D3");

    let sdio_d1 = board
        .connect(
            radio.pin(Mm8108Mf15457::SDIO_D1_SPI_INT),
            rpi.pin(Rp2354a::GPIO4),
        )
        .context("SDIO_D1")?;
    board.set_net_name(sdio_d1, "SDIO_D1");

    // SDIO bus pull-ups (10 kΩ → VDD_IO), excluding the clock line.
    let vdd_io_pin = rpi.pin(Rp2354a::IOVDD);
    pullup(
        &mut board,
        "R1",
        radio.pin(Mm8108Mf15457::SDIO_D3_SPI_CS),
        vdd_io_pin,
        Package::M0603,
    )?;
    pullup(
        &mut board,
        "R2",
        radio.pin(Mm8108Mf15457::SDIO_CMD_SPI_MOSI),
        vdd_io_pin,
        Package::M0603,
    )?;
    pullup(
        &mut board,
        "R3",
        radio.pin(Mm8108Mf15457::SDIO_D0_SPI_MISO),
        vdd_io_pin,
        Package::M0603,
    )?;
    pullup(
        &mut board,
        "R4",
        radio.pin(Mm8108Mf15457::SDIO_D1_SPI_INT),
        vdd_io_pin,
        Package::M0603,
    )?;

    // ═══ HaLow control signals ══════════════════════════════════════
    let mm_reset = board
        .connect(radio.pin(Mm8108Mf15457::RESET_N), rpi.pin(Rp2354a::GPIO5))
        .context("MM_RESET_N")?;
    board.set_net_name(mm_reset, "MM_RESET_N");

    let mm_wake = board
        .connect(radio.pin(Mm8108Mf15457::WAKE), rpi.pin(Rp2354a::GPIO6))
        .context("MM_WAKE")?;
    board.set_net_name(mm_wake, "MM_WAKE");

    let mm_busy = board
        .connect(radio.pin(Mm8108Mf15457::BUSY), rpi.pin(Rp2354a::GPIO7))
        .context("MM_BUSY")?;
    board.set_net_name(mm_busy, "MM_BUSY");

    // ═══ MHF4 antenna connector ═══════════════════════════════════════
    let rf = board
        .connect(radio.pin(Mm8108Mf15457::ANT), ant.pin(Conmhf4SmdGT::Signal))
        .context("ANT")?;
    board.set_net_name(rf, "RF");

    // MHF4 ground pads to GND.
    board.connect(ant.pin(Conmhf4SmdGT::GND1), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(ant.pin(Conmhf4SmdGT::GND2), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(ant.pin(Conmhf4SmdGT::GND3), rpi.pin(Rp2354a::VREG_PGND))?;

    // ═══ SPI1: W5500 ↔ RP2354A (33 MHz) ═════════════════════════════
    let w_sclk = board
        .connect(eth.pin(W5500::SCLK), rpi.pin(Rp2354a::GPIO10))
        .context("W5500_SCLK")?;
    board.set_net_name(w_sclk, "W5500_SCLK");

    let w_mosi = board
        .connect(eth.pin(W5500::MOSI), rpi.pin(Rp2354a::GPIO11))
        .context("W5500_MOSI")?;
    board.set_net_name(w_mosi, "W5500_MOSI");

    let w_miso = board
        .connect(eth.pin(W5500::MISO), rpi.pin(Rp2354a::GPIO8))
        .context("W5500_MISO")?;
    board.set_net_name(w_miso, "W5500_MISO");

    let w_scsn = board
        .connect(eth.pin(W5500::SCSn), rpi.pin(Rp2354a::GPIO9))
        .context("W5500_SCSn")?;
    board.set_net_name(w_scsn, "W5500_SCSn");

    // ═══ W5500 control signals ══════════════════════════════════════
    let w_rstn = board
        .connect(eth.pin(W5500::RSTn), rpi.pin(Rp2354a::GPIO12))
        .context("W5500_RSTn")?;
    board.set_net_name(w_rstn, "W5500_RSTn");

    let w_intn = board
        .connect(eth.pin(W5500::INTn), rpi.pin(Rp2354a::GPIO13))
        .context("W5500_INTn")?;
    board.set_net_name(w_intn, "W5500_INTn");

    // ═══ W5500 crystal (25 MHz) ═════════════════════════════════════
    let y2 = board.add("Y2", Crystal::new(25.0.mhz()));
    let w_xi = board
        .connect(eth.pin(W5500::XI_CLKIN), y2.pin(Crystal::PIN1))
        .context("W5500_XI")?;
    board.set_net_name(w_xi, "W5500_XI");

    let w_xo = board
        .connect(eth.pin(W5500::XO), y2.pin(Crystal::PIN2))
        .context("W5500_XO")?;
    board.set_net_name(w_xo, "W5500_XO");

    // ═══ W5500 bias / reference components ══════════════════════════
    let r23 = board.add("R23", Resistor::new(12.4.kohm(), Package::M0603));
    let w_exres = board
        .connect(eth.pin(W5500::EXRES1), r23.pin(Resistor::PIN1))
        .context("W5500_EXRES")?;
    board.set_net_name(w_exres, "W5500_EXRES");
    board.connect(r23.pin(Resistor::PIN2), rpi.pin(Rp2354a::VREG_PGND))?;

    let c10 = board.add("C10", Capacitor::new(4.7.uf(), Package::M0603));
    let w_tocap = board
        .connect(eth.pin(W5500::TOCAP), c10.pin(Capacitor::PIN1))
        .context("W5500_TOCAP")?;
    board.set_net_name(w_tocap, "W5500_TOCAP");
    board.connect(c10.pin(Capacitor::PIN2), rpi.pin(Rp2354a::VREG_PGND))?;

    let c11 = board.add("C11", Capacitor::new(10.0.nf(), Package::M0603));
    let w_1v2o = board
        .connect(eth.pin(W5500::PIN_1V2O), c11.pin(Capacitor::PIN1))
        .context("W5500_1V2O")?;
    board.set_net_name(w_1v2o, "W5500_1V2O");
    board.connect(c11.pin(Capacitor::PIN2), rpi.pin(Rp2354a::VREG_PGND))?;

    // ═══ W5500 PHY mode select (pull-ups → 111 = auto-neg) ══════════
    // PMODE0/1/2 are wired to the LED outputs and pulled up to AVDD.
    let r24 = board.add("R24", Resistor::new(10.0.kohm(), Package::M0603));
    let spd = board
        .connect(eth.pin(W5500::PMODE0), eth.pin(W5500::SPDLED))
        .context("W5500_SPDLED")?;
    board.set_net_name(spd, "W5500_SPDLED");
    board.connect(eth.pin(W5500::PMODE0), r24.pin(Resistor::PIN1))?;
    let avdd = board.connect(eth.pin(W5500::AVDD), r24.pin(Resistor::PIN2))?;
    board.set_net_voltage(avdd, 3.3.volt());
    board.set_net_name(avdd, "AVDD");

    let r25 = board.add("R25", Resistor::new(10.0.kohm(), Package::M0603));
    let link = board
        .connect(eth.pin(W5500::PMODE1), eth.pin(W5500::LINKLED))
        .context("W5500_LINKLED")?;
    board.set_net_name(link, "W5500_LINKLED");
    board.connect(eth.pin(W5500::PMODE1), r25.pin(Resistor::PIN1))?;
    board.connect(eth.pin(W5500::AVDD), r25.pin(Resistor::PIN2))?;

    let r26 = board.add("R26", Resistor::new(10.0.kohm(), Package::M0603));
    let dup = board
        .connect(eth.pin(W5500::PMODE2), eth.pin(W5500::DUPLED))
        .context("W5500_DUPLED")?;
    board.set_net_name(dup, "W5500_DUPLED");
    board.connect(eth.pin(W5500::PMODE2), r26.pin(Resistor::PIN1))?;
    board.connect(eth.pin(W5500::AVDD), r26.pin(Resistor::PIN2))?;

    // ═══ W5500 reserved pins to GND ═════════════════════════════════
    board.connect(eth.pin(W5500::RSVD), rpi.pin(Rp2354a::VREG_PGND))?;

    // ═══ HaLow USB pins tied to GND in SPI mode ═════════════════════
    board.connect(
        radio.pin(Mm8108Mf15457::USB_D_N),
        rpi.pin(Rp2354a::VREG_PGND),
    )?;
    board.connect(
        radio.pin(Mm8108Mf15457::USB_D_P),
        rpi.pin(Rp2354a::VREG_PGND),
    )?;
    board.connect(
        radio.pin(Mm8108Mf15457::VDD_USB),
        rpi.pin(Rp2354a::VREG_PGND),
    )?;

    // ═══ HaLow unused-pin pull-downs (10 kΩ → GND) ══════════════════
    let gnd_pin = rpi.pin(Rp2354a::VREG_PGND);
    pulldown(
        &mut board,
        "R5",
        radio.pin(Mm8108Mf15457::JTAG_TMS),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R6",
        radio.pin(Mm8108Mf15457::JTAG_TCK),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R7",
        radio.pin(Mm8108Mf15457::JTAG_TDO),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R8",
        radio.pin(Mm8108Mf15457::JTAG_TDI),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R9",
        radio.pin(Mm8108Mf15457::SDIO_D2),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R10",
        radio.pin(Mm8108Mf15457::GPIO5),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R11",
        radio.pin(Mm8108Mf15457::GPIO4),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R12",
        radio.pin(Mm8108Mf15457::GPIO3),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R13",
        radio.pin(Mm8108Mf15457::GPIO1),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R14",
        radio.pin(Mm8108Mf15457::GPIO0),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R15",
        radio.pin(Mm8108Mf15457::GPIO6),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R16",
        radio.pin(Mm8108Mf15457::GPIO7),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R17",
        radio.pin(Mm8108Mf15457::GPIO8),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R18",
        radio.pin(Mm8108Mf15457::GPIO9),
        gnd_pin,
        Package::M0603,
    )?;
    pulldown(
        &mut board,
        "R19",
        radio.pin(Mm8108Mf15457::GPIO10),
        gnd_pin,
        Package::M0603,
    )?;

    // ═══ RP2354A free GPIOs (GPIO14–GPIO29) tied to GND ═════════════
    let free_gpio: &[PinRef] = &[
        Rp2354a::GPIO14,
        Rp2354a::GPIO15,
        Rp2354a::GPIO16,
        Rp2354a::GPIO17,
        Rp2354a::GPIO18,
        Rp2354a::GPIO19,
        Rp2354a::GPIO20,
        Rp2354a::GPIO21,
        Rp2354a::GPIO22,
        Rp2354a::GPIO23,
        Rp2354a::GPIO24,
        Rp2354a::GPIO25,
        Rp2354a::GPIO26_ADC0,
        Rp2354a::GPIO27_ADC1,
        Rp2354a::GPIO28_ADC2,
        Rp2354a::GPIO29_ADC3,
    ];
    for &pin in free_gpio {
        board.connect(rpi.pin(pin), rpi.pin(Rp2354a::VREG_PGND))?;
    }

    // ═══ RP2354A unused I/O — tie to GND ═════════════════════════════
    // QSPI pins (no external flash — RP2354A has internal).
    for &pin in &[
        Rp2354a::QSPI_SD0,
        Rp2354a::QSPI_SD1,
        Rp2354a::QSPI_SD2,
        Rp2354a::QSPI_SD3,
        Rp2354a::QSPI_SS,
    ] {
        board.connect(rpi.pin(pin), rpi.pin(Rp2354a::VREG_PGND))?;
    }

    // USB pins unused in this design.
    board.connect(rpi.pin(Rp2354a::USB_DM), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(rpi.pin(Rp2354a::USB_DP), rpi.pin(Rp2354a::VREG_PGND))?;

    // ═══ RP2354A control pins ══════════════════════════════════════════
    let vdd_io_pin = rpi.pin(Rp2354a::IOVDD);

    // RUN: 10 kΩ pull-up to VDD_IO for normal operation.
    pullup(
        &mut board,
        "R_RUN",
        rpi.pin(Rp2354a::RUN),
        vdd_io_pin,
        Package::M0603,
    )?;

    // SWDIO: 10 kΩ pull-up to VDD_IO for debug interface.
    pullup(
        &mut board,
        "R_SWD",
        rpi.pin(Rp2354a::SWDIO),
        vdd_io_pin,
        Package::M0603,
    )?;

    Ok(board)
}

#[cfg(test)]
mod tests {
    use copperleaf_compile::CompileOptions;
    use copperleaf_parts_passives::footprint::Package;

    use super::*;

    #[test]
    fn design_has_three_ics() {
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        let ics: Vec<_> = report
            .board
            .components
            .iter()
            .filter(|c| c.refdes.starts_with('U'))
            .collect();
        assert_eq!(ics.len(), 3, "U1=HaLow, U2=RP2354A, U3=W5500");
    }

    #[test]
    fn spi0_connects_halow_and_rp2354a() {
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        for net_name in ["SDIO_CLK", "SDIO_CMD", "SDIO_D0", "SDIO_D3"] {
            let refdes: Vec<String> = report
                .board
                .connections
                .iter()
                .filter(|c| report.board.net(c.net).name == net_name)
                .map(|c| report.board.components[c.component].refdes.clone())
                .collect();
            assert!(refdes.contains(&"U1".into()), "{} missing HaLow", net_name);
            assert!(
                refdes.contains(&"U2".into()),
                "{} missing RP2354A",
                net_name
            );
        }
    }

    #[test]
    fn spi1_connects_w5500_and_rp2354a() {
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        for net_name in ["W5500_SCLK", "W5500_MOSI", "W5500_MISO", "W5500_SCSn"] {
            let refdes: Vec<String> = report
                .board
                .connections
                .iter()
                .filter(|c| report.board.net(c.net).name == net_name)
                .map(|c| report.board.components[c.component].refdes.clone())
                .collect();
            assert!(refdes.contains(&"U3".into()), "{} missing W5500", net_name);
            assert!(
                refdes.contains(&"U2".into()),
                "{} missing RP2354A",
                net_name
            );
        }
    }

    #[test]
    fn halow_control_signals_connected() {
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        for net_name in ["MM_RESET_N", "MM_WAKE", "MM_BUSY"] {
            let refdes: Vec<String> = report
                .board
                .connections
                .iter()
                .filter(|c| report.board.net(c.net).name == net_name)
                .map(|c| report.board.components[c.component].refdes.clone())
                .collect();
            assert!(refdes.contains(&"U1".into()), "{} missing HaLow", net_name);
            assert!(
                refdes.contains(&"U2".into()),
                "{} missing RP2354A",
                net_name
            );
        }
    }

    #[test]
    fn vdd_usb_tied_to_ground() {
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        let gnd_pins: Vec<_> = report
            .board
            .connections
            .iter()
            .filter(|c| {
                report.board.net(c.net).name == "GND"
                    && report.board.components[c.component].refdes == "U1"
                    && c.pin == "VDD_USB"
            })
            .collect();
        assert!(
            !gnd_pins.is_empty(),
            "VDD_USB must be tied to GND in SPI mode"
        );
    }

    #[test]
    fn no_vdd_fem_rail() {
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        assert!(
            report.board.nets.iter().all(|n| n.name != "VDD_FEM"),
            "VDD_FEM must not exist in single-supply design"
        );
    }
}
