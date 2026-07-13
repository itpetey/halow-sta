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

use copperleaf_model::{Component, Pin, PinRef, units::UnitExt};

pub struct Rp2354a {
    pins: [Pin; 60],
}

#[allow(dead_code)]
impl Rp2354a {
    pub const IOVDD: PinRef = PinRef("IOVDD");
    pub const DVDD: PinRef = PinRef("DVDD");
    pub const GPIO0: PinRef = PinRef("GPIO0");
    pub const GPIO1: PinRef = PinRef("GPIO1");
    pub const GPIO2: PinRef = PinRef("GPIO2");
    pub const GPIO3: PinRef = PinRef("GPIO3");
    pub const GPIO4: PinRef = PinRef("GPIO4");
    pub const GPIO5: PinRef = PinRef("GPIO5");
    pub const GPIO6: PinRef = PinRef("GPIO6");
    pub const GPIO7: PinRef = PinRef("GPIO7");
    pub const GPIO8: PinRef = PinRef("GPIO8");
    pub const GPIO9: PinRef = PinRef("GPIO9");
    pub const GPIO10: PinRef = PinRef("GPIO10");
    pub const GPIO11: PinRef = PinRef("GPIO11");
    pub const GPIO12: PinRef = PinRef("GPIO12");
    pub const GPIO13: PinRef = PinRef("GPIO13");
    pub const GPIO14: PinRef = PinRef("GPIO14");
    pub const GPIO15: PinRef = PinRef("GPIO15");
    pub const GPIO16: PinRef = PinRef("GPIO16");
    pub const GPIO17: PinRef = PinRef("GPIO17");
    pub const GPIO18: PinRef = PinRef("GPIO18");
    pub const GPIO19: PinRef = PinRef("GPIO19");
    pub const GPIO20: PinRef = PinRef("GPIO20");
    pub const GPIO21: PinRef = PinRef("GPIO21");
    pub const GPIO22: PinRef = PinRef("GPIO22");
    pub const GPIO23: PinRef = PinRef("GPIO23");
    pub const GPIO24: PinRef = PinRef("GPIO24");
    pub const GPIO25: PinRef = PinRef("GPIO25");
    pub const GPIO26_ADC0: PinRef = PinRef("GPIO26_ADC0");
    pub const GPIO27_ADC1: PinRef = PinRef("GPIO27_ADC1");
    pub const GPIO28_ADC2: PinRef = PinRef("GPIO28_ADC2");
    pub const GPIO29_ADC3: PinRef = PinRef("GPIO29_ADC3");
    pub const XIN: PinRef = PinRef("XIN");
    pub const XOUT: PinRef = PinRef("XOUT");
    pub const SWCLK: PinRef = PinRef("SWCLK");
    pub const SWDIO: PinRef = PinRef("SWDIO");
    pub const RUN: PinRef = PinRef("RUN");
    pub const ADC_AVDD: PinRef = PinRef("ADC_AVDD");
    pub const VREG_AVDD: PinRef = PinRef("VREG_AVDD");
    pub const VREG_PGND: PinRef = PinRef("VREG_PGND");
    pub const VREG_LX: PinRef = PinRef("VREG_LX");
    pub const VREG_VIN: PinRef = PinRef("VREG_VIN");
    pub const VREG_FB: PinRef = PinRef("VREG_FB");
    pub const USB_DM: PinRef = PinRef("USB_DM");
    pub const USB_DP: PinRef = PinRef("USB_DP");
    pub const USB_OTP_VDD: PinRef = PinRef("USB_OTP_VDD");
    pub const QSPI_IOVDD: PinRef = PinRef("QSPI_IOVDD");
    pub const QSPI_SD3: PinRef = PinRef("QSPI_SD3");
    pub const QSPI_SCLK: PinRef = PinRef("QSPI_SCLK");
    pub const QSPI_SD0: PinRef = PinRef("QSPI_SD0");
    pub const QSPI_SD2: PinRef = PinRef("QSPI_SD2");
    pub const QSPI_SD1: PinRef = PinRef("QSPI_SD1");
    pub const QSPI_SS: PinRef = PinRef("QSPI_SS");

    /// Create a new RP2354A model instance
    pub fn new() -> Self {
        let iovdd = Pin::build("IOVDD")
            .pwr(1.8.volt(), 3.3.volt(), 0.1.amp())
            .pin();
        let dvdd = Pin::build("DVDD")
            .pwr(1.1.volt(), 1.1.volt(), 0.1.amp())
            .pin();

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
            Pin::build("XIN").clk(12.0),
            Pin::build("XOUT").clk(12.0),
            dvdd.clone(),
            Pin::build("SWCLK").clk(1.0),
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
            Pin::build("ADC_AVDD")
                .pwr_fixed(3.3.volt(), 0.1.amp())
                .pin(),
            iovdd,
            Pin::build("VREG_AVDD")
                .pwr(1.1.volt(), 1.1.volt(), 0.0.amp())
                .pin(), // @todo
            Pin::build("VREG_PGND").gnd(),
            Pin::build("VREG_LX")
                .pwr(0.0.volt(), 0.0.volt(), 0.0.amp())
                .pin(), // @todo
            Pin::build("VREG_VIN")
                .pwr(2.7.volt(), 5.5.volt(), 0.0.amp())
                .pin(), // @todo
            Pin::build("VREG_FB")
                .pwr(0.0.volt(), 0.0.volt(), 0.0.amp())
                .pin(), // @todo
            Pin::build("USB_DM").dio(),
            Pin::build("USB_DP").dio(),
            Pin::build("USB_OTP_VDD")
                .pwr_fixed(3.3.volt(), 0.1.amp())
                .pin(),
            Pin::build("QSPI_IOVDD")
                .pwr(1.8.volt(), 3.3.volt(), 0.1.amp())
                .pin(), // @todo
            Pin::build("QSPI_SD3").dio(),
            Pin::build("QSPI_SCLK").clk(50.0),
            Pin::build("QSPI_SD0").dio(),
            Pin::build("QSPI_SD2").dio(),
            Pin::build("QSPI_SD1").dio(),
            Pin::build("QSPI_SS").dio(),
        ];

        Self { pins }
    }
}

impl Component for Rp2354a {
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
}

impl Default for Rp2354a {
    fn default() -> Self {
        Self::new()
    }
}
