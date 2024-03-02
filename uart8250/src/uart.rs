#[cfg(feature = "embedded")]
use core::convert::Infallible;
use core::fmt::{self, Display, Formatter};
use tock_registers::{
    fields::FieldValue,
    interfaces::{ReadWriteable, Readable, Writeable},
};

use crate::registers::{Registers, FCR, IER, IIR, LCR, LSR, MSR};

pub type ChipFifoInfo = IIR::FifoInfo::Value;
pub type InterruptType = IIR::InterruptType::Value;
pub type Parity = LCR::Parity::Value;

/// An error encountered which trying to transmit data.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TransmitError {
    /// The transmit buffer is full, try again later.
    BufferFull,
}

impl Display for TransmitError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::BufferFull => "UART buffer full",
        })
    }
}

/// # MMIO version of an 8250 UART.
///
/// **Note** This is only tested on the NS16550 compatible UART used in QEMU 5.0 virt machine of RISC-V.
pub struct MmioUart8250<'a> {
    reg: &'a mut Registers,
}

impl<'a> MmioUart8250<'a> {
    /// Creates a new UART.
    ///
    /// # Safety
    ///
    /// The given base address must point to the 8 MMIO control registers of an appropriate UART
    /// device, which must be mapped into the address space of the process as device memory and not
    /// have any other aliases.
    pub unsafe fn new(base_address: usize) -> Self {
        Self {
            reg: Registers::from_base_address(base_address),
        }
    }

    /// Initialises the UART with common settings and interrupts enabled.
    ///
    /// More customised initialisation can be done using other methods below.
    pub fn init(&self, clock: usize, baud_rate: usize) {
        // Enable DLAB and set divisor
        self.set_divisor(clock, baud_rate);

        // Disable DLAB and set word length 8 bits, no parity, 1 stop bit
        self.reg
            .lcr
            .write(LCR::Parity::No + LCR::STOP_BITS::One + LCR::WORD_LENGTH::Bits8);
        // Enable FIFO
        self.reg.iir_fcr.write(FCR::Enable::SET);
        // No modem control
        self.reg.mcr.write(FieldValue::<u8, _>::new(0, 0, 0));
        // Enable received_data_available_interrupt
        self.enable_received_data_available_interrupt();
        // Enable transmitter_holding_register_empty_interrupt
        // self.enable_transmitter_holding_register_empty_interrupt();
    }

    /// Sets a new base address for the UART.
    ///
    /// # Safety
    ///
    /// The given base address must point to the 8 MMIO control registers of an appropriate UART
    /// device, which must be mapped into the address space of the process as device memory and not
    /// have any other aliases.
    pub unsafe fn set_base_address(&mut self, base_address: usize) {
        self.reg = Registers::from_base_address(base_address);
    }

    /// Reads a byte from the UART.
    ///
    /// Returns `None` when data is not ready (RBR\[0\] != 1)
    pub fn read_byte(&self) -> Option<u8> {
        if self.is_data_ready() {
            Some(self.reg.thr_rbr_dll.get())
        } else {
            None
        }
    }

    /// Writes a byte to the UART.
    pub fn write_byte(&self, byte: u8) -> Result<(), TransmitError> {
        if self.is_transmitter_holding_register_empty() {
            self.reg.thr_rbr_dll.set(byte);
            Ok(())
        } else {
            Err(TransmitError::BufferFull)
        }
    }

