//! HT-HC01 V2 + RP2354A + W5500 reference design.
//!
//! Reproduces a WiFi HaLow ↔ Ethernet bridge using:
//!
//! - **U1**: HT-HC01 V2 WiFi HaLow module (SPI0 ↔ RP2354A)
//! - **U2**: RP2354A host MCU (RP2350 with 2 MB flash, QFN-60)
//! - **U3**: W5500 hardwired TCP/IP Ethernet controller (SPI1 ↔ RP2354A)
//!
//! ## Architecture
//!
//! ```text
//!  ┌─────────┐  SPI0 (50 MHz)  ┌──────────┐  SPI1 (33 MHz)  ┌──────┐
//!  │ HT-HC01 │←───────────────→│ RP2354A  │←───────────────→│ W5500│
//!  │  V2     │  GPIO4-7 (ctrl) │ (flash)  │  GPIO12-13      │ PHY  │
//!  │ HaLow   │                 │          │                 │      │
//!  │ 868/915 │                 │          │                 │ RJ45 │
//!  └────┬────┘                 └──────────┘                 └──┬───┘
//!       │ ANT                                                  │
//!       └──── 1-2 km RF link                                   │
//!                                                              │
//!                                                          Ethernet
//!                                                          to LAN
//! ```
//!
//! ## GPIO pin mapping (RP2354A QFN-60)
//!
//! | GPIO | Alt  | Net        | Destination         |
//! |------|------|------------|---------------------|
//! | 0    | F0   | SDIO_D0    | U1 pad 17 (MISO)    |
//! | 1    | F0   | SDIO_D3    | U1 pad 22 (CS)      |
//! | 2    | F0   | SDIO_CLK   | U1 pad 18 (SCK)     |
//! | 3    | F0   | SDIO_CMD   | U1 pad 21 (MOSI)    |
//! | 4    | SIO  | SDIO_D1    | U1 pad 16 (INT)     |
//! | 5    | SIO  | MM_RESET_N | U1 pad 35 (RESET)  |
//! | 6    | SIO  | MM_WAKE    | U1 pad 36 (WAKE)   |
//! | 7    | SIO  | MM_BUSY    | U1 pad 34 (BUSY)   |
//! | 8    | F0   | W5500_MISO | U3 pin 34 (MISO)   |
//! | 9    | F0   | W5500_SCSn | U3 pin 32 (SCSn)   |
//! | 10   | F0   | W5500_SCLK | U3 pin 33 (SCLK)   |
//! | 11   | F0   | W5500_MOSI | U3 pin 35 (MOSI)   |
//! | 12   | SIO  | W5500_RSTn | U3 pin 37 (RSTn)   |
//! | 13   | SIO  | W5500_INTn | U3 pin 36 (INTn)   |
//!
//! GPIOs 14–29 free for USB, SWD, ADC, status LEDs, future expansion.

use copperleaf::{ComponentInst, Constraint, Design, DesignExt, Net, NetClass, NetKind, SigSpec, UnitExt};
use copperleaf::parts::Crystal;

use crate::parts::{HtHc01, Rp2354a, W5500};

/// Shorthand for a signal net carrying `spec`.
macro_rules! signal_net {
    ($name:expr, $spec:expr) => {
        Net {
            name: $name.into(),
            kind: NetKind::Signal { spec: $spec },
            class: NetClass::default(),
            constraints: vec![],
        }
    };
}

