# SPI Buttons Rust Implementation

This is a Rust re-implementation of the spi_buttons repository for Beaglebone Black, using the Linux SPI device interface.

## Overview

The original project controls buttons with lights using SPI shift registers. This version implements the same logic in Rust, using the `spidev` library for SPI communication via the Linux SPI device driver. GPIO is used for the latch pin to control shift register parallel loading.

## SPI Implementation

The implementation uses the Linux `spidev` interface to communicate with SPI shift registers. The SPI peripheral handles the low-level pin toggling for SCK (clock), MOSI (data out), MISO (data in), and CS (chip select). GPIO is used for the latch signal to control when data is latched into the shift registers for parallel output.

### Pin Usage Diagram

```
Beaglebone Black Pinout (SPI via spidev)

P8 Header:
  10: GPIO68 (Lamp Latch) - Controls shift register latch for parallel load
  11: SPI1_D1 (MOSI)      - Master Out Slave In (data to shift registers)
  12: SPI1_SCLK (SCK)     - Serial Clock (SPI clock signal)
  15: SPI1_D0 (MISO)      - Master In Slave Out (data from shift registers)

P9 Header:
  28: SPI1_CS0 (CS)       - Chip Select (enables SPI communication)

Shift Register Connections:
- Serial Data In  <- MOSI (P8_11)
- Serial Clock    <- SCK (P8_12)
- Latch Clock     <- GPIO68 (P8_10)
- Serial Data Out -> MISO (P8_15)
- Chip Select     <- CS (P9_28)

Note: SPI pins are managed by the Linux SPI driver; GPIO68 is controlled via sysfs.
```

### SPI Pins Description

- **MOSI (P8_11, SPI1_D1)**: Outputs serial data to the shift registers' data input.
- **SCK (P8_12, SPI1_SCLK)**: Provides the clock signal for synchronizing data transfer.
- **MISO (P8_15, SPI1_D0)**: Reads serial data from the shift registers' data output.
- **CS (P9_28, SPI1_CS0)**: Chip select signal to enable/disable SPI communication.
- **Lamp Latch (P8_10, GPIO68)**: GPIO-controlled pin to latch data into the shift registers for parallel output.

## Dependencies

- `spidev`: For SPI communication via the Linux SPI device interface.

## Building

The PRU firmware is no longer used; SPI communication is handled by the Linux SPI driver.

To build the project:

```bash
cargo build
```

## Running

1. Build the Rust program: `cargo build`
2. Run as root (for GPIO access): `sudo cargo run`

The program uses the SPI device at `/dev/spidev1.0` and GPIO68 for latch control.