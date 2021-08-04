#[cfg(feature = "embedded")]
use core::convert::Infallible;
#[cfg(feature = "embedded")]
use embedded_hal::serial;
#[cfg(feature = "embedded")]
use nb;

#[cfg(feature = "fmt")]
use core::fmt;

use crate::registers::Registers;

#[derive(Debug, Clone, PartialEq)]
pub enum ChipFifoInfo {
    NoFifo,
    Reserved,
    EnabledNoFunction,
    Enabled,
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

/// # MMIO version of uart 8250
///
/// **Noticed** This is only tested on the NS16550 compatible UART used in QEMU 5.0 virt machine of RISC-V
pub struct MmioUart8250<'a> {
    reg: &'a mut Registers,
}

impl<'a> MmioUart8250<'a> {
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
        self.enable_divisor_latch_accessible();
        let divisor = clock / (16 * baud_rate);
        self.write_dll(divisor as u8);
        self.write_dlh((divisor >> 8) as u8);

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
            Some(self.read_rbr())
        } else {
            None
        }
    }

    /// Write a byte to uart
    ///
    /// Error are not concerned now **MAYBE TODO**
    pub fn write_byte(&self, byte: u8) {
        self.write_thr(byte);
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
    pub fn write_thr(&self, value: u8) {
        unsafe { self.reg.rw[0].write(value) }
    }

    /// read RBR (offset + 0)
    ///
    /// Read Receiver Buffer to get data
    pub fn read_rbr(&self) -> u8 {
        self.reg.rw[0].read()
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
    pub fn read_dll(&self) -> u8 {
        self.reg.rw[0].read()
    }

    /// write DLL (offset + 0)
    ///
    /// set divisor latch low byte in the register
    pub fn write_dll(&self, value: u8) {
        unsafe { self.reg.rw[0].write(value) }
    }

    /// read DLH (offset + 1)
    ///
    /// get divisor latch high byte in the register
    pub fn read_dlh(&self) -> u8 {
        self.reg.rw[1].read()
    }

    /// write DLH (offset + 1)
    ///
    /// set divisor latch high byte in the register
    pub fn write_dlh(&self, value: u8) {
        unsafe { self.reg.rw[1].write(value) }
    }

    /// Set divisor latch according to clock and baud_rate, then set DLAB to false
    pub fn set_divisor(&self, clock: usize, baud_rate: usize) {
        self.enable_divisor_latch_accessible();
        let divisor = clock / (16 * baud_rate);
        self.write_dll(divisor as u8);
        self.write_dlh((divisor >> 8) as u8);
        self.disable_divisor_latch_accessible();
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
    pub fn read_ier(&self) -> u8 {
        self.reg.rw[1].read()
    }

    /// Write IER (offset + 1)
    ///
    /// Write Interrupt Enable Register to turn on/off interrupts
    pub fn write_ier(&self, value: u8) {
        unsafe { self.reg.rw[1].write(value) }
    }

    /// get whether low power mode (16750) is enabled (IER\[5\])
    pub fn is_low_power_mode_enabled(&self) -> bool {
        self.reg.rw[1].read() & 0b0010_0000 != 0
    }

    /// toggle low power mode (16750) (IER\[5\])
    pub fn toggle_low_power_mode(&self) {
        unsafe { self.reg.rw[1].modify(|v| v ^ 0b0010_0000) }
    }

    /// enable low power mode (16750) (IER\[5\])
    pub fn enable_low_power_mode(&self) {
        unsafe { self.reg.rw[1].modify(|v| v | 0b0010_0000) }
    }

    /// disable low power mode (16750) (IER\[5\])
    pub fn disable_low_power_mode(&self) {
        unsafe { self.reg.rw[1].modify(|v| v & !0b0010_0000) }
    }

    /// get whether sleep mode (16750) is enabled (IER\[4\])
    pub fn is_sleep_mode_enabled(&self) -> bool {
        self.reg.rw[1].read() & 0b0001_0000 != 0
    }

    /// toggle sleep mode (16750) (IER\[4\])
    pub fn toggle_sleep_mode(&self) {
        unsafe { self.reg.rw[1].modify(|v| v ^ 0b0001_0000) }
    }

    /// enable sleep mode (16750) (IER\[4\])
    pub fn enable_sleep_mode(&self) {
        unsafe { self.reg.rw[1].modify(|v| v | 0b0001_0000) }
    }

    /// disable sleep mode (16750) (IER\[4\])
    pub fn disable_sleep_mode(&self) {
        unsafe { self.reg.rw[1].modify(|v| v & !0b0001_0000) }
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
    pub fn read_iir(&self) -> u8 {
        self.reg.rw[2].read()
    }

    /// Read IIR\[7:6\] to get FIFO status
    pub fn read_fifo_status(&self) -> ChipFifoInfo {
        match self.reg.rw[2].read() & 0b1100_0000 {
            0 => ChipFifoInfo::NoFifo,
            0b0100_0000 => ChipFifoInfo::Reserved,
            0b1000_0000 => ChipFifoInfo::EnabledNoFunction,
            0b1100_0000 => ChipFifoInfo::Enabled,
            _ => panic!("Can't reached"),
        }
    }

    /// get whether 64 Byte fifo (16750 only) is enabled (IIR\[5\])
    pub fn is_64byte_fifo_enabled(&self) -> bool {
        self.reg.rw[2].read() & 0b0010_0000 != 0
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
    pub fn write_fcr(&self, value: u8) {
        unsafe { self.reg.rw[2].write(value) }
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
    pub fn read_lcr(&self) -> u8 {
        self.reg.rw[3].read()
    }

    /// Write LCR (offset + 3)
    ///
    /// Write Line Control Register to set DLAB and the serial data protocol
    pub fn write_lcr(&self, value: u8) {
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
    pub fn enable_divisor_latch_accessible(&self) {
        unsafe { self.reg.rw[3].modify(|v| v | 0b1000_0000) }
    }

    /// disable DLAB
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
    pub fn get_stop_bit(&self) -> u8 {
        ((self.reg.rw[3].read() & 0b100) >> 2) + 1
    }

    /// set stop bit, only 1 and 2 can be used as `stop_bit`
    pub fn set_stop_bit(&self, stop_bit: u8) {
        match stop_bit {
            1 => unsafe { self.reg.rw[3].modify(|v| v & 0b1111_1011) },
            2 => unsafe { self.reg.rw[3].modify(|v| v | 0b0000_0100) },
            _ => panic!("Invalid stop bit"),
        }
    }

    /// get word length of used data protocol
    pub fn get_word_length(&self) -> u8 {
        (self.reg.rw[3].read() & 0b11) + 5
    }

    /// set word length, only 5..=8 can be used as `length`
    pub fn set_word_length(&self, length: u8) {
        if (5..=8).contains(&length) {
            unsafe { self.reg.rw[3].modify(|v| v | (length - 5)) }
        } else {
            panic!("Invalid word length")
        }
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
    pub fn read_mcr(&self) -> u8 {
        self.reg.rw[4].read()
    }

    /// Write MCR (offset + 4)
    ///
    /// Write Modem Control Register to control flow
    pub fn write_mcr(&self, value: u8) {
        unsafe { self.reg.rw[4].write(value) }
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
    pub fn read_lsr(&self) -> u8 {
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
    pub fn read_msr(&self) -> u8 {
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

    pub fn read_sr(&self) -> u8 {
        self.reg.scratch.read()
    }

    pub fn write_sr(&self, value: u8) {
        unsafe { self.reg.scratch.write(value) }
    }
}

/// ## embedded-hal::serial::Read
///
/// This is a very simple implementation, based on [rustsbi/rustsbi-qemu](https://github.com/rustsbi/rustsbi-qemu/blob/main/rustsbi-qemu/src/ns16550a.rs#L36-L52)
#[cfg(feature = "embedded")]
impl<'a> serial::Read<u8> for MmioUart8250<'a> {
    type Error = Infallible;

    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.is_interrupt_pending() {
            Ok(self.read_rbr())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// ## embedded-hal::serial::Write
///
/// This is a very simple implementation, based on [rustsbi/rustsbi-qemu](https://github.com/rustsbi/rustsbi-qemu/blob/main/rustsbi-qemu/src/ns16550a.rs#L54-L75)
#[cfg(feature = "embedded")]
impl<'a> serial::Write<u8> for MmioUart8250<'a> {
    type Error = Infallible;

    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.write_thr(word);
        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        if self.is_interrupt_pending() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

/// ## fmt::Write
///
/// A simple implementation, may be changed in the future
#[cfg(feature = "fmt")]
impl<'a> fmt::Write for MmioUart8250<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            self.write_thr(*c);
        }
        Ok(())
    }
}
