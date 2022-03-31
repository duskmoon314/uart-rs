use core::u8;

use volatile_register::{RO, RW};

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
#[repr(C, packed)]
pub struct Registers {
    thr_rbr_dll: RW<u8>,
    ier_dlh: RW<u8>,
    iir_fcr: RW<u8>,
    pub lcr: RW<u8>,
    mcr: RW<u8>,
    lsr: RO<u8>,
    msr: RO<u8>,
    scratch: RW<u8>,
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
        unsafe { self.thr_rbr_dll.write(value) }
    }

    /// read RBR (offset + 0)
    ///
    /// Read Receiver Buffer to get data
    #[inline]
    pub fn read_rbr(&self) -> u8 {
        self.thr_rbr_dll.read()
    }

    /// read DLL (offset + 0)
    ///
    /// get divisor latch low byte in the register
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
    pub fn read_dll(&self) -> u8 {
        self.thr_rbr_dll.read()
    }

    /// write DLL (offset + 0)
    ///
    /// set divisor latch low byte in the register
    #[inline]
    pub fn write_dll(&self, value: u8) {
        unsafe { self.thr_rbr_dll.write(value) }
    }

    /// read DLH (offset + 1)
    ///
    /// get divisor latch high byte in the register
    #[inline]
    pub fn read_dlh(&self) -> u8 {
        self.ier_dlh.read()
    }

    /// write DLH (offset + 1)
    ///
    /// set divisor latch high byte in the register
    #[inline]
    pub fn write_dlh(&self, value: u8) {
        unsafe { self.ier_dlh.write(value) }
    }

    /// Read IER (offset + 1)
    ///
    /// Read IER to get what interrupts are enabled
    ///
    /// > ## Interrupt Enable Register
    /// >
    /// > Offset: +1 . This register allows you to control when and how the UART is going to trigger an interrupt event with the hardware interrupt associated with the serial COM port. If used properly, this can enable an efficient use of system resources and allow you to react to information being sent across a serial data line in essentially real-time conditions. Some more on that will be covered later, but the point here is that you can use the UART to let you know exactly when you need to extract some data. This register has both read- and write-access.
    /// >
    /// > The following is a table showing each bit in this register and what events that it will enable to allow you check on the status of this chip:
    /// >
    /// > | Bit | Notes                                               |
    /// > | --- | --------------------------------------------------- |
    /// > | 7   | Reserved                                            |
    /// > | 6   | Reserved                                            |
    /// > | 5   | Enables Low Power Mode (16750)                      |
    /// > | 4   | Enables Sleep Mode (16750)                          |
    /// > | 3   | Enable Modem Status Interrupt                       |
    /// > | 2   | Enable Receiver Line Status Interrupt               |
    /// > | 1   | Enable Transmitter Holding Register Empty Interrupt |
    /// > | 0   | Enable Received Data Available Interrupt            |
    #[inline]
    pub fn read_ier(&self) -> u8 {
        self.ier_dlh.read()
    }

    /// Write IER (offset + 1)
    ///
    /// Write Interrupt Enable Register to turn on/off interrupts
    #[inline]
    pub fn write_ier(&self, value: u8) {
        unsafe { self.ier_dlh.write(value) }
    }

    /// Read IIR (offset + 2)
    ///
    /// > ## Interrupt Identification Register
    /// >
    /// > Offset: +2 . This register is to be used to help identify what the unique characteristics of the UART chip that you are using has. This chip has two uses:
    /// >
    /// > - Identification of why the UART triggered an interrupt.
    /// > - Identification of the UART chip itself.
    /// >
    /// > Of these, identification of why the interrupt service routine has been invoked is perhaps the most important.
    /// >
    /// > The following table explains some of the details of this register, and what each bit on it represents:
    /// >
    /// > | Bit        | Notes                             |       |                                   |                                              |                                                                                           |
    /// > | ---------- | --------------------------------- | ----- | --------------------------------- | -------------------------------------------- | ----------------------------------------------------------------------------------------- |
    /// > | 7 and 6    | Bit 7                             | Bit 6 |                                   |                                              |                                                                                           |
    /// > |            | 0                                 | 0     | No FIFO on chip                   |                                              |                                                                                           |
    /// > |            | 0                                 | 1     | Reserved condition                |                                              |                                                                                           |
    /// > |            | 1                                 | 0     | FIFO enabled, but not functioning |                                              |                                                                                           |
    /// > |            | 1                                 | 1     | FIFO enabled                      |                                              |                                                                                           |
    /// > | 5          | 64 Byte FIFO Enabled (16750 only) |       |                                   |                                              |                                                                                           |
    /// > | 4          | Reserved                          |       |                                   |                                              |                                                                                           |
    /// > | 3, 2 and 1 | Bit 3                             | Bit 2 | Bit 1                             |                                              | Reset Method                                                                              |
    /// > |            | 0                                 | 0     | 0                                 | Modem Status Interrupt                       | Reading Modem Status Register(MSR)                                                        |
    /// > |            | 0                                 | 0     | 1                                 | Transmitter Holding Register Empty Interrupt | Reading Interrupt Identification Register(IIR) or Writing to Transmit Holding Buffer(THR) |
    /// > |            | 0                                 | 1     | 0                                 | Received Data Available Interrupt            | Reading Receive Buffer Register(RBR)                                                      |
    /// > |            | 0                                 | 1     | 1                                 | Receiver Line Status Interrupt               | Reading Line Status Register(LSR)                                                         |
    /// > |            | 1                                 | 0     | 0                                 | Reserved                                     | N/A                                                                                       |
    /// > |            | 1                                 | 0     | 1                                 | Reserved                                     | N/A                                                                                       |
    /// > |            | 1                                 | 1     | 0                                 | Time-out Interrupt Pending (16550 & later)   | Reading Receive Buffer Register(RBR)                                                      |
    /// > |            | 1                                 | 1     | 1                                 | Reserved                                     | N/A                                                                                       |
    /// > | 0          | Interrupt Pending Flag            |       |                                   |                                              |                                                                                           |
    #[inline]
    pub fn read_iir(&self) -> u8 {
        self.iir_fcr.read()
    }

    /// Write FCR (offset + 2) to control FIFO buffers
    ///
    /// > ## FIFO Control Register
    /// >
    /// > Offset: +2 . This is a relatively "new" register that was not a part of the original 8250 UART implementation. The purpose of this register is to control how the First In/First Out (FIFO) buffers will behave on the chip and to help you fine-tune their performance in your application. This even gives you the ability to "turn on" or "turn off" the FIFO.
    /// >
    /// > Keep in mind that this is a "write only" register. Attempting to read in the contents will only give you the Interrupt Identification Register (IIR), which has a totally different context.
    /// >
    /// > | Bit   | Notes                       |       |                                   |                         |
    /// > | ----- | --------------------------- | ----- | --------------------------------- | ----------------------- |
    /// > | 7 & 6 | Bit 7                       | Bit 6 | Interrupt Trigger Level (16 byte) | Trigger Level (64 byte) |
    /// > |       | 0                           | 0     | 1 Byte                            | 1 Byte                  |
    /// > |       | 0                           | 1     | 4 Bytes                           | 16 Bytes                |
    /// > |       | 1                           | 0     | 8 Bytes                           | 32 Bytes                |
    /// > |       | 1                           | 1     | 14 Bytes                          | 56 Bytes                |
    /// > | 5     | Enable 64 Byte FIFO (16750) |       |                                   |                         |
    /// > | 4     | Reserved                    |       |                                   |                         |
    /// > | 3     | DMA Mode Select             |       |                                   |                         |
    /// > | 2     | Clear Transmit FIFO         |       |                                   |                         |
    /// > | 1     | Clear Receive FIFO          |       |                                   |                         |
    /// > | 0     | Enable FIFOs                |       |                                   |                         |
    #[inline]
    pub fn write_fcr(&self, value: u8) {
        unsafe { self.iir_fcr.write(value) }
    }

    /// Read LCR (offset + 3)
    ///
    /// Read Line Control Register to get the data protocol and DLAB
    ///
    /// > ## Line Control Register
    /// >
    /// > Offset: +3 . This register has two major purposes:
    /// >
    /// > - Setting the Divisor Latch Access Bit (DLAB), allowing you to set the values of the Divisor Latch Bytes.
    /// > - Setting the bit patterns that will be used for both receiving and transmitting the serial data. In other words, the serial data protocol you will be using (8-1-None, 5-2-Even, etc.).
    /// >
    /// > | Bit      | Notes                    |                              |             |               |
    /// > | -------- | ------------------------ | ---------------------------- | ----------- | ------------- |
    /// > | 7        | Divisor Latch Access Bit |                              |             |               |
    /// > | 6        | Set Break Enable         |                              |             |               |
    /// > | 3, 4 & 5 | Bit 5                    | Bit 4                        | Bit 3       | Parity Select |
    /// > |          | 0                        | 0                            | 0           | No Parity     |
    /// > |          | 0                        | 0                            | 1           | Odd Parity    |
    /// > |          | 0                        | 1                            | 1           | Even Parity   |
    /// > |          | 1                        | 0                            | 1           | Mark          |
    /// > |          | 1                        | 1                            | 1           | Space         |
    /// > | 2        | 0                        | One Stop Bit                 |             |               |
    /// > |          | 1                        | 1.5 Stop Bits or 2 Stop Bits |             |               |
    /// > | 0 & 1    | Bit 1                    | Bit 0                        | Word Length |               |
    /// > |          | 0                        | 0                            | 5 Bits      |               |
    /// > |          | 0                        | 1                            | 6 Bits      |               |
    /// > |          | 1                        | 0                            | 7 Bits      |               |
    /// > |          | 1                        | 1                            | 8 Bits      |               |
    #[inline]
    pub fn read_lcr(&self) -> u8 {
        self.lcr.read()
    }

    /// Write LCR (offset + 3)
    ///
    /// Write Line Control Register to set DLAB and the serial data protocol
    #[inline]
    pub fn write_lcr(&self, value: u8) {
        unsafe { self.lcr.write(value) }
    }

    /// Read MCR (offset + 4)
    ///
    /// Read Modem Control Register to get how flow is controlled
    ///
    /// > ## Modem Control Register
    /// >
    /// > Offset: +4 . This register allows you to do "hardware" flow control, under software control. Or in a more practical manner, it allows direct manipulation of four different wires on the UART that you can set to any series of independent logical states, and be able to offer control of the modem. It should also be noted that most UARTs need Auxiliary Output 2 set to a logical "1" to enable interrupts.
    /// >
    /// > | Bit | Notes                            |
    /// > | --- | -------------------------------- |
    /// > | 7   | Reserved                         |
    /// > | 6   | Reserved                         |
    /// > | 5   | Autoflow Control Enabled (16750) |
    /// > | 4   | Loopback Mode                    |
    /// > | 3   | Auxiliary Output 2               |
    /// > | 2   | Auxiliary Output 1               |
    /// > | 1   | Request To Send                  |
    /// > | 0   | Data Terminal Ready              |
    #[inline]
    pub fn read_mcr(&self) -> u8 {
        self.mcr.read()
    }

    /// Write MCR (offset + 4)
    ///
    /// Write Modem Control Register to control flow
    #[inline]
    pub fn write_mcr(&self, value: u8) {
        unsafe { self.mcr.write(value) }
    }

    /// Read LSR (offset + 5)
    ///
    /// > ## Line Status Register
    /// >
    /// > Offset: +5 . This register is used primarily to give you information on possible error conditions that may exist within the UART, based on the data that has been received. Keep in mind that this is a "read only" register, and any data written to this register is likely to be ignored or worse, cause different behavior in the UART. There are several uses for this information, and some information will be given below on how it can be useful for diagnosing problems with your serial data connection:
    /// >
    /// > | Bit | Notes                              |
    /// > | --- | ---------------------------------- |
    /// > | 7   | Error in Received FIFO             |
    /// > | 6   | Empty Data Holding Registers       |
    /// > | 5   | Empty Transmitter Holding Register |
    /// > | 4   | Break Interrupt                    |
    /// > | 3   | Framing Error                      |
    /// > | 2   | Parity Error                       |
    /// > | 1   | Overrun Error                      |
    /// > | 0   | Data Ready                         |
    #[inline]
    pub fn read_lsr(&self) -> u8 {
        self.lsr.read()
    }

    /// Read MSR (offset + 6)
    ///
    /// > ## Modem Status Register
    /// >
    /// > Offset: +6 . This register is another read-only register that is here to inform your software about the current status of the modem. The modem accessed in this manner can either be an external modem, or an internal modem that uses a UART as an interface to the computer.
    /// >
    /// > | Bit | Notes                        |
    /// > | --- | ---------------------------- |
    /// > | 7   | Carrier Detect               |
    /// > | 6   | Ring Indicator               |
    /// > | 5   | Data Set Ready               |
    /// > | 4   | Clear To Send                |
    /// > | 3   | Delta Data Carrier Detect    |
    /// > | 2   | Trailing Edge Ring Indicator |
    /// > | 1   | Delta Data Set Ready         |
    /// > | 0   | Delta Clear To Send          |
    #[inline]
    pub fn read_msr(&self) -> u8 {
        self.msr.read()
    }

    #[inline]
    pub fn read_sr(&self) -> u8 {
        self.scratch.read()
    }

    #[inline]
    pub fn write_sr(&self, value: u8) {
        unsafe { self.scratch.write(value) }
    }
}