/// Build the complete HT-HC01 V2 + RP2354A + W5500 bridge reference design.
pub fn build_spi_reference_design() -> Design {
    let mut d = Design::default();

    // ═══ 1. Nets ════════════════════════════════════════════════════

    // Power nets
    let vbat = Net::power("VBAT", 3.3.volt()).ripple(100.0.millivolt());
    let vdd_io = Net::power("VDD_IO", 3.3.volt()).ripple(50.0.millivolt());
    let vdd_fem = Net::power("VDD_FEM", 5.0.volt()).ripple(100.0.millivolt());
    let gnd = Net::ground();
    // W5500 analog supply (same 3.3V rail but separately decoupled)
    let avdd = Net::power("AVDD", 3.3.volt()).ripple(50.0.millivolt());

    d.add_net(vbat);
    d.add_net(vdd_io);
    d.add_net(vdd_fem);
    d.add_net(gnd);
    d.add_net(avdd);

    // SPI0 bus → HaLow module (50 MHz)
    d.add_net(signal_net!("SDIO_CLK", SigSpec::spi_clk(50.0)));
    d.add_net(signal_net!("SDIO_CMD", SigSpec::spi(50.0)));
    d.add_net(signal_net!("SDIO_D0", SigSpec::spi(50.0)));
    d.add_net(signal_net!("SDIO_D1", SigSpec::spi(50.0)));
    d.add_net(signal_net!("SDIO_D3", SigSpec::spi(50.0)));

    // SPI1 bus → W5500 (33 MHz)
    d.add_net(signal_net!("W5500_SCLK", SigSpec::spi_clk(33.0)));
    d.add_net(signal_net!("W5500_MOSI", SigSpec::spi(33.0)));
    d.add_net(signal_net!("W5500_MISO", SigSpec::spi(33.0)));
    d.add_net(signal_net!("W5500_SCSn", SigSpec::spi(33.0)));

    // W5500 control
    d.add_net(signal_net!("W5500_RSTn", SigSpec::control()));
    d.add_net(signal_net!("W5500_INTn", SigSpec::control()));

    // HaLow control signals
    d.add_net(signal_net!("MM_RESET_N", SigSpec::control()));
    d.add_net(signal_net!("MM_WAKE", SigSpec::control()));
    d.add_net(signal_net!("MM_BUSY", SigSpec::control()));

    // HaLow antenna
    d.add_net(signal_net!("ANT", SigSpec::rf_50ohm()));
    d.add_net(signal_net!("ANT_CONN", SigSpec::rf_50ohm()));

    // W5500 crystal / reference pins
    d.add_net(signal_net!("W5500_XI", SigSpec::spi_clk(25.0)));
    d.add_net(signal_net!("W5500_XO", SigSpec::control()));
    d.add_net(signal_net!("W5500_EXRES", SigSpec::control()));
    d.add_net(signal_net!("W5500_TOCAP", SigSpec::control()));
    d.add_net(signal_net!("W5500_1V2O", SigSpec::control()));

    // W5500 LED nets
    for name in [
        "W5500_SPDLED",
        "W5500_LINKLED",
        "W5500_DUPLED",
        "W5500_ACTLED",
    ] {
        d.add_net(signal_net!(name, SigSpec::control()));
    }

    // HaLow pull-down nets (unused GPIO + JTAG)
    for name in [
        "GPIO_9",
        "GPIO_8",
        "GPIO_7",
        "GPIO_6",
        "GPIO_5",
        "SDIO_D2",
        "JTAG_TCK",
        "JTAG_TDI",
        "JTAG_TMS",
        "JTAG_TRST_N",
        "JTAG_TDO",
    ] {
        d.add_net(signal_net!(name, SigSpec::control()));
    }

    // ═══ 2. Active components ════════════════════════════════════════

    d.add_component(ComponentInst::new("U1", HtHc01::new()));
    d.add_component(ComponentInst::new("U2", Rp2354a::new()));
    d.add_component(ComponentInst::new("U3", W5500::new()));

    // ═══ 3. W5500 external components ═════════════════════════════════

    // Crystal: 25 MHz (Y2) — connected between XI and XO
    d.add_component(ComponentInst::new("Y2", Crystal::new(25.0.mhz())));
    d.wire("Y2.1", "W5500_XI");
    d.wire("Y2.2", "W5500_XO");

    // EXRES1: 12.4 kΩ 1% bias resistor (R23)
    d.add_res("R23", 12.4.kohm(), "W5500_EXRES", "GND");

    // TOCAP: 4.7 µF reference capacitor (C10)
    d.add_cap("C10", 4.7.uf(), "W5500_TOCAP", "GND");

    // 1V2O: 10 nF regulator bypass capacitor (C11)
    d.add_cap("C11", 10.0.nf(), "W5500_1V2O", "GND");

    // Decoupling capacitors for W5500 VDD + AVDD
    d.add_cap("C12", 100.0.nf(), "VDD_IO", "GND");
    d.add_cap("C13", 10.0.uf(), "VDD_IO", "GND");
    d.add_cap("C14", 100.0.nf(), "AVDD", "GND");
    d.add_cap("C15", 10.0.uf(), "AVDD", "GND");

    // PMODE pull-ups (10 kΩ → auto-negotiation enabled, PMODE[2:0]=111)
    d.add_res("R24", 10.0.kohm(), "AVDD", "W5500_SPDLED");
    d.add_res("R25", 10.0.kohm(), "AVDD", "W5500_LINKLED");
    d.add_res("R26", 10.0.kohm(), "AVDD", "W5500_DUPLED");

    // ═══ 4. HaLow decoupling capacitors (from original SPI ref design) ═══

    d.add_cap("C1", 100.0.pf(), "VDD_FEM", "GND");
    d.add_cap("C2", 22.0.uf(), "VDD_FEM", "GND");
    d.add_cap("C3", 22.0.uf(), "VDD_FEM", "GND");
    d.add_cap("C4", 10.0.uf(), "VDD_FEM", "GND");
    d.add_cap("C5", 100.0.nf(), "VDD_FEM", "GND");

    d.add_cap("C6", 100.0.nf(), "VDD_IO", "GND");
    d.add_cap("C7", 10.0.uf(), "VDD_IO", "GND");
    d.add_cap("C8", 100.0.nf(), "VBAT", "GND");
    d.add_cap("C9", 10.0.uf(), "VBAT", "GND");

    // ═══ 5. HaLow SPI bus pull-up resistors ══════════════════════════

    d.add_res("R1", 10.0.kohm(), "VDD_IO", "SDIO_D3");
    d.add_res("R2", 10.0.kohm(), "VDD_IO", "SDIO_CLK");
    d.add_res("R3", 10.0.kohm(), "VDD_IO", "SDIO_CMD");
    d.add_res("R5", 10.0.kohm(), "VDD_IO", "SDIO_D1");

    // ═══ 6. HaLow pull-down resistors (JTAG + unused GPIO) ═══════════

    d.add_res("R7", 10.0.kohm(), "GPIO_9", "GND");
    d.add_res("R8", 10.0.kohm(), "GPIO_8", "GND");
    d.add_res("R9", 10.0.kohm(), "GPIO_7", "GND");
    d.add_res("R11", 10.0.kohm(), "GPIO_6", "GND");
    d.add_res("R13", 10.0.kohm(), "GPIO_5", "GND");
    d.add_res("R22", 10.0.kohm(), "SDIO_D2", "GND");
    d.add_res("R15", 10.0.kohm(), "JTAG_TCK", "GND");
    d.add_res("R16", 10.0.kohm(), "JTAG_TDI", "GND");
    d.add_res("R17", 10.0.kohm(), "JTAG_TMS", "GND");
    d.add_res("R19", 10.0.kohm(), "JTAG_TRST_N", "GND");
    d.add_res("R21", 10.0.kohm(), "JTAG_TDO", "GND");

    // ═══ 7. Antenna jumper (0 Ω) ═════════════════════════════════════

    d.add_res("R6", 0.0.ohm(), "ANT", "ANT_CONN");

    // ═══ 8. HaLow module ↔ RP2354A connections (SPI0) ════════════════

    // SPI bus
    d.connect_net("SDIO_CLK", &["U1.SDIO_CLK", "U2.GPIO2"]);
    d.connect_net("SDIO_CMD", &["U1.SDIO_CMD", "U2.GPIO3"]);
    d.connect_net("SDIO_D0", &["U1.SDIO_D0", "U2.GPIO0"]);
    d.connect_net("SDIO_D3", &["U1.SDIO_D3", "U2.GPIO1"]);
    d.connect_net("SDIO_D1", &["U1.SDIO_D1", "U2.GPIO4"]);

    // Control signals
    d.connect_net("MM_RESET_N", &["U1.RESET_N", "U2.GPIO5"]);
    d.connect_net("MM_WAKE", &["U1.WAKE", "U2.GPIO6"]);
    d.connect_net("MM_BUSY", &["U1.BUSY", "U2.GPIO7"]);

    // HaLow power
    d.wire("U1.VBAT", "VBAT");
    d.wire("U1.VDD_IO", "VDD_IO");
    d.wire("U1.VDD_FEM", "VDD_FEM");
    d.wire("U1.GND", "GND");

    // HaLow antenna
    d.wire("U1.ANT", "ANT");

    // HaLow unused-pin pull-downs
    d.wire("U1.GPIO_9", "GPIO_9");
    d.wire("U1.GPIO_8", "GPIO_8");
    d.wire("U1.GPIO_7", "GPIO_7");
    d.wire("U1.GPIO_6", "GPIO_6");
    d.wire("U1.GPIO_5", "GPIO_5");
    d.wire("U1.SDIO_D2", "SDIO_D2");
    d.wire("U1.JTAG_TCK", "JTAG_TCK");
    d.wire("U1.JTAG_TDI", "JTAG_TDI");
    d.wire("U1.JTAG_TMS", "JTAG_TMS");
    d.wire("U1.JTAG_TRST_N", "JTAG_TRST_N");
    d.wire("U1.JTAG_TDO", "JTAG_TDO");

    // Remaining unused GPIOs → GND directly
    d.wire("U1.GPIO_4", "GND");
    d.wire("U1.GPIO_3", "GND");
    d.wire("U1.GPIO_2", "GND");
    d.wire("U1.GPIO_1", "GND");

    // ═══ 9. W5500 ↔ RP2354A connections (SPI1) ═══════════════════════

    d.connect_net("W5500_SCLK", &["U3.SCLK", "U2.GPIO10"]);
    d.connect_net("W5500_MOSI", &["U3.MOSI", "U2.GPIO11"]);
    d.connect_net("W5500_MISO", &["U3.MISO", "U2.GPIO8"]);
    d.connect_net("W5500_SCSn", &["U3.SCSn", "U2.GPIO9"]);

    // Control
    d.connect_net("W5500_RSTn", &["U3.RSTn", "U2.GPIO12"]);
    d.connect_net("W5500_INTn", &["U3.INTn", "U2.GPIO13"]);

    // W5500 power
    d.wire("U3.VDD", "VDD_IO");
    d.wire("U3.GND", "GND");
    d.wire("U3.AVDD", "AVDD");
    d.wire("U3.AGND", "GND");

    // W5500 crystal
    d.wire("U3.XI", "W5500_XI");
    d.wire("U3.XO", "W5500_XO");

    // W5500 bias/reference
    d.wire("U3.EXRES1", "W5500_EXRES");
    d.wire("U3.TOCAP", "W5500_TOCAP");
    d.wire("U3.1V2O", "W5500_1V2O");
    // VBG: band-gap reference output — leave unconnected (floating) per datasheet

    // W5500 PMODE (pulled up via LED nets = auto-neg enabled)
    d.connect_net("W5500_SPDLED", &["U3.PMODE0", "U3.SPDLED"]);
    d.connect_net("W5500_LINKLED", &["U3.PMODE1", "U3.LINKLED"]);
    d.connect_net("W5500_DUPLED", &["U3.PMODE2", "U3.DUPLED"]);
    d.wire("U3.ACTLED", "W5500_ACTLED");

    // RP2354A power
    d.wire("U2.IOVDD", "VDD_IO");
    d.wire("U2.GND", "GND");

    // ═══ 10. Top-level constraints ═════════════════════════════════════

    d.add_constraint(Constraint::ResonanceIndex { max: 0.5 });
    d.add_constraint(Constraint::MaxJunction {
        temp: 85.0.celsius(),
    });
    d.add_constraint(Constraint::ReturnPath {
        requires_plane: true,
    });

    d
}

