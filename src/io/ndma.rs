use core::ptr;

const NDMA_BASE: u32 = 0x10002000u32;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
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

bf!(RegGlobalCnt[u32] {
    global_enable: 0:0,
    cycle_sel: 16:19,
    use_round_robin: 31:31
});

bf!(RegCnt[u32] {
    dst_update_method: 10:11,
    dst_reload: 12:12,
    src_update_method: 13:14,
    src_reload: 15:15,
    blk_xfer_size: 16:19,
    startup_mode: 24:27,
    mode_immediate: 28:28,
    mode_repeating: 29:29,
    irq_enabled: 30:30,
    busy: 31:31
});

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg, channel: u32) -> T {
    unsafe { ptr::read_volatile((NDMA_BASE + channel*0x1C + reg as u32) as *const T) }
}

#[inline(never)]
fn write_reg<T: Copy>(reg: Reg, val: T, channel: u32) {
    unsafe { ptr::write_volatile((NDMA_BASE + channel*0x1C + reg as u32) as *mut T, val); }
}

pub enum NdmaSrc {
    FillData(u32),
    FixedAddr(*const u32),
    LinearBuf(*const u32, usize),
}

pub enum NdmaDst {
    FixedAddr(*mut u32),
    LinearBuf(*mut u32, usize)
}

impl NdmaSrc {
    fn max_xfer_words(&self) -> Option<usize> {
        match *self {
            NdmaSrc::FillData(_) | NdmaSrc::FixedAddr(_)  => None,
            NdmaSrc::LinearBuf(_, len) => Some(len),
        }
    }

    fn src_type_index(&self) -> u32 {
        match *self {
            NdmaSrc::FillData(_) => 3,
            NdmaSrc::FixedAddr(_) => 2,
            NdmaSrc::LinearBuf(_, _) => 0 // Increasing
        }
    }
}

impl NdmaDst {
    fn max_xfer_words(&self) -> Option<usize> {
        match *self {
            NdmaDst::FixedAddr(_) => None,
            NdmaDst::LinearBuf(_, len) => Some(len),
        }
    }

    fn dst_type_index(&self) -> u32 {
        match *self {
            NdmaDst::FixedAddr(_) => 2,
            NdmaDst::LinearBuf(_, _) => 0 // Increasing
        }
    }
}

fn max_xfer_words(src: &NdmaSrc, dst: &NdmaDst, limit: Option<usize>) -> usize {
    let vals = [src.max_xfer_words(), dst.max_xfer_words(), limit];
    let mut it = vals.iter().filter_map(|x| *x);
    let size = it.next()
        .expect("Cannot NDMA with inconsistent buffer sizes!");
    if !it.all(|item| item == size) {
        panic!("Cannot NDMA with no defined transfer size!");
    }
    size
}

pub fn mem_transfer(src: NdmaSrc, dst: NdmaDst) {
    // Ensure global settings
    let channel = 1;

    let mut global_cnt = RegGlobalCnt::new(0);
    global_cnt.global_enable.set(1);
    write_reg(Reg::GLOBAL_CNT, global_cnt, 0);

    let mut cnt = RegCnt::new(0);
    write_reg(Reg::CNT, cnt, channel);
    while { cnt = read_reg(Reg::CNT, channel); cnt.busy.get() == 1 } { }

    cnt.val = 0;

    match src {
        NdmaSrc::FillData(data) => {
            write_reg(Reg::FILL_DATA, data, channel);
            cnt.src_update_method.set(3); // Fill
        }
        NdmaSrc::LinearBuf(ptr, _) | NdmaSrc::FixedAddr(ptr) => {
            if (ptr as u32) & 0b11 != 0 {
                panic!("Tried to NDMA from a non-word-aligned address!");
            }
            write_reg(Reg::SRC_ADDR, ptr as u32, channel);
        }
    }

    match dst {
        NdmaDst::LinearBuf(ptr, _) | NdmaDst::FixedAddr(ptr) => {
            if (ptr as u32) & 0b11 != 0 {
                panic!("Tried to NDMA to a non-word-aligned address!");
            }
            write_reg(Reg::DST_ADDR, ptr as u32, channel);
        }
    }

    let xfer_size = max_xfer_words(&src, &dst, None);
    write_reg(Reg::WRITE_CNT, xfer_size as u32, channel);

    cnt.src_update_method.set(src.src_type_index());
    cnt.dst_update_method.set(dst.dst_type_index());
    cnt.mode_immediate.set(1);
    cnt.busy.set(1); // Start
    write_reg(Reg::CNT, cnt, channel);

    while { cnt = read_reg(Reg::CNT, channel); cnt.busy.get() == 1 } { }
}
