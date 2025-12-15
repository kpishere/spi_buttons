# SPI Buttons Rust Implementation

This is a Rust re-implementation of the spi_buttons repository for Beaglebone Black, using PRU 0 as SPI Master.

## Overview

The original project controls buttons with lights using SPI shift registers. This version implements the same logic in Rust, using the Beaglebone's PRU (Programmable Real-Time Unit) for real-time SPI communication via bitbanging.

## PRU SPI Implementation

The PRU 0 is used as an SPI Master, implementing bitbanged SPI communication with the shift registers. The PRU firmware handles the low-level pin toggling for SCK (clock), MOSI (data out), MISO (data in), and CS (chip select) to transfer data serially. This provides deterministic, real-time SPI without relying on the main CPU's SPI peripheral.

### Pin Usage Diagram

```
Beaglebone Black Pinout (PRU 0 SPI Master)

P8 Header:
  10: GPIO68 (Lamp Latch) - Controls shift register latch for parallel load
  11: PRU0_R30_15 (MOSI) - Master Out Slave In (data to shift registers)
  12: PRU0_R30_14 (SCK)  - Serial Clock (SPI clock signal)
  15: PRU0_R31_15 (MISO) - Master In Slave Out (data from shift registers)

P9 Header:
  27: PRU0_R30_5 (CS)    - Chip Select (enables SPI communication)

Shift Register Connections:
- Serial Data In  <- MOSI (P8_11)
- Serial Clock    <- SCK (P8_12)
- Latch Clock     <- GPIO68 (P8_10)
- Serial Data Out -> MISO (P8_15)
- Chip Select     <- CS (P9_27)

Note: Pins are controlled directly by PRU 0 registers for precise timing.
```

### SPI Pins Description

- **MOSI (P8_11, PRU0_R30_15)**: Outputs serial data to the shift registers' data input.
- **SCK (P8_12, PRU0_R30_14)**: Provides the clock signal for synchronizing data transfer.
- **MISO (P8_15, PRU0_R31_15)**: Reads serial data from the shift registers' data output.
- **CS (P9_27, PRU0_R30_5)**: Chip select signal to enable/disable SPI communication.
- **Lamp Latch (P8_10, GPIO68)**: GPIO-controlled pin to latch data into the shift registers for parallel output.

## Dependencies

- `libc`: For low-level system calls (mmap, etc.).

## Building the PRU Firmware

The PRU firmware source files are included: `pru-spi-master.p` and `pru-spi-common.ph`.

The firmware is automatically compiled during the Rust build process using `build.rs`. The compiled `pru-spi-master.bin` is placed in the target directory.

To compile manually, install the TI PRU compiler tools (pasm) and run:

```bash
pasm -V2 -L -b pru-spi-master.p
```

## Running

1. Build the Rust program: `cargo build`
2. Run as root (for GPIO, memory mapping, and PRU access): `sudo cargo run`

The program automatically loads the PRU firmware and uses it for SPI communication via shared memory.