/// Run ERC and print a human-readable summary to stdout.
pub fn run_analysis(d: &Design) {
    println!("{}", copperleaf::report(d));

    // ── Free GPIO summary ───────────────────────────────────────────
    println!("── RP2354A GPIO allocation ──────────────────────────────────");
    let used_gpio: Vec<u8> = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13].to_vec();
    let free_gpio: Vec<u8> = (14..30).collect();
    println!(
        "  Used:   GPIO{} ({}/30)",
        used_gpio
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(","),
        used_gpio.len()
    );
    println!(
        "  Free:   GPIO{} ({}/30 free)",
        free_gpio
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(","),
        free_gpio.len()
    );
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use copperleaf::{Block, ComponentRecord, Limits, Net, Pin, Role};

    use super::*;

    // ── Part-level tests ─────────────────────────────────────────────

    #[test]
    fn halow_module_has_38_pins() {
        let m = HtHc01::new();
        assert_eq!(m.pins().len(), 38);
    }

    #[test]
    fn halow_module_has_three_power_inputs() {
        let m = HtHc01::new();
        let power_pins: Vec<&Pin> = m
            .pins()
            .iter()
            .filter(|p| matches!(p.role, Role::PowerIn))
            .collect();
        assert_eq!(power_pins.len(), 3);
        let names: Vec<&str> = power_pins.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"VBAT"));
        assert!(names.contains(&"VDD_IO"));
        assert!(names.contains(&"VDD_FEM"));
    }

    #[test]
    fn rp2354a_has_30_gpio_plus_power() {
        let mcu = Rp2354a::new();
        let gpio_count = mcu
            .pins()
            .iter()
            .filter(|p| p.name.starts_with("GPIO"))
            .count();
        assert_eq!(gpio_count, 30);
        assert!(mcu.pins().iter().any(|p| p.name == "IOVDD"));
        assert!(mcu.pins().iter().any(|p| p.name == "GND"));
    }

    #[test]
    fn w5500_has_spi_and_power_pins() {
        let w = W5500::new();
        let names: Vec<&str> = w.pins().iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"SCSn"));
        assert!(names.contains(&"SCLK"));
        assert!(names.contains(&"MISO"));
        assert!(names.contains(&"MOSI"));
        assert!(names.contains(&"INTn"));
        assert!(names.contains(&"RSTn"));
        assert!(names.contains(&"VDD"));
        assert!(names.contains(&"AVDD"));
        assert!(names.contains(&"XI"));
        assert!(names.contains(&"XO"));
        assert!(names.contains(&"EXRES1"));
        assert!(names.contains(&"TOCAP"));
        assert!(names.contains(&"1V2O"));
    }

    // ── Design-level tests ───────────────────────────────────────────

    #[test]
    fn design_has_three_ics() {
        let d = build_spi_reference_design();
        let ics: Vec<_> = d
            .components
            .iter()
            .filter(|c| c.refdes.starts_with('U'))
            .collect();
        assert_eq!(ics.len(), 3, "U1=HaLow, U2=RP2354A, U3=W5500");
    }

    #[test]
    fn spi0_connects_halog_and_rp2354a() {
        let d = build_spi_reference_design();
        for net_name in ["SDIO_CLK", "SDIO_CMD", "SDIO_D0", "SDIO_D3"] {
            let pins = d.pins_on_net(net_name);
            let refdes: Vec<&str> = pins.iter().map(|(r, _)| r.as_str()).collect();
            assert!(
                refdes.contains(&"U1"),
                "{} not connected to HaLow module",
                net_name
            );
            assert!(
                refdes.contains(&"U2"),
                "{} not connected to RP2354A",
                net_name
            );
        }
    }

    #[test]
    fn spi1_connects_w5500_and_rp2354a() {
        let d = build_spi_reference_design();
        for net_name in ["W5500_SCLK", "W5500_MOSI", "W5500_MISO", "W5500_SCSn"] {
            let pins = d.pins_on_net(net_name);
            let refdes: Vec<&str> = pins.iter().map(|(r, _)| r.as_str()).collect();
            assert!(
                refdes.contains(&"U3"),
                "{} not connected to W5500",
                net_name
            );
            assert!(
                refdes.contains(&"U2"),
                "{} not connected to RP2354A",
                net_name
            );
        }
    }

    #[test]
    fn w5500_control_lines_connected() {
        let d = build_spi_reference_design();
        // RSTn
        let rst_pins = d.pins_on_net("W5500_RSTn");
        assert!(rst_pins.iter().any(|(r, _)| r == "U3"));
        assert!(rst_pins.iter().any(|(r, _)| r == "U2"));
        // INTn
        let int_pins = d.pins_on_net("W5500_INTn");
        assert!(int_pins.iter().any(|(r, _)| r == "U3"));
        assert!(int_pins.iter().any(|(r, _)| r == "U2"));
    }

    #[test]
    fn w5500_has_25mhz_crystal() {
        let d = build_spi_reference_design();
        let xi_pins = d.pins_on_net("W5500_XI");
        assert!(xi_pins.iter().any(|(r, _)| r == "Y2"));
        assert!(xi_pins.iter().any(|(r, _)| r == "U3"));
    }

    #[test]
    fn w5500_has_bias_resistor() {
        let d = build_spi_reference_design();
        let exres_pins = d.pins_on_net("W5500_EXRES");
        assert!(exres_pins.iter().any(|(r, _)| r == "R23"));
        assert!(exres_pins.iter().any(|(r, _)| r == "U3"));
    }

    #[test]
    fn w5500_has_ref_caps() {
        let d = build_spi_reference_design();
        let tocap_pins = d.pins_on_net("W5500_TOCAP");
        assert!(tocap_pins.iter().any(|(r, _)| r == "C10"));
        let v1o_pins = d.pins_on_net("W5500_1V2O");
        assert!(v1o_pins.iter().any(|(r, _)| r == "C11"));
    }

    #[test]
    fn vbat_and_vdd_io_are_3v3() {
        let d = build_spi_reference_design();
        for name in &["VBAT", "VDD_IO", "AVDD"] {
            let net = d.nets.iter().find(|n| &n.name == name).unwrap();
            match &net.kind {
                NetKind::Power { v_nom, .. } => {
                    assert!(
                        (v_nom.as_base() - 3.3).abs() < 1e-9,
                        "{} must be 3.3 V",
                        name
                    );
                }
                _ => panic!("{} must be a power net", name),
            }
        }
    }

    #[test]
    fn vdd_fem_is_5v() {
        let d = build_spi_reference_design();
        let net = d.nets.iter().find(|n| n.name == "VDD_FEM").unwrap();
        match &net.kind {
            NetKind::Power { v_nom, .. } => {
                assert!(
                    (v_nom.as_base() - 5.0).abs() < 1e-9,
                    "VDD_FEM must be 5.0 V"
                );
            }
            _ => panic!("VDD_FEM must be a power net"),
        }
    }

    #[test]
    fn no_overvoltage_violations() {
        let d = build_spi_reference_design();
        let diags = copperleaf::run_erc(&d);
        let overvoltage: Vec<_> = diags.iter().filter(|d| d.code == "ERC:OVERVOLT").collect();
        assert!(
            overvoltage.is_empty(),
            "ERC overvoltage violations: {:?}",
            overvoltage
        );
    }

    #[test]
    fn json_serializes_without_error() {
        let d = build_spi_reference_design();
        let json = serde_json::to_string(&d);
        assert!(json.is_ok(), "design must serialize to JSON");
    }

    #[test]
    fn nc_pins_are_not_connected() {
        let d = build_spi_reference_design();
        let diags = copperleaf::run_erc(&d);
        let nc_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "ERC:NC_CONNECTED")
            .collect();
        assert!(
            nc_diags.is_empty(),
            "NC pin(s) must be floating: {:?}",
            nc_diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reports_nc_pin_connected() {
        let mut d = Design::default();
        d.add_net(Net::ground());
        d.components.push(ComponentRecord {
            refdes: "U1".into(),
            pins: vec![Pin {
                name: "NC".into(),
                role: Role::DigitalIO,
                limits: Limits::new(0.0.volt(), 3.6.volt(), 0.01.amp()),
                sig: None,
            }],
            constraints: vec![],
        });
        d.connect("U1", "NC", "GND");
        let diags = copperleaf::run_erc(&d);
        let nc_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "ERC:NC_CONNECTED")
            .collect();
        assert_eq!(nc_diags.len(), 1);
        assert!(nc_diags[0].message.contains("U1.NC"));
    }
}
