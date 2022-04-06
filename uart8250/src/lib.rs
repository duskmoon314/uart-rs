/*!
# uart8250

This crate provides a struct with many methods to operate an 8250 UART.

[REF: Serial Programming/8250 UART Programming](https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming#UART_Registers)
*/

#![no_std]

mod registers;
mod uart;

pub use uart::{ChipFifoInfo, InterruptType, MmioUart8250, Parity};
