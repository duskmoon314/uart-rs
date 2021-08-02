/*!
# uartXPS

This crate provide a struct with many methods to operate Xilinx XPS UART Lite.

[REF: DS571 - XPS UART Lite (v1.02a) Data Sheet (v1.8)](https://china.xilinx.com/support/documentation/ip_documentation/xps_uartlite/v1_02_a/xps_uartlite.pdf)
*/

#![no_std]

#[macro_use]
pub mod registers;
pub mod uart;

pub use uart::MmioUartXPS;