    /// Sets DLAB to true, sets divisor latch according to clock and baud_rate, then sets DLAB to
    /// false.
    ///
    /// > ## Divisor Latch Bytes
    /// >
    /// > Offset: +0 and +1 . The Divisor Latch Bytes are what control the baud rate of the modem. As you might guess from the name of this register, it is used as a divisor to determine what baud rate that the chip is going to be transmitting at.
    ///
    /// Used clock 1.8432 MHz as example, first divide 16 and get 115200. Then use the formula to get divisor latch value:
    ///
    /// *DivisorLatchValue = 115200 / BaudRate*
    ///
    /// This gives the following table:
    ///
    /// | Baud Rate | Divisor (in decimal) | Divisor Latch High Byte | Divisor Latch Low Byte |
    /// | --------- | -------------------- | ----------------------- | ---------------------- |
    /// | 50        | 2304                 | $09                     | $00                    |
    /// | 110       | 1047                 | $04                     | $17                    |
    /// | 220       | 524                  | $02                     | $0C                    |
    /// | 300       | 384                  | $01                     | $80                    |
    /// | 600       | 192                  | $00                     | $C0                    |
    /// | 1200      | 96                   | $00                     | $60                    |
    /// | 2400      | 48                   | $00                     | $30                    |
    /// | 4800      | 24                   | $00                     | $18                    |
    /// | 9600      | 12                   | $00                     | $0C                    |
    /// | 19200     | 6                    | $00                     | $06                    |
    /// | 38400     | 3                    | $00                     | $03                    |
    /// | 57600     | 2                    | $00                     | $02                    |
    /// | 115200    | 1                    | $00                     | $01                    |
    #[inline]
    pub fn set_divisor(&self, clock: usize, baud_rate: usize) {
        // Enable DLAB.
        self.reg.lcr.modify(LCR::DLAB::SET);

        let divisor = clock / (16 * baud_rate);
        self.reg.thr_rbr_dll.set(divisor as u8);
        self.reg.ier_dlh.set((divisor >> 8) as u8);

        // Disable DLAB.
        self.reg.lcr.modify(LCR::DLAB::CLEAR);
    }

    /// get whether low power mode (16750) is enabled (IER\[5\])
    pub fn is_low_power_mode_enabled(&self) -> bool {
        self.reg.ier_dlh.is_set(IER::LPM)
    }

    /// enable low power mode (16750) (IER\[5\])
    pub fn enable_low_power_mode(&self) {
        self.reg.ier_dlh.modify(IER::LPM::SET)
    }

    /// disable low power mode (16750) (IER\[5\])
    pub fn disable_low_power_mode(&self) {
        self.reg.ier_dlh.modify(IER::LPM::CLEAR)
    }

    /// get whether sleep mode (16750) is enabled (IER\[4\])
    pub fn is_sleep_mode_enabled(&self) -> bool {
        self.reg.ier_dlh.is_set(IER::SM)
    }

    /// enable sleep mode (16750) (IER\[4\])
    pub fn enable_sleep_mode(&self) {
        self.reg.ier_dlh.modify(IER::SM::SET)
    }

    /// disable sleep mode (16750) (IER\[4\])
    pub fn disable_sleep_mode(&self) {
        self.reg.ier_dlh.modify(IER::SM::CLEAR)
    }

    /// get whether modem status interrupt is enabled (IER\[3\])
    pub fn is_modem_status_interrupt_enabled(&self) -> bool {
        self.reg.ier_dlh.is_set(IER::MSI)
    }

