use core::u8;
use tock_registers::{
    register_bitfields, register_structs,
    registers::{Aliased, ReadOnly, ReadWrite},
};

register_structs! {
    /// # UART Registers
    ///
    /// The chip has a total of 12 different registers that are mapped into 8 different Port I/O locations / Memory Mapped I/O addresses.
    ///
    /// The following is a table of each of the registers that can be found in a typical UART chip
    ///
    /// | Base Address | DLAB | I/O Access | Abbrv. | Register Name                     |
    /// | ------------ | ---- | ---------- | ------ | --------------------------------- |
    /// | +0           | 0    | Write      | THR    | Transmitter Holding Buffer        |
    /// | +0           | 0    | Read       | RBR    | Receiver Buffer                   |
    /// | +0           | 1    | Read/Write | DLL    | Divisor Latch Low Byte            |
    /// | +1           | 0    | Read/Write | IER    | Interrupt Enable Register         |
    /// | +1           | 1    | Read/Write | DLH    | Divisor Latch High Byte           |
    /// | +2           | x    | Read       | IIR    | Interrupt Identification Register |
    /// | +2           | x    | Write      | FCR    | FIFO Control Register             |
    /// | +3           | x    | Read/Write | LCR    | Line Control Register             |
    /// | +4           | x    | Read/Write | MCR    | Modem Control Register            |
    /// | +5           | x    | Read       | LSR    | Line Status Register              |
    /// | +6           | x    | Read       | MSR    | Modem Status Register             |
    /// | +7           | x    | Read/Write | SR     | Scratch Register                  |
    pub Registers {
        (0x00 => pub thr_rbr_dll: ReadWrite<u8>),
        (0x01 => pub ier_dlh: ReadWrite<u8, IER::Register>),
        (0x02 => pub iir_fcr: Aliased<u8, IIR::Register, FCR::Register>),
        (0x03 => pub lcr: ReadWrite<u8, LCR::Register>),
        (0x04 => pub mcr: ReadWrite<u8, MCR::Register>),
        (0x05 => pub lsr: ReadOnly<u8, LSR::Register>),
        (0x06 => pub msr: ReadOnly<u8, MSR::Register>),
        (0x07 => pub scratch: ReadWrite<u8>),
        (0x08 => @END),
    }
}

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
        /// Data Holding Registers Empty
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
            /// Reserved value.
            Reserved = 1,
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
            Reserved = 4,
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

impl Registers {
    /// Constructs a new instance of the UART registers starting at the given base address.
    pub unsafe fn from_base_address(base_address: usize) -> &'static mut Self {
        &mut *(base_address as *mut crate::registers::Registers)
    }
}
