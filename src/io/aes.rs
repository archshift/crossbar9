use core::intrinsics;
use core::iter;
use core::mem;

const AES_BASE: u32 = 0x10009000u32;

#[derive(Clone, Copy)]
enum Reg {
    CNT = 0x000,
    BLK_CNT = 0x006,
    FIFO_IN = 0x008,
    FIFO_OUT = 0x00C,
    KEY_SEL = 0x010,
    KEY_CNT = 0x011,
    CTR = 0x020,
    KEY_FIFO = 0x100,
}

bfdesc!(CntReg: u32, {
    fifo_in_count: 0 => 4,
    fifo_out_count: 5 => 9,
    flush_fifo_in: 10 => 10,
    flush_fifo_out: 11 => 11,
    fifo_in_dma_size: 12 => 13,
    fifo_out_dma_size: 14 => 15,
    mac_size: 16 => 18,
    mac_source_reg: 20 => 20,
    mac_verified: 21 => 21,
    out_big_endian: 22 => 22,
    in_big_endian: 23 => 23,
    out_normal_order: 24 => 24,
    in_normal_order: 25 => 25,
    update_keyslot: 26 => 26,
    mode: 27 => 29,
    enable_irq: 30 => 30,
    busy: 31 => 31
});

bfdesc!(KeyCntReg: u8, {
    keyslot: 0 => 5,
    use_dsi_keygen: 6 => 6,
    enable_fifo_flush: 7 => 7
});

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { intrinsics::volatile_load((AES_BASE + reg as u32) as *const T) }
}

#[inline(never)]
fn write_reg<T: Copy>(reg: Reg, val: T) {
    unsafe { intrinsics::volatile_store((AES_BASE + reg as u32) as *mut T, val); }
}

#[derive(Clone, Copy)]
pub enum Direction {
    Encrypt,
    Decrypt
}

pub fn crypt_cbc128(key: &[u8], iv: &[u8], msg: &mut [u8], direction: Direction) {
    { // Init
        let mut cnt = 0;
        bf!(cnt @ CntReg::flush_fifo_in = 1);
        bf!(cnt @ CntReg::flush_fifo_out = 1);
        bf!(cnt @ CntReg::out_big_endian = 1);
        bf!(cnt @ CntReg::out_normal_order = 1);
        bf!(cnt @ CntReg::in_big_endian = 1);
        bf!(cnt @ CntReg::in_normal_order = 1);
        write_reg(Reg::CNT, cnt);
    }

    { // Write key
        let mut key_cnt = 0;
        bf!(key_cnt @ KeyCntReg::keyslot = 0x3F);
        bf!(key_cnt @ KeyCntReg::enable_fifo_flush = 1);
        write_reg(Reg::KEY_CNT, key_cnt);

        assert!(key.len() == 0x10);
        let mut key_it = key.iter();
        while let (Some(b0), Some(b1), Some(b2), Some(b3))
            = (key_it.next(), key_it.next(), key_it.next(), key_it.next()) {

            let bytes = [*b0, *b1, *b2, *b3];
            write_reg::<[u8;4]>(Reg::KEY_FIFO, bytes);
        }
    }

    { // Write IV
        let mut iv_words = [0u32; 4];
        {
            let mut iv_it = iv.iter();
            let mut iv_word_it = iv_words.iter_mut().rev();

            assert!(iv.len() == 0x10);
            while let (Some(word), Some(b0), Some(b1), Some(b2), Some(b3))
                = (iv_word_it.next(), iv_it.next(), iv_it.next(), iv_it.next(), iv_it.next()) {

                let bytes = [*b0, *b1, *b2, *b3];
                *word = unsafe { mem::transmute(bytes) };
            }

        }
        write_reg(Reg::CTR, iv_words);
    }

    { // Select keyslot
        write_reg(Reg::KEY_SEL, 0x3Fu8);

        let mut cnt = read_reg::<u32>(Reg::CNT);
        bf!(cnt @ CntReg::update_keyslot = 1);
        write_reg(Reg::CNT, cnt);
    }

    { // Start processing
        assert!(msg.len() % 16 == 0);
        write_reg(Reg::BLK_CNT, (msg.len() >> 4) as u16);

        let mode = match direction {
            Direction::Decrypt => 4, // AES-CBC
            Direction::Encrypt => 5,
        };

        let mut cnt = read_reg::<u32>(Reg::CNT);
        bf!(cnt @ CntReg::mode = mode);
        bf!(cnt @ CntReg::busy = 1);
        write_reg(Reg::CNT, cnt);
    }

    { // Perform crypto
        let fifo_in_full = || {
            let cnt: u32 = read_reg(Reg::CNT);
            bf!(cnt @ CntReg::fifo_in_count) == 16
        };
        let fifo_out_empty = || {
            let cnt: u32 = read_reg(Reg::CNT);
            bf!(cnt @ CntReg::fifo_out_count) == 0
        };

        let mut pos = 0;
        while pos < msg.len() {
            while fifo_in_full() { }

            for i in 0..4 {
                let mut bytes = [0u8;4];
                bytes.copy_from_slice(&msg[pos + i*4 .. pos + i*4+4]);
                write_reg::<[u8;4]>(Reg::FIFO_IN, bytes);
            }

            while fifo_out_empty() { }

            for i in 0..4 {
                let bytes: [u8; 4] = read_reg(Reg::FIFO_OUT);
                msg[pos + i*4 .. pos + i*4+4].copy_from_slice(&bytes[..]);
            }

            pos += 16;
        }
    }
}
