//! A WiFi HaLow bridge using:
//!
//! - **U1**: MM8108-MF15457 Wi-Fi HaLow module (SPI0 ↔ RP2354A)
//! - **U2**: RP2354A host MCU
//! - **U3**: TPS63031DSKR buck-boost converter (3.3 V output)
//!
//! Power is supplied by a single-cell 16340/CR123A, while the TPS63031
//! buck-boost maintains a stable 3.3 V rail across the full LiPo discharge
//! curve (3.0–3.7 V).
//!
//! Decoupling capacitors are synthesised automatically from the decoupling
//! constraints declared on each part definition.

use anyhow::{Context, Result};
use copperleaf::{Board, PinRef, UnitExt, helpers::join};
use copperleaf_parts_connectors::{Bh123a, ConSmaEdgeS};
use copperleaf_parts_morsemicro::Mm8108Mf15457;
use copperleaf_parts_passives::{
    B82472p6152m000, B82472p6222m000, footprint::Package, pulldown, pullup,
};
use copperleaf_parts_raspberrypi::Rp2354a;
use copperleaf_parts_texas_instruments::Tps63031dskr;

pub fn create() -> Result<Board> {
    let mut board = Board::new("halow-sta-low-min");
    board.set_dimensions(18.3, 48.0); // 18.3mm wide, 48mm high

    //
    // Components
    //

    // Raspberry Pi RP2354A microcontroller
    let rpi = board.add("U1", Rp2354a::new());
    // Morse Micro MM8108-MF15457 HaLow module
    let radio = board.add("U2", Mm8108Mf15457::new());
    // Buck-Boost converter
    let reg = board.add("U3", Tps63031dskr::new());
    // Through-hole battery terminals
    let batt = board.add("J1", Bh123a::new());
    // SMA Female antenna socket
    let ant = board.add("J2", ConSmaEdgeS::new());
    // Power inductors
    let l_pwr = board.add("L_PWR", B82472p6152m000::new());
    let l_vreg = board.add("L_VREG", B82472p6222m000::new());

    //
    // Ground Net
    //

    let gnd = join(
        &mut board,
        &[
            rpi.pin(Rp2354a::VREG_PGND),
            radio.pin(Mm8108Mf15457::GND_1),
            reg.pin(Tps63031dskr::GND),
            reg.pin(Tps63031dskr::PGND),
            batt.pin(Bh123a::NEGATIVE),
        ],
    )?;
    board.set_net_name(gnd, "GND");

    //
    // Battery net (BAT)
    //

    // Battery connector positive, buck-boost input,
    // and RP2354A internal regulator input (VREG_VIN: 2.7–5.5 V).
    let bat = join(
        &mut board,
        &[
            batt.pin(Bh123a::POSITIVE),
            reg.pin(Tps63031dskr::VIN),
            rpi.pin(Rp2354a::VREG_VIN),
        ],
    )?;
    board.set_net_voltage(bat, 3.7.volt());
    board.set_net_name(bat, "BAT");

    //
    // Buck-boost converter (TPS63031, fixed 3.3 V)
    //

    // Enable: tie HIGH (to BAT) for always-on.
    board.connect(reg.pin(Tps63031dskr::EN), reg.pin(Tps63031dskr::VIN))?;

    // Fixed-output configuration: FB → GND.
    board.connect(reg.pin(Tps63031dskr::FB), reg.pin(Tps63031dskr::GND))?;

    // Power-save / SYNC: tie LOW for automatic PFM/PWM switching.
    board.connect(reg.pin(Tps63031dskr::PS_SYNC), reg.pin(Tps63031dskr::GND))?;

    // VINA (control-stage supply): tie to VIN.
    board.connect(reg.pin(Tps63031dskr::VINA), reg.pin(Tps63031dskr::VIN))?;

    // Inductor between L1 and L2 (1.5 µH typ., e.g. LPS4018-152ML).
    board.connect(reg.pin(Tps63031dskr::L1), l_pwr.pin(B82472p6152m000::PIN_1))?;
    board.connect(reg.pin(Tps63031dskr::L2), l_pwr.pin(B82472p6152m000::PIN_2))?;

    // Exposed thermal pad to GND.
    board.connect(reg.pin(Tps63031dskr::EXP), rpi.pin(Rp2354a::VREG_PGND))?;

    //
    // 3.3 V net (V3V3)
    //

    let v3v3 = board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::IOVDD_1))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::IOVDD_2))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::IOVDD_3))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::IOVDD_4))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::IOVDD_5))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::IOVDD_6))?;
    board.set_net_voltage(v3v3, 3.3.volt());
    board.set_net_name(v3v3, "V3V3");
    board.connect(reg.pin(Tps63031dskr::VOUT), radio.pin(Mm8108Mf15457::VDDIO))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), radio.pin(Mm8108Mf15457::VBAT))?;
    board.connect(
        reg.pin(Tps63031dskr::VOUT),
        radio.pin(Mm8108Mf15457::VBAT_TX),
    )?;

    //
    // RP2354A internal voltage regulator (1.1 V core)
    //

    // VREG_VIN ← BAT (already connected above).
    // Inductor from VREG_LX to VREG_AVDD (2.2 µH typ.).
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
    board.connect(rpi.pin(Rp2354a::DVDD_1), rpi.pin(Rp2354a::VREG_AVDD))?;
    board.connect(rpi.pin(Rp2354a::DVDD_2), rpi.pin(Rp2354a::VREG_AVDD))?;
    board.connect(rpi.pin(Rp2354a::DVDD_3), rpi.pin(Rp2354a::VREG_AVDD))?;

    // Remaining supply pins ← V3V3.
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::ADC_AVDD))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::USB_OTP_VDD))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::QSPI_IOVDD))?;

    //
    // HaLow module ↔ RP2354A (50 MHz)
    //

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

    // SDIO bus pull-ups (10 kΩ → V3V3), excluding the clock line.
    let v3v3_pin = reg.pin(Tps63031dskr::VOUT);
    pullup(
        &mut board,
        "R1",
        radio.pin(Mm8108Mf15457::SDIO_D3_SPI_CS),
        v3v3_pin,
        Package::M0603,
    )?;
    pullup(
        &mut board,
        "R2",
        radio.pin(Mm8108Mf15457::SDIO_CMD_SPI_MOSI),
        v3v3_pin,
        Package::M0603,
    )?;
    pullup(
        &mut board,
        "R3",
        radio.pin(Mm8108Mf15457::SDIO_D0_SPI_MISO),
        v3v3_pin,
        Package::M0603,
    )?;
    pullup(
        &mut board,
        "R4",
        radio.pin(Mm8108Mf15457::SDIO_D1_SPI_INT),
        v3v3_pin,
        Package::M0603,
    )?;

    //
    // HaLow control signals
    //

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

    // HaLow USB pins tied to GND in SPI mode
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

    //
    // Antenna connector
    //

    let rf = board
        .connect(radio.pin(Mm8108Mf15457::ANT), ant.pin(ConSmaEdgeS::Signal))
        .context("ANT")?;
    board.set_net_name(rf, "RF");

    // Ground pads to GND
    board.connect(ant.pin(ConSmaEdgeS::GND1), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(ant.pin(ConSmaEdgeS::GND2), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(ant.pin(ConSmaEdgeS::GND3), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(ant.pin(ConSmaEdgeS::GND4), rpi.pin(Rp2354a::VREG_PGND))?;

    //
    // Unused pin pull-downs
    //

    // Arbitrary ground pin in the ground net
    let gnd_pin = rpi.pin(Rp2354a::VREG_PGND);

    // Radio (10 kΩ → GND)
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

    // RP2354A
    let free_gpio: &[PinRef] = &[
        Rp2354a::GPIO8,
        Rp2354a::GPIO9,
        Rp2354a::GPIO10,
        Rp2354a::GPIO11,
        Rp2354a::GPIO12,
        Rp2354a::GPIO13,
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
        Rp2354a::QSPI_SD0,
        Rp2354a::QSPI_SD1,
        Rp2354a::QSPI_SD2,
        Rp2354a::QSPI_SD3,
        Rp2354a::QSPI_SS,
        Rp2354a::USB_DM,
        Rp2354a::USB_DP,
    ];
    for &pin in free_gpio {
        board.connect(rpi.pin(pin), rpi.pin(Rp2354a::VREG_PGND))?;
    }

    //
    // RP2354A control pins
    //

    // RUN: 10 kΩ pull-up to V3V3 for normal operation.
    pullup(
        &mut board,
        "R_RUN",
        rpi.pin(Rp2354a::RUN),
        v3v3_pin,
        Package::M0603,
    )?;

    // SWDIO: 10 kΩ pull-up to V3V3 for debug interface.
    pullup(
        &mut board,
        "R_SWD",
        rpi.pin(Rp2354a::SWDIO),
        v3v3_pin,
        Package::M0603,
    )?;

    Ok(board)
}

#[cfg(test)]
mod tests {
    use copperleaf_compile::CompileOptions;

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
        assert_eq!(ics.len(), 3, "U1=RP2354A, U2=HaLow, U3=Regulator");
    }

    #[test]
    fn v3v3_is_3v3() {
        use copperleaf::NetKind;
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        let v3v3 = report
            .board
            .nets
            .iter()
            .find(|n| n.name == "V3V3")
            .expect("V3V3 net must exist");
        let NetKind::Power { v_nom, .. } = &v3v3.kind else {
            panic!("V3V3 must be a power net");
        };
        assert!(
            (v_nom.as_base() - 3.3).abs() < 0.1,
            "V3V3 must be 3.3 V, got {}",
            v_nom.as_base()
        );
    }

    #[test]
    fn battery_net_exists() {
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        assert!(
            report.board.nets.iter().any(|n| n.name == "BAT"),
            "BAT net must exist"
        );
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
                    && report.board.components[c.component].refdes == "U2"
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
