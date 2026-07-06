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

use copperleaf::{
    ComponentInst, Constraint, Design, Diagnostic, Net, NetClass, NetKind, Severity, SigKind,
    SigSpec, UnitExt,
    erc_voltage_pin_to_net, synthesize_decoupling,
};

use crate::parts::{Capacitor, Crystal, HtHc01, Resistor, Rp2354a, W5500};

// ── Net helpers ───────────────────────────────────────────────────────

fn sig_net(name: &str, spec: SigSpec) -> Net {
    Net {
        name: name.into(),
        kind: NetKind::Signal { spec },
        class: NetClass::default(),
        constraints: vec![],
    }
}

fn spi_spec() -> SigSpec {
    SigSpec {
        kind: SigKind::Generic,
        bandwidth: Some(50.0.mhz()),
        edge_rate: None,
        target_impedance: Some(50.0.ohm()),
    }
}

fn spi_clk_spec() -> SigSpec {
    SigSpec {
        kind: SigKind::Clock,
        bandwidth: Some(50.0.mhz()),
        edge_rate: None,
        target_impedance: Some(50.0.ohm()),
    }
}

fn spi1_spec() -> SigSpec {
    SigSpec {
        kind: SigKind::Generic,
        bandwidth: Some(33.0.mhz()),
        edge_rate: None,
        target_impedance: Some(50.0.ohm()),
    }
}

fn spi1_clk_spec() -> SigSpec {
    SigSpec {
        kind: SigKind::Clock,
        bandwidth: Some(33.0.mhz()),
        edge_rate: None,
        target_impedance: Some(50.0.ohm()),
    }
}

fn ctrl_spec() -> SigSpec {
    SigSpec {
        kind: SigKind::Generic,
        bandwidth: None,
        edge_rate: None,
        target_impedance: None,
    }
}

fn rf_spec() -> SigSpec {
    SigSpec {
        kind: SigKind::AnalogLowNoise,
        bandwidth: None,
        edge_rate: None,
        target_impedance: Some(50.0.ohm()),
    }
}

