# ag32-hal

Experimental Rust HAL (Hardware Abstraction Library) for the mysterious AG32VF303 microcontroller and FPGA combo chip from AGM, with Embassy framework.

AG32VF303 features 128K SRAM, 256K Flash, a RV32IMAFC CPU runs up to 248MHz and a built-in FPGA (namely AGRV2K) with 2K LUTs.

The FPGA connects to both GPIO lines from the MCU and chip pins, effectively being a GPIO pin mux matrix. It is attached to the AHB, capable of operating as master and slave, and is able to initiate and respond to DMA requests.

Try out the blinky example: install [this fork of probe-rs](https://github.com/NKID00/probe-rs/tree/ag32vf303), navigate to examples/ag32vf303, then execute `cargo embed --bin blinky`.

This implementation is largely inspired by ch32-hal and embassy-stm32.

## About the chip, AG32VF303

Documentation of this chip is pretty vague in general and register usages are mostly either guessed from their names or learned from the SDK. Some of the peripherals are standard ones like the PL080 DMA controller.

Since chip pins and PLL are wired to the FPGA, their configurations (pull-up/down, open-drain, etc.) can only be accessed through the FCB0 (FPGA Control Block 0) peripheral. These configurations along side the FPGA can be automatically set up by the bootloader after reset according to option bytes.

The chip has an ADIv5 Serial Wire/JTAG Debug Port, which incorporates a Memory Access Port to access RISC-V Debug Module Registers conforming to RISC-V Debug Specification v0.13. Use [this fork of probe-rs](https://github.com/NKID00/probe-rs/tree/ag32vf303) (chip description file is included in that fork) to flash it.

## Development

Minimum supported Rust version (MSRV) is latest nightly.

To build the PAC (Peripheral Access Crate), install svdtools, svd2rust and form, then execute `generate-pac.sh`.

To build target-rs flash algorithm and chip description file, install target-gen from the [this fork of probe-rs](https://github.com/NKID00/probe-rs/tree/ag32vf303), then execute `cargo run -r` in `flash/`. `flash/target/AG32VF303.yaml` is the generated chip description file.

## License

Distributed under GPL-3.0-or-later, except for the file svd/AltaRiscv.svd borrowed from official SDK (but I do suspect that their SDK is violating GPL by redistributing free software without including license information).

<sub>
Copyright (C) 2025 NKID00 <br>
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. <br>
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details. <br>
You should have received a copy of the GNU General Public License along with this program.  If not, see &le;https://www.gnu.org/licenses/&gt;.
</sub>
