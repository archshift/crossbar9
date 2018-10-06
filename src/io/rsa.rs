use core::intrinsics;
use core::iter;

const RSA_BASE: u32 = 0x1000B000u32;

#[derive(Clone, Copy)]
struct RegSlot {
    cnt: u32,
    size: u32,
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum Reg {
    CNT = 0x000,
    UNK = 0x0F0,
    SLOT0 = 0x100,
    SLOT1 = 0x110,
    SLOT2 = 0x120,
    SLOT3 = 0x130,
    EXPFIFO = 0x200,
    MOD = 0x400,
    TXT = 0x800,
}

bf!(CntReg[u32] {
    busy: 0:0,
    keyslot: 4:5,
    little_endian: 8:8,
    normal_word_order: 9:9
});

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { intrinsics::volatile_load((RSA_BASE + reg as u32) as *const T) }
}

#[inline(never)]
fn write_reg<T: Copy>(reg: Reg, val: T) {
    unsafe { intrinsics::volatile_store((RSA_BASE + reg as u32) as *mut T, val); }
}

pub fn crypt_2048_opt(key: &[u8], modulus: &[u8], msg: &[u8], little_endian: bool, normal_word_order: bool) -> [u8; 0x100] {
    write_reg(Reg::UNK, 0u32);

    { // Update slot information
        let mut slot = read_reg::<RegSlot>(Reg::SLOT3);
        assert!(slot.cnt & 0x2 == 0);
        if slot.cnt & (1 << 31) == 0 {
            slot.cnt &= !0x1;
        }
        slot.size = 0x40;
        write_reg(Reg::SLOT3, slot);
    }

    { // Update CNT
        let mut cnt = CntReg::new(0);
        cnt.keyslot.set(3);
        cnt.little_endian.set(little_endian as u32);
        cnt.normal_word_order.set(normal_word_order as u32);
        write_reg(Reg::CNT, cnt);
    }

    { // Copy exponent into FIFO one u32 at a time
        let remainder = 0x100 - key.len();
        let mut k_it = iter::repeat(&0u8).take(remainder).chain(key.iter());

        while let (Some(b0), Some(b1), Some(b2), Some(b3))
                = (k_it.next(), k_it.next(), k_it.next(), k_it.next()) {
            let word = (*b3 as u32) << 24 | (*b2 as u32) << 16 | (*b1 as u32) << 8 | (*b0 as u32);
            write_reg::<u32>(Reg::EXPFIFO, word);
        }
    }

    { // Modulus
        let mut mod_buf = [0u8; 0x100];
        mod_buf[..].copy_from_slice(modulus);
        write_reg(Reg::MOD, mod_buf);
    }

    { // Signal keydata updated and ready
        let mut slot = read_reg::<RegSlot>(Reg::SLOT3);
        slot.cnt |= 0x1;
        write_reg(Reg::SLOT3, slot);
    }

    { // Write message/signature
        let mut msg_buf = [0u8; 0x100];
        msg_buf[0x100 - msg.len()..0x100].copy_from_slice(msg);
        write_reg(Reg::TXT, msg_buf);
    }

    { // Start processing
        let mut cnt: CntReg::Bf = read_reg(Reg::CNT);
        cnt.busy.set(1);
        write_reg(Reg::CNT, cnt);
    }

    while read_reg::<CntReg::Bf>(Reg::CNT).busy.get() == 1 { }

    read_reg(Reg::TXT)
}

pub fn crypt_2048(key: &[u8], modulus: &[u8], msg: &[u8]) -> [u8; 0x100] {
    crypt_2048_opt(key, modulus, msg, true, true)
}
