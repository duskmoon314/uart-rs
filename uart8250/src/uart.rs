use bitflags::bitflags;
#[cfg(feature = "embedded")]
use core::convert::Infallible;
use core::fmt::{self, Display, Formatter};
use tock_registers::{
    fields::FieldValue,
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, LocalRegisterCopy,
};

use crate::registers::Registers;

register_bitfields![
    u8,

    /// Interrupt Enable Register
    pub IER2 [
        /// Enable Received Data Available Interrupt
        RDAI OFFSET(0) NUMBITS(1) [],
        /// Enable Transmitter Holding Register Empty Interrupt
        THREI OFFSET(1) NUMBITS(1) [],
        /// Enable Receiver Line Status Interrupt
        RLSI OFFSET(2) NUMBITS(1) [],
        /// Enable Modem Status Interrupt
        MSI OFFSET(3) NUMBITS(1) [],
        /// Enable Sleep Mode (16750)
        SM OFFSET(4) NUMBITS(1) [],
        /// Enable Low Power Mode (16750)
        LPM OFFSET(5) NUMBITS(1) [],
    ],

    /// Line Control Register
    pub LCR [
        /// Divisor Latch Access Bit
        DLAB OFFSET(7) NUMBITS(1) [], // = 0b1000_0000;
        /// Set Break Enable
        SBE OFFSET(6) NUMBITS(1) [], // = 0b0100_0000;
        /// Parity
        Parity OFFSET(3) NUMBITS(3) [
            /// No parity
            None = 0,
            /// Odd parity
            Odd = 1,
            /// Even parity
            Even = 3,
            /// Mark
            Mark = 5,
            /// Space
            Space = 7,
        ],
        /// Number of stop bits
        STOP_BITS OFFSET(2) NUMBITS(1) [
            /// One stop bit
            One = 0,
            /// 1.5 or 2 stop bits
            Two = 1,
        ],
        /// Word length
        WORD_LENGTH OFFSET(0) NUMBITS(2) [
            /// 5 bit word length
            Bits5 = 0,
            /// 6 bit word length
            Bits6 = 1,
            /// 7 bit word length
            Bits7 = 2,
            /// 8 bit word length
            Bits8 = 3,
        ],
    ],
];

bitflags! {
    /// Interrupt Enable Register (bitflags)
    pub struct IER: u8 {
        /// Enable Received Data Available Interrupt
        const RDAI  = 0b0000_0001;
        /// Enable Transmitter Holding Register Empty Interrupt
        const THREI = 0b0000_0010;
        /// Enable Receiver Line Status Interrupt
        const RLSI  = 0b0000_0100;
        /// Enable Modem Status Interrupt
        const MSI   = 0b0000_1000;
        /// Enable Sleep Mode (16750)
        const SM    = 0b0001_0000;
        /// Enable Low Power Mode (16750)
        const LPM   = 0b0010_0000;
    }
}

bitflags! {
    /// Line Status Register (bitflags)
    pub struct LSR: u8 {
        /// Data Ready
        const DR = 0b0000_0001;
        /// Overrun Error
        const OE = 0b0000_0010;
        /// Parity Error
        const PE = 0b0000_0100;
        /// Framing Error
        const FE = 0b0000_1000;
        /// Break Interrupt
        const BI = 0b0001_0000;
        /// Transmitter Holding Register Empty
        const THRE = 0b0010_0000;
        /// Data Holding Regiters Empty
        const DHRE = 0b0100_0000;
        /// Error in Received FIFO
        const RFE = 0b1000_0000;
    }
}

bitflags! {
    /// Modem Status Register (bitflags)
    pub struct MSR: u8 {
        /// Delta Clear To Send
        const DCTS = 0b0000_0001;
        ///Delta Data Set Ready
        const DDSR = 0b0000_0010;
        ///Trailing Edge Ring Indicator
        const TERI = 0b0000_0100;
        ///Delta Data Carrier Detect
        const DDCD = 0b0000_1000;
        ///Clear To Send
        const CTS = 0b0001_0000;
        ///Data Set Ready
        const DSR = 0b0010_0000;
        ///Ring Indicator
        const RI = 0b0100_0000;
        ///Carrier Detect
        const CD = 0b1000_0000;
    }
}

