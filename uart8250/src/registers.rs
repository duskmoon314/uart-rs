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
#[repr(C)]
pub struct Registers<const W: usize> {
    pub thr_rbr_dll: RwReg<W, u8>,
    pub ier_dlh: RwReg<W, u8>,
    pub iir_fcr: RwReg<W, u8>,
    pub lcr: RwReg<W, u8>,
    pub mcr: RwReg<W, u8>,
    pub lsr: RoReg<W, u8>,
    pub msr: RoReg<W, u8>,
    pub scratch: RwReg<W, u8>,
}

#[repr(C)]
pub struct RoReg<const W: usize, T: Copy>([RO<T>; W]);

impl<const W: usize, T: Copy> RoReg<W, T> {
    /// Reads the value of the register
    #[inline(always)]
    pub fn read(&self) -> T {
        self.0[0].read()
    }
}

#[repr(C)]
pub struct RwReg<const W: usize, T: Copy>([RW<T>; W]);

impl<const W: usize, T: Copy> RwReg<W, T> {
    /// Performs a read-modify-write operation
    ///
    /// NOTE: `unsafe` because writes to a register are side effectful
    #[inline(always)]
    pub unsafe fn modify<F>(&self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        self.0[0].modify(f);
    }

    /// Reads the value of the register
    #[inline(always)]
    pub fn read(&self) -> T {
        self.0[0].read()
    }

    /// Writes a `value` into the register
    ///
    /// NOTE: `unsafe` because writes to a register are side effectful
    #[inline(always)]
    pub unsafe fn write(&self, value: T) {
        self.0[0].write(value);
    }
}

impl<const W: usize> Registers<W> {
    /// Constructs a new instance of the UART registers starting at the given base address.
    pub unsafe fn from_base_address(base_address: usize) -> &'static mut Self {
        &mut *(base_address as *mut Registers<W>)
    }
}
