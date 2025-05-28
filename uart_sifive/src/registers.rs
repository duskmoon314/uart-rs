use volatile_register::{RO, RW, WO};

/// # UART Registers
#[repr(C)]
pub struct Registers {
    pub tx: RW<u32>,
    pub rx: RO<u32>,
    pub txctrl: RW<u32>,
    pub rxctrl: RW<u32>,
    pub ie: RW<u32>,
    pub ip: RO<u32>,
    pub div: RW<u32>,
}
