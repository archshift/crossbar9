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

pub struct Byte4Iter<'a>(slice::Chunks<'a, u8>);
impl<'a> Byte4Iter<'a> {
    pub fn new(slice: &'a [u8]) -> Byte4Iter<'a> {
        assert!(slice.len() % 4 == 0);
        Byte4Iter(slice.chunks(4))
    }
}
impl<'a> Iterator for Byte4Iter<'a> {
    type Item = [u8;4];
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(b) = self.0.next() {
            Some([b[0], b[1], b[2], b[3]])
        } else {
            None
        }
    }
}

fn u128_bytes(mut num: u128) -> [u8;0x10] {
    let mut data = [0u8; 0x10];
    for b in data.iter_mut().rev() {
        *b = num as u8;
        num >>= 8;
    }
    data
}

fn u128_from_bytes(data: &[u8]) -> u128 {
    assert!(data.len() == 16);
    let mut new = 0u128;
    for b in data.iter() {
        new <<= 8;
        new |= *b as u128;
    }
    new
}

// Returns Ok(blocks) if aligned to block length, Err(rounded_up) if not aligned
pub fn buf_num_blocks(buf: &[u8]) -> Result<usize, usize> {
    if buf.len() % 16 == 0 {
        Ok(buf.len() >> 4)
    } else {
        Err(buf.len() >> 4 + 1)
    }
}

pub fn ctr_add(ctr: &[u8], blocks: usize) -> [u8;0x10] {
    let num = u128_from_bytes(ctr) + blocks as u128;
    u128_bytes(num)
}

pub struct AesContext<'a> {
    keyslot: u8,
    keywriter: fn(&AesContext, u8, &[u8], Option<&[u8]>),
    key: Option<&'a [u8]>,
    key_y: Option<&'a [u8]>,
    output_le: bool,
}

impl<'a> AesContext<'a> {
    pub fn new() -> Result<AesContext<'a>, ()> {
        // TODO: Check if other contexts are active and fail
        Ok(AesContext {
            keyslot: 0x3F,
            keywriter: keywriter::anykey,
            key: None,
            key_y: None,
            output_le: false,
        })
    }

    pub fn with_keyslot(mut self, keyslot: u8) -> AesContext<'a> {
        AesContext { keyslot: keyslot, ..self }
    }

    pub fn with_keywriter(mut self, keywriter: fn(&AesContext, u8, &[u8], Option<&[u8]>)) -> AesContext<'a> {
        AesContext { keywriter: keywriter, ..self }
    }

    pub fn with_normalkey(mut self, key: &'a [u8]) -> AesContext<'a> {
        AesContext { key: Some(key), key_y: None, ..self }
    }

    pub fn with_keypair(mut self, keyx: &'a [u8], keyy: &'a [u8]) -> AesContext<'a> {
        AesContext { key: Some(keyx), key_y: Some(keyy), ..self }
    }

    pub fn with_output_le(mut self, state: bool) -> AesContext<'a> {
        AesContext { output_le: state, ..self }
    }

    pub fn crypt128(&self, mode: Mode, direction: Direction, msg: &mut [u8], iv_ctr: Option<&[u8]>) {
        let mut cnt = 0;
        bf!(cnt @ CntReg::flush_fifo_in = 1);
        bf!(cnt @ CntReg::flush_fifo_out = 1);
        bf!(cnt @ CntReg::out_big_endian = !self.output_le as u32);
        bf!(cnt @ CntReg::out_normal_order = 1);
        bf!(cnt @ CntReg::in_big_endian = 1);
        bf!(cnt @ CntReg::in_normal_order = 1);
        write_reg(Reg::CNT, cnt);

        if let Some(key) = self.key {
            (self.keywriter)(self, self.keyslot, key, self.key_y);
        }

        let (mode_base, requires_iv) = match mode {
            Mode::CCM => (0, true),
            Mode::CTR => (2, true),
            Mode::CBC => (4, true),
            Mode::ECB => (6, false)
        };

        // Write IV
        if requires_iv {
            let mut iv_words = [0u32; 4];
            if let Some(iv) = iv_ctr {
                assert!(iv.len() == 0x10);

                let mut iv_it = Byte4Iter::new(iv);
                let mut iv_word_it = iv_words.iter_mut().rev();

                for (word, bytes4) in iv_word_it.zip(iv_it) {
                    *word = unsafe { mem::transmute(bytes4) };
                }
            } else {
                panic!("This crypto mode requires an IV/CTR");
            }
            write_reg(Reg::CTR, iv_words);
        }

        { // Select keyslot
            write_reg(Reg::KEY_SEL, self.keyslot);

            let mut cnt = read_reg::<u32>(Reg::CNT);
            bf!(cnt @ CntReg::update_keyslot = 1);
            write_reg(Reg::CNT, cnt);
        }

        { // Start processing
            let msg_blocks = buf_num_blocks(msg).unwrap();
            write_reg(Reg::BLK_CNT, msg_blocks as u16);

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
}

pub mod keywriter {
    use super::*;
    pub fn anykey(ctx: &AesContext, keyslot: u8, key: &[u8], key_y: Option<&[u8]>) {
        let mut key_cnt = 0;
        bf!(key_cnt @ KeyCntReg::keyslot = keyslot);
        bf!(key_cnt @ KeyCntReg::enable_fifo_flush = 1);
        write_reg(Reg::KEY_CNT, key_cnt);

        let key_reg = if key_y.is_some() { Reg::KEYX_FIFO }
                      else { Reg::KEY_FIFO };

        assert!(key.len() == 0x10);
        for bytes4 in Byte4Iter::new(key) {
            write_reg::<[u8;4]>(key_reg, bytes4);
        }

        if let Some(y) = key_y {
            assert!(y.len() == 0x10);
            for bytes4 in Byte4Iter::new(y) {
                write_reg::<[u8;4]>(Reg::KEYY_FIFO, bytes4);
            }

        }
    }
}