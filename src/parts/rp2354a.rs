//! Raspberry Pi RP2354A — RP2350 with 2 MB internal flash (QFN-60).
//!
//! # Pinout
//!
//! | Pin | Name       | Purpose | Notes |
//! |-----|------------|------------|--------------------------------------|
//! | 0   | SPI0 RX  | SDIO_D0        | MM8108-MF15457 pin 12 (SPI_MISO)    |
//! | 1   | SPI0 CSn | SDIO_D3        | MM8108-MF15457 pin 13 (SPI_CS)      |
//! | 2   | SPI0 SCK | SDIO_CLK       | MM8108-MF15457 pin 17 (SPI_SCK)     |
//! | 3   | SPI0 TX  | SDIO_CMD       | MM8108-MF15457 pin 16 (SPI_MOSI)    |
//! | 4   | SIO (IRQ)| SDIO_D1    | MM8108-MF15457 pin 14 (SPI_INT)     |
//! | 5   | SIO (OUT)| MM_RESET_N | MM8108-MF15457 pin 4  (RESET_N)     |
//! | 6   | SIO (OUT)| MM_WAKE        | MM8108-MF15457 pin 5  (WAKE)        |
//! | 7   | SIO (IN) | MM_BUSY        | MM8108-MF15457 pin 29 (BUSY)        |
//! | 8   | SPI1 RX  | W5500_MISO     | W5500 pin 34 (MISO)                 |
//! | 9   | SPI1 CSn | W5500_SCSn     | W5500 pin 32 (SCSn)                 |
//! | 10  | SPI1 SCK | W5500_SCLK     | W5500 pin 33 (SCLK)                 |
//! | 11  | SPI1 TX  | W5500_MOSI     | W5500 pin 35 (MOSI)                 |
//! | 12  | SIO (OUT)| W5500_RSTn     | W5500 pin 37 (RSTn)                 |
//! | 13  | SIO (IN) | W5500_INTn     | W5500 pin 36 (INTn)                 |
//!
//! ~TODO - Fix Pinout!~ Name/Purpose columns are switched around, Notes should be deleted to make this generic, Pin should be 1-based

use copperleaf_model::{Component, Pin, PinId, units::UnitExt};

pub struct Rp2354a {
    pins: [Pin; 60],
}

impl Rp2354a {
    /// Create a new RP2354A model instance
    pub fn new() -> Self {
        let iovdd = Pin::build("IOVDD").pwr(1.8.volt(), 3.3.volt(), 0.1.amp());
        let dvdd = Pin::build("DVDD").pwr(1.1.volt(), 1.1.volt(), 0.1.amp());

        let pins = [
            iovdd.clone(),
            Pin::build("GPIO0").dio(),
            Pin::build("GPIO1").dio(),
            Pin::build("GPIO2").dio(),
            Pin::build("GPIO3").dio(),
            dvdd.clone(),
            Pin::build("GPIO4").dio(),
            Pin::build("GPIO5").dio(),
            Pin::build("GPIO6").dio(),
            Pin::build("GPIO7").dio(),
            iovdd.clone(),
            Pin::build("GPIO8").dio(),
            Pin::build("GPIO9").dio(),
            Pin::build("GPIO10").dio(),
            Pin::build("GPIO11").dio(),
            Pin::build("GPIO12").dio(),
            Pin::build("GPIO13").dio(),
            Pin::build("GPIO14").dio(),
            Pin::build("GPIO15").dio(),
            iovdd.clone(),
            Pin::build("XIN").clk(),
            Pin::build("XOUT").clk(),
            dvdd.clone(),
            Pin::build("SWCLK").clk(),
            Pin::build("SWDIO").dio(),
            Pin::build("RUN").dio(),
            Pin::build("GPIO16").dio(),
            Pin::build("GPIO17").dio(),
            Pin::build("GPIO18").dio(),
            iovdd.clone(),
            Pin::build("GPIO19").dio(),
            Pin::build("GPIO20").dio(),
            Pin::build("GPIO21").dio(),
            Pin::build("GPIO22").dio(),
            Pin::build("GPIO23").dio(),
            Pin::build("GPIO24").dio(),
            Pin::build("GPIO25").dio(),
            iovdd.clone(),
            dvdd,
            Pin::build("GPIO26_ADC0").dio(),
            Pin::build("GPIO27_ADC1").dio(),
            Pin::build("GPIO28_ADC2").dio(),
            Pin::build("GPIO29_ADC3").dio(),
            Pin::build("ADC_AVDD").pwr(3.3.volt(), 3.3.volt(), 0.1.amp()),
            iovdd,
            Pin::build("VREG_AVDD").pwr(0.0.volt(), 0.0.volt(), 0.0.amp()), // @todo
            Pin::build("VREG_PGND").gnd(),
            Pin::build("VREG_LX").pwr(0.0.volt(), 0.0.volt(), 0.0.amp()), // @todo
            Pin::build("VREG_VIN").pwr(2.7.volt(), 5.5.volt(), 0.0.amp()), // @todo
            Pin::build("VREG_FB").pwr(0.0.volt(), 0.0.volt(), 0.0.amp()), // @todo
            Pin::build("USB_DM").dio(),
            Pin::build("USB_DP").dio(),
            Pin::build("USB_OTP_VDD").pwr(3.3.volt(), 3.3.volt(), 0.1.amp()),
            Pin::build("QSPI_IOVDD").pwr(1.8.volt(), 3.3.volt(), 0.1.amp()), // @todo
            Pin::build("QSPI_SD3").dio(),
            Pin::build("QSPI_SCLK").clk(),
            Pin::build("QSPI_SD0").dio(),
            Pin::build("QSPI_SD2").dio(),
            Pin::build("QSPI_SD1").dio(),
            Pin::build("QSPI_SS").dio(),
        ];

        // let dio_limits = Limits {
        //     v_min: 0.0.volt(),
        //     v_max: 3.63.volt(), // IOVDD abs max
        //     i_max: 0.012.amp(), // 12 mA drive strength max
        // };

        // for n in 0..30u8 {
        // let sig = if n <= 3 {
        //     // SPI0 pins — 50 MHz capable
        //     if n == 2 {
        //         Some(SigSpec::spi_clk(50.0))
        //     } else {
        //         Some(SigSpec::spi(50.0))
        //     }
        // } else if (8..=11).contains(&n) {
        //     // SPI1 pins — 33 MHz (W5500 practical max)
        //     if n == 10 {
        //         Some(SigSpec::spi_clk(33.0))
        //     } else {
        //         Some(SigSpec::spi(33.0))
        //     }
        // } else {
        //     None // plain GPIO / control
        // // };
        // pins.push(Pin::new(name, Role::DigitalIO, dio_limits, sig));
        // }

        Self { pins }
    }
}

impl Component for Rp2354a {
    fn pin(&self, id: PinId) -> Option<&Pin> {
        self.pins.iter().find(|pin| pin.id() == id)
    }

    fn pin_name(&self, name: &str) -> Option<&Pin> {
        self.pins.iter().find(|pin| pin.name() == name)
    }

    fn pins(&self) -> &[Pin] {
        &self.pins
    }
}