bitflags! {
    /// FIFO Control Register (bitflags)
    pub struct FCR: u8 {
        /// Interrupt trigger level is 1 byte.
        const INTERRUPT_TRIGGER_LEVEL_1 = 0b0000_0000;
        /// Interrupt trigger level is 4 or 16 bytes, for 16 or 64 byte FIFO respectively.
        const INTERRUPT_TRIGGER_LEVEL_4_16 = 0b0100_0000;
        /// Interrupt trigger level is 8 or 32 bytes, for 16 or 64 byte FIFO respectively.
        const INTERRUPT_TRIGGER_LEVEL_8_32 = 0b1000_0000;
        /// Interrupt trigger level is 14 or 56 bytes, for 16 or 64 byte FIFO respectively.
        const INTERRUPT_TRIGGER_LEVEL_14_56 = 0b1100_0000;
        /// Enable 64 byte FIFO (16750)
        const ENABLE_64_BYTE = 0b0010_0000;
        /// DMA mode select
        const DMA_MODE = 0b0000_1000;
        /// Clear transmit FIFO
        const CLEAR_TX = 0b0000_0100;
        /// Clear receive FIFO
        const CLEAR_RX = 0b0000_0010;
        /// Enable FIFOs.
        const ENABLE = 0b0000_0001;
    }
}

