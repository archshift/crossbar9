use core::ptr;

const NDMA_BASE: u32 = 0x10002000u32;

#[derive(Clone, Copy)]
enum Reg {
    GLOBAL_CNT = 0x00,
    SRC_ADDR = 0x04,
    DST_ADDR = 0x08,
    TRANSFER_CNT = 0x0C,
    WRITE_CNT = 0x10,
    BLOCK_CNT = 0x14,
    FILL_DATA = 0x18,
    CNT = 0x1C
}

bfdesc!(RegGlobalCnt: u32, {
    global_enable: 0 => 0,
    cycle_sel: 16 => 19,
    use_round_robin: 31 => 31
});

bfdesc!(RegCnt: u32, {
    dst_update_method: 10 => 11,
    dst_reload: 12 => 12,
    src_update_method: 13 => 14,
    src_reload: 15 => 15,
    blk_xfer_size: 16 => 19,
    startup_mode: 24 => 27,
    mode_immediate: 28 => 28,
    mode_repeating: 29 => 29,
    irq_enabled: 30 => 30,
    busy: 31 => 31
});

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg, channel: u32) -> T {
    unsafe { ptr::read_volatile((NDMA_BASE + channel*0x1C + reg as u32) as *const T) }
}

#[inline(never)]
fn write_reg<T: Copy>(reg: Reg, val: T, channel: u32) {
    unsafe { ptr::write_volatile((NDMA_BASE + channel*0x1C + reg as u32) as *mut T, val); }
}

pub fn mem_fill_words(dest: *mut u32, words: usize, data: u32) {
    // Ensure global settings
    let channel = 1;

    if (dest as u32) & 0b11 != 0 {
        panic!("Tried to NDMA to a non-word-aligned address!");
    }

    let mut global_cnt = 0;
    bf!(global_cnt @ RegGlobalCnt::global_enable = 1);
    write_reg(Reg::GLOBAL_CNT, global_cnt, 0);

    write_reg(Reg::CNT, 0, channel);
    while bf!((read_reg(Reg::CNT, channel)) @ RegCnt::busy) == 1 { }

    write_reg(Reg::FILL_DATA, data, channel);
    write_reg(Reg::DST_ADDR, dest as u32, channel);
    write_reg(Reg::WRITE_CNT, words as u32, channel);

    let mut cnt = 0;
    bf!(cnt @ RegCnt::dst_update_method = 0); // Increment
    bf!(cnt @ RegCnt::src_update_method = 3); // Fill
    bf!(cnt @ RegCnt::mode_immediate = 1);
    bf!(cnt @ RegCnt::busy = 1); // Start
    write_reg(Reg::CNT, cnt, channel);

    while bf!((read_reg(Reg::CNT, channel)) @ RegCnt::busy) == 1 { }
}