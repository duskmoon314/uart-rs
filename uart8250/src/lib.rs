/*!
# uart8250

This crate provides a struct with many methods to operate an 8250 UART.

[REF: Serial Programming/8250 UART Programming](https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming#UART_Registers)
*/

#![no_std]

mod registers;
mod uart;

use core::ops::{Deref, DerefMut};
pub use uart::{ChipFifoInfo, InterruptType, Parity, TransmitError};
use crate::registers::{Register};

pub struct MmioUart8250<R:Register + Copy+'static>(uart::MmioUart8250<'static, R>);


unsafe impl<R: Register + Copy + 'static> Send for MmioUart8250<R> {}
unsafe impl<R: Register + Copy + 'static> Sync for MmioUart8250<R> {}


impl MmioUart8250<u32> {
    pub fn new(base_addr: usize) -> Self {
        let uart_raw = unsafe { uart::MmioUart8250::<u32>::new(base_addr) };
        MmioUart8250(uart_raw)
    }
    pub fn set_base_address(&mut self, base_address: usize) {
        unsafe { self.0.set_base_address(base_address); }
    }
}

impl Deref for MmioUart8250<u32> {
    type Target = uart::MmioUart8250<'static, u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MmioUart8250<u32> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}