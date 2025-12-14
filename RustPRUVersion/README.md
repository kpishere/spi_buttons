# SPI Buttons Rust Implementation

This is a Rust re-implementation of the spi_buttons repository for Beaglebone Black, using PRU 0 as SPI Master.

## Overview

The original project controls buttons with lights using SPI shift registers. This version implements the same logic in Rust, using the Beaglebone's PRU (Programmable Real-Time Unit) for real-time SPI communication via bitbanging.

## Dependencies

- `pru`: For PRU firmware loading and memory mapping.

## Building the PRU Firmware

The PRU firmware source files are included: `pru-spi-master.p` and `pru-spi-common.ph`.

To compile the firmware, install the TI PRU compiler tools (pasm) and run:

```bash
pasm -V2 -L -b pru-spi-master.p
```

This generates `pru-spi-master.bin`, which is loaded by the Rust program.

## Hardware Setup

Connect the SPI shift registers and buttons/lights to the Beaglebone Black as follows:

### PRU SPI Pins (Master on PRU 0)
- CS (Chip Select): P9_27 (R30.5)
- MISO (Master In Slave Out): P8_15 (R31.15)
- MOSI (Master Out Slave In): P8_11 (R30.15)
- SCK (Serial Clock): P8_12 (R30.14)

### GPIO Latch Pin
- Lamp Latch: P8_10 (GPIO68)

Ensure the shift registers (e.g., MC14021B for buttons, MC14094B for lights) are wired according to their datasheets, with power and ground connected appropriately.

## Running

1. Compile the PRU firmware as described above.
2. Build the Rust program: `cargo build`
3. Run as root (for GPIO and PRU access): `sudo cargo run`

Note: Requires Beaglebone Black with PRU firmware loaded, GPIO68 configured, and appropriate hardware setup for buttons and lights via shift registers. The SPI is handled by the PRU firmware for real-time performance.