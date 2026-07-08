//! Raspberry Pi RP2354A — RP2350 with 2 MB internal flash (QFN-60).
//!
//! Models the 30 GPIO pins available on the QFN-60 package, plus power
//! and ground. GPIO function selection is done at the net/connection level
//! in the reference design (e.g. connecting GPIO2 to net "SDIO_CLK" assigns
//! it to SPI0 SCK via F0).
//!
//! # GPIO function mapping used in this design
//!
//! | GPIO | Function | Net name       | Connected to                         |
//! |------|----------|----------------|--------------------------------------|
//! | 0    | SPI0 RX  | SDIO_D0        | MM8108-MF15457 pin 12 (SPI_MISO)    |
//! | 1    | SPI0 CSn | SDIO_D3        | MM8108-MF15457 pin 13 (SPI_CS)      |
//! | 2    | SPI0 SCK | SDIO_CLK       | MM8108-MF15457 pin 17 (SPI_SCK)     |
//! | 3    | SPI0 TX  | SDIO_CMD       | MM8108-MF15457 pin 16 (SPI_MOSI)    |
//! | 4    | SIO (IRQ)| SDIO_D1        | MM8108-MF15457 pin 14 (SPI_INT)     |
//! | 5    | SIO (OUT)| MM_RESET_N     | MM8108-MF15457 pin 4  (RESET_N)     |
//! | 6    | SIO (OUT)| MM_WAKE        | MM8108-MF15457 pin 5  (WAKE)        |
//! | 7    | SIO (IN) | MM_BUSY        | MM8108-MF15457 pin 29 (BUSY)        |
//! | 8    | SPI1 RX  | W5500_MISO     | W5500 pin 34 (MISO)                 |
//! | 9    | SPI1 CSn | W5500_SCSn     | W5500 pin 32 (SCSn)                 |
//! | 10   | SPI1 SCK | W5500_SCLK     | W5500 pin 33 (SCLK)                 |
//! | 11   | SPI1 TX  | W5500_MOSI     | W5500 pin 35 (MOSI)                 |
//! | 12   | SIO (OUT)| W5500_RSTn     | W5500 pin 37 (RSTn)                 |
//! | 13   | SIO (IN) | W5500_INTn     | W5500 pin 36 (INTn)                 |
//!
//! GPIOs 14–29 remain free for USB, SWD, ADC, or future expansion.

use copperleaf::{Component, Constraint, Limits, Pin, Role, SigSpec, UnitExt};

/// Raspberry Pi RP2354A (QFN-60, 2 MB internal flash, 30 GPIOs).
#[derive(Clone, Debug, Component)]
#[component(
    symbol = "MCU_RaspberryPi:RP2354A",
    constraints(
        Constraint::LengthMatch { group: "SPI0_BUS".into(), skew_ps: 200.0 },
        Constraint::LengthMatch { group: "SPI1_BUS".into(), skew_ps: 500.0 },
        Constraint::MaxJunction { temp: 85.0.celsius() },
    )
)]
pub struct Rp2354a {
    pins: Vec<Pin>,
}

impl Rp2354a {
    /// Create an RP2354A model with its 30 GPIOs and power pins.
    pub fn new() -> Self {
        let dio_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 3.63.volt(), // IOVDD abs max
            i_max: 0.012.amp(), // 12 mA drive strength max
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
        // Note: GPIO26–GPIO29 have the ADC function on the same pin; the KiCad
        // symbol names them "GPIO26/ADC0"–"GPIO29/ADC3".
        for n in 0..30u8 {
            let sig = if n <= 3 {
                // SPI0 pins — 50 MHz capable
                if n == 2 {
                    Some(SigSpec::spi_clk(50.0))
                } else {
                    Some(SigSpec::spi(50.0))
                }
            } else if (8..=11).contains(&n) {
                // SPI1 pins — 33 MHz (W5500 practical max)
                if n == 10 {
                    Some(SigSpec::spi_clk(33.0))
                } else {
                    Some(SigSpec::spi(33.0))
                }
            } else {
                None // plain GPIO / control
            };
            let name = match n {
                26 => "GPIO26/ADC0".to_string(),
                27 => "GPIO27/ADC1".to_string(),
                28 => "GPIO28/ADC2".to_string(),
                29 => "GPIO29/ADC3".to_string(),
                _ => format!("GPIO{}", n),
            };
            pins.push(Pin::new(name, Role::DigitalIO, dio_limits, sig));
        }

        // Power pins (IOVDD — IO supply, shared with HaLow VDDIO and W5500)
        pins.push(Pin::new("IOVDD", Role::PowerIn, pwr_limits, None));
        // Main digital ground and voltage-regulator ground (both tie to GND net).
        // The KiCad symbol exposes both GND and VREG_PGND; modelling both keeps
        // ERC clean and matches the QFN-60 package pinout.
        pins.push(Pin::new("GND", Role::Gnd, gnd_limits, None));
        pins.push(Pin::new("VREG_PGND", Role::Gnd, gnd_limits, None));

        Self { pins }
    }
}
