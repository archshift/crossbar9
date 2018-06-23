use core::intrinsics;
use core::ptr;

const SHA_BASE: u32 = 0x1000A000u32;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
enum Reg {
    CNT = 0x00,
    BLK_CNT = 0x04,
    HASH = 0x40,
    FIFO = 0x80,
}

#[derive(Clone, Copy)]
pub enum HashMode {
    SHA256 = 0,
    SHA224 = 1,
    SHA1 = 2,
    SHA1_ = 3,
}

bfdesc!(CntReg: u32, {
    start: 0 => 0,
    final_round: 1 => 1,
    enable_irq0: 2 => 2,
    big_endian: 3 => 3,
    hash_mode: 4 => 5,
    clear_fifo: 8 => 8,
    enable_fifo: 9 => 9,
    enable_irq1: 10 => 10
});

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { ptr::read_volatile((SHA_BASE + reg as u32) as *const T) }
}

fn write_reg<T: Copy>(reg: Reg, val: T) {
    unsafe { ptr::write_volatile((SHA_BASE + reg as u32) as *mut T, val); }
}

fn write_fifo<F: Fn()>(reg: Reg, fifo_size: usize, buf: &[u8], sync_fn: F) {
    let mut buf_ptr = buf.as_ptr();
    let mut bytes_remaining = buf.len();
    while bytes_remaining >= fifo_size {
        sync_fn();
        unsafe {
            intrinsics::volatile_copy_nonoverlapping_memory((SHA_BASE + reg as u32) as *mut u8,
                                                            buf_ptr, fifo_size);
        }
        bytes_remaining -= fifo_size;
        buf_ptr = unsafe { buf_ptr.offset(fifo_size as isize) };
    }

    if bytes_remaining > 0 {
        sync_fn();
        unsafe {
            intrinsics::volatile_copy_nonoverlapping_memory((SHA_BASE + reg as u32) as *mut u8,
                                                            buf_ptr, bytes_remaining);
        }
    }
}

fn sha_is_working() -> bool {
    bf!((read_reg(Reg::CNT)) @ CntReg::start) == 1
}

fn run_hasher(mode: HashMode, buf: &[u8]) {
    while sha_is_working() {}

    // Reset SHA device
    let mut cnt_reg = 0u32;
    bf!(cnt_reg @ CntReg::start = 1);
    bf!(cnt_reg @ CntReg::clear_fifo = 1);
    write_reg(Reg::CNT, cnt_reg);
    write_reg(Reg::CNT, 0u32);

    // Set the amount of bytes to hash
    write_reg(Reg::BLK_CNT, buf.len() as u32);

    // Enable SHA device with proper parameters
    cnt_reg = 0u32;
    bf!(cnt_reg @ CntReg::start = 1);
    bf!(cnt_reg @ CntReg::big_endian = 1);
    bf!(cnt_reg @ CntReg::hash_mode = mode as u32);
    write_reg(Reg::CNT, cnt_reg);

    write_fifo(Reg::FIFO, 0x40, buf, || {
        while sha_is_working() {}
    });

    // Halt SHA device
    cnt_reg = read_reg(Reg::CNT);
    bf!(cnt_reg @ CntReg::start = 0);
    bf!(cnt_reg @ CntReg::final_round = 1);
    write_reg(Reg::CNT, cnt_reg);

    while sha_is_working() {}
}

pub fn hash_256(buf: &[u8]) -> [u8; 32] {
    run_hasher(HashMode::SHA256, buf);
    read_reg(Reg::HASH)
}

pub fn hash_224(buf: &[u8]) -> [u8; 28] {
    run_hasher(HashMode::SHA224, buf);
    read_reg(Reg::HASH)
}

pub fn hash_160(buf: &[u8]) -> [u8; 20] {
    run_hasher(HashMode::SHA1, buf);
    read_reg(Reg::HASH)
}
