use core::intrinsics;
use core::mem;
use core::slice;

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
    KEYX_FIFO = 0x104,
    KEYY_FIFO = 0x108
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
pub enum Mode {
    CCM,
    CTR,
    CBC,
    ECB
}

#[derive(Clone, Copy)]
pub enum Direction {
    Encrypt,
    Decrypt
}

struct Byte4Iter<'a>(slice::Iter<'a, u8>);
impl<'a> Byte4Iter<'a> {
    fn new(slice: &'a [u8]) -> Byte4Iter<'a> {
        assert!(slice.len() % 4 == 0);
        Byte4Iter(slice.iter())
    }
}
impl<'a> Iterator for Byte4Iter<'a> {
    type Item = [u8;4];
    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(b0), Some(b1), Some(b2), Some(b3))
            = (self.0.next(), self.0.next(), self.0.next(), self.0.next()) {
            Some([*b0, *b1, *b2, *b3])
        } else {
            None
        }
    }
}

pub fn crypt128(key: &[u8], key_y: Option<&[u8]>, iv: Option<&[u8]>, msg: &mut [u8],
                    mode: Mode, direction: Direction) {
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

        let key_reg = if key_y.is_some() { Reg::KEYX_FIFO }
                      else { Reg::KEY_FIFO };

        for bytes4 in Byte4Iter::new(key) {
            write_reg::<[u8;4]>(key_reg, bytes4);
        }

        if let Some(y) = key_y {
            for bytes4 in Byte4Iter::new(y) {
                write_reg::<[u8;4]>(Reg::KEYY_FIFO, bytes4);
            }
        }
    }

    { // Write IV
        let mut iv_words = [0u32; 4];
        if let Some(iv) = iv {
            assert!(iv.len() == 0x10);
            let mut iv_it = Byte4Iter::new(iv);
            let mut iv_word_it = iv_words.iter_mut().rev();

            for (word, bytes4) in iv_word_it.zip(iv_it) {
                *word = unsafe { mem::transmute(bytes4) };
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

        let mode_base = match mode {
            Mode::CCM => 0,
            Mode::CTR => 2,
            Mode::CBC => 4,
            Mode::ECB => 6
        };
        let mode_num = match direction {
            Direction::Decrypt => mode_base,
            Direction::Encrypt => mode_base + 1,
        };

        let mut cnt = read_reg::<u32>(Reg::CNT);
        bf!(cnt @ CntReg::mode = mode_num);
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

            for bytes4 in Byte4Iter::new(&msg[pos .. pos + 16]) {
                write_reg::<[u8;4]>(Reg::FIFO_IN, bytes4);
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