    /// enable modem status interrupt (IER\[3\])
    pub fn enable_modem_status_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::MSI::SET)
    }

    /// disable modem status interrupt (IER\[3\])
    pub fn disable_modem_status_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::MSI::CLEAR)
    }

    /// get whether receiver line status interrupt is enabled (IER\[2\])
    pub fn is_receiver_line_status_interrupt_enabled(&self) -> bool {
        self.reg.ier_dlh.is_set(IER::RLSI)
    }

    /// enable receiver line status interrupt (IER\[2\])
    pub fn enable_receiver_line_status_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::RLSI::SET)
    }

    /// disable receiver line status interrupt (IER\[2\])
    pub fn disable_receiver_line_status_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::RLSI::CLEAR)
    }

    /// get whether transmitter holding register empty interrupt is enabled (IER\[1\])
    pub fn is_transmitter_holding_register_empty_interrupt_enabled(&self) -> bool {
        self.reg.ier_dlh.is_set(IER::THREI)
    }

    /// enable transmitter holding register empty interrupt (IER\[1\])
    pub fn enable_transmitter_holding_register_empty_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::THREI::SET)
    }

    /// disable transmitter holding register empty interrupt (IER\[1\])
    pub fn disable_transmitter_holding_register_empty_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::THREI::CLEAR)
    }

    /// get whether received data available is enabled (IER\[0\])
    pub fn is_received_data_available_interrupt_enabled(&self) -> bool {
        self.reg.ier_dlh.is_set(IER::RDAI)
    }

    /// enable received data available (IER\[0\])
    pub fn enable_received_data_available_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::RDAI::SET);
    }

    /// disable received data available (IER\[0\])
    pub fn disable_received_data_available_interrupt(&self) {
        self.reg.ier_dlh.modify(IER::RDAI::CLEAR);
    }

    /// Read IIR\[7:6\] to get FIFO status
    pub fn read_fifo_status(&self) -> ChipFifoInfo {
        self.reg.iir_fcr.read_as_enum(IIR::FifoInfo).unwrap()
    }

    /// get whether 64 Byte fifo (16750 only) is enabled (IIR\[5\])
    pub fn is_64byte_fifo_enabled(&self) -> bool {
        self.reg.iir_fcr.is_set(IIR::Fifo64Byte)
    }

    /// Read IIR\[3:1\] to get interrupt type
    pub fn read_interrupt_type(&self) -> Option<InterruptType> {
        let iir = self.reg.iir_fcr.extract();
        if iir.is_set(IIR::InterruptPending) {
            None
        } else {
            Some(
                iir.read_as_enum(IIR::InterruptType)
                    .unwrap_or(IIR::InterruptType::Value::Reserved),
            )
        }
    }

    /// get parity of used data protocol
    pub fn get_parity(&self) -> Parity {
        self.reg
            .lcr
            .read_as_enum(LCR::Parity)
            .expect("Invalid Parity! Please check your UART.")
    }

    /// set parity
    pub fn set_parity(&self, parity: Parity) {
        self.reg.lcr.modify(LCR::Parity.val(parity as u8));
    }

    /// get stop bit of used data protocol
    ///
    /// Simply return a u8 to indicate 1 or 1.5/2 bits
    pub fn get_stop_bit(&self) -> u8 {
        self.reg.lcr.read(LCR::STOP_BITS) + 1
    }

    /// set stop bit, only 1 and 2 can be used as `stop_bit`
    pub fn set_stop_bit(&self, stop_bit: u8) {
        match stop_bit {
            1 => self.reg.lcr.modify(LCR::STOP_BITS::One),
            2 => self.reg.lcr.modify(LCR::STOP_BITS::Two),
            _ => panic!("Invalid stop bit"),
        }
    }

    /// get word length of used data protocol
    pub fn get_word_length(&self) -> u8 {
        self.reg.lcr.read(LCR::WORD_LENGTH) + 5
    }

    /// set word length, only 5..=8 can be used as `length`
    pub fn set_word_length(&self, length: u8) {
        if (5..=8).contains(&length) {
            self.reg.lcr.modify(LCR::WORD_LENGTH.val(length - 5))
        } else {
            panic!("Invalid word length")
        }
    }

    /// get whether there is an error in received FIFO
    pub fn is_received_fifo_error(&self) -> bool {
        self.reg.lsr.is_set(LSR::RFE)
    }

    /// Gets whether data holding registers are empty, i.e. the UART has finished transmitting all
    /// the data it has been given.
    pub fn is_data_holding_registers_empty(&self) -> bool {
        self.reg.lsr.is_set(LSR::DHRE)
    }

    /// Gets whether transmitter holding register is empty, i.e. the UART is ready to be given more
    /// data to transmit.
    pub fn is_transmitter_holding_register_empty(&self) -> bool {
        self.reg.lsr.is_set(LSR::THRE)
    }

    pub fn is_break_interrupt(&self) -> bool {
        self.reg.lsr.is_set(LSR::BI)
    }

    pub fn is_framing_error(&self) -> bool {
        self.reg.lsr.is_set(LSR::FE)
    }

    pub fn is_parity_error(&self) -> bool {
        self.reg.lsr.is_set(LSR::PE)
    }

    pub fn is_overrun_error(&self) -> bool {
        self.reg.lsr.is_set(LSR::OE)
    }

    pub fn is_data_ready(&self) -> bool {
        self.reg.lsr.is_set(LSR::DR)
    }

    pub fn is_carrier_detect(&self) -> bool {
        self.reg.msr.is_set(MSR::CD)
    }

    pub fn is_ring_indicator(&self) -> bool {
        self.reg.msr.is_set(MSR::RI)
    }

    pub fn is_data_set_ready(&self) -> bool {
        self.reg.msr.is_set(MSR::DSR)
    }

    pub fn is_clear_to_send(&self) -> bool {
        self.reg.msr.is_set(MSR::CTS)
    }

    pub fn is_delta_data_carrier_detect(&self) -> bool {
        self.reg.msr.is_set(MSR::DDCD)
    }

    pub fn is_trailing_edge_ring_indicator(&self) -> bool {
        self.reg.msr.is_set(MSR::TERI)
    }

    pub fn is_delta_data_set_ready(&self) -> bool {
        self.reg.msr.is_set(MSR::DDSR)
    }

    pub fn is_delta_clear_to_send(&self) -> bool {
        self.reg.msr.is_set(MSR::DCTS)
    }
}

