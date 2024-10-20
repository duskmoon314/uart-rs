#[cfg(feature = "fmt")]
use core::fmt;

use super::registers::Registers;

bitflags! {
    /// Status Register Bit Definitions
    pub struct Status: u8 {
        const RX_FIFO_VALID     = 0b0000_0001;
        const RX_FIFO_FULL      = 0b0000_0010;
        const TX_FIFO_EMPTY     = 0b0000_0100;
        const TX_FIFO_FULL      = 0b0000_1000;
        const INTERRUPT_ENABLED = 0b0001_0000;
        const OVERRUN_ERROR     = 0b0010_0000;
        const FRAME_ERROR       = 0b0100_0000;
        const PARITY_ERROR      = 0b1000_0000;
    }
}

bitflags! {
    /// Control Register Bit Definitions
    struct Control: u8 {
        const ENABLE_INTERRUPT  = 0b0001_0000;
        const REST_RX_FIFO      = 0b0000_0010;
        const REST_TX_FIFO      = 0b0000_0001;
    }
}

/// # MMIO version of XPS UART Lite
///
/// **Noticed** This hasn't been tested.
pub struct MmioUartXpsLite {
    reg_pointer: *mut Registers,
}

impl MmioUartXpsLite {
    /// New a uart
    pub const fn new(base_address: usize) -> Self {
        Self {
            reg_pointer: base_address as _,
        }
    }

    pub fn reg(&self) -> &mut Registers {
        unsafe { &mut *self.reg_pointer }
    }

    /// Set a new base_address
    pub fn set_base_address(&mut self, base_address: usize) {
        self.reg_pointer = base_address as _;
    }

    /// Read a byte
    pub fn read_byte(&self) -> Option<u8> {
        if self.is_rx_fifo_valid() {
            Some(self.read_rx().reverse_bits() as u8)
        } else {
            None
        }
    }

    /// Write a byte
    pub fn write_byte(&self, value: u8) {
        self.write_tx((value as u32).reverse_bits())
    }

    /// Read Rx FIFO
    #[inline]
    pub fn read_rx(&self) -> u32 {
        self.reg().rx.read()
    }

    /// Write Tx FIFO
    #[inline]
    pub fn write_tx(&self, value: u32) {
        unsafe { self.reg().tx.write(value) }
    }

    /// Read Uart Lite Status Register
    #[inline]
    pub fn read_stat(&self) -> u32 {
        self.reg().stat.read()
    }

    /// Get Uart Lite Status
    #[inline]
    pub fn status(&self) -> Status {
        Status::from_bits_truncate(self.reg().stat.read().reverse_bits() as u8)
    }

    pub fn is_rx_fifo_valid(&self) -> bool {
        self.status().contains(Status::RX_FIFO_VALID)
    }

    pub fn is_rx_fifo_full(&self) -> bool {
        self.status().contains(Status::RX_FIFO_FULL)
    }

    pub fn is_tx_fifo_empty(&self) -> bool {
        self.status().contains(Status::TX_FIFO_EMPTY)
    }

    pub fn is_tx_fifo_full(&self) -> bool {
        self.status().contains(Status::TX_FIFO_FULL)
    }

    pub fn is_interrupt_enabled(&self) -> bool {
        self.status().contains(Status::INTERRUPT_ENABLED)
    }

    pub fn is_overrun_error(&self) -> bool {
        self.status().contains(Status::OVERRUN_ERROR)
    }

    pub fn is_frame_error(&self) -> bool {
        self.status().contains(Status::FRAME_ERROR)
    }

    pub fn is_parity_error(&self) -> bool {
        self.status().contains(Status::PARITY_ERROR)
    }

    /// Write Uart Lite Control Register
    #[inline]
    pub fn write_ctrl(&self, value: u32) {
        unsafe { self.reg().ctrl.write(value) }
    }

    pub fn enable_interrupt(&self) {
        self.write_ctrl((Control::ENABLE_INTERRUPT.bits() as u32).reverse_bits());
    }

    pub fn disable_interrupt(&self) {
        self.write_ctrl((Control::ENABLE_INTERRUPT.bits() as u32).reverse_bits());
    }

    pub fn clear_rx_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            ((Control::ENABLE_INTERRUPT | Control::REST_RX_FIFO).bits() as u32).reverse_bits()
        } else {
            (Control::REST_RX_FIFO.bits() as u32).reverse_bits()
        });
    }

    pub fn clear_tx_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            ((Control::ENABLE_INTERRUPT | Control::REST_TX_FIFO).bits() as u32).reverse_bits()
        } else {
            (Control::REST_TX_FIFO.bits() as u32).reverse_bits()
        });
    }

    pub fn clear_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            ((Control::ENABLE_INTERRUPT | Control::REST_RX_FIFO | Control::REST_TX_FIFO).bits()
                as u32)
                .reverse_bits()
        } else {
            ((Control::REST_RX_FIFO | Control::REST_TX_FIFO).bits() as u32).reverse_bits()
        });
    }
}

