/*!
# uart_16550

This mod provide structures with many methods to operate AXI Uart 16550
*/

#[macro_use]
pub mod registers;
pub mod uart;

pub use uart::{InterruptType, MmioUartAxi16550, Parity};
