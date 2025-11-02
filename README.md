# ag32-hal

Experimental rust HAL (Hardware Abstraction Library) for the mysterious AG32VF303 microcontroller and FPGA combo chip from AGM.

AG32VF303 features 128K SRAM, 256K Flash, a RV32IMAFC CPU runs up to 248MHz and a built-in FPGA (namely AGRV2K) with 2K LUTs.

The FPGA connects to both GPIO lines from the MCU and chip pins, effectively being a GPIO pin mux matrix. It is attached to the AHB, capable of operating as master and slave, and is able to initiate and respond to DMA requests.

## About the chip, AG32VF303

Documentation of this chip is pretty vague in general and register usages are mostly either guessed from their names or learned from the SDK. Some of the peripherals are standard ones like the PL080 DMA controller.

Since chip pins and PLL are wired to the FPGA, their configurations (pull-up/down, open-drain, etc.) are not directly accessible from the MCU part. They can only be configured along side the FPGA via the FCB0 peripheral (I have no idea what FCB stands for, FPGA Control Block?).

The chip has an ADIv5 Serial Wire/JTAG Debug Port, which incorporates a Memory Access Port to access RISC-V Debug Module Registers conforming to RISC-V Debug Specification v0.13. Use [this forked version of probe-rs](https://github.com/NKID00/probe-rs/tree/riscv-behind-adi) with chip description file and flash algorithms in this repo to flash it.

## Development

Minimum supported Rust version (MSRV) is latest nightly.

To build the PAC (Peripheral Access Crate), execute generate-pac.sh.

This implementation is highly inspired by ch32-hal and embassy-stm32.

To build flash algorithm, install target-gen.

## License

Distributed under GPL-3.0-or-later, except for the file svd/AltaRiscv.svd borrowed from official SDK.

<sub>
Copyright (C) 2025 NKID00 <br>
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. <br>
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details. <br>
You should have received a copy of the GNU General Public License along with this program.  If not, see &le;https://www.gnu.org/licenses/&gt;.
</sub>