/// ## fmt::Write
///
/// A simple implementation, may be changed in the future
#[cfg(feature = "fmt")]
impl<'a> fmt::Write for MmioUartXpsLite<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            self.write_byte(*c);
        }
        Ok(())
    }
}

/// # MMIO version of AXI UART Lite
///
/// **Noticed** This hasn't been tested.
pub struct MmioUartAxiLite {
    reg_pointer: *mut Registers,
}

impl MmioUartAxiLite {
    /// New a uart
    pub const fn new(base_address: usize) -> Self {
        Self {
            reg_pointer: base_address as _,
        }
    }

    pub fn reg(&self) -> &mut Registers {
        unsafe { &mut *self.reg_pointer }
    }

    /// Set a new base_address
    pub fn set_base_address(&mut self, base_address: usize) {
        self.reg_pointer = base_address as _;
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

    /// Read a slice
    pub fn read(&self, buf: &mut [u8]) -> usize {
        let mut count = 0;
        for current in buf {
            if let Some(ch) = self.read_byte() {
                count += 1;
                *current = ch;
            } else {
                break;
            }
        }
        count
    }

    /// Write a slice
    pub fn write(&self, buf: &[u8]) -> usize {
        let mut count = 0;
        for current in buf {
            if self.is_tx_fifo_full() {
                break;
            }
            count += 1;
            self.write_byte(*current);
        }
        count
    }

    /// Read Rx FIFO
    #[inline]
    pub fn read_rx(&self) -> u32 {
        self.reg().rx.read()
    }

    /// Write Tx FIFO
    #[inline]
    pub fn write_tx(&self, value: u32) {
        unsafe { self.reg().tx.write(value) }
    }

    /// Read Uart Lite Status Register
    #[inline]
    pub fn read_stat(&self) -> u32 {
        self.reg().stat.read()
    }

    /// Get Uart Lite Status
    #[inline]
    pub fn status(&self) -> Status {
        Status::from_bits_truncate(self.reg().stat.read() as u8)
    }

    pub fn is_rx_fifo_valid(&self) -> bool {
        self.status().contains(Status::RX_FIFO_VALID)
    }

    pub fn is_rx_fifo_full(&self) -> bool {
        self.status().contains(Status::RX_FIFO_FULL)
    }

    pub fn is_tx_fifo_empty(&self) -> bool {
        self.status().contains(Status::TX_FIFO_EMPTY)
    }

    pub fn is_tx_fifo_full(&self) -> bool {
        self.status().contains(Status::TX_FIFO_FULL)
    }

    pub fn is_interrupt_enabled(&self) -> bool {
        self.status().contains(Status::INTERRUPT_ENABLED)
    }

    pub fn is_overrun_error(&self) -> bool {
        self.status().contains(Status::OVERRUN_ERROR)
    }

    pub fn is_frame_error(&self) -> bool {
        self.status().contains(Status::FRAME_ERROR)
    }

    pub fn is_parity_error(&self) -> bool {
        self.status().contains(Status::PARITY_ERROR)
    }

    /// Write Uart Lite Control Register
    #[inline]
    pub fn write_ctrl(&self, value: u32) {
        unsafe { self.reg().ctrl.write(value) }
    }

    pub fn enable_interrupt(&self) {
        self.write_ctrl(Control::ENABLE_INTERRUPT.bits() as u32);
    }

    pub fn disable_interrupt(&self) {
        self.write_ctrl(Control::ENABLE_INTERRUPT.bits() as u32);
    }

    pub fn clear_rx_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            (Control::ENABLE_INTERRUPT | Control::REST_RX_FIFO).bits() as u32
        } else {
            Control::REST_RX_FIFO.bits() as u32
        });
    }

    pub fn clear_tx_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            (Control::ENABLE_INTERRUPT | Control::REST_TX_FIFO).bits() as u32
        } else {
            Control::REST_TX_FIFO.bits() as u32
        });
    }

    pub fn clear_fifo(&self, enable_interrupt: bool) {
        self.write_ctrl(if enable_interrupt {
            (Control::ENABLE_INTERRUPT | Control::REST_RX_FIFO | Control::REST_TX_FIFO).bits()
                as u32
        } else {
            (Control::REST_RX_FIFO | Control::REST_TX_FIFO).bits() as u32
        });
    }
}

/// ## fmt::Write
///
/// A simple implementation, may be changed in the future
#[cfg(feature = "fmt")]
impl<'a> fmt::Write for MmioUartAxiLite<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            self.write_byte(*c);
        }
        Ok(())
    }
}