// ── Design builder ───────────────────────────────────────────────────

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
    d.add_net(sig_net("SDIO_CLK", spi_clk_spec())); // SCK
    d.add_net(sig_net("SDIO_CMD", spi_spec())); // MOSI
    d.add_net(sig_net("SDIO_D0", spi_spec())); // MISO
    d.add_net(sig_net("SDIO_D1", spi_spec())); // INT
    d.add_net(sig_net("SDIO_D3", spi_spec())); // CS

    // SPI1 bus → W5500 (33 MHz)
    d.add_net(sig_net("W5500_SCLK", spi1_clk_spec()));
    d.add_net(sig_net("W5500_MOSI", spi1_spec()));
    d.add_net(sig_net("W5500_MISO", spi1_spec()));
    d.add_net(sig_net("W5500_SCSn", spi1_spec()));

    // W5500 control
    d.add_net(sig_net("W5500_RSTn", ctrl_spec()));
    d.add_net(sig_net("W5500_INTn", ctrl_spec()));

    // HaLow control signals
    d.add_net(sig_net("MM_RESET_N", ctrl_spec()));
    d.add_net(sig_net("MM_WAKE", ctrl_spec()));
    d.add_net(sig_net("MM_BUSY", ctrl_spec()));

    // HaLow antenna
    d.add_net(sig_net("ANT", rf_spec()));
    d.add_net(sig_net("ANT_CONN", rf_spec()));

    // W5500 crystal / reference pins (need their own nets for the external components)
    d.add_net(sig_net(
        "W5500_XI",
        SigSpec {
            kind: SigKind::Clock,
            bandwidth: Some(25.0.mhz()),
            edge_rate: None,
            target_impedance: None,
        },
    ));
    d.add_net(sig_net("W5500_XO", ctrl_spec()));
    d.add_net(sig_net("W5500_EXRES", ctrl_spec()));
    d.add_net(sig_net("W5500_TOCAP", ctrl_spec()));
    d.add_net(sig_net("W5500_1V2O", ctrl_spec()));

    // W5500 LED nets
    for name in [
        "W5500_SPDLED",
        "W5500_LINKLED",
        "W5500_DUPLED",
        "W5500_ACTLED",
    ] {
        d.add_net(sig_net(name, ctrl_spec()));
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
        d.add_net(sig_net(name, ctrl_spec()));
    }

    // ═══ 2. Active components ════════════════════════════════════════

    let halow = HtHc01::new("HT-HC01_V2");
    let u1 = ComponentInst::new("U1", halow);
    d.add_component(&u1);

    let mcu = Rp2354a::new("RP2354A");
    let u2 = ComponentInst::new("U2", mcu);
    d.add_component(&u2);

    let eth = W5500::new("W5500");
    let u3 = ComponentInst::new("U3", eth);
    d.add_component(&u3);

    // ═══ 3. W5500 external components ═════════════════════════════════

    // Crystal: 25 MHz (Y2) — connected between XI and XO
    let y2 = Crystal::new("Y2", 25.0.mhz());
    let y2_inst = ComponentInst::new("Y2", y2);
    d.add_component(&y2_inst);
    d.connect("Y2", "1", "W5500_XI");
    d.connect("Y2", "2", "W5500_XO");

    // EXRES1: 12.4 kΩ 1% bias resistor (R23)
    let r23 = Resistor::new("R23", 12.4.kohm());
    let r23_inst = ComponentInst::new("R23", r23);
    d.add_component(&r23_inst);
    d.connect("R23", "1", "W5500_EXRES");
    d.connect("R23", "2", "GND");

    // TOCAP: 4.7 µF reference capacitor (C10)
    let c10 = Capacitor::new("C10", 4.7.uf());
    let c10_inst = ComponentInst::new("C10", c10);
    d.add_component(&c10_inst);
    d.connect("C10", "1", "W5500_TOCAP");
    d.connect("C10", "2", "GND");

    // 1V2O: 10 nF regulator bypass capacitor (C11)
    let c11 = Capacitor::new("C11", 10.0.nf());
    let c11_inst = ComponentInst::new("C11", c11);
    d.add_component(&c11_inst);
    d.connect("C11", "1", "W5500_1V2O");
    d.connect("C11", "2", "GND");

    // Decoupling capacitors for W5500 VDD + AVDD
    // VDD: 100 nF + 10 µF, AVDD: 100 nF + 10 µF
    let w5500_decoupling = [
        ("C12", 100.0.nf(), "VDD_IO"), // W5500 VDD (ties to same 3.3V as VDD_IO net)
        ("C13", 10.0.uf(), "VDD_IO"),
        ("C14", 100.0.nf(), "AVDD"),
        ("C15", 10.0.uf(), "AVDD"),
    ];
    for (refdes, value, rail) in w5500_decoupling {
        let c = Capacitor::new(refdes, value);
        let inst = ComponentInst::new(refdes, c);
        d.add_component(&inst);
        d.connect(refdes, "1", rail);
        d.connect(refdes, "2", "GND");
    }

    // PMODE pull-ups (10 kΩ → auto-negotiation enabled, PMODE[2:0]=111)
    for (refdes, net_name) in [
        ("R24", "W5500_SPDLED"),  // PMODE0 uses SPDLED pin; pulled up
        ("R25", "W5500_LINKLED"), // PMODE1 uses LINKLED pin
        ("R26", "W5500_DUPLED"),  // PMODE2 uses DUPLED pin
    ] {
        let r = Resistor::new(refdes, 10.0.kohm());
        let inst = ComponentInst::new(refdes, r);
        d.add_component(&inst);
        d.connect(refdes, "1", "AVDD");
        d.connect(refdes, "2", net_name);
    }

    // ═══ 4. HaLow decoupling capacitors (from original SPI ref design) ═══

    let halow_caps = [
        ("C1", 100.0.pf(), "VDD_FEM"),
        ("C2", 22.0.uf(), "VDD_FEM"),
        ("C3", 22.0.uf(), "VDD_FEM"),
        ("C4", 10.0.uf(), "VDD_FEM"),
        ("C5", 100.0.nf(), "VDD_FEM"),
        ("C6", 100.0.nf(), "VDD_IO"),
        ("C7", 10.0.uf(), "VDD_IO"),
        ("C8", 100.0.nf(), "VBAT"),
        ("C9", 10.0.uf(), "VBAT"),
    ];

    for (refdes, value, rail) in halow_caps {
        let c = Capacitor::new(refdes, value);
        let inst = ComponentInst::new(refdes, c);
        d.add_component(&inst);
        d.connect(refdes, "1", rail);
        d.connect(refdes, "2", "GND");
    }

    // ═══ 5. HaLow SPI bus pull-up resistors ══════════════════════════

    let pullups = [
        ("R1", 10.0.kohm(), "SDIO_D3"),
        ("R2", 10.0.kohm(), "SDIO_CLK"),
        ("R3", 10.0.kohm(), "SDIO_CMD"),
        ("R5", 10.0.kohm(), "SDIO_D1"),
    ];

    for (refdes, value, signal_net) in pullups {
        let r = Resistor::new(refdes, value);
        let inst = ComponentInst::new(refdes, r);
        d.add_component(&inst);
        d.connect(refdes, "1", "VDD_IO");
        d.connect(refdes, "2", signal_net);
    }

    // ═══ 6. HaLow pull-down resistors (JTAG + unused GPIO) ═══════════

    let pulldowns: [(&str, &str); 11] = [
        ("R7", "GPIO_9"),
        ("R8", "GPIO_8"),
        ("R9", "GPIO_7"),
        ("R11", "GPIO_6"),
        ("R13", "GPIO_5"),
        ("R22", "SDIO_D2"),
        ("R15", "JTAG_TCK"),
        ("R16", "JTAG_TDI"),
        ("R17", "JTAG_TMS"),
        ("R19", "JTAG_TRST_N"),
        ("R21", "JTAG_TDO"),
    ];

    for (refdes, signal_net) in pulldowns {
        let r = Resistor::new(refdes, 10.0.kohm());
        let inst = ComponentInst::new(refdes, r);
        d.add_component(&inst);
        d.connect(refdes, "1", signal_net);
        d.connect(refdes, "2", "GND");
    }

    // ═══ 7. Antenna jumper (0 Ω) ═════════════════════════════════════

    let r6 = Resistor::new("R6", 0.0.ohm());
    let r6_inst = ComponentInst::new("R6", r6);
    d.add_component(&r6_inst);
    d.connect("R6", "1", "ANT");
    d.connect("R6", "2", "ANT_CONN");

    // ═══ 8. HaLow module ↔ RP2354A connections (SPI0) ════════════════

    // SPI bus
    d.connect("U1", "SDIO_CLK", "SDIO_CLK");
    d.connect("U2", "GPIO2", "SDIO_CLK"); // F0: SPI0 SCK

    d.connect("U1", "SDIO_CMD", "SDIO_CMD");
    d.connect("U2", "GPIO3", "SDIO_CMD"); // F0: SPI0 TX (MOSI)

    d.connect("U1", "SDIO_D0", "SDIO_D0");
    d.connect("U2", "GPIO0", "SDIO_D0"); // F0: SPI0 RX (MISO)

    d.connect("U1", "SDIO_D3", "SDIO_D3");
    d.connect("U2", "GPIO1", "SDIO_D3"); // F0: SPI0 CSn

    d.connect("U1", "SDIO_D1", "SDIO_D1");
    d.connect("U2", "GPIO4", "SDIO_D1"); // SIO: level IRQ (SPI_INT)

    // Control signals
    d.connect("U1", "RESET_N", "MM_RESET_N");
    d.connect("U2", "GPIO5", "MM_RESET_N");

    d.connect("U1", "WAKE", "MM_WAKE");
    d.connect("U2", "GPIO6", "MM_WAKE");

    d.connect("U1", "BUSY", "MM_BUSY");
    d.connect("U2", "GPIO7", "MM_BUSY");

    // HaLow power
    d.connect("U1", "VBAT", "VBAT");
    d.connect("U1", "VDD_IO", "VDD_IO");
    d.connect("U1", "VDD_FEM", "VDD_FEM");
    d.connect("U1", "GND", "GND");

    // HaLow antenna
    d.connect("U1", "ANT", "ANT");

    // HaLow unused-pin pull-downs
    for (pin_name, net_name) in [
        ("GPIO_9", "GPIO_9"),
        ("GPIO_8", "GPIO_8"),
        ("GPIO_7", "GPIO_7"),
        ("GPIO_6", "GPIO_6"),
        ("GPIO_5", "GPIO_5"),
        ("SDIO_D2", "SDIO_D2"),
        ("JTAG_TCK", "JTAG_TCK"),
        ("JTAG_TDI", "JTAG_TDI"),
        ("JTAG_TMS", "JTAG_TMS"),
        ("JTAG_TRST_N", "JTAG_TRST_N"),
        ("JTAG_TDO", "JTAG_TDO"),
    ] {
        d.connect("U1", pin_name, net_name);
    }
    // Remaining unused GPIOs → GND directly
    d.connect("U1", "GPIO_4", "GND");
    d.connect("U1", "GPIO_3", "GND");
    d.connect("U1", "GPIO_2", "GND");
    d.connect("U1", "GPIO_1", "GND");

    // ═══ 9. W5500 ↔ RP2354A connections (SPI1) ═══════════════════════

    d.connect("U3", "SCLK", "W5500_SCLK");
    d.connect("U2", "GPIO10", "W5500_SCLK"); // F0: SPI1 SCK

    d.connect("U3", "MOSI", "W5500_MOSI");
    d.connect("U2", "GPIO11", "W5500_MOSI"); // F0: SPI1 TX

    d.connect("U3", "MISO", "W5500_MISO");
    d.connect("U2", "GPIO8", "W5500_MISO"); // F0: SPI1 RX

    d.connect("U3", "SCSn", "W5500_SCSn");
    d.connect("U2", "GPIO9", "W5500_SCSn"); // F0: SPI1 CSn

    // Control
    d.connect("U3", "RSTn", "W5500_RSTn");
    d.connect("U2", "GPIO12", "W5500_RSTn");

    d.connect("U3", "INTn", "W5500_INTn");
    d.connect("U2", "GPIO13", "W5500_INTn");

    // W5500 power
    d.connect("U3", "VDD", "VDD_IO"); // digital 3.3V (same rail)
    d.connect("U3", "GND", "GND");
    d.connect("U3", "AVDD", "AVDD"); // analog 3.3V
    d.connect("U3", "AGND", "GND"); // analog ground ties to GND

    // W5500 crystal
    d.connect("U3", "XI", "W5500_XI");
    d.connect("U3", "XO", "W5500_XO");

    // W5500 bias/reference
    d.connect("U3", "EXRES1", "W5500_EXRES");
    d.connect("U3", "TOCAP", "W5500_TOCAP");
    d.connect("U3", "1V2O", "W5500_1V2O");
    // VBG: band-gap reference output — leave unconnected (floating) per datasheet

    // W5500 PMODE (pulled up via LED nets = auto-neg enabled)
    d.connect("U3", "PMODE0", "W5500_SPDLED");
    d.connect("U3", "PMODE1", "W5500_LINKLED");
    d.connect("U3", "PMODE2", "W5500_DUPLED");

    // W5500 LEDs
    d.connect("U3", "SPDLED", "W5500_SPDLED");
    d.connect("U3", "LINKLED", "W5500_LINKLED");
    d.connect("U3", "DUPLED", "W5500_DUPLED");
    d.connect("U3", "ACTLED", "W5500_ACTLED");

    // RP2354A power
    d.connect("U2", "IOVDD", "VDD_IO"); // MCU IO supply = 3.3V (same as HaLow VDD_IO)
    d.connect("U2", "GND", "GND");

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

