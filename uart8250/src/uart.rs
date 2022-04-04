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
    pub IER [
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
        DLAB OFFSET(7) NUMBITS(1) [],
        /// Set Break Enable
        SBE OFFSET(6) NUMBITS(1) [],
        /// Parity
        Parity OFFSET(3) NUMBITS(3) [
            /// No parity
            No = 0,
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

    /// Line Status Register
    pub LSR [
        /// Data Ready
        DR OFFSET(0) NUMBITS(1) [],
        /// Overrun Error
        OE OFFSET(1) NUMBITS(1) [],
        /// Parity Error
        PE OFFSET(2) NUMBITS(1) [],
        /// Framing Error
        FE OFFSET(3) NUMBITS(1) [],
        /// Break Interrupt
        BI OFFSET(4) NUMBITS(1) [],
        /// Transmitter Holding Register Empty
        THRE OFFSET(5) NUMBITS(1) [],
        /// Data Holding Regiters Empty
        DHRE OFFSET(6) NUMBITS(1) [],
        /// Error in Received FIFO
        RFE OFFSET(7) NUMBITS(1) [],
    ],

    /// Modem Status Register
    pub MSR [
        /// Delta Clear To Send
        DCTS OFFSET(0) NUMBITS(1) [],
        /// Delta Data Set Ready
        DDSR OFFSET(1) NUMBITS(1) [],
        /// Trailing Edge Ring Indicator
        TERI OFFSET(2) NUMBITS(1) [],
        /// Delta Data Carrier Detect
        DDCD OFFSET(3) NUMBITS(1) [],
        /// Clear To Send
        CTS OFFSET(4) NUMBITS(1) [],
        /// Data Set Ready
        DSR OFFSET(5) NUMBITS(1) [],
        /// Ring Indicator
        RI OFFSET(6) NUMBITS(1) [],
        /// Carrier Detect
        CD OFFSET(7) NUMBITS(1) [],
    ],

    /// FIFO Control Register
    pub FCR [
        /// Interrupt trigger level.
        InterruptTriggerLevel OFFSET(6) NUMBITS(2) [
            /// Interrupt trigger level is 1 byte.
            Bytes1 = 0,
            /// Interrupt trigger level is 4 or 16 bytes, for 16 or 64 byte FIFO respectively.
            Bytes4Or16 = 1,
            /// Interrupt trigger level is 8 or 32 bytes, for 16 or 64 byte FIFO respectively.
            Bytes8Or32 = 2,
            /// Interrupt trigger level is 14 or 56 bytes, for 16 or 64 byte FIFO respectively.
            Bytes14Or56 = 3,
        ],
        /// Enable 64 byte FIFO (16750)
        Enable64Byte OFFSET(5) NUMBITS(1) [],
        /// DMA mode select
        DmaMode OFFSET(3) NUMBITS(1) [],
        /// Clear transmit FIFO
        ClearTx OFFSET(2) NUMBITS(1) [],
        /// Clear receive FIFO
        ClearRx OFFSET(1) NUMBITS(1) [],
        /// Enable FIFOs.
        Enable OFFSET(0) NUMBITS(1) [],
    ],

    /// Interrupt Identification Register
    pub IIR [
        FifoInfo OFFSET(6) NUMBITS(2) [
            /// No FIFO on chip.
            None = 0,
            /// FIFO enabled but not functioning.
            EnabledNotFunctioning = 2,
            /// FIFO enabled.
            Enabled = 3,
        ],
        /// 64 byte FIFO enabled (16750 only).
        Fifo64Byte OFFSET(5) NUMBITS(1) [],
        InterruptType OFFSET(1) NUMBITS(3) [
            ModemStatus = 0,
            TransmitterHoldingRegisterEmpty = 1,
            ReceivedDataAvailable = 2,
            ReceiverLineStatus = 3,
            Timeout = 6,
        ],
        /// Interrupt pending flag.
        InterruptPending OFFSET(0) NUMBITS(1) [],
    ],

    /// Modem Control Register (bitflags)
    pub MCR [
        /// Autoflow control enabled (16750)
        AUTOFLOW_CONTROL_ENABLED OFFSET(5) NUMBITS(1) [],
        /// Loopback mode
        LOOPBACK_MODE OFFSET(4) NUMBITS(1) [],
        /// Auxiliary output 2
        AUX_OUTPUT_2 OFFSET(3) NUMBITS(1) [],
        /// Auxiliary output 1
        AUX_OUTPUT_1 OFFSET(2) NUMBITS(1) [],
        /// Request to Send
        RTS OFFSET(1) NUMBITS(1) [],
        /// Data Terminal Ready
        DTR OFFSET(0) NUMBITS(1) [],
    ],
];

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
        self.set_lcr(LCR::Parity::No + LCR::STOP_BITS::One + LCR::WORD_LENGTH::Bits8);
        // Enable FIFO
        self.set_fcr(FCR::Enable::SET);
        // No modem control
        self.set_mcr(FieldValue::<u8, _>::new(0, 0, 0));
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
        match self.reg.iir_fcr.read_as_enum(IIR::FifoInfo) {
            Some(IIR::FifoInfo::Value::None) => ChipFifoInfo::NoFifo,
            Some(IIR::FifoInfo::Value::EnabledNotFunctioning) => ChipFifoInfo::EnabledNoFunction,
            Some(IIR::FifoInfo::Value::Enabled) => ChipFifoInfo::Enabled,
            None => ChipFifoInfo::Reserved,
        }
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
            match iir.read_as_enum(IIR::InterruptType) {
                Some(IIR::InterruptType::Value::ModemStatus) => Some(InterruptType::ModemStatus),
                Some(IIR::InterruptType::Value::TransmitterHoldingRegisterEmpty) => {
                    Some(InterruptType::TransmitterHoldingRegisterEmpty)
                }
                Some(IIR::InterruptType::Value::ReceivedDataAvailable) => {
                    Some(InterruptType::ReceivedDataAvailable)
                }
                Some(IIR::InterruptType::Value::ReceiverLineStatus) => {
                    Some(InterruptType::ReceiverLineStatus)
                }
                Some(IIR::InterruptType::Value::Timeout) => Some(InterruptType::Timeout),
                None => Some(InterruptType::Reserved),
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
            LCR::Parity::Value::No => Parity::No,
            LCR::Parity::Value::Odd => Parity::Odd,
            LCR::Parity::Value::Even => Parity::Even,
            LCR::Parity::Value::Mark => Parity::Mark,
            LCR::Parity::Value::Space => Parity::Space,
        }
    }

    /// set parity
    pub fn set_parity(&self, parity: Parity) {
        let parity = match parity {
            Parity::No => LCR::Parity::No,
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

    /// Sets FCR bitflags
    #[inline]
    pub fn set_fcr(&self, fcr: FieldValue<u8, FCR::Register>) {
        self.reg.iir_fcr.write(fcr)
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
    pub fn mcr(&self) -> LocalRegisterCopy<u8, MCR::Register> {
        self.reg.mcr.extract()
    }

    /// Sets MCR bitflags
    #[inline]
    pub fn set_mcr(&self, mcr: FieldValue<u8, MCR::Register>) {
        self.reg.mcr.write(mcr)
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
