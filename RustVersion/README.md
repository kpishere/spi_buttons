# SPI Buttons Rust Implementation

This is a Rust re-implementation of the spi_buttons repository for Beaglebone Black, using the Linux SPI device interface.

## Overview

The original project controls buttons with lights using SPI shift registers. This version implements the same logic in Rust, using the `spidev` library for SPI communication via the Linux SPI device driver. 

## SPI Implementation

The implementation uses the Linux `spidev` interface to communicate with SPI shift registers. The SPI peripheral handles the low-level pin toggling for SCK (clock), MOSI (data out), MISO (data in), and CS (chip select). 

### Pin Usage Diagram

```
Beaglebone Black Pinout (SPI via spidev)

P8 Header:

P9 Header:
  31: SPI1_SCLK (SCK)     - Serial Clock (SPI clock signal)
  29: SPI1_D0 (MISO)      - Master In Slave Out (data from shift registers)
  30: SPI1_D1 (MOSI)      - Master Out Slave In (data to shift registers)
  28: SPI1_CS0 (CS)       - Chip Select (enables SPI communication)

Shift Register Connections:
- Serial Data In  <- MOSI (P9_30)
- Serial Clock    <- SCK (P9_31)
- Serial Data Out -> MISO (P9_29)
- Chip Select     <- CS (P9_28)
```

### SPI Pins Description

- **MOSI (P9_30, SPI1_D1)**: Outputs serial data to the shift registers' data input.
- **SCK (P9_31, SPI1_SCLK)**: Provides the clock signal for synchronizing data transfer.
- **MISO (P9_29, SPI1_D0)**: Reads serial data from the shift registers' data output.
- **CS (P9_28, SPI1_CS0)**: Chip select signal to enable/disable SPI communication.

## GPIO Access

## Dependencies

- `spidev`: For SPI communication via the Linux SPI device interface.

## Building

The PRU firmware is no longer used; SPI communication is handled by the Linux SPI driver.

To build the project:

```bash
cargo build
```

## Testing

Run the unit tests to verify the button logic:

```bash
cargo test
```

## Running

1. Build the Rust program: `cargo build`

The program uses the SPI device at `/dev/spidev1.0`.
