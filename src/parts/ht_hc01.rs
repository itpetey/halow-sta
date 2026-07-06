//! HT-HC01 V2 WiFi HaLow module (MM6108 IQ).
//!
//! All 38 pad pins are modelled per the datasheet pin table (section 2).
//! Pin names mirror the datasheet pad names so that connections in the
//! reference design read naturally.
//!
//! # Electrical summary
//!
//! | Supply      | Range       | Typ  | Min current |
//! |-------------|-------------|------|------------|
//! | VBAT        | 3.0–3.6 V   | 3.3  | 200 mA*    |
//! | VDD_IO      | 1.62–3.6 V  | 3.3  | (shared*)  |
//! | VDD_FEM     | 3.0–5.25 V  | 5.0  | 800 mA     |
//!
//! \* VBAT + VDD_IO combined input current > 200 mA.
//!
//! SPI mode pin multiplexing (alternate function column from datasheet):
//!
//! | Pad | Primary  | SPI alt  |
//! |-----|----------|----------|
//! | 16  | SDIO_D1  | SPI_INT  |
//! | 17  | SDIO_D0  | SPI_MISO |
//! | 18  | SDIO_CLK | SPI_SCK  |
//! | 21  | SDIO_CMD | SPI_MOSI |
//! | 22  | SDIO_D3  | SPI_CS   |

use copperleaf::{Block, Limits, Pin, Role, SigKind, SigSpec, UnitExt};

/// Wi-Fi HaLow module with 38 pads, SPI interface active.
#[derive(Clone, Debug)]
pub struct HtHc01 {
    id: String,
    pins: Vec<Pin>,
}

// ── Helper builders ───────────────────────────────────────────────────

/// Operating limits for digital I/O pins (VDD_IO domain, 0–VDD_IO).
fn digital_limits() -> Limits {
    Limits {
        v_min: 0.0.volt(),
        v_max: 3.6.volt(),
        i_max: 0.02.amp(), // 20 mA
    }
}

/// Limits for the analogue / RF antenna pin.
fn rf_limits() -> Limits {
    Limits {
        v_min: 0.0.volt(),
        v_max: 1.2.volt(), // abs. max for analog/RF pin per datasheet §3.2.1
        i_max: 1.0.amp(),
    }
}

/// SPI signal specification: up to 50 MHz, 50 Ω single-ended.
fn spi_sig() -> SigSpec {
    SigSpec {
        kind: SigKind::Generic,
        bandwidth: Some(50.0.mhz()), // 20 ns period
        edge_rate: None,
        target_impedance: Some(50.0.ohm()),
    }
}

/// SPI clock signal specification.
fn spi_clk_sig() -> SigSpec {
    SigSpec {
        kind: SigKind::Clock,
        bandwidth: Some(50.0.mhz()),
        edge_rate: None,
        target_impedance: Some(50.0.ohm()),
    }
}

fn gnd_pin(name: &str) -> Pin {
    Pin::new(name, Role::Gnd, Limits {
        v_min: 0.0.volt(),
        v_max: 0.0.volt(),
        i_max: 100.0.amp(),
    }, None)
}

fn pwr_pin(name: &str, v_min: f64, v_max: f64, i_max: f64) -> Pin {
    Pin::new(name, Role::PowerIn, Limits {
        v_min: v_min.volt(),
        v_max: v_max.volt(),
        i_max: i_max.amp(),
    }, None)
}

fn dio_pin(name: &str) -> Pin {
    Pin::new(name, Role::DigitalIO, digital_limits(), None)
}

fn dio_spi_pin(name: &str) -> Pin {
    Pin::new(name, Role::DigitalIO, digital_limits(), Some(spi_sig()))
}

fn dio_clk_pin(name: &str) -> Pin {
    Pin::new(name, Role::DigitalIO, digital_limits(), Some(spi_clk_sig()))
}

// ── Implementation ────────────────────────────────────────────────────

