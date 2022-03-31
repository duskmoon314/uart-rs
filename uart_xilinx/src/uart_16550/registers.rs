use volatile_register::{RO, RW};

macro_rules! cast {
    ($expr:expr) => {
        unsafe { &mut *(($expr) as *mut super::registers::Registers) }
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
/// | +0x0         | 0    | Write      | THR    | Transmitter Holding Buffer        |
/// | +0x0         | 0    | Read       | RBR    | Receiver Buffer                   |
/// | +0x0         | 1    | Read/Write | DLL    | Divisor Latch Low Byte            |
/// | +0x4         | 0    | Read/Write | IER    | Interrupt Enable Register         |
/// | +0x4         | 1    | Read/Write | DLH    | Divisor Latch High Byte           |
/// | +0x8         | x    | Read       | IIR    | Interrupt Identification Register |
/// | +0x8         | x    | Write      | FCR    | FIFO Control Register             |
/// | +0x8         | 1    | Read       | FCR    | FIFO Control Register             |
/// | +0xc         | x    | Read/Write | LCR    | Line Control Register             |
/// | +0x10        | x    | Read/Write | MCR    | Modem Control Register            |
/// | +0x14        | x    | Read       | LSR    | Line Status Register              |
/// | +0x18        | x    | Read       | MSR    | Modem Status Register             |
/// | +0x1c        | x    | Read/Write | SR     | Scratch Register                  |
#[repr(C)]
pub struct Registers {
    pub rw: [RW<u32>; 5],
    pub ro: [RO<u32>; 2],
    pub scratch: RW<u32>,
}
