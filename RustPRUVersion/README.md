# SPI Buttons Rust Implementation

This is a Rust re-implementation of the spi_buttons repository for Beaglebone Black, using PRU 0 as SPI Master.

## Overview

The original project controls buttons with lights using SPI shift registers. This version implements the same logic in Rust, using the Beaglebone's PRU (Programmable Real-Time Unit) for real-time SPI communication via bitbanging.

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