//! Raspberry Pi RP2354A — RP2350 with 2 MB internal flash (QFN-60).
//!
//! Models the 30 GPIO pins available on the QFN-60 package, plus power
//! and ground. GPIO function selection is done at the net/connection level
//! in the reference design (e.g. connecting GPIO2 to net "SDIO_CLK" assigns
//! it to SPI0 SCK via F0).
//!
//! # GPIO function mapping used in this design
//!
//! | GPIO | Function | Net name       | Connected to          |
//! |------|----------|----------------|-----------------------|
//! | 0    | SPI0 RX  | SDIO_D0        | HT-HC01 pad 17 (MISO) |
//! | 1    | SPI0 CSn | SDIO_D3        | HT-HC01 pad 22 (CS)   |
//! | 2    | SPI0 SCK | SDIO_CLK       | HT-HC01 pad 18 (SCK)  |
//! | 3    | SPI0 TX  | SDIO_CMD       | HT-HC01 pad 21 (MOSI) |
//! | 4    | SIO (IRQ)| SDIO_D1        | HT-HC01 pad 16 (INT)  |
//! | 5    | SIO (OUT)| MM_RESET_N     | HT-HC01 pad 35 (RESET)|
//! | 6    | SIO (OUT)| MM_WAKE        | HT-HC01 pad 36 (WAKE) |
//! | 7    | SIO (IN) | MM_BUSY        | HT-HC01 pad 34 (BUSY) |
//! | 8    | SPI1 RX  | W5500_MISO     | W5500 pin 34 (MISO)   |
//! | 9    | SPI1 CSn | W5500_SCSn     | W5500 pin 32 (SCSn)   |
//! | 10   | SPI1 SCK | W5500_SCLK     | W5500 pin 33 (SCLK)   |
//! | 11   | SPI1 TX  | W5500_MOSI     | W5500 pin 35 (MOSI)   |
//! | 12   | SIO (OUT)| W5500_RSTn     | W5500 pin 37 (RSTn)   |
//! | 13   | SIO (IN) | W5500_INTn     | W5500 pin 36 (INTn)   |
//!
//! GPIOs 14–29 remain free for USB, SWD, ADC, or future expansion.

use copperleaf::{Block, Limits, Pin, Role, SigKind, SigSpec, UnitExt};

/// Raspberry Pi RP2354A (QFN-60, 2 MB internal flash, 30 GPIOs).
#[derive(Clone, Debug)]
pub struct Rp2354a {
    id: String,
    pins: Vec<Pin>,
}

impl Rp2354a {
    /// Create an RP2354A model with its 30 GPIOs and power pins.
    pub fn new(id: &str) -> Self {
        let dio_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 3.63.volt(), // IOVDD abs max
            i_max: 0.012.amp(),  // 12 mA drive strength max
        };

        let pwr_limits = Limits {
            v_min: 1.62.volt(),
            v_max: 3.63.volt(),
            i_max: 0.100.amp(), // max total IOVDD current
        };

        let gnd_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 0.0.volt(),
            i_max: 100.0.amp(),
        };

        let mut pins: Vec<Pin> = Vec::new();

        // 30 GPIO pins (GPIO0–GPIO29, QFN-60 package)
        for n in 0..30u8 {
            let sig = if n <= 3 {
                // SPI0 pins — 50 MHz capable
                Some(SigSpec {
                    kind: if n == 2 { SigKind::Clock } else { SigKind::Generic },
                    bandwidth: Some(50.0.mhz()),
                    edge_rate: None,
                    target_impedance: Some(50.0.ohm()),
                })
            } else if (8..=11).contains(&n) {
                // SPI1 pins — 33 MHz (W5500 practical max)
                Some(SigSpec {
                    kind: if n == 10 { SigKind::Clock } else { SigKind::Generic },
                    bandwidth: Some(33.0.mhz()),
                    edge_rate: None,
                    target_impedance: Some(50.0.ohm()),
                })
            } else {
                None // plain GPIO / control
            };
            pins.push(Pin::new(format!("GPIO{}", n), Role::DigitalIO, dio_limits, sig));
        }

        // Power pins (IOVDD — IO supply, shared with HaLow VDD_IO and W5500)
        pins.push(Pin::new("IOVDD", Role::PowerIn, pwr_limits, None));
        // Multiple GND pins modelled as a single logical pin (all tie to GND net)
        pins.push(Pin::new("GND", Role::Gnd, gnd_limits, None));

        Self {
            id: id.to_owned(),
            pins,
        }
    }
}

impl Block for Rp2354a {
    fn id(&self) -> &str {
        &self.id
    }
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
    fn constraints(&self) -> Vec<copperleaf::Constraint> {
        use copperleaf::Constraint;
        vec![
            // SPI0 length matching for HaLow bus
            Constraint::LengthMatch {
                group: "SPI0_BUS".into(),
                skew_ps: 200.0,
            },
            // SPI1 length matching for W5500 bus
            Constraint::LengthMatch {
                group: "SPI1_BUS".into(),
                skew_ps: 500.0, // more relaxed for 33 MHz
            },
            // Max junction temp per RP2350 datasheet
            Constraint::MaxJunction {
                temp: 85.0.celsius(),
            },
        ]
    }
}