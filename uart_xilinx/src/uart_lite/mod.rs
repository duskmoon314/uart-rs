/*!
# uart_lite

This mod provide structures with many methods to operate XPS Uart Lite and AXI Uart Lite
*/

#[macro_use]
pub mod registers;
pub mod uart;

pub use uart::{MmioUartAxiLite, MmioUartXpsLite, Status};
