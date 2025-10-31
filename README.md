# ag32-hal

Rust HAL (Hardware Abstraction Library) for the mysterious AG32VF303 microcontroller and FPGA combo chip from AGM.

AG32VF303 features 128K SRAM, 256K Flash, a RV32IMAFC CPU up to 248MHz and a built-in FPGA (namely AGRV2K) with 2K LUTs.

The FPGA connects to both GPIO lines from the MCU and chip pins, effectively being a GPIO pin mux matrix. It is attached to the AHB, capable of operating as master and slave, and is able to initiate and respond to DMA requests.

The documentations are pretty vague and register usages are either guessed from their names or learned from the SDK.

To build the PAC (Peripheral Access Crate), execute generate-pac.sh.

## License

Distributed under GPL-3.0-or-later, except for the file svd/AltaRiscv.svd borrowed from official SDK.

<sub>
Copyright (C) 2025 NKID00 <br>
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. <br>
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details. <br>
You should have received a copy of the GNU General Public License along with this program.  If not, see &le;https://www.gnu.org/licenses/&gt;.
</sub>
