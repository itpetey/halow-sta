//! Morse Micro MM8108-MF15457 Wi-Fi HaLow module (MM8108 SoC).
//!
//! All 38 pins modelled per the datasheet pin table (Rev. 4, §2).
//! Pin names mirror the datasheet so connections read naturally.
//!
//! # Key differences from HT-HC01 V2 (previous design)
//!
//! - **Single 3.3V supply** — no separate VDD_FEM (front-end integrated)
//! - **Integrated 26dBm PA + LNA** — better range, simpler power rail
//! - **Up to 43Mbps PHY** — 256-QAM @ 8MHz (vs 32.5Mbps on MM6108)
//! - **Pre-certified** — FCC modular approval (FCC ID: 2A74O-737B5B)
//! - **USB 2.0 HS** interface available (not used in this SPI design)
//!
//! # SPI mode pin multiplexing
//!
//! | Pin | Primary  | SPI alt  |
//! |-----|----------|----------|
//! | 12  | SDIO_D0  | SPI_MISO |
//! | 13  | SDIO_D3  | SPI_CS   |
//! | 14  | SDIO_D1  | SPI_INT  |
//! | 16  | SDIO_CMD | SPI_MOSI |
//! | 17  | SDIO_CLK | SPI_SCK  |
//!
//! # Power rails (all 3.3V in this design)
//!
//! | Supply    | Range     | Typ  | Notes                        |
//! |-----------|-----------|------|------------------------------|
//! | VBAT      | 3.0–3.6 V | 3.3  | Main SoC supply              |
//! | VBAT_TX   | 3.0–3.6 V | 3.3  | PA supply                    |
//! | VDDIO     | 1.8–3.6 V | 3.3  | Digital I/O supply           |
//! | VDD_USB   | 3.0–3.6 V | —    | USB supply (GND when unused) |

use copperleaf::{Component, Constraint, Limits, Pin, Role, SigSpec, UnitExt};

/// Wi-Fi HaLow module with 38 pins, SPI interface active.
#[derive(Clone, Debug, Component)]
#[component(
    symbol = "MM8108-MF15457",
    symbol_lib_path = "components/mm8108-mf15457.kicad_sym",
    constraints(
        Constraint::Decoupling { values: vec![100.0.nf(), 10.0.uf()], per_pin: true },
        Constraint::LengthMatch { group: "SPI_BUS".into(), skew_ps: 200.0 },
        Constraint::MaxJunction { temp: 85.0.celsius() },
    )
)]
pub struct Mm8108Mf15457 {
    pins: Vec<Pin>,
}

