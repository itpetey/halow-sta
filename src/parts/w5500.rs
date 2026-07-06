//! WIZnet W5500 hardwired TCP/IP Ethernet controller (48-LQFP).
//!
//! Models the 48-pin package with SPI slave interface, integrated 10/100
//! Ethernet PHY, and the external component requirements (crystal, bias
//! resistor, reference capacitors) as described in the W5500 datasheet.
//!
//! # SPI interface
//!
//! The W5500 operates as an SPI slave supporting Mode 0 and Mode 3, with
//! a practical SCLK limit of ~33.3 MHz (theoretical 80 MHz, but signal
//! integrity limits this per the datasheet footnote).
//!
//! # Key external components (placed in the reference design)
//!
//! | Component | Value     | Pin(s)     | Purpose                     |
//! |-----------|-----------|------------|-----------------------------|
//! | Y2        | 25 MHz    | XI/XO      | Crystal oscillator          |
//! | R23       | 12.4 kΩ 1%| EXRES1→GND | PHY bias reference resistor |
//! | C10       | 4.7 µF    | TOCAP→GND  | Internal reference capacitor|
//! | C11       | 10 nF     | 1V2O→GND   | 1.2V regulator bypass      |
//! | C12-C19   | 100 nF    | VDD/AVDD   | Decoupling per supply pin   |
//! | R24-R26   | 10 kΩ     | PMODE[2:0] | Pull-ups (auto-neg enabled)|

use copperleaf::{Block, Limits, Pin, Role, SigSpec, UnitExt};

/// W5500 Ethernet controller (48-LQFP, integrated 10/100 PHY).
#[derive(Clone, Debug)]
pub struct W5500 {
    pins: Vec<Pin>,
}

impl W5500 {
    /// Create a W5500 model with all relevant pins from the 48-LQFP package.
    pub fn new() -> Self {
        let spi_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 5.5.volt(), // 5V-tolerant I/O per datasheet
            i_max: 0.008.amp(), // 8 mA drive
        };

        let dio_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 5.5.volt(),
            i_max: 0.008.amp(),
        };

        let pwr_limits = Limits {
            v_min: 2.97.volt(),
            v_max: 3.63.volt(),
            i_max: 0.150.amp(), // ~132 mA typical at 100M TX
        };

        let gnd_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 0.0.volt(),
            i_max: 100.0.amp(),
        };

        let pins: Vec<Pin> = vec![
            // ── Ethernet PHY analog pins ──────────────────────────────────
            Pin::new("TXP", Role::AnalogOut, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.050.amp(),
            }, Some(SigSpec::rf_50ohm())),
            Pin::new("TXN", Role::AnalogOut, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.050.amp(),
            }, Some(SigSpec::rf_50ohm())),
            Pin::new("RXP", Role::AnalogIn, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.050.amp(),
            }, Some(SigSpec::rf_50ohm())),
            Pin::new("RXN", Role::AnalogIn, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.050.amp(),
            }, Some(SigSpec::rf_50ohm())),

            // ── Analog power/ground ────────────────────────────────────────
            Pin::new("AVDD", Role::PowerIn, pwr_limits, None),
            Pin::new("AGND", Role::Gnd, gnd_limits, None),

            // ── External bias / reference pins ─────────────────────────────
            Pin::new("EXRES1", Role::AnalogOut, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.001.amp(),
            }, None), // → 12.4 kΩ to AGND
            Pin::new("TOCAP", Role::AnalogOut, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.001.amp(),
            }, None), // → 4.7 µF to AGND
            Pin::new("1V2O", Role::PowerOut, Limits {
                v_min: 1.0.volt(), v_max: 1.4.volt(), i_max: 0.010.amp(),
            }, None), // → 10 nF to GND (1.2V regulator output)
            Pin::new("VBG", Role::AnalogOut, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.001.amp(),
            }, None), // band gap, leave floating

            // ── Crystal oscillator ─────────────────────────────────────────
            Pin::new("XI", Role::AnalogIn, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.001.amp(),
            }, Some(SigSpec::spi_clk(25.0))),
            Pin::new("XO", Role::AnalogOut, Limits {
                v_min: 0.0.volt(), v_max: 3.63.volt(), i_max: 0.001.amp(),
            }, None),

            // ── SPI interface (slave mode) ────────────────────────────────
            Pin::new("SCSn", Role::DigitalIO, spi_limits, Some(SigSpec::spi(33.0))),
            Pin::new("SCLK", Role::DigitalIO, spi_limits, Some(SigSpec::spi_clk(33.0))),
            Pin::new("MISO", Role::DigitalIO, spi_limits, Some(SigSpec::spi(33.0))),
            Pin::new("MOSI", Role::DigitalIO, spi_limits, Some(SigSpec::spi(33.0))),

            // ── Control / interrupt ───────────────────────────────────────
            Pin::new("INTn", Role::DigitalIO, dio_limits, None), // active-low IRQ out
            Pin::new("RSTn", Role::DigitalIO, dio_limits, None), // active-low reset in

            // ── PHY mode select (pull-ups → 111 = auto-neg) ──────────────
            Pin::new("PMODE0", Role::DigitalIO, dio_limits, None),
            Pin::new("PMODE1", Role::DigitalIO, dio_limits, None),
            Pin::new("PMODE2", Role::DigitalIO, dio_limits, None),

            // ── LED outputs ───────────────────────────────────────────────
            Pin::new("SPDLED", Role::DigitalIO, dio_limits, None),
            Pin::new("LINKLED", Role::DigitalIO, dio_limits, None),
            Pin::new("DUPLED", Role::DigitalIO, dio_limits, None),
            Pin::new("ACTLED", Role::DigitalIO, dio_limits, None),

            // ── Digital power/ground ───────────────────────────────────────
            Pin::new("VDD", Role::PowerIn, pwr_limits, None),
            Pin::new("GND", Role::Gnd, gnd_limits, None),
        ];

        Self { pins }
    }
}

impl Block for W5500 {
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
    fn constraints(&self) -> Vec<copperleaf::Constraint> {
        use copperleaf::Constraint;
        vec![
            // Decoupling for the digital + analog supply pins
            Constraint::Decoupling {
                values: vec![100.0.nf(), 10.0.uf()],
                per_pin: true,
            },
            // Max junction temperature
            Constraint::MaxJunction {
                temp: 125.0.celsius(), // per datasheet §5.3
            },
        ]
    }
}