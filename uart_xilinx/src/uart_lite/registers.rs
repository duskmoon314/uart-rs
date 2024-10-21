use volatile_register::{RO, WO};

/// # UART Registers
#[repr(C)]
pub struct Registers {
    pub rx: RO<u32>,
    pub tx: WO<u32>,
    pub stat: RO<u32>,
    pub ctrl: WO<u32>,
}