impl Mm8108Mf15457 {
    /// Create an MM8108-MF15457 module instance with all 38 pins defined.
    pub fn new() -> Self {
        let pins: Vec<Pin> = vec![
            // ── Pins 1–5: GND, ANT, GND, RESET, WAKE ──────────────────
            gnd_pin("GND_1"),                                    // Pin 1
            Pin::new("ANT", Role::AnalogIn, rf_limits(), None), // Pin 2
            gnd_pin("GND_2"),                                    // Pin 3
            dio_pin("RESET_N"),                                 // Pin 4  (active-low reset)
            dio_pin("WAKE"),                                    // Pin 5  (external wake)
            // ── Pins 6–9: JTAG (pull-down when unused) ────────────────
            dio_pin("JTAG_TMS"), // Pin 6  alt: GPIO15
            dio_pin("JTAG_TCK"), // Pin 7  alt: GPIO13
            dio_pin("JTAG_TDO"), // Pin 8  alt: GPIO16
            dio_pin("JTAG_TDI"), // Pin 9  alt: GPIO14
            // ── Pins 10–11: VBAT, GND ──────────────────────────────────
            pwr_pin("VBAT", 3.0, 3.6, 0.3), // Pin 10  3.3 V main supply
            gnd_pin("GND_3"),                // Pin 11
            // ── Pins 12–17: SDIO / SPI bus ────────────────────────────
            dio_spi_pin("SDIO_D0"),  // Pin 12  alt: SPI_MISO
            dio_spi_pin("SDIO_D3"),  // Pin 13  alt: SPI_CS
            dio_spi_pin("SDIO_D1"),  // Pin 14  alt: SPI_INT
            dio_pin("SDIO_D2"),      // Pin 15  (unused in SPI mode)
            dio_spi_pin("SDIO_CMD"), // Pin 16  alt: SPI_MOSI
            dio_clk_pin("SDIO_CLK"), // Pin 17  alt: SPI_SCK
            // ── Pins 18–19: GPIO5, GPIO4 ──────────────────────────────
            dio_pin("GPIO5"), // Pin 18  (pull-down, unused)
            dio_pin("GPIO4"), // Pin 19  (pull-down, unused)
            // ── Pins 20–22: GND, GPIO3, VDDIO ─────────────────────────
            gnd_pin("GND_4"),                  // Pin 20
            dio_pin("GPIO3"),                 // Pin 21  (pull-down, unused)
            pwr_pin("VDDIO", 1.8, 3.6, 0.05), // Pin 22  3.3 V I/O supply
            // ── Pins 23–26: GND, VBAT_TX, VDD_USB, GND ───────────────
            gnd_pin("GND_5"),                   // Pin 23
            pwr_pin("VBAT_TX", 3.0, 3.6, 0.5), // Pin 24  3.3 V PA supply
            pwr_pin("VDD_USB", 3.0, 3.6, 0.1), // Pin 25  USB supply (GND when unused)
            gnd_pin("GND_6"),                   // Pin 26
            // ── Pins 27–28: USB (floating in SPI mode) ───────────────
            dio_pin("USB_D_N"), // Pin 27  USB DM
            dio_pin("USB_D_P"), // Pin 28  USB DP
            // ── Pin 29: BUSY ──────────────────────────────────────────
            dio_pin("BUSY"), // Pin 29  BUSY signal output
            // ── Pins 30–38: GND, GPIOs, GND ───────────────────────────
            gnd_pin("GND_7"),   // Pin 30
            dio_pin("GPIO1"),  // Pin 31  (pull-down, unused)
            dio_pin("GPIO0"),  // Pin 32  (pull-down, unused)
            dio_pin("GPIO6"),  // Pin 33  (pull-down, unused)
            dio_pin("GPIO7"),  // Pin 34  (pull-down, unused)
            dio_pin("GPIO8"),  // Pin 35  (pull-down, unused)
            dio_pin("GPIO9"),  // Pin 36  (pull-down, unused)
            dio_pin("GPIO10"), // Pin 37  (pull-down, unused)
            gnd_pin("GND_8"),   // Pin 38
        ];

        Self { pins }
    }
}

/// Operating limits for digital I/O pins (VDDIO domain, 0–VDDIO).
fn digital_limits() -> Limits {
    Limits {
        v_min: 0.0.volt(),
        v_max: 3.6.volt(),
        i_max: 0.02.amp(),
    }
}

fn dio_clk_pin(name: &str) -> Pin {
    Pin::new(
        name,
        Role::DigitalIO,
        digital_limits(),
        Some(SigSpec::spi_clk(50.0)),
    )
}

fn dio_pin(name: &str) -> Pin {
    Pin::new(name, Role::DigitalIO, digital_limits(), None)
}

fn dio_spi_pin(name: &str) -> Pin {
    Pin::new(
        name,
        Role::DigitalIO,
        digital_limits(),
        Some(SigSpec::spi(50.0)),
    )
}

fn gnd_pin(name: &str) -> Pin {
    Pin::new(
        name,
        Role::Gnd,
        Limits {
            v_min: 0.0.volt(),
            v_max: 0.0.volt(),
            i_max: 100.0.amp(),
        },
        None,
    )
}

fn pwr_pin(name: &str, v_min: f64, v_max: f64, i_max: f64) -> Pin {
    Pin::new(
        name,
        Role::PowerIn,
        Limits {
            v_min: v_min.volt(),
            v_max: v_max.volt(),
            i_max: i_max.amp(),
        },
        None,
    )
}

/// Limits for the analogue / RF antenna pin.
fn rf_limits() -> Limits {
    Limits {
        v_min: 0.0.volt(),
        v_max: 1.2.volt(),
        i_max: 1.0.amp(),
    }
}
