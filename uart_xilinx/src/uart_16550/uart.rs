#[cfg(feature = "fmt")]
use core::fmt;

use super::registers::Registers;

bitflags! {
    /// Interrupt Enable Register (bitflags)
    pub struct IER: u8 {
        /// Enable Received Data Available Interrupt
        const ERBFI = 0b0000_0001;
        /// Enable Transmitter Holding Register Empty Interrupt
        const ETBEI = 0b0000_0010;
        /// Enable Receiver Line Status Interrupt
        const ELSI  = 0b0000_0100;
        /// Enable Modem Status Interrupt
        const EDSSI = 0b0000_1000;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterruptType {
    ModemStatus,
    TransmitterHoldingRegisterEmpty,
    ReceivedDataAvailable,
    ReceiverLineStatus,
    Timeout,
    Reserved,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Parity {
    No,
    Odd,
    Even,
    Mark,
    Space,
}

/// # MMIO version of AXI UART 16550
///
/// **Noticed** This hasn't been tested.
pub struct MmioUartAxi16550<'a> {
    reg: &'a mut Registers,
}

impl<'a> MmioUartAxi16550<'a> {
    /// New a uart
    pub fn new(base_address: usize) -> Self {
        Self {
            reg: cast!(base_address),
        }
    }

    /// A basic way to init the uart with interrupt enable
    ///
    /// Other way to init can be done by using other methods below
    pub fn init(&self, clock: usize, baud_rate: usize) {
        // Enable DLAB and Set divisor
        self.set_divisor(clock, baud_rate);

        // Disable DLAB and set word length 8 bits, no parity, 1 stop bit
        self.write_lcr(3);
        // Enable FIFO
        self.write_fcr(1);
        // No modem control
        self.write_mcr(0);
        // Enable received_data_available_interrupt
        self.enable_received_data_available_interrupt();
        // Enable transmitter_holding_register_empty_interrupt
        // self.enable_transmitter_holding_register_empty_interrupt();
    }

    /// Set a new base_address
    pub fn set_base_address(&mut self, base_address: usize) {
        self.reg = cast!(base_address);
    }

    /// Read a byte from uart
    ///
    /// Return `None` when data is not ready (RBR\[0\] != 1)
    pub fn read_byte(&self) -> Option<u8> {
        if self.is_data_ready() {
            Some(self.read_rbr() as u8)
        } else {
            None
        }
    }

    /// Write a byte to uart
    ///
    /// Error are not concerned now **MAYBE TODO**
    pub fn write_byte(&self, byte: u8) {
        self.write_thr(byte as u32);
    }

    /// write THR (offset + 0x0)
    ///
    /// Write Transmitter Holding Buffer to send data
    #[inline]
    pub fn write_thr(&self, value: u32) {
        unsafe { self.reg.rw[0].write(value) }
    }

    /// read RBR (offset + 0x0)
    ///
    /// Read Receiver Buffer to get data
    #[inline]
    pub fn read_rbr(&self) -> u32 {
        self.reg.rw[0].read()
    }

    /// read DLL (offset + 0x0)
    ///
    /// get divisor latch low byte in the register
    #[inline]
    pub fn read_dll(&self) -> u32 {
        self.reg.rw[0].read()
    }

    /// write DLL (offset + 0x0)
    ///
    /// set divisor latch low byte in the register
    #[inline]
    pub fn write_dll(&self, value: u32) {
        unsafe { self.reg.rw[0].write(value) }
    }

    /// read DLH (offset + 0x4)
    ///
    /// get divisor latch high byte in the register
    #[inline]
    pub fn read_dlh(&self) -> u32 {
        self.reg.rw[1].read()
    }

    /// write DLH (offset + 0x4)
    ///
    /// set divisor latch high byte in the register
    #[inline]
    pub fn write_dlh(&self, value: u32) {
        unsafe { self.reg.rw[1].write(value) }
    }

    /// Set divisor latch according to clock and baud_rate, then set DLAB to false
    #[inline]
    pub fn set_divisor(&self, clock: usize, baud_rate: usize) {
        self.enable_divisor_latch_accessible();
        let divisor = clock / (16 * baud_rate);
        self.write_dll((divisor & 0b1111_1111) as u32);
        self.write_dlh(((divisor >> 8) & 0b1111_1111) as u32);
        self.disable_divisor_latch_accessible();
    }

    /// Read IER (offset + 0x4)
    ///
    /// Read IER to get what interrupts are enabled
    #[inline]
    pub fn read_ier(&self) -> u32 {
        self.reg.rw[1].read()
    }

    /// Write IER (offset + 0x4)
    ///
    /// Write Interrupt Enable Register to turn on/off interrupts
    #[inline]
    pub fn write_ier(&self, value: u32) {
        unsafe { self.reg.rw[1].write(value) }
    }

    /// Get IER bitflags
    pub fn ier(&self) -> IER {
        IER::from_bits_truncate(self.read_ier() as u8)
    }

    /// Set IER via bitflags
    pub fn set_ier(&self, flag: IER) {
        self.write_ier(flag.bits() as u32)
    }

    /// get whether modem status interrupt is enabled (IER\[3\])
    pub fn is_modem_status_interrupt_enabled(&self) -> bool {
        self.reg.rw[1].read() & 0b0000_1000 != 0
    }

    /// toggle modem status interrupt (IER\[3\])
    pub fn toggle_modem_status_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v ^ 0b0000_1000) }
    }

    /// enable modem status interrupt (IER\[3\])
    pub fn enable_modem_status_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v | 0b0000_1000) }
    }

    /// disable modem status interrupt (IER\[3\])
    pub fn disable_modem_status_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v & !0b0000_1000) }
    }

    /// get whether receiver line status interrupt is enabled (IER\[2\])
    pub fn is_receiver_line_status_interrupt_enabled(&self) -> bool {
        self.reg.rw[1].read() & 0b0000_0100 != 0
    }

    /// toggle receiver line status interrupt (IER\[2\])
    pub fn toggle_receiver_line_status_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v ^ 0b0000_0100) }
    }

    /// enable receiver line status interrupt (IER\[2\])
    pub fn enable_receiver_line_status_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v | 0b0000_0100) }
    }

    /// disable receiver line status interrupt (IER\[2\])
    pub fn disable_receiver_line_status_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v & !0b0000_0100) }
    }

    /// get whether transmitter holding register empty interrupt is enabled (IER\[1\])
    pub fn is_transmitter_holding_register_empty_interrupt_enabled(&self) -> bool {
        self.reg.rw[1].read() & 0b0000_0010 != 0
    }

    /// toggle transmitter holding register empty interrupt (IER\[1\])
    pub fn toggle_transmitter_holding_register_empty_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v ^ 0b0000_0010) }
    }

    /// enable transmitter holding register empty interrupt (IER\[1\])
    pub fn enable_transmitter_holding_register_empty_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v | 0b0000_0010) }
    }

    /// disable transmitter holding register empty interrupt (IER\[1\])
    pub fn disable_transmitter_holding_register_empty_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v & !0b0000_0010) }
    }

    /// get whether received data available is enabled (IER\[0\])
    pub fn is_received_data_available_interrupt_enabled(&self) -> bool {
        self.reg.rw[1].read() & 0b0000_0001 != 0
    }

    /// toggle received data available (IER\[0\])
    pub fn toggle_received_data_available_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v ^ 0b0000_0001) }
    }

    /// enable received data available (IER\[0\])
    pub fn enable_received_data_available_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v | 0b0000_0001) }
    }

    /// disable received data available (IER\[0\])
    pub fn disable_received_data_available_interrupt(&self) {
        unsafe { self.reg.rw[1].modify(|v| v & !0b0000_0001) }
    }

    /// Read IIR (offset + 0x8)
    #[inline]
    pub fn read_iir(&self) -> u32 {
        self.reg.rw[2].read()
    }

    /// Read IIR\[7:6\] to get whether FIFO is enabled
    pub fn is_fifo_enabled(&self) -> bool {
        self.reg.rw[2].read() & 0b1100_0000 != 0
    }

    /// Read IIR\[3:1\] to get interrupt type
    pub fn read_interrupt_type(&self) -> InterruptType {
        match self.reg.rw[2].read() & 0b0000_1110 {
            0b0000 => InterruptType::ModemStatus,
            0b0010 => InterruptType::TransmitterHoldingRegisterEmpty,
            0b0100 => InterruptType::ReceivedDataAvailable,
            0b0110 => InterruptType::ReceiverLineStatus,
            0b1100 => InterruptType::Timeout,
            0b1000 | 0b1010 | 0b1110 => InterruptType::Reserved,
            _ => panic!("Can't reached"),
        }
    }

    /// get whether interrupt is pending (IIR\[0\])
    pub fn is_interrupt_pending(&self) -> bool {
        self.reg.rw[2].read() & 1 == 0
    }

    /// Read FCR (offset + 0x8)
    ///
    /// # Safety
    ///
    /// In Xilinx doc, FCR can be read when DLAB == 1.
    /// Please make sure DLAB is 1 when using this method.
    #[inline]
    pub unsafe fn read_fcr(&self) -> u32 {
        self.reg.rw[2].read()
    }

    /// Write FCR (offset + 0x8) to control FIFO buffers
    #[inline]
    pub fn write_fcr(&self, value: u32) {
        unsafe { self.reg.rw[2].write(value) }
    }

    /// Read LCR (offset + 0xc)
    ///
    /// Read Line Control Register to get the data protocol and DLAB
    #[inline]
    pub fn read_lcr(&self) -> u32 {
        self.reg.rw[3].read()
    }

    /// Write LCR (offset + 0xc)
    ///
    /// Write Line Control Register to set DLAB and the serial data protocol
    #[inline]
    pub fn write_lcr(&self, value: u32) {
        unsafe { self.reg.rw[3].write(value) }
    }

    /// get whether DLAB is enabled
    pub fn is_divisor_latch_accessible(&self) -> bool {
        self.reg.rw[3].read() & 0b1000_0000 != 0
    }

    /// toggle DLAB
    pub fn toggle_divisor_latch_accessible(&self) {
        unsafe { self.reg.rw[3].modify(|v| v ^ 0b1000_0000) }
    }

    /// enable DLAB
    #[inline]
    pub fn enable_divisor_latch_accessible(&self) {
        unsafe { self.reg.rw[3].modify(|v| v | 0b1000_0000) }
    }

    /// disable DLAB
    #[inline]
    pub fn disable_divisor_latch_accessible(&self) {
        unsafe { self.reg.rw[3].modify(|v| v & !0b1000_0000) }
    }

    /// get parity of used data protocol
    pub fn get_parity(&self) -> Parity {
        match self.reg.rw[3].read() & 0b0011_1000 {
            0b0000_0000 => Parity::No,
            0b0000_1000 => Parity::Odd,
            0b0001_1000 => Parity::Even,
            0b0010_1000 => Parity::Mark,
            0b0011_1000 => Parity::Space,
            _ => panic!("Invalid Parity! Please check your uart"),
        }
    }

    /// set parity
    pub fn set_parity(&self, parity: Parity) {
        match parity {
            Parity::No => unsafe { self.reg.rw[3].modify(|v| (v & 0b1100_0111)) },
            Parity::Odd => unsafe { self.reg.rw[3].modify(|v| (v & 0b1100_0111) | 0b0000_1000) },
            Parity::Even => unsafe { self.reg.rw[3].modify(|v| (v & 0b1100_0111) | 0b0001_1000) },
            Parity::Mark => unsafe { self.reg.rw[3].modify(|v| (v & 0b1100_0111) | 0b0010_1000) },
            Parity::Space => unsafe { self.reg.rw[3].modify(|v| v | 0b0011_1000) },
        }
    }

    /// get stop bit of used data protocol
    ///
    /// Simply return a u8 to indicate 1 or 1.5/2 bits
    pub fn get_stop_bit(&self) -> u32 {
        ((self.reg.rw[3].read() & 0b100) >> 2) + 1
    }

    /// set stop bit, only 1 and 2 can be used as `stop_bit`
    pub fn set_stop_bit(&self, stop_bit: u32) {
        match stop_bit {
            1 => unsafe { self.reg.rw[3].modify(|v| v & 0b1111_1011) },
            2 => unsafe { self.reg.rw[3].modify(|v| v | 0b0000_0100) },
            _ => panic!("Invalid stop bit"),
        }
    }

    /// get word length of used data protocol
    pub fn get_word_length(&self) -> u32 {
        (self.reg.rw[3].read() & 0b11) + 5
    }

    /// set word length, only 5..=8 can be used as `length`
    pub fn set_word_length(&self, length: u32) {
        if (5..=8).contains(&length) {
            unsafe { self.reg.rw[3].modify(|v| v | (length - 5)) }
        } else {
            panic!("Invalid word length")
        }
    }

    /// Read MCR (offset + 0x10)
    ///
    /// Read Modem Control Register to get how flow is controlled
    #[inline]
    pub fn read_mcr(&self) -> u32 {
        self.reg.rw[4].read()
    }

    /// Write MCR (offset + 0x10)
    ///
    /// Write Modem Control Register to control flow
    #[inline]
    pub fn write_mcr(&self, value: u32) {
        unsafe { self.reg.rw[4].write(value) }
    }

    /// Read LSR (offset + 0x14)
    #[inline]
    pub fn read_lsr(&self) -> u32 {
        self.reg.ro[0].read()
    }

    /// get whether there is an error in received FIFO
    pub fn is_received_fifo_error(&self) -> bool {
        self.reg.ro[0].read() & 0b1000_0000 != 0
    }

    /// get whether data holding registers are empty
    pub fn is_data_holding_registers_empty(&self) -> bool {
        self.reg.ro[0].read() & 0b0100_0000 != 0
    }

    /// get whether transmitter holding register is empty
    pub fn is_transmitter_holding_register_empty(&self) -> bool {
        self.reg.ro[0].read() & 0b0010_0000 != 0
    }

    pub fn is_break_interrupt(&self) -> bool {
        self.reg.ro[0].read() & 0b0001_0000 != 0
    }

    pub fn is_framing_error(&self) -> bool {
        self.reg.ro[0].read() & 0b0000_1000 != 0
    }

    pub fn is_parity_error(&self) -> bool {
        self.reg.ro[0].read() & 0b0000_0100 != 0
    }

    pub fn is_overrun_error(&self) -> bool {
        self.reg.ro[0].read() & 0b0000_0010 != 0
    }

    pub fn is_data_ready(&self) -> bool {
        self.reg.ro[0].read() & 0b0000_0001 != 0
    }

    /// Read MSR (offset + 0x18)
    #[inline]
    pub fn read_msr(&self) -> u32 {
        self.reg.ro[1].read()
    }

    pub fn is_carrier_detect(&self) -> bool {
        self.reg.ro[1].read() & 0b1000_0000 != 0
    }

    pub fn is_ring_indicator(&self) -> bool {
        self.reg.ro[1].read() & 0b0100_0000 != 0
    }

    pub fn is_data_set_ready(&self) -> bool {
        self.reg.ro[1].read() & 0b0010_0000 != 0
    }

    pub fn is_clear_to_send(&self) -> bool {
        self.reg.ro[1].read() & 0b0001_0000 != 0
    }

    pub fn is_delta_data_carrier_detect(&self) -> bool {
        self.reg.ro[1].read() & 0b0000_1000 != 0
    }

    pub fn is_trailing_edge_ring_indicator(&self) -> bool {
        self.reg.ro[1].read() & 0b0000_0100 != 0
    }

    pub fn is_delta_data_set_ready(&self) -> bool {
        self.reg.ro[1].read() & 0b0000_0010 != 0
    }

    pub fn is_delta_clear_to_send(&self) -> bool {
        self.reg.ro[1].read() & 0b0000_0001 != 0
    }

    #[inline]
    pub fn read_sr(&self) -> u32 {
        self.reg.scratch.read()
    }

    #[inline]
    pub fn write_sr(&self, value: u32) {
        unsafe { self.reg.scratch.write(value) }
    }
}

/// ## fmt::Write
///
/// A simple implementation, may be changed in the future
#[cfg(feature = "fmt")]
impl<'a> fmt::Write for MmioUartAxi16550<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            self.write_thr((*c) as u32);
        }
        Ok(())
    }
}
