//! A WiFi HaLow bridge using:
//!
//! - **U1**: MM8108-MF15457 Wi-Fi HaLow module (SPI0 ↔ RP2354A)
//! - **U2**: RP2354A host MCU
//! - **U4**: MCP73831T-2ATI/OT LiPo charge controller
//! - **U5**: TPS63031DSKR buck-boost converter (3.3 V output)
//!
//! Power is supplied by a single-cell LiPo battery (J2) with USB-C charging
//! (J1). A 5.1 kΩ resistor on each CC pin requests the default 5 V USB
//! profile. The MCP73831 charges the battery while the TPS63031 buck-boost
//! maintains a stable 3.3 V rail across the full LiPo discharge curve
//! (3.0–4.2 V).
//!
//! Decoupling capacitors are synthesised automatically from the decoupling
//! constraints declared on each part definition.

use anyhow::{Context, Result};
use copperleaf::{Board, PinRef, UnitExt, helpers::join};
use copperleaf_parts_connectors::{Conmhf4SmdGT, S2bPhSm4TbLfSn, UsbC23409011};
use copperleaf_parts_microchip::Mcp73831t2atiOt;
use copperleaf_parts_morsemicro::Mm8108Mf15457;
use copperleaf_parts_passives::{
    B82472p6152m000, B82472p6222m000, Resistor, footprint::Package, pulldown, pullup,
};
use copperleaf_parts_raspberrypi::Rp2354a;
use copperleaf_parts_texas_instruments::Tps63031dskr;

