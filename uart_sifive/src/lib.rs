/*!
# uart_sifive

A simple struct and helper function for Sifive UART (`sifive,uart0`).

## REF

- <https://static.dev.sifive.com/SiFive-E300-platform-reference-manual-v1.0.1.pdf>
*/

#![no_std]

#[macro_use]
extern crate bitflags;

pub mod registers;
pub mod uart;

pub use uart::MmioUartSifive;
