use core::u8;

use volatile_register::{RO, RW};

#[macro_export]
macro_rules! cast {
    ($expr:expr) => {
        unsafe { &mut *(($expr) as *mut crate::registers::Registers) }
    };
}

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
#[repr(C)]
pub struct Registers {
    pub rw: [RW<u8>; 5],
    pub ro: [RO<u8>; 2],
    pub scratch: RW<u8>,
}
