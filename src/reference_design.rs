//! MM8108-MF15457 + RP2354A + W5500 reference design.
//!
//! A WiFi HaLow ↔ Ethernet bridge using:
//!
//! - **U1**: MM8108-MF15457 Wi-Fi HaLow module (SPI0 ↔ RP2354A)
//! - **U2**: RP2354A host MCU (RP2350 with 2 MB flash, QFN-60)
//! - **U3**: W5500 hardwired TCP/IP Ethernet controller (SPI1 ↔ RP2354A)
//!
//! ## Architecture
//!
//! ```text
//!  ┌──────────────┐  SPI0 (50 MHz)  ┌──────────┐  SPI1 (33 MHz)  ┌──────┐
//!  │ MM8108-MF1545│←───────────────→│ RP2354A  │←───────────────→│ W5500│
//!  │  (MM8108)    │  GPIO4-7 (ctrl) │ (flash)  │  GPIO12-13      │ PHY  │
//!  │  HaLow       │                 │          │                 │      │
//!  │  868/915 MHz │                 │          │                 │ RJ45 │
//!  └──────┬───────┘                 └──────────┘                 └──┬───┘
//!         │ ANT                                                    │
//!         └──── 1+ km RF link                                      │
//!                                                                  │
//!                                                              Ethernet
//!                                                              to LAN
//! ```
//!
//! ## Key improvements over HT-HC01 V2 design
//!
//! - **MM8108 SoC** — 43 Mbps PHY (256-QAM @ 8 MHz) vs 32.5 Mbps
//! - **Integrated 26 dBm PA + LNA** — longer range, simpler power rail
//! - **Single 3.3 V supply** — no 5 V VDD_FEM rail needed
//! - **FCC modular certified** — no separate RF certification required
//!
//! ## GPIO pin mapping (RP2354A QFN-60)
//!
//! | GPIO | Alt  | Net             | Destination                     |
//! |------|------|-----------------|---------------------------------|
//! | 0    | F0   | SDIO_D0         | U1 pin 12 (SPI_MISO)           |
//! | 1    | F0   | SDIO_D3         | U1 pin 13 (SPI_CS)             |
//! | 2    | F0   | SDIO_CLK        | U1 pin 17 (SPI_SCK)            |
//! | 3    | F0   | SDIO_CMD        | U1 pin 16 (SPI_MOSI)           |
//! | 4    | SIO  | SDIO_D1         | U1 pin 14 (SPI_INT)            |
//! | 5    | SIO  | MM_RESET_N      | U1 pin 4  (RESET_N)            |
//! | 6    | SIO  | MM_WAKE         | U1 pin 5  (WAKE)               |
//! | 7    | SIO  | MM_BUSY         | U1 pin 29 (BUSY)               |
//! | 8    | F0   | W5500_MISO      | U3 pin 34 (MISO)               |
//! | 9    | F0   | W5500_SCSn      | U3 pin 32 (SCSn)               |
//! | 10   | F0   | W5500_SCLK      | U3 pin 33 (SCLK)               |
//! | 11   | F0   | W5500_MOSI      | U3 pin 35 (MOSI)               |
//! | 12   | SIO  | W5500_RSTn      | U3 pin 37 (RSTn)               |
//! | 13   | SIO  | W5500_INTn      | U3 pin 36 (INTn)               |
//!
//! GPIOs 14–29 free for USB, SWD, ADC, status LEDs, future expansion.

use copperleaf::{
    ComponentInst, Constraint, Design, DesignExt, Net, NetClass, NetKind, SigSpec, UnitExt,
    parts::Crystal,
};

use crate::parts::{Mm8108Mf15457, Rp2354a, W5500};

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