impl HtHc01 {
    /// Create an HT-HC01 V2 module instance with all 38 pads defined.
    pub fn new(id: &str) -> Self {
        let pins: Vec<Pin> = vec![
            // ── Row 1 (pads 1–11): JTAG & GND ───────────────────────
            gnd_pin("GND"),          // Pad 1
            gnd_pin("GND"),          // Pad 2
            gnd_pin("GND"),          // Pad 3
            dio_pin("JTAG_TCK"),     // Pad 4  (pull-down in ref design)
            dio_pin("JTAG_TDI"),     // Pad 5  (pull-down)
            dio_pin("NC"),           // Pad 6  (no connect)
            dio_pin("JTAG_TMS"),     // Pad 7  (pull-down)
            dio_pin("JTAG_TRST_N"),  // Pad 8  (pull-down)
            dio_pin("JTAG_TDO"),     // Pad 9  (pull-down)
            dio_pin("NC"),           // Pad 10 (no connect)
            dio_pin("NC"),           // Pad 11 (no connect)

            // ── Row 2 (pads 12–20): GPIO, SPI bus, VDD_IO ──────────
            gnd_pin("GND"),                    // Pad 12
            dio_pin("GPIO_9"),                 // Pad 13 (pull-down, unused)
            dio_pin("GPIO_8"),                 // Pad 14 (pull-down, unused)
            dio_pin("GPIO_7"),                 // Pad 15 (pull-down; alt: UART1_TX)
            dio_spi_pin("SDIO_D1"),            // Pad 16  alt: SPI_INT
            dio_spi_pin("SDIO_D0"),            // Pad 17  alt: SPI_MISO
            dio_clk_pin("SDIO_CLK"),           // Pad 18  alt: SPI_SCK
            pwr_pin("VDD_IO", 1.62, 3.6, 0.2),// Pad 19  3.3 V
            gnd_pin("GND"),                    // Pad 20

            // ── Row 3 (pads 21–30): SPI bus, GPIO, VBAT ────────────
            dio_spi_pin("SDIO_CMD"),            // Pad 21  alt: SPI_MOSI
            dio_spi_pin("SDIO_D3"),             // Pad 22  alt: SPI_CS
            dio_pin("SDIO_D2"),                 // Pad 23 (pull-down, unused in SPI)
            dio_pin("GPIO_6"),                  // Pad 24 (pull-down; alt: UART1_RX)
            pwr_pin("VBAT", 3.0, 3.6, 0.2),     // Pad 25  3.3 V
            gnd_pin("GND"),                     // Pad 26
            dio_pin("GPIO_5"),                  // Pad 27 (pull-down; alt: I2C_SCL)
            dio_pin("GPIO_4"),                  // Pad 28 (pull-down; alt: I2C_SDA)
            dio_pin("GPIO_3"),                  // Pad 29 (pull-down; alt: UART0_TX)
            dio_pin("GPIO_2"),                  // Pad 30 (pull-down; alt: UART0_RX)

            // ── Row 4 (pads 31–38): GND, VDD_FEM, GPIO, control, ANT
            gnd_pin("GND"),                              // Pad 31
            pwr_pin("VDD_FEM", 3.0, 5.25, 0.8),         // Pad 32  5 V frontend
            dio_pin("GPIO_1"),                           // Pad 33 (pull-down; alt: PWM1_1)
            dio_pin("BUSY"),                            // Pad 34 (Wi-Fi BUSY)
            dio_pin("RESET_N"),                         // Pad 35 (active-low reset)
            dio_pin("WAKE"),                            // Pad 36 (wake)
            gnd_pin("GND"),                             // Pad 37
            Pin::new("ANT", Role::AnalogIn, rf_limits(), None), // Pad 38
        ];

        Self {
            id: id.to_owned(),
            pins,
        }
    }
}

impl Block for HtHc01 {
    fn id(&self) -> &str {
        &self.id
    }
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
    fn constraints(&self) -> Vec<copperleaf::Constraint> {
        use copperleaf::Constraint;
        vec![
            // VBAT & VDD_IO decoupling: 100 nF + 10 µF per power pin.
            Constraint::Decoupling {
                values: vec![100.0.nf(), 10.0.uf()],
                per_pin: true,
            },
            // SPI bus length matching: SCK / MOSI / MISO / CS within 200 ps skew.
            Constraint::LengthMatch {
                group: "SPI_BUS".into(),
                skew_ps: 200.0,
            },
            // Operating temperature per datasheet: −40 to 85 °C.
            Constraint::MaxJunction {
                temp: 85.0.celsius(),
            },
        ]
    }
}