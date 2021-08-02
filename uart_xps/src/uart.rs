#[cfg(feature = "fmt")]
use core::fmt;

use crate::registers::Registers;

/// # MMIO version of uart XPS Lite
///
/// **Noticed** This hasn't been tested.
pub struct MmioUartXPS<'a> {
    reg: &'a mut Registers,
}

impl<'a> MmioUartXPS<'a> {
    /// New a uart
    pub fn new(base_address: usize) -> Self {
        Self {
            reg: cast!(base_address),
        }
    }

    /// Set a new base_address
    pub fn set_base_address(&mut self, base_address: usize) {
        self.reg = cast!(base_address);
    }

    /// Read a byte
    pub fn read_byte(&self) -> Option<u8> {
        if self.is_rx_fifo_valid() {
            Some(self.read_rx() as u8)
        } else {
            None
        }
    }

    /// Write a byte
    pub fn write_byte(&self, value: u8) {
        self.write_tx(value as u32)
    }

    /// Read Rx FIFO
    pub fn read_rx(&self) -> u32 {
        self.reg.rx.read()
    }

    /// Write Tx FIFO
    #[inline]
    pub fn write_tx(&self, value: u32) {
        unsafe { self.reg.tx.write(value) }
    }

    /// Read Uart Lite Status Register
    #[inline]
    pub fn read_stat(&self) -> u32 {
        self.reg.stat.read()
    }

    pub fn is_rx_fifo_valid(&self) -> bool {
        self.read_stat() & 0x8000_0000 != 0
    }

    pub fn is_rx_fifo_full(&self) -> bool {
        self.read_stat() & 0x4000_0000 != 0
    }

    pub fn is_tx_fifo_empty(&self) -> bool {
        self.read_stat() & 0x2000_0000 != 0
    }

    pub fn is_tx_fifo_full(&self) -> bool {
        self.read_stat() & 0x1000_0000 != 0
    }

    pub fn is_interrupt_enabled(&self) -> bool {
        self.read_stat() & 0x0800_0000 != 0
    }

    pub fn is_overrun_error(&self) -> bool {
        self.read_stat() & 0x0400_0000 != 0
    }

    pub fn is_frame_error(&self) -> bool {
        self.read_stat() & 0x0200_0000 != 0
    }

    pub fn is_parity_error(&self) -> bool {
        self.read_stat() & 0x0100_0000 != 0
    }

    /// Write Uart Lite Control Register
    #[inline]
    pub fn write_ctrl(&self, value: u32) {
        unsafe { self.reg.ctrl.write(value) }
    }

    pub fn enable_interrupt(&self) {
        self.write_ctrl(0x0800_0000)
    }

    pub fn clear_rx_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            0x4800_0000
        } else {
            0x4000_0000
        })
    }

    pub fn clear_tx_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            0x8800_0000
        } else {
            0x8000_0000
        })
    }

    pub fn clear_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            0xC800_0000
        } else {
            0xC000_0000
        })
    }
}

/// ## fmt::Write
///
/// A simple implementation, may be changed in the future
#[cfg(feature = "fmt")]
impl<'a> fmt::Write for MmioUartXPS<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            self.write_byte(*c);
        }
        Ok(())
    }
}
