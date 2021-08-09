# uart_xilinx

**Work In Progress**

This crate provide a struct with many methods to operate uarts in Xilinx's FPGA: XPS UART Lite, AXI UART Lite...

## REF

- [DS571 - XPS UART Lite (v1.02a) Data Sheet (v1.8)](https://china.xilinx.com/support/documentation/ip_documentation/xps_uartlite/v1_02_a/xps_uartlite.pdf)
- [PG142 - AXI UART Lite v2.0 Product Guide (v2.0)](https://www.xilinx.com/support/documentation/ip_documentation/axi_uartlite/v2_0/pg142-axi-uartlite.pdf)

## Intro

**Noticed:** This crate may have problems. Any help would be welcomed, even if your help will bring about **breaking change**. Please feel free to start an Issue or a PR.

Currently I **cannot guarantee** the stability of this crate, and it is likely to introduce destructive updates (including but not limited to renaming of structs, renaming of functions and methods, code restructuring). So fixing the dependency version should be a good way to go.

Besides, this crate currently is not following [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). Please feel free to start an Issue or a PR to help me fix this.
