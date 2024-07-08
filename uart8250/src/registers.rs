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
pub struct Registers<R: Register + Copy> {
    pub thr_rbr_dll: RwReg<R>,
    pub ier_dlh: RwReg<R>,
    pub iir_fcr: RwReg<R>,
    pub lcr: RwReg<R>,
    pub mcr: RwReg<R>,
    pub lsr: RoReg<R>,
    pub msr: RoReg<R>,
    pub scratch: RwReg<R>,
}

#[repr(C)]
pub struct RoReg<R: Register + Copy>(RO<R>);

impl<R: Register + Copy> RoReg<R> {
    /// Reads the value of the register
    #[inline(always)]
    pub fn read(&self) -> u8 {
        self.0.read().val()
    }
}

#[repr(C)]
pub struct RwReg<R: Register + Copy>(RW<R>);

impl<R: Register + Copy> RwReg<R> {
    /// Performs a read-modify-write operation
    ///
    /// NOTE: `unsafe` because writes to a register are side effectful
    #[inline(always)]
    pub unsafe fn modify<F>(&self, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        self.0.write(f(self.read()).into());
    }

    /// Reads the value of the register
    #[inline(always)]
    pub fn read(&self) -> u8 {
        self.0.read().val()
    }

    /// Writes a `value` into the register
    ///
    /// NOTE: `unsafe` because writes to a register are side effectful
    #[inline(always)]
    pub unsafe fn write(&self, value: u8) {
        self.0.write(value.into())
    }
}

impl<R: Register + Copy> Registers<R> {
    /// Constructs a new instance of the UART registers starting at the given base address.
    pub unsafe fn from_base_address(base_address: usize) -> &'static mut Self {
        &mut *(base_address as *mut Registers<R>)
    }
}

pub trait Register: From<u8> {
    /// 取出寄存器中的有效位。
    fn val(self) -> u8;
}

/// 寄存器的 8 位模式。
impl Register for u8 {
    #[inline]
    fn val(self) -> u8 {
        self
    }
}

/// 寄存器的 32 位模式。
impl Register for u32 {
    #[inline]
    fn val(self) -> u8 {
        self as _
    }
}