bitflags! {
    /// Modem Control Register (bitflags)
    pub struct MCR: u8 {
        /// Autoflow control enabled (16750)
        const AUTOFLOW_CONTROL_ENABLED = 0b0010_0000;
        /// Loopback mode
        const LOOPBACK_MODE = 0b0001_0000;
        /// Auxiliary output 2
        const AUX_OUTPUT_2 = 0b0000_1000;
        /// Auxiliary output 1
        const AUX_OUTPUT_1 = 0b0000_0100;
        /// Request to Send
        const RTS = 0b0000_0010;
        /// Data Terminal Ready
        const DTR = 0b0000_0001;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChipFifoInfo {
    NoFifo,
    Reserved,
    EnabledNoFunction,
    Enabled,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InterruptType {
    ModemStatus,
    TransmitterHoldingRegisterEmpty,
    ReceivedDataAvailable,
    ReceiverLineStatus,
    Timeout,
    Reserved,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Parity {
    No,
    Odd,
    Even,
    Mark,
    Space,
}

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
        self.set_lcr(LCR::Parity::None + LCR::STOP_BITS::One + LCR::WORD_LENGTH::Bits8);
        // Enable FIFO
        self.set_fcr(FCR::ENABLE);
        // No modem control
        self.set_mcr(MCR::empty());
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
            Some(self.reg.read_rbr())
        } else {
            None
        }
    }

    /// Writes a byte to the UART.
    pub fn write_byte(&self, byte: u8) -> Result<(), TransmitError> {
        if self.is_transmitter_holding_register_empty() {
            self.reg.write_thr(byte);
            Ok(())
        } else {
            Err(TransmitError::BufferFull)
        }
    }

    /// Set divisor latch according to clock and baud_rate, then set DLAB to false
    #[inline]
    pub fn set_divisor(&self, clock: usize, baud_rate: usize) {
        self.enable_divisor_latch_accessible();
        let divisor = clock / (16 * baud_rate);
        self.reg.write_dll(divisor as u8);
        self.reg.write_dlh((divisor >> 8) as u8);
        self.disable_divisor_latch_accessible();
    }

    /// Get IER bitflags
    #[inline]
    fn ier(&self) -> IER {
        IER::from_bits_truncate(self.reg.read_ier())
    }

    /// Set IER via bitflags
    #[inline]
    fn set_ier(&self, flag: IER) {
        self.reg.write_ier(flag.bits())
    }

    /// get whether low power mode (16750) is enabled (IER\[5\])
    pub fn is_low_power_mode_enabled(&self) -> bool {
        self.ier().contains(IER::LPM)
    }

    /// enable low power mode (16750) (IER\[5\])
    pub fn enable_low_power_mode(&self) {
        self.set_ier(self.ier() | IER::LPM)
    }

    /// disable low power mode (16750) (IER\[5\])
    pub fn disable_low_power_mode(&self) {
        self.set_ier(self.ier() & !IER::LPM)
    }

    /// get whether sleep mode (16750) is enabled (IER\[4\])
    pub fn is_sleep_mode_enabled(&self) -> bool {
        self.ier().contains(IER::SM)
    }

    /// enable sleep mode (16750) (IER\[4\])
    pub fn enable_sleep_mode(&self) {
        self.set_ier(self.ier() | IER::SM)
    }

    /// disable sleep mode (16750) (IER\[4\])
    pub fn disable_sleep_mode(&self) {
        self.set_ier(self.ier() & !IER::SM)
    }

    /// get whether modem status interrupt is enabled (IER\[3\])
    pub fn is_modem_status_interrupt_enabled(&self) -> bool {
        self.ier().contains(IER::MSI)
    }

    /// enable modem status interrupt (IER\[3\])
    pub fn enable_modem_status_interrupt(&self) {
        self.set_ier(self.ier() | IER::MSI)
    }

    /// disable modem status interrupt (IER\[3\])
    pub fn disable_modem_status_interrupt(&self) {
        self.set_ier(self.ier() & !IER::MSI)
    }

    /// get whether receiver line status interrupt is enabled (IER\[2\])
    pub fn is_receiver_line_status_interrupt_enabled(&self) -> bool {
        self.ier().contains(IER::RLSI)
    }

    /// enable receiver line status interrupt (IER\[2\])
    pub fn enable_receiver_line_status_interrupt(&self) {
        self.set_ier(self.ier() | IER::RLSI)
    }

    /// disable receiver line status interrupt (IER\[2\])
    pub fn disable_receiver_line_status_interrupt(&self) {
        self.set_ier(self.ier() & !IER::RLSI)
    }

    /// get whether transmitter holding register empty interrupt is enabled (IER\[1\])
    pub fn is_transmitter_holding_register_empty_interrupt_enabled(&self) -> bool {
        self.ier().contains(IER::THREI)
    }

    /// enable transmitter holding register empty interrupt (IER\[1\])
    pub fn enable_transmitter_holding_register_empty_interrupt(&self) {
        self.set_ier(self.ier() | IER::THREI)
    }

    /// disable transmitter holding register empty interrupt (IER\[1\])
    pub fn disable_transmitter_holding_register_empty_interrupt(&self) {
        self.set_ier(self.ier() & !IER::THREI)
    }

    /// get whether received data available is enabled (IER\[0\])
    pub fn is_received_data_available_interrupt_enabled(&self) -> bool {
        self.ier().contains(IER::RDAI)
    }

    /// enable received data available (IER\[0\])
    pub fn enable_received_data_available_interrupt(&self) {
        self.set_ier(self.ier() | IER::RDAI)
    }

    /// disable received data available (IER\[0\])
    pub fn disable_received_data_available_interrupt(&self) {
        self.set_ier(self.ier() & !IER::RDAI)
    }

    /// Read IIR\[7:6\] to get FIFO status
    pub fn read_fifo_status(&self) -> ChipFifoInfo {
        match self.reg.read_iir() & 0b1100_0000 {
            0 => ChipFifoInfo::NoFifo,
            0b0100_0000 => ChipFifoInfo::Reserved,
            0b1000_0000 => ChipFifoInfo::EnabledNoFunction,
            0b1100_0000 => ChipFifoInfo::Enabled,
            _ => panic!("Can't reached"),
        }
    }

    /// get whether 64 Byte fifo (16750 only) is enabled (IIR\[5\])
    pub fn is_64byte_fifo_enabled(&self) -> bool {
        self.reg.read_iir() & 0b0010_0000 != 0
    }

    /// Read IIR\[3:1\] to get interrupt type
    pub fn read_interrupt_type(&self) -> Option<InterruptType> {
        let iir = self.reg.read_iir() & 0b0000_1111;
        if iir & 1 != 0 {
            None
        } else {
            match iir {
                0b0000 => Some(InterruptType::ModemStatus),
                0b0010 => Some(InterruptType::TransmitterHoldingRegisterEmpty),
                0b0100 => Some(InterruptType::ReceivedDataAvailable),
                0b0110 => Some(InterruptType::ReceiverLineStatus),
                0b1100 => Some(InterruptType::Timeout),
                0b1000 | 0b1010 | 0b1110 => Some(InterruptType::Reserved),
                _ => panic!("Can't reached"),
            }
        }
    }

    /// enable DLAB
    fn enable_divisor_latch_accessible(&self) {
        self.reg.lcr.modify(LCR::DLAB::SET)
    }

    /// disable DLAB
    fn disable_divisor_latch_accessible(&self) {
        self.reg.lcr.modify(LCR::DLAB::CLEAR)
    }

    /// get parity of used data protocol
    pub fn get_parity(&self) -> Parity {
        match self
            .reg
            .lcr
            .read_as_enum(LCR::Parity)
            .expect("Invalid Parity! Please check your UART.")
        {
            LCR::Parity::Value::None => Parity::No,
            LCR::Parity::Value::Odd => Parity::Odd,
            LCR::Parity::Value::Even => Parity::Even,
            LCR::Parity::Value::Mark => Parity::Mark,
            LCR::Parity::Value::Space => Parity::Space,
        }
    }

    /// set parity
    pub fn set_parity(&self, parity: Parity) {
        let parity = match parity {
            Parity::No => LCR::Parity::None,
            Parity::Odd => LCR::Parity::Odd,
            Parity::Even => LCR::Parity::Even,
            Parity::Mark => LCR::Parity::Mark,
            Parity::Space => LCR::Parity::Space,
        };
        self.reg.lcr.modify(parity);
    }

    /// get stop bit of used data protocol
    ///
    /// Simply return a u8 to indicate 1 or 1.5/2 bits
    pub fn get_stop_bit(&self) -> u8 {
        ((self.reg.read_lcr() & 0b100) >> 2) + 1
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

    /// Sets FCR bitflags
    #[inline]
    pub fn set_fcr(&self, fcr: FCR) {
        self.reg.write_fcr(fcr.bits())
    }

    /// Gets LCR bitflags
    #[inline]
    pub fn lcr(&self) -> LocalRegisterCopy<u8, LCR::Register> {
        self.reg.lcr.extract()
    }

    /// Sets LCR bitflags
    #[inline]
    pub fn set_lcr(&self, lcr: FieldValue<u8, LCR::Register>) {
        self.reg.lcr.write(lcr)
    }

    /// Gets MCR bitflags
    #[inline]
    pub fn mcr(&self) -> MCR {
        MCR::from_bits_truncate(self.reg.read_mcr())
    }

    /// Sets MCR bitflags
    #[inline]
    pub fn set_mcr(&self, mcr: MCR) {
        self.reg.write_mcr(mcr.bits())
    }

    /// Get LSR bitflags
    #[inline]
    fn lsr(&self) -> LSR {
        LSR::from_bits_truncate(self.reg.read_lsr())
    }

    /// get whether there is an error in received FIFO
    pub fn is_received_fifo_error(&self) -> bool {
        self.lsr().contains(LSR::RFE)
    }

    /// Gets whether data holding registers are empty, i.e. the UART has finished transmitting all
    /// the data it has been given.
    pub fn is_data_holding_registers_empty(&self) -> bool {
        self.lsr().contains(LSR::DHRE)
    }

    /// Gets whether transmitter holding register is empty, i.e. the UART is ready to be given more
    /// data to transmit.
    pub fn is_transmitter_holding_register_empty(&self) -> bool {
        self.lsr().contains(LSR::THRE)
    }

    pub fn is_break_interrupt(&self) -> bool {
        self.lsr().contains(LSR::BI)
    }

    pub fn is_framing_error(&self) -> bool {
        self.lsr().contains(LSR::FE)
    }

    pub fn is_parity_error(&self) -> bool {
        self.lsr().contains(LSR::PE)
    }

    pub fn is_overrun_error(&self) -> bool {
        self.lsr().contains(LSR::OE)
    }

    pub fn is_data_ready(&self) -> bool {
        self.lsr().contains(LSR::DR)
    }

    /// Get MSR bitflags
    #[inline]
    fn msr(&self) -> MSR {
        MSR::from_bits_truncate(self.reg.read_msr())
    }

    pub fn is_carrier_detect(&self) -> bool {
        self.msr().contains(MSR::CD)
    }

    pub fn is_ring_indicator(&self) -> bool {
        self.msr().contains(MSR::RI)
    }

    pub fn is_data_set_ready(&self) -> bool {
        self.msr().contains(MSR::DSR)
    }

    pub fn is_clear_to_send(&self) -> bool {
        self.msr().contains(MSR::CTS)
    }

    pub fn is_delta_data_carrier_detect(&self) -> bool {
        self.msr().contains(MSR::DDCD)
    }

    pub fn is_trailing_edge_ring_indicator(&self) -> bool {
        self.msr().contains(MSR::TERI)
    }

    pub fn is_delta_data_set_ready(&self) -> bool {
        self.msr().contains(MSR::DDSR)
    }

    pub fn is_delta_clear_to_send(&self) -> bool {
        self.msr().contains(MSR::DCTS)
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
