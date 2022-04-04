use crate::uart::{FCR, IER, IIR, LCR, LSR, MCR, MSR};
use core::u8;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_structs,
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
        (0x00 => thr_rbr_dll: ReadWrite<u8>),
        (0x01 => pub ier_dlh: ReadWrite<u8, IER::Register>),
        (0x02 => pub iir_fcr: Aliased<u8, IIR::Register, FCR::Register>),
        (0x03 => pub lcr: ReadWrite<u8, LCR::Register>),
        (0x04 => pub mcr: ReadWrite<u8, MCR::Register>),
        (0x05 => pub lsr: ReadOnly<u8, LSR::Register>),
        (0x06 => pub msr: ReadOnly<u8, MSR::Register>),
        (0x07 => scratch: ReadWrite<u8>),
        (0x08 => @END),
    }
}

impl Registers {
    /// Constructs a new instance of the UART registers starting at the given base address.
    pub unsafe fn from_base_address(base_address: usize) -> &'static mut Self {
        &mut *(base_address as *mut crate::registers::Registers)
    }

    /// write THR (offset + 0)
    ///
    /// Write Transmitter Holding Buffer to send data
    ///
    /// > ## Transmitter Holding Buffer/Receiver Buffer
    /// >
    /// > Offset: +0 . The Transmit and Receive buffers are related, and often even use the very same memory. This is also one of the areas where later versions of the 8250 chip have a significant impact, as the later models incorporate some internal buffering of the data within the chip before it gets transmitted as serial data. The base 8250 chip can only receive one byte at a time, while later chips like the 16550 chip will hold up to 16 bytes either to transmit or to receive (sometimes both... depending on the manufacturer) before you have to wait for the character to be sent. This can be useful in multi-tasking environments where you have a computer doing many things, and it may be a couple of milliseconds before you get back to dealing with serial data flow.
    /// >
    /// > These registers really are the "heart" of serial data communication, and how data is transferred from your software to another computer and how it gets data from other devices. Reading and Writing to these registers is simply a matter of accessing the Port I/O address for the respective UART.
    /// >
    /// > If the receive buffer is occupied or the FIFO is full, the incoming data is discarded and the Receiver Line Status interrupt is written to the IIR register. The Overrun Error bit is also set in the Line Status Register.
    #[inline]
    pub fn write_thr(&self, value: u8) {
        self.thr_rbr_dll.set(value)
    }

    /// read RBR (offset + 0)
    ///
    /// Read Receiver Buffer to get data
    #[inline]
    pub fn read_rbr(&self) -> u8 {
        self.thr_rbr_dll.get()
    }

    /// write DLL (offset + 0)
    ///
    /// set divisor latch low byte in the register
    #[inline]
    pub fn write_dll(&self, value: u8) {
        self.thr_rbr_dll.set(value)
    }

    /// write DLH (offset + 1)
    ///
    /// set divisor latch high byte in the register
    #[inline]
    pub fn write_dlh(&self, value: u8) {
        self.ier_dlh.set(value)
    }
}
