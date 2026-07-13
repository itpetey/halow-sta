//! Morse Micro MM8108-MF15457 Wi-Fi HaLow module (MM8108 SoC).
//!
//! # Pinout
//!
//! | Pin | Name     | Purpose     | Notes                 |
//! |-----|----------|-------------|-----------------------|
//! | 1   | GND_1    | Ground      |                       |
//! | 2   | ANT      | Antenna     |                       |
//! | 3   | GND_2    | Ground      |                       |
//! | 4   | RESET_N  | Reset       | Pull low to reset     |
//! | 5   | WAKE     | Wake        | Pull ? to wake        |
//! | 6   | JTAG_TMS | JTAG        | alt: GPIO15           |
//! | 7   | JTAG_TCK | JTAG        | alt: GPIO13           |
//! | 8   | JTAG_TDO | JTAG        | alt: GPIO16           |
//! | 9   | JTAG_TDI | JTAG        | alt: GPIO14           |
//! | 10  | VBAT     | Supply      | 3V3 battery power     |
//! | 11  | GND_3    | Ground      |                       |
//! | 12  | SDIO_D0  | SDIO/SPI    | alt: SPI_MISO         |
//! | 13  | SDIO_D3  | SDIO/SPI    | alt: SPI_CS           |
//! | 14  | SDIO_D1  | SDIO/SPI    | alt: SPI_INT          |
//! | 15  | SDIO_D2  | SDIO/SPI    | (unused in SPI mode)  |
//! | 16  | SDIO_CMD | SDIO/SPI    | alt: SPI_MOSI         |
//! | 17  | SDIO_CLK | SDIO/SPI    | alt: SPI_SCK          |
//! | 18  | GPIO5    | GPIO        |                       |
//! | 19  | GPIO4    | GPIO        |                       |
//! | 20  | GND_4    | Ground      |                       |
//! | 21  | GPIO3    | GPIO        |                       |
//! | 22  | VDDIO    | Supply      | Host power for I/O    |
//! | 23  | GND_5    | Ground      |                       |
//! | 24  | VBAT_TX  | Supply      | 3V3 TX power          |
//! | 25  | VDD_USB  | Supply      | USB power             |
//! | 26  | GND_6    | Ground      |                       |
//! | 27  | USB_D_N  | USB DM      | Floating in SPI mode  |
//! | 28  | USB_D_P  | USB DP      | Floating in SPI mode  |
//! | 29  | BUSY     | Busy signal |                       |
//! | 30  | GND_7    | Ground      |                       |
//! | 31  | GPIO1    | GPIO        |                       |
//! | 32  | GPIO0    | GPIO        |                       |
//! | 33  | GPIO6    | GPIO        |                       |
//! | 34  | GPIO7    | GPIO        |                       |
//! | 35  | GPIO8    | GPIO        |                       |
//! | 36  | GPIO9    | GPIO        |                       |
//! | 37  | GPIO10   | GPIO        |                       |
//! | 38  | GND_8    | Ground      |                       |

use copperleaf_model::{Component, Pin, PinRef, Role, units::UnitExt};

/// Morse Micro Wi-Fi HaLow module
pub struct Mm8108Mf15457 {
    pins: [Pin; 38],
}

#[allow(dead_code)]
impl Mm8108Mf15457 {
    pub const GND_1: PinRef = PinRef("GND_1");
    pub const ANT: PinRef = PinRef("ANT");
    pub const GND_2: PinRef = PinRef("GND_2");
    pub const RESET_N: PinRef = PinRef("RESET_N");
    pub const WAKE: PinRef = PinRef("WAKE");
    pub const JTAG_TMS: PinRef = PinRef("JTAG_TMS");
    pub const JTAG_TCK: PinRef = PinRef("JTAG_TCK");
    pub const JTAG_TDO: PinRef = PinRef("JTAG_TDO");
    pub const JTAG_TDI: PinRef = PinRef("JTAG_TDI");
    pub const VBAT: PinRef = PinRef("VBAT");
    pub const GND_3: PinRef = PinRef("GND_3");
    pub const SDIO_D0: PinRef = PinRef("SDIO_D0");
    pub const SDIO_D3: PinRef = PinRef("SDIO_D3");
    pub const SDIO_D1: PinRef = PinRef("SDIO_D1");
    pub const SDIO_D2: PinRef = PinRef("SDIO_D2");
    pub const SDIO_CMD: PinRef = PinRef("SDIO_CMD");
    pub const SDIO_CLK: PinRef = PinRef("SDIO_CLK");
    pub const GPIO5: PinRef = PinRef("GPIO5");
    pub const GPIO4: PinRef = PinRef("GPIO4");
    pub const GND_4: PinRef = PinRef("GND_4");
    pub const GPIO3: PinRef = PinRef("GPIO3");
    pub const VDDIO: PinRef = PinRef("VDDIO");
    pub const GND_5: PinRef = PinRef("GND_5");
    pub const VBAT_TX: PinRef = PinRef("VBAT_TX");
    pub const VDD_USB: PinRef = PinRef("VDD_USB");
    pub const GND_6: PinRef = PinRef("GND_6");
    pub const USB_D_N: PinRef = PinRef("USB_D_N");
    pub const USB_D_P: PinRef = PinRef("USB_D_P");
    pub const BUSY: PinRef = PinRef("BUSY");
    pub const GND_7: PinRef = PinRef("GND_7");
    pub const GPIO1: PinRef = PinRef("GPIO1");
    pub const GPIO0: PinRef = PinRef("GPIO0");
    pub const GPIO6: PinRef = PinRef("GPIO6");
    pub const GPIO7: PinRef = PinRef("GPIO7");
    pub const GPIO8: PinRef = PinRef("GPIO8");
    pub const GPIO9: PinRef = PinRef("GPIO9");
    pub const GPIO10: PinRef = PinRef("GPIO10");
    pub const GND_8: PinRef = PinRef("GND_8");

    /// Create an MM8108-MF15457 module instance
    pub fn new() -> Self {
        let pins = [
            Pin::build("GND_1").gnd(),
            Pin::build("ANT").role(Role::AnalogIn).rf_limits().pin(),
            Pin::build("GND_2").gnd(),
            Pin::build("RESET_N").dio(),
            Pin::build("WAKE").dio(),
            Pin::build("JTAG_TMS").dio(),
            Pin::build("JTAG_TCK").clk(1.0),
            Pin::build("JTAG_TDO").dio(),
            Pin::build("JTAG_TDI").dio(),
            Pin::build("VBAT")
                .pwr(3.0.volt(), 3.6.volt(), 0.3.amp())
                .pin(),
            Pin::build("GND_3").gnd(),
            Pin::build("SDIO_D0").spi(50.0),
            Pin::build("SDIO_D3").spi(50.0),
            Pin::build("SDIO_D1").spi(50.0),
            Pin::build("SDIO_D2").dio(),
            Pin::build("SDIO_CMD").spi(50.0),
            Pin::build("SDIO_CLK").clk(50.0),
            Pin::build("GPIO5").dio(),
            Pin::build("GPIO4").dio(),
            Pin::build("GND_4").gnd(),
            Pin::build("GPIO3").dio(),
            Pin::build("VDDIO")
                .pwr(1.8.volt(), 3.6.volt(), 0.05.amp())
                .pin(),
            Pin::build("GND_5").gnd(),
            Pin::build("VBAT_TX")
                .pwr(3.0.volt(), 3.6.volt(), 0.5.amp())
                .pin(),
            Pin::build("VDD_USB")
                .pwr(3.0.volt(), 3.6.volt(), 0.1.amp())
                .pin(),
            Pin::build("GND_6").gnd(),
            Pin::build("USB_D_N").dio(),
            Pin::build("USB_D_P").dio(),
            Pin::build("BUSY").dio(),
            Pin::build("GND_7").gnd(),
            Pin::build("GPIO1").dio(),
            Pin::build("GPIO0").dio(),
            Pin::build("GPIO6").dio(),
            Pin::build("GPIO7").dio(),
            Pin::build("GPIO8").dio(),
            Pin::build("GPIO9").dio(),
            Pin::build("GPIO10").dio(),
            Pin::build("GND_8").gnd(),
        ];

        Self { pins }
    }
}

impl Component for Mm8108Mf15457 {
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
}

impl Default for Mm8108Mf15457 {
    fn default() -> Self {
        Self::new()
    }
}