/// ## fmt::Write
///
/// A simple implementation, may be changed in the future
#[cfg(feature = "fmt")]
impl<'a> fmt::Write for MmioUart8250<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            // If buffer is full, keep retrying.
            while self.write_byte(*c) == Err(TransmitError::BufferFull) {}
        }
        Ok(())
    }
}

#[cfg(feature = "embedded")]
impl embedded_hal::serial::Read<u8> for MmioUart8250<'_> {
    type Error = Infallible;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.read_byte().ok_or(nb::Error::WouldBlock)
    }
}

#[cfg(feature = "embedded")]
impl embedded_hal::serial::Write<u8> for MmioUart8250<'_> {
    type Error = Infallible;

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        self.write_byte(byte).map_err(|e| match e {
            TransmitError::BufferFull => nb::Error::WouldBlock,
        })
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        if self.is_data_holding_registers_empty() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests treat normal memory as device memory, which is not necessarily guaranteed to
    // work, but it seems to for now.

    #[test]
    fn initialise() {
        // Create a fake UART using an in-memory buffer, and check that it is initialised as
        // expected.
        let mut fake_registers: [u8; 8] = [0xff; 8];
        let uart = unsafe { MmioUart8250::new(&mut fake_registers as *mut u8 as usize) };

        uart.init(11_059_200, 115200);

        assert!(matches!(uart.get_parity(), Parity::No));
        assert_eq!(uart.get_stop_bit(), 1);
        assert_eq!(uart.get_word_length(), 8);
    }

    #[test]
    fn write() {
        let mut fake_registers: [u8; 8] = [0; 8];
        let uart = unsafe { MmioUart8250::new(&mut fake_registers as *mut u8 as usize) };

        // Pretend that the transmit buffer is full.
        fake_registers[5] = 0;
        assert_eq!(uart.write_byte(0x42), Err(TransmitError::BufferFull));
        assert_eq!(fake_registers[0], 0);

        // Pretend that the transmit buffer is available.
        fake_registers[5] = 0b0010_0000;
        assert_eq!(uart.write_byte(0x42), Ok(()));
        assert_eq!(fake_registers[0], 0x42);
    }

    #[test]
    fn read() {
        let mut fake_registers: [u8; 8] = [0; 8];
        let uart = unsafe { MmioUart8250::new(&mut fake_registers as *mut u8 as usize) };

        // First try to read when there is nothing available.
        assert_eq!(uart.read_byte(), None);

        // Set the UART up to have a byte available to read and read it.
        fake_registers[0] = 0xab;
        fake_registers[5] = 0b0000_0001;

        assert_eq!(uart.read_byte(), Some(0xab));
    }
}
