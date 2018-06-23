use io::RegEnum;

pub const IRQ_BASE: u32 = 0x10001000u32;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
enum Reg {
    ENABLED = 0x00,
    PENDING = 0x04,
}

impl RegEnum for Reg {
    fn addr_of(&self) -> u32 {
        IRQ_BASE + (*self as u32)
    }
}

#[allow(non_snake_case)]
pub mod Interrupt {
    pub const DMAC_1_0: u32      = 1 << 0;
    pub const DMAC_1_1: u32      = 1 << 1;
    pub const DMAC_1_2: u32      = 1 << 2;
    pub const DMAC_1_3: u32      = 1 << 3;
    pub const DMAC_1_4: u32      = 1 << 4;
    pub const DMAC_1_5: u32      = 1 << 5;
    pub const DMAC_1_6: u32      = 1 << 6;
    pub const DMAC_1_7: u32      = 1 << 7;
    pub const TIMER_0: u32       = 1 << 8;
    pub const TIMER_1: u32       = 1 << 9;
    pub const TIMER_2: u32       = 1 << 10;
    pub const TIMER_3: u32       = 1 << 11;
    pub const PXI_SYNC: u32      = 1 << 12;
    pub const PXI_NOT_FULL: u32  = 1 << 13;
    pub const PXI_NOT_EMPTY: u32 = 1 << 14;
    pub const AES: u32           = 1 << 15;
    pub const SDIO_1: u32        = 1 << 16;
    pub const SDIO_1_ASYNC: u32  = 1 << 17;
    pub const SDIO_3: u32        = 1 << 18;
    pub const SDIO_3_ASYNC: u32  = 1 << 19;
    pub const DEBUG_RECV: u32    = 1 << 20;
    pub const DEBUG_SEND: u32    = 1 << 21;
    pub const RSA: u32           = 1 << 22;
    pub const CTR_CARD_1: u32    = 1 << 23;
    pub const CTR_CARD_2: u32    = 1 << 24;
    pub const CGC: u32           = 1 << 25;
    pub const CGC_DET: u32       = 1 << 26;
    pub const DS_CARD: u32       = 1 << 27;
    pub const DMAC_2: u32        = 1 << 28;
    pub const DMAC_2_ABORT: u32  = 1 << 29;
}

pub fn set_enabled(interrupts: u32) {
    Reg::ENABLED.write32(get_enabled() | interrupts);
}

pub fn set_disabled(interrupts: u32) {
    Reg::ENABLED.write32(get_enabled() & !interrupts);
}

pub fn get_enabled() -> u32 {
    Reg::ENABLED.read32()
}

pub fn get_pending() -> u32 {
    Reg::PENDING.read32()
}

pub fn clear_pending(interrupts: u32) {
    Reg::PENDING.write32(interrupts)
}

pub fn clear_all_pending() {
    Reg::PENDING.write32(!0u32);
}