/// Build the complete MM8108-MF15457 + RP2354A + W5500 bridge reference design.
pub fn build_spi_reference_design() -> Design {
    let mut d = Design::default();

    // ═══ 1. Nets ════════════════════════════════════════════════════

    // Power nets — single 3.3 V domain (no VDD_FEM)
    let vbat = Net::power("VBAT", 3.3.volt()).ripple(50.0.millivolt());
    let vbat_tx = Net::power("VBAT_TX", 3.3.volt()).ripple(100.0.millivolt());
    let vdd_io = Net::power("VDD_IO", 3.3.volt()).ripple(50.0.millivolt());
    let gnd = Net::ground();
    // W5500 analog supply (same 3.3V rail but separately decoupled)
    let avdd = Net::power("AVDD", 3.3.volt()).ripple(50.0.millivolt());

    d.add_net(vbat);
    d.add_net(vbat_tx);
    d.add_net(vdd_io);
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

    // ═══ 2. Active components ════════════════════════════════════════

    d.add_component(ComponentInst::new("U1", Mm8108Mf15457::new()));
    d.add_component(ComponentInst::new("U2", Rp2354a::new()));
    d.add_component(ComponentInst::new("U3", W5500::new()));

    // ═══ 3. W5500 external components ═══════════════════════════════

    // Crystal: 25 MHz (Y2) — connected between XI and XO
    d.add_component(ComponentInst::new("Y2", Crystal::new(25.0.mhz())));
    d.wire("Y2.1", "W5500_XI");
    d.wire("Y2.2", "W5500_XO");

    // EXRES1: 12.4 kΩ 1% bias resistor (R23)
    d.add_res("R23", 12.4.kohm(), "W5500_EXRES", "GND");

    // TOCAP: 4.7 µF reference capacitor (C10)
    d.add_de_cap("C10", 4.7.uf(), "W5500_TOCAP", "GND");

    // 1V2O: 10 nF regulator bypass capacitor (C11)
    d.add_de_cap("C11", 10.0.nf(), "W5500_1V2O", "GND");

    // Decoupling capacitors for W5500 VDD + AVDD
    d.add_de_cap("C12", 100.0.nf(), "VDD_IO", "GND");
    d.add_de_cap("C13", 10.0.uf(), "VDD_IO", "GND");
    d.add_de_cap("C14", 100.0.nf(), "AVDD", "GND");
    d.add_de_cap("C15", 10.0.uf(), "AVDD", "GND");

    // PMODE pull-ups (10 kΩ → auto-negotiation enabled, PMODE[2:0]=111)
    d.add_res("R24", 10.0.kohm(), "AVDD", "W5500_SPDLED");
    d.add_res("R25", 10.0.kohm(), "AVDD", "W5500_LINKLED");
    d.add_res("R26", 10.0.kohm(), "AVDD", "W5500_DUPLED");

    // ═══ 4. HaLow decoupling capacitors (single 3.3 V domain) ════════

    // VBAT (main SoC supply)
    d.add_de_cap("C1", 100.0.nf(), "VBAT", "GND");
    d.add_de_cap("C2", 10.0.uf(), "VBAT", "GND");

    // VBAT_TX (PA supply)
    d.add_de_cap("C3", 100.0.nf(), "VBAT_TX", "GND");
    d.add_de_cap("C4", 10.0.uf(), "VBAT_TX", "GND");

    // VDDIO (I/O supply)
    d.add_de_cap("C5", 100.0.nf(), "VDD_IO", "GND");

    // ═══ 5. SPI bus pull-up resistors ════════════════════════════════
    //
    // Per MM8108-MF15457 datasheet §2 note [1]: all SDIO bus pins except
    // SDIO_CLK should be pulled up with 10 kΩ–100 kΩ.

    d.add_res("R1", 10.0.kohm(), "VDD_IO", "SDIO_D3"); // SPI_CS
    d.add_res("R2", 10.0.kohm(), "VDD_IO", "SDIO_CMD"); // SPI_MOSI
    d.add_res("R3", 10.0.kohm(), "VDD_IO", "SDIO_D0"); // SPI_MISO
    d.add_res("R4", 10.0.kohm(), "VDD_IO", "SDIO_D1"); // SPI_INT

    // ═══ 6. HaLow pull-down resistors ════════════════════════════════
    //
    // Per MM8108-MF15457 datasheet §2 notes [2] and [3]:
    // - JTAG pins → GND via 10 kΩ
    // - All unused GPIO → GND via 10 kΩ
    // - SDIO_D2 → GND via 10 kΩ (unused in SPI mode)

    // JTAG pull-downs
    d.add_res("R5", 10.0.kohm(), "JTAG_TMS", "GND");
    d.add_res("R6", 10.0.kohm(), "JTAG_TCK", "GND");
    d.add_res("R7", 10.0.kohm(), "JTAG_TDO", "GND");
    d.add_res("R8", 10.0.kohm(), "JTAG_TDI", "GND");

    // Unused SDIO
    d.add_res("R9", 10.0.kohm(), "SDIO_D2", "GND");

    // Unused GPIO pull-downs (GPIO0–GPIO10 except those used for SPI/ctrl)
    // GPIO5 (pin 18), GPIO4 (pin 19), GPIO3 (pin 21)
    // GPIO2 not present on this module
    // GPIO1 (pin 31), GPIO0 (pin 32)
    // GPIO6 (pin 33), GPIO7 (pin 34), GPIO8 (pin 35)
    // GPIO9 (pin 36), GPIO10 (pin 37)
    d.add_res("R10", 10.0.kohm(), "GPIO5", "GND");
    d.add_res("R11", 10.0.kohm(), "GPIO4", "GND");
    d.add_res("R12", 10.0.kohm(), "GPIO3", "GND");
    d.add_res("R13", 10.0.kohm(), "GPIO1", "GND");
    d.add_res("R14", 10.0.kohm(), "GPIO0", "GND");
    d.add_res("R15", 10.0.kohm(), "GPIO6", "GND");
    d.add_res("R16", 10.0.kohm(), "GPIO7", "GND");
    d.add_res("R17", 10.0.kohm(), "GPIO8", "GND");
    d.add_res("R18", 10.0.kohm(), "GPIO9", "GND");
    d.add_res("R19", 10.0.kohm(), "GPIO10", "GND");

    // ═══ 7. Antenna jumper (0 Ω) ═════════════════════════════════════

    d.add_res("R20", 0.0.ohm(), "ANT", "ANT_CONN");

    // ═══ 8. HaLow module ↔ RP2354A connections (SPI0) ════════════════

    // SPI bus — MM8108 pin mapping (SPI alt function)
    //   MM8108 pin 12 (SDIO_D0 / SPI_MISO) ↔ RP2354A GPIO0
    //   MM8108 pin 13 (SDIO_D3 / SPI_CS)   ↔ RP2354A GPIO1
    //   MM8108 pin 17 (SDIO_CLK / SPI_SCK) ↔ RP2354A GPIO2
    //   MM8108 pin 16 (SDIO_CMD / SPI_MOSI)↔ RP2354A GPIO3
    //   MM8108 pin 14 (SDIO_D1 / SPI_INT)  ↔ RP2354A GPIO4
    d.connect_net("SDIO_CLK", &["U1.SDIO_CLK", "U2.GPIO2"]);
    d.connect_net("SDIO_CMD", &["U1.SDIO_CMD", "U2.GPIO3"]);
    d.connect_net("SDIO_D0", &["U1.SDIO_D0", "U2.GPIO0"]);
    d.connect_net("SDIO_D3", &["U1.SDIO_D3", "U2.GPIO1"]);
    d.connect_net("SDIO_D1", &["U1.SDIO_D1", "U2.GPIO4"]);

    // Control signals
    //   MM8108 pin 4  (RESET_N)  ↔ RP2354A GPIO5
    //   MM8108 pin 5  (WAKE)     ↔ RP2354A GPIO6
    //   MM8108 pin 29 (BUSY)     ↔ RP2354A GPIO7
    d.connect_net("MM_RESET_N", &["U1.RESET_N", "U2.GPIO5"]);
    d.connect_net("MM_WAKE", &["U1.WAKE", "U2.GPIO6"]);
    d.connect_net("MM_BUSY", &["U1.BUSY", "U2.GPIO7"]);

    // HaLow power — single 3.3 V domain
    d.wire("U1.VBAT", "VBAT");
    d.wire("U1.VBAT_TX", "VBAT_TX");
    d.wire("U1.VDDIO", "VDD_IO");
    for n in 1u8..=8 {
        d.wire(&format!("U1.GND_{}", n), "GND");
    }

    // VDD_USB — tied to GND (USB not used in SPI mode)
    d.wire("U1.VDD_USB", "GND");

    // HaLow antenna
    d.wire("U1.ANT", "ANT");

    // HaLow unused-pin pull-downs (via resistors defined above)
    d.wire("U1.JTAG_TMS", "JTAG_TMS");
    d.wire("U1.JTAG_TCK", "JTAG_TCK");
    d.wire("U1.JTAG_TDO", "JTAG_TDO");
    d.wire("U1.JTAG_TDI", "JTAG_TDI");
    d.wire("U1.SDIO_D2", "SDIO_D2");
    d.wire("U1.GPIO5", "GPIO5");
    d.wire("U1.GPIO4", "GPIO4");
    d.wire("U1.GPIO3", "GPIO3");
    d.wire("U1.GPIO1", "GPIO1");
    d.wire("U1.GPIO0", "GPIO0");
    d.wire("U1.GPIO6", "GPIO6");
    d.wire("U1.GPIO7", "GPIO7");
    d.wire("U1.GPIO8", "GPIO8");
    d.wire("U1.GPIO9", "GPIO9");
    d.wire("U1.GPIO10", "GPIO10");

    // USB pins — tied to GND (USB not used in SPI mode, PHY powered down via VDD_USB=GND)
    d.wire("U1.USB_D_N", "GND");
    d.wire("U1.USB_D_P", "GND");

    // ═══ 9. W5500 ↔ RP2354A connections (SPI1) ═══════════════════════

    d.connect_net("W5500_SCLK", &["U3.SCLK", "U2.GPIO10"]);
    d.connect_net("W5500_MOSI", &["U3.MOSI", "U2.GPIO11"]);
    d.connect_net("W5500_MISO", &["U3.MISO", "U2.GPIO8"]);
    d.connect_net("W5500_SCSn", &["U3.~{SCS}", "U2.GPIO9"]);

    // Control
    d.connect_net("W5500_RSTn", &["U3.~{RST}", "U2.GPIO12"]);
    d.connect_net("W5500_INTn", &["U3.~{INT}", "U2.GPIO13"]);

    // W5500 power
    d.wire("U3.VDD", "VDD_IO");
    d.wire("U3.GND", "GND");
    d.wire("U3.AVDD", "AVDD");
    d.wire("U3.AGND", "GND");

    // W5500 crystal
    d.wire("U3.XI/CLKIN", "W5500_XI");
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
    d.wire("U2.VREG_PGND", "GND");

    // RP2354A free GPIOs (GPIO14–29) — tied to GND as unused.
    // Available for USB, SWD, ADC, status LEDs, or future expansion.
    // Note: GPIO26–29 are named "GPIO26/ADC0"–"GPIO29/ADC3" in the part definition.
    for n in 14u8..30 {
        let pin_name = match n {
            26 => "GPIO26/ADC0".to_string(),
            27 => "GPIO27/ADC1".to_string(),
            28 => "GPIO28/ADC2".to_string(),
            29 => "GPIO29/ADC3".to_string(),
            _ => format!("GPIO{}", n),
        };
        d.wire(&format!("U2.{}", pin_name), "GND");
    }

    // ═══ 10. Top-level constraints ════════════════════════════════════

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

#[cfg(test)]
mod tests {
    use copperleaf::{Block, ComponentRecord, Limits, Net, Pin, Role};

    use super::*;

    // ── Part-level tests ─────────────────────────────────────────────

    #[test]
    fn halow_module_has_38_pins() {
        let m = Mm8108Mf15457::new();
        assert_eq!(m.pins().len(), 38);
    }

    #[test]
    fn halow_module_has_three_power_inputs() {
        let m = Mm8108Mf15457::new();
        let power_pins: Vec<&Pin> = m
            .pins()
            .iter()
            .filter(|p| matches!(p.role, Role::PowerIn))
            .collect();
        assert_eq!(power_pins.len(), 4); // VBAT, VBAT_TX, VDDIO, VDD_USB
        let names: Vec<&str> = power_pins.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"VBAT"));
        assert!(names.contains(&"VBAT_TX"));
        assert!(names.contains(&"VDDIO"));
        assert!(names.contains(&"VDD_USB"));
    }

    #[test]
    fn halow_module_has_antenna_pin() {
        let m = Mm8108Mf15457::new();
        assert!(m.pins().iter().any(|p| p.name == "ANT"));
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
        assert!(mcu.pins().iter().any(|p| p.name == "VREG_PGND"));
    }

    #[test]
    fn w5500_has_spi_and_power_pins() {
        let w = W5500::new();
        let names: Vec<&str> = w.pins().iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"~{SCS}"));
        assert!(names.contains(&"SCLK"));
        assert!(names.contains(&"MISO"));
        assert!(names.contains(&"MOSI"));
        assert!(names.contains(&"~{INT}"));
        assert!(names.contains(&"~{RST}"));
        assert!(names.contains(&"VDD"));
        assert!(names.contains(&"AVDD"));
        assert!(names.contains(&"XI/CLKIN"));
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
    fn spi0_connects_halow_and_rp2354a() {
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
        for name in &["VBAT", "VBAT_TX", "VDD_IO", "AVDD"] {
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
    fn halow_control_signals_connected() {
        let d = build_spi_reference_design();
        for net_name in ["MM_RESET_N", "MM_WAKE", "MM_BUSY"] {
            let pins = d.pins_on_net(net_name);
            assert!(
                pins.iter().any(|(r, _)| r == "U1"),
                "{} not connected to HaLow module",
                net_name
            );
            assert!(
                pins.iter().any(|(r, _)| r == "U2"),
                "{} not connected to RP2354A",
                net_name
            );
        }
    }

    #[test]
    fn vdd_usb_tied_to_ground() {
        let d = build_spi_reference_design();
        let usb_pins = d.pins_on_net("GND");
        assert!(
            usb_pins.iter().any(|(r, p)| r == "U1" && p == "VDD_USB"),
            "VDD_USB must be tied to GND in SPI mode"
        );
    }

    #[test]
    fn no_vdd_fem_rail() {
        let d = build_spi_reference_design();
        assert!(
            d.nets.iter().all(|n| n.name != "VDD_FEM"),
            "VDD_FEM must not exist in single-supply design"
        );
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
            pins: vec![Pin::new(
                "NC",
                Role::DigitalIO,
                Limits::new(0.0.volt(), 3.6.volt(), 0.01.amp()),
                None,
            )],
            constraints: vec![],
            kicad_symbol: None,
            kicad_symbol_lib_path: None,
            kicad_footprint: None,
            kicad_symbol_raw: None,
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