// ── Analysis & reporting ─────────────────────────────────────────────

/// ERC: flag any pin named "NC" that is connected to a net.
///
/// Pins named "NC" (no-connect) are defined in the part datasheet as
/// "do not connect externally". A connection indicates a wiring error.
pub fn erc_nc_pin_connected(d: &Design) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for component in &d.components {
        for pin in &component.pins {
            if pin.name == "NC" || pin.name.starts_with("NC_") {
                let nets = d.nets_of_pin(&component.refdes, &pin.name);
                if !nets.is_empty() {
                    diags.push(Diagnostic {
                        code: "ERC:NC_CONNECTED".into(),
                        severity: Severity::Error,
                        message: format!(
                            "NC pin {}.{} is connected to net(s): {:?}",
                            component.refdes, pin.name, nets
                        ),
                        entities: vec![format!("{}.{}", component.refdes, pin.name)],
                        hint: Some("Remove the connection — NC pins must float".into()),
                    });
                }
            }
        }
    }
    diags
}

/// Run ERC and decoupling synthesis on the reference design and print
/// a human-readable summary to stdout.
pub fn run_analysis(d: &Design) {
    let (nodes, edges) = d.graph.counts();
    println!("════════════════════════════════════════════════════════════");
    println!(" HT-HC01 V2 + RP2354A + W5500 — HaLow/Ethernet Bridge");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!(" Design graph:  {} nodes, {} edges", nodes, edges);
    println!(" Components:    {}", d.components.len());
    println!(" Nets:          {}", d.nets.len());
    println!(" Constraints:   {}", d.constraints.len());
    println!();

    // ── Component list ──────────────────────────────────────────────
    println!("── Components ──────────────────────────────────────────────────");
    for c in &d.components {
        if c.refdes.starts_with('U') {
            println!("  {}  [IC]      {} pins", c.refdes, c.pins.len());
        } else if c.refdes.starts_with('C') {
            println!("  {}  [Cap]", c.refdes);
        } else if c.refdes.starts_with('R') {
            println!("  {}  [Res]", c.refdes);
        } else if c.refdes.starts_with('Y') {
            println!("  {}  [XTAL]", c.refdes);
        } else {
            println!("  {}  [???]     {} pins", c.refdes, c.pins.len());
        }
    }
    println!();

    // ── Net summary ────────────────────────────────────────────────
    println!("── Power & signal nets ────────────────────────────────────────");
    for n in &d.nets {
        let pins = d.pins_on_net(&n.name);
        match &n.kind {
            NetKind::Power { v_nom, ripple } => {
                let rip = ripple
                    .map(|r| format!(" ripple {:.0}mV", r.as_base() * 1000.0))
                    .unwrap_or_default();
                println!(
                    "  {:14}  POWER  {:.1}V{}  [{} pins]",
                    n.name,
                    v_nom.as_base(),
                    rip,
                    pins.len()
                );
            }
            NetKind::Signal { spec } => {
                let bw = spec
                    .bandwidth
                    .map(|b| {
                        let mhz = 1.0 / b.as_base() / 1.0e6;
                        format!(" {:.0}MHz", mhz)
                    })
                    .unwrap_or_default();
                let z = spec
                    .target_impedance
                    .map(|z| format!(" {:.0}Ω", z.as_base()))
                    .unwrap_or_default();
                println!("  {:14}  SIG{}{}  [{} pins]", n.name, bw, z, pins.len());
            }
        }
    }
    println!();

    // ── SPI bus connectivity ────────────────────────────────────────
    println!("── SPI0 bus (HaLow, 50 MHz) ─────────────────────────────────────");
    for net_name in ["SDIO_CLK", "SDIO_CMD", "SDIO_D0", "SDIO_D3", "SDIO_D1"] {
        let pins = d.pins_on_net(net_name);
        let pin_strs: Vec<String> = pins.iter().map(|(r, p)| format!("{}.{}", r, p)).collect();
        println!("  {:10} : {}", net_name, pin_strs.join(", "));
    }
    println!();

    println!("── SPI1 bus (W5500, 33 MHz) ────────────────────────────────────");
    for net_name in ["W5500_SCLK", "W5500_MOSI", "W5500_MISO", "W5500_SCSn"] {
        let pins = d.pins_on_net(net_name);
        let pin_strs: Vec<String> = pins.iter().map(|(r, p)| format!("{}.{}", r, p)).collect();
        println!("  {:10} : {}", net_name, pin_strs.join(", "));
    }
    println!();

    // ── ERC: overvoltage checks ─────────────────────────────────────
    println!("── ERC: overvoltage checks ──────────────────────────────────");
    let mut erc_count = 0;
    for c in &d.components {
        for pin in &c.pins {
            let nets = d.nets_of_pin(&c.refdes, &pin.name);
            for net_name in &nets {
                if let Some(net) = d.nets.iter().find(|n| &n.name == net_name)
                    && let Some(diag) = erc_voltage_pin_to_net(net, pin)
                {
                    println!(
                        "  [{:?}] {} — {} ({})",
                        diag.severity, diag.code, diag.message, c.refdes
                    );
                    erc_count += 1;
                }
            }
        }
    }
    if erc_count == 0 {
        println!("  No overvoltage violations detected.");
    }
    println!();

    // ── ERC: NC-pin checks ─────────────────────────────────────────
    println!("── ERC: NC-pin checks ────────────────────────────────────────");
    let nc_diags = erc_nc_pin_connected(d);
    if nc_diags.is_empty() {
        println!("  No NC-pin connection violations detected.");
    } else {
        for diag in &nc_diags {
            println!(
                "  [{:?}] {} — {}",
                diag.severity, diag.code, diag.message
            );
        }
    }
    println!();

    // ── Decoupling synthesis ────────────────────────────────────────
    println!("── Decoupling synthesis ────────────────────────────────────");
    let result = synthesize_decoupling(d);
    if result.caps.is_empty() {
        println!("  No decoupling caps synthesized.");
    } else {
        for cap in &result.caps {
            println!(
                "  {}  {:.3} µF  on {}  (from {}.{})",
                cap.refdes,
                cap.value.as_base() * 1.0e6,
                cap.net,
                cap.source_component,
                cap.source_pin
            );
        }
    }
    for diag in &result.diagnostics {
        if diag.code != "DECOUPLE:SUMMARY" {
            println!("  [{:?}] {} — {}", diag.severity, diag.code, diag.message);
        }
    }
    println!();

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
    use copperleaf::{Block, ComponentRecord, Limits, Pin, Role};

    use super::*;

    // ── Part-level tests ─────────────────────────────────────────────

    #[test]
    fn halow_module_has_38_pins() {
        let m = HtHc01::new("HT-HC01_V2");
        assert_eq!(m.pins().len(), 38);
    }

    #[test]
    fn halow_module_has_three_power_inputs() {
        let m = HtHc01::new("HT-HC01_V2");
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
        let mcu = Rp2354a::new("RP2354A");
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
        let w = W5500::new("W5500");
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
        let mut violations = 0;
        for c in &d.components {
            for pin in &c.pins {
                let nets = d.nets_of_pin(&c.refdes, &pin.name);
                for net_name in &nets {
                    if let Some(net) = d.nets.iter().find(|n| &n.name == net_name)
                        && erc_voltage_pin_to_net(net, pin).is_some()
                    {
                        violations += 1;
                    }
                }
            }
        }
        assert_eq!(violations, 0, "ERC should find no overvoltage violations");
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
        let diags = erc_nc_pin_connected(&d);
        assert!(
            diags.is_empty(),
            "NC pin(s) must be floating: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
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
        let diags = erc_nc_pin_connected(&d);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "ERC:NC_CONNECTED");
        assert!(diags[0].message.contains("U1.NC"));
    }
}
