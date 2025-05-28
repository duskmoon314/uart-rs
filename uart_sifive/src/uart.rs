use super::registers::Registers;

bitflags! {
    /// TxData Register
    pub struct TxData: u32 {
        const FULL = 1 << 31;
        // const DATA = 0b1111_1111;
    }

    /// RxData Register
    pub struct RxData: u32 {
        const EMPTY = 1 << 31;
        // const DATA = 0b1111_1111;
    }

    /// TxControl Register
    pub struct TxControl: u32 {
        const ENABLE = 0b01;
        const NSTOP  = 0b10;
        // const COUNT  = 0b111 << 15;
    }

    /// RxControl Register
    pub struct RxControl: u32 {
        const ENABLE = 0b01;
        const NSTOP  = 0b10;
        // const COUNT  = 0b111 << 15;
    }

    /// This sturct use for `ie` and `ip` register
    pub struct InterruptRegister: u32 {
        const RXWM = 0b10;
        const TXWM = 0b01;
    }

    // struct DivRegister: u32 {
    //     const div = 1 << 16 - 1;
    // }
}

/// # MMIO version of Sifive UART
///
/// **Noticed** This hasn't been tested.
pub struct MmioUartSifive {
    reg_pointer: *mut Registers,
}

impl MmioUartSifive {
    /// New a uart
    pub const fn new(base_address: usize) -> Self {
        Self {
            reg_pointer: base_address as _,
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn reg(&self) -> &mut Registers {
        unsafe { &mut *self.reg_pointer }
    }

    /// Set a new base_address
    pub fn set_base_address(&mut self, base_address: usize) {
        self.reg_pointer = base_address as _;
    }

    /// Read a byte
    pub fn read_byte(&self) -> Option<u8> {
        let rx = self.read_rx();
        let rx_empty = RxData::from_bits_truncate(rx).contains(RxData::EMPTY);
        if !rx_empty {
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
    #[inline]
    pub fn read_rx(&self) -> u32 {
        self.reg().rx.read()
    }

    /// Read Tx Status
    #[inline]
    pub fn read_tx(&self) -> u32 {
        self.reg().tx.read()
    }

    /// Write Tx FIFO
    #[inline]
    pub fn write_tx(&self, value: u32) {
        unsafe { self.reg().tx.write(value) }
    }

    /// Read RxCtrl
    #[inline]
    pub fn read_rxctrl(&self) -> u32 {
        self.reg().rxctrl.read()
    }

    /// Write RxCtrl
    #[inline]
    pub fn write_rxctrl(&self, value: u32) {
        unsafe { self.reg().rxctrl.write(value) }
    }

    /// Read TxCtrl
    #[inline]
    pub fn read_txctrl(&self) -> u32 {
        self.reg().txctrl.read()
    }

    /// Write TxCtrl
    #[inline]
    pub fn write_txctrl(&self, value: u32) {
        unsafe { self.reg().txctrl.write(value) }
    }

    /// Read ip register
    #[inline]
    pub fn read_ip(&self) -> InterruptRegister {
        InterruptRegister::from_bits_truncate(self.reg().ip.read())
    }

    /// Read ie register
    #[inline]
    pub fn read_ie(&self) -> InterruptRegister {
        InterruptRegister::from_bits_truncate(self.reg().ie.read())
    }

    /// Write ie register
    #[inline]
    pub fn write_ie(&self, value: u32) {
        unsafe { self.reg().ie.write(value) }
    }

    /// Read div register
    #[inline]
    pub fn read_div(&self) -> u32 {
        self.reg().div.read()
    }

    /// Write div register
    #[inline]
    pub fn write_div(&self, value: u32) {
        unsafe { self.reg().div.write(value) }
    }

    /// Check if tx FIFO is full
    pub fn is_tx_fifo_full(&self) -> bool {
        TxData::from_bits_truncate(self.read_tx()).contains(TxData::FULL)
    }

    /// Check if read interrupt has been enable
    pub fn is_read_interrupt_enabled(&self) -> bool {
        self.read_ie().contains(InterruptRegister::RXWM)
    }

    /// Check if write interrupt has been enable
    pub fn is_write_interrupt_enabled(&self) -> bool {
        self.read_ie().contains(InterruptRegister::TXWM)
    }

    /// Enable write
    pub fn enable_write(&self) {
        self.write_txctrl(self.read_txctrl() | TxControl::ENABLE.bits())
    }

    /// Enable read
    pub fn enable_read(&self) {
        self.write_rxctrl(self.read_rxctrl() | RxControl::ENABLE.bits())
    }

    /// Disable write
    pub fn disable_write(&self) {
        self.write_txctrl(self.read_txctrl() & !TxControl::ENABLE.bits())
    }

    /// Disable read
    pub fn disable_read(&self) {
        self.write_rxctrl(self.read_rxctrl() & !RxControl::ENABLE.bits())
    }

    /// Disable all interrupt
    pub fn disable_interrupt(&self) {
        self.write_ie(0)
    }

    /// Enable read interrupt (and keep other bit in ie register)
    pub fn enable_read_interrupt(&self) {
        self.write_ie((self.read_ie() | InterruptRegister::RXWM).bits() as u32)
    }

    /// Enable write interrupt (and keep other bit in ie register)
    pub fn enable_write_interrupt(&self) {
        self.write_ie((self.read_ie() | InterruptRegister::TXWM).bits() as u32)
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
}
