# HaLow STA

A WiFi HaLow Station (STA) board using the **Morse Micro MM8108-MF15457** module with an **RP2354A** host MCU.

## Design

- **U1**: MM8108-MF15457 — Wi-Fi HaLow module (MM8108 SoC, 26 dBm, 43 Mbps)
- **U2**: RP2354A — Raspberry Pi MCU (Cortex-M33, 2 MB flash)
- **U3**: W5500 — Hardwired TCP/IP Ethernet controller

Communication: SPI0 (50 MHz) connects the HaLow module to the RP2354A, SPI1 (33 MHz) connects the W5500 Ethernet controller.
