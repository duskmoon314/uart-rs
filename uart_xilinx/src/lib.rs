/*!
# uart_xilinx

This crate provide a struct with many methods to operate uarts in Xilinx's FPGA: XPS UART Lite, AXI UART Lite...

## REF

- [DS571 - XPS UART Lite (v1.02a) Data Sheet (v1.8)](https://china.xilinx.com/support/documentation/ip_documentation/xps_uartlite/v1_02_a/xps_uartlite.pdf)
- [PG142 - AXI UART Lite v2.0 Product Guide (v2.0)](https://www.xilinx.com/support/documentation/ip_documentation/axi_uartlite/v2_0/pg142-axi-uartlite.pdf)
*/

#![no_std]

#[macro_use]
extern crate bitflags;

pub mod uart_16550;
pub mod uart_lite;

pub use uart_16550::MmioUartAxi16550;
pub use uart_lite::{MmioUartAxiLite, MmioUartXpsLite};