pub fn create() -> Result<Board> {
    let mut board = Board::new("halow-sta-low-min");
    board.set_dimensions(60.0, 40.0); // 60mm wide, 40mm high

    let rpi = board.add("U2", Rp2354a::new());
    let radio = board.add("U1", Mm8108Mf15457::new());

    // ── Power components ───────────────────────────────────────────
    let usb = board.add("J1", UsbC23409011::new());
    let charger = board.add("U4", Mcp73831t2atiOt::new());
    let reg = board.add("U5", Tps63031dskr::new());
    let batt = board.add("J2", S2bPhSm4TbLfSn::new());
    let ant = board.add("J3", Conmhf4SmdGT::new());

    // ═══ Ground net ═════════════════════════════════════════════════
    let gnd = join(
        &mut board,
        &[
            rpi.pin(Rp2354a::VREG_PGND),
            radio.pin(Mm8108Mf15457::GND_1),
            usb.pin(UsbC23409011::GND_A),
            usb.pin(UsbC23409011::GND_B),
            charger.pin(Mcp73831t2atiOt::VSS),
            reg.pin(Tps63031dskr::GND),
            reg.pin(Tps63031dskr::PGND),
            batt.pin(PinRef("2")),
            batt.pin(S2bPhSm4TbLfSn::SHIELD_1),
            batt.pin(S2bPhSm4TbLfSn::SHIELD_2),
        ],
    )?;
    board.set_net_name(gnd, "GND");

    // Shield tie-downs — all four USB-C shield pads to GND.
    board.connect(usb.pin(UsbC23409011::SHIELD_1), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(usb.pin(UsbC23409011::SHIELD_2), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(usb.pin(UsbC23409011::SHIELD_3), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(usb.pin(UsbC23409011::SHIELD_4), rpi.pin(Rp2354a::VREG_PGND))?;

    // ═══ USB 5 V rail (VBUS) ════════════════════════════════════════
    let vbus = join(
        &mut board,
        &[usb.pin(UsbC23409011::VBUS_A), usb.pin(UsbC23409011::VBUS_B)],
    )?;
    board.set_net_voltage(vbus, 5.0.volt());
    board.set_net_name(vbus, "VBUS");

    // CC pull-downs: 5.1 kΩ to GND requests the default 5 V profile.
    let gnd_pin = rpi.pin(Rp2354a::VREG_PGND);

    let r_cc1 = board.add("R_CC1", Resistor::new(5.1.kohm(), Package::M0603));
    board.connect(usb.pin(UsbC23409011::CC1), r_cc1.pin(Resistor::PIN1))?;
    board.connect(gnd_pin, r_cc1.pin(Resistor::PIN2))?;

    let r_cc2 = board.add("R_CC2", Resistor::new(5.1.kohm(), Package::M0603));
    board.connect(usb.pin(UsbC23409011::CC2), r_cc2.pin(Resistor::PIN1))?;
    board.connect(gnd_pin, r_cc2.pin(Resistor::PIN2))?;

    // ═══ Charger (MCP73831) ═════════════════════════════════════════
    // VDD ← VBUS
    board.connect(
        usb.pin(UsbC23409011::VBUS_A),
        charger.pin(Mcp73831t2atiOt::VDD),
    )?;

    // PROG → R_PROG (2 kΩ ≈ 500 mA charge current) → GND
    let r_prog = board.add("R_PROG", Resistor::new(2.0.kohm(), Package::M0603));
    board.connect(
        charger.pin(Mcp73831t2atiOt::PROG),
        r_prog.pin(Resistor::PIN1),
    )?;
    board.connect(gnd_pin, r_prog.pin(Resistor::PIN2))?;

    // STAT (open-drain) → LED + current limiting resistor → GND.
    // The LED is skippable; omitting R_LED just leaves STAT floating
    // (the charger is fine with that).
    // For now we leave STAT unconnected — the PROG resistor alone
    // configures charging.

    // ═══ Battery net (BAT) ══════════════════════════════════════════
    // Charger output, battery connector positive, buck-boost input,
    // and RP2354A internal regulator input (VREG_VIN: 2.7–5.5 V).
    let bat = join(
        &mut board,
        &[
            charger.pin(Mcp73831t2atiOt::VBAT),
            batt.pin(PinRef("1")),
            reg.pin(Tps63031dskr::VIN),
            rpi.pin(Rp2354a::VREG_VIN),
        ],
    )?;
    board.set_net_name(bat, "BAT");

    // ═══ Buck-boost converter (TPS63031, fixed 3.3 V) ═══════════════
    // Enable: tie HIGH (to BAT) for always-on.
    board.connect(reg.pin(Tps63031dskr::EN), reg.pin(Tps63031dskr::VIN))?;

    // Fixed-output configuration: FB → GND.
    board.connect(reg.pin(Tps63031dskr::FB), reg.pin(Tps63031dskr::GND))?;

    // Power-save / SYNC: tie LOW for automatic PFM/PWM switching.
    board.connect(reg.pin(Tps63031dskr::PS_SYNC), reg.pin(Tps63031dskr::GND))?;

    // VINA (control-stage supply): tie to VIN.
    board.connect(reg.pin(Tps63031dskr::VINA), reg.pin(Tps63031dskr::VIN))?;

    // Inductor between L1 and L2 (1.5 µH typ., e.g. LPS4018-152ML).
    let l_pwr = board.add("L_PWR", B82472p6152m000::new());
    board.connect(reg.pin(Tps63031dskr::L1), l_pwr.pin(B82472p6152m000::PIN_1))?;
    board.connect(reg.pin(Tps63031dskr::L2), l_pwr.pin(B82472p6152m000::PIN_2))?;

    // Exposed thermal pad to GND.
    board.connect(reg.pin(Tps63031dskr::EXP), rpi.pin(Rp2354a::VREG_PGND))?;

    // ═══ 3.3 V rail (V3V3) — regulated output ═══════════════════════
    // Wire all 3.3 V consumers.
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

    // ═══ RP2354A internal voltage regulator (1.1 V core) ═══════════════
    // VREG_VIN ← BAT (already connected above).
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
    board.connect(rpi.pin(Rp2354a::DVDD_1), rpi.pin(Rp2354a::VREG_AVDD))?;
    board.connect(rpi.pin(Rp2354a::DVDD_2), rpi.pin(Rp2354a::VREG_AVDD))?;
    board.connect(rpi.pin(Rp2354a::DVDD_3), rpi.pin(Rp2354a::VREG_AVDD))?;

    // Remaining supply pins ← V3V3.
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::ADC_AVDD))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::USB_OTP_VDD))?;
    board.connect(reg.pin(Tps63031dskr::VOUT), rpi.pin(Rp2354a::QSPI_IOVDD))?;

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
    // GPIO8-13 (used by Ethernet variant, free here).
    for &pin in &[
        Rp2354a::GPIO8,
        Rp2354a::GPIO9,
        Rp2354a::GPIO10,
        Rp2354a::GPIO11,
        Rp2354a::GPIO12,
        Rp2354a::GPIO13,
    ] {
        board.connect(rpi.pin(pin), rpi.pin(Rp2354a::VREG_PGND))?;
    }

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

    // USB pins — route channel 1 to the MCU.
    board.connect(rpi.pin(Rp2354a::USB_DM), usb.pin(UsbC23409011::DN1))?;
    board.connect(rpi.pin(Rp2354a::USB_DP), usb.pin(UsbC23409011::DP1))?;

    // Unused connector pins — tie to GND.
    board.connect(usb.pin(UsbC23409011::DP2), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(usb.pin(UsbC23409011::DN2), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(usb.pin(UsbC23409011::SBU1), rpi.pin(Rp2354a::VREG_PGND))?;
    board.connect(usb.pin(UsbC23409011::SBU2), rpi.pin(Rp2354a::VREG_PGND))?;

    // ═══ RP2354A control pins ══════════════════════════════════════════
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

    // STAT: pull-up to VBUS (open-drain, active-low — ignored when not used).
    pullup(
        &mut board,
        "R_STAT",
        charger.pin(Mcp73831t2atiOt::STAT),
        usb.pin(UsbC23409011::VBUS_A),
        Package::M0603,
    )?;

    Ok(board)
}

#[cfg(test)]
mod tests {
    use copperleaf_compile::CompileOptions;

    use super::*;

    #[test]
    fn design_has_five_ics() {
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
        assert_eq!(
            ics.len(),
            4,
            "U1=HaLow, U2=RP2354A, U4=Charger, U5=Regulator"
        );
    }

    #[test]
    fn vbus_is_5v() {
        use copperleaf::NetKind;
        let report = copperleaf_compile::run(
            create().unwrap(),
            &CompileOptions {
                decoupling_footprint: Package::M0603,
            },
        )
        .unwrap();
        let vbus = report
            .board
            .nets
            .iter()
            .find(|n| n.name == "VBUS")
            .expect("VBUS net must exist");
        let NetKind::Power { v_nom, .. } = &vbus.kind else {
            panic!("VBUS must be a power net");
        };
        assert!(
            (v_nom.as_base() - 5.0).abs() < 0.1,
            "VBUS must be 5 V, got {}",
            v_nom.as_base()
        );
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
