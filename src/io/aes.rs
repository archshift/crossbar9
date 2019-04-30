use core::ptr;
use core::mem;

const AES_BASE: u32 = 0x10009000u32;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum Reg {
    CNT = 0x000,
    BLK_CNT = 0x006,
    FIFO_IN = 0x008,
    FIFO_OUT = 0x00C,
    KEY_SEL = 0x010,
    KEY_CNT = 0x011,
    CTR = 0x020,
    TWL_KEY0 = 0x040,
    TWL_KEY1 = 0x070,
    TWL_KEY2 = 0x0A0,
    TWL_KEY3 = 0x0D0,
    KEY_FIFO = 0x100,
    KEYX_FIFO = 0x104,
    KEYY_FIFO = 0x108
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum TwlKeyReg {
    NORMAL = 0x00,
    KEYX = 0x10,
    KEYY = 0x20
}

bf!(CntReg[u32] {
    fifo_in_count: 0:4,
    fifo_out_count: 5:9,
    flush_fifo_in: 10:10,
    flush_fifo_out: 11:11,
    fifo_in_dma_size: 12:13,
    fifo_out_dma_size: 14:15,
    mac_size: 16:18,
    mac_source_reg: 20:20,
    mac_verified: 21:21,
    out_big_endian: 22:22,
    in_big_endian: 23:23,
    out_normal_order: 24:24,
    in_normal_order: 25:25,
    update_keyslot: 26:26,
    mode: 27:29,
    enable_irq: 30:30,
    busy: 31:31
});

bf!(KeyCntReg[u8] {
    keyslot: 0:5,
    force_dsi_keygen: 6:6,
    enable_fifo_flush: 7:7
});

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { ptr::read_volatile((AES_BASE + reg as u32) as *const T) }
}

#[inline(never)]
fn write_reg<T: Copy>(reg: Reg, val: T) {
    unsafe { ptr::write_volatile((AES_BASE + reg as u32) as *mut T, val); }
}

#[inline(never)]
fn write_reg_twlkey<T: Copy>(keyslot: u8, target: TwlKeyReg, val: T) {
    assert!(keyslot < 4);
    let reg = Reg::TWL_KEY0 as u32 + (keyslot as u32) * 0x30;
    unsafe { ptr::write_volatile((AES_BASE + reg + target as u32) as *mut T, val); }
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

fn byte4iter<'a>(slice: &'a [u8]) -> impl Iterator<Item = [u8;4]> + 'a {
    assert!(slice.len() % 4 == 0);
    slice.chunks(4).map(|c| [c[0], c[1], c[2], c[3]])
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
    input_le: bool,
    output_le: bool,
    output_rev_words: bool,
    force_dsi_keygen: bool
}

impl<'a> AesContext<'a> {
    pub fn new() -> Result<AesContext<'a>, ()> {
        Ok(AesContext {
            keyslot: 0x3F,
            keywriter: keywriter::anykey,
            key: None,
            key_y: None,
            input_le: false,
            output_le: false,
            output_rev_words: false,
            force_dsi_keygen: false
        })
    }

    pub fn with_keyslot(self, keyslot: u8) -> AesContext<'a> {
        AesContext { keyslot: keyslot, ..self }
    }

    pub fn with_keywriter(self, keywriter: fn(&AesContext, u8, &[u8], Option<&[u8]>)) -> AesContext<'a> {
        AesContext { keywriter: keywriter, ..self }
    }

    pub fn with_normalkey(self, key: &'a [u8]) -> AesContext<'a> {
        AesContext { key: Some(key), key_y: None, ..self }
    }

    pub fn with_keypair(self, keyx: &'a [u8], keyy: &'a [u8]) -> AesContext<'a> {
        AesContext { key: Some(keyx), key_y: Some(keyy), ..self }
    }

    pub fn with_input_le(self, state: bool) -> AesContext<'a> {
        AesContext { input_le: state, ..self }
    }

    pub fn with_output_le(self, state: bool) -> AesContext<'a> {
        AesContext { output_le: state, ..self }
    }

    pub fn with_output_rev_words(self, state: bool) -> AesContext<'a> {
        AesContext { output_rev_words: state, ..self }
    }

    pub fn force_dsi_keygen(self, force: bool) -> AesContext<'a> {
        AesContext { force_dsi_keygen: force, ..self }
    }

    pub fn crypt128(&self, mode: Mode, direction: Direction, msg: &mut [u8], iv_ctr: Option<&[u8]>) {
        let mut cnt = CntReg::new(0);
        cnt.flush_fifo_in.set(1);
        cnt.flush_fifo_out.set(1);
        cnt.out_big_endian.set(!self.output_le as u32);
        cnt.out_normal_order.set(!self.output_rev_words as u32);
        cnt.in_big_endian.set(!self.input_le as u32);
        cnt.in_normal_order.set(1);
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

                let iv_it = byte4iter(iv);
                let iv_word_it = iv_words.iter_mut().rev();

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

            let mut cnt = read_reg::<CntReg::Bf>(Reg::CNT);
            cnt.update_keyslot.set(1);
            write_reg(Reg::CNT, cnt);
        }

        { // Start processing
            let msg_blocks = buf_num_blocks(msg).unwrap();
            write_reg(Reg::BLK_CNT, msg_blocks as u16);

            let mode_num = match direction {
                Direction::Decrypt => mode_base,
                Direction::Encrypt => mode_base + 1,
            };

            let mut cnt = read_reg::<CntReg::Bf>(Reg::CNT);
            cnt.mode.set(mode_num);
            cnt.busy.set(1);
            write_reg(Reg::CNT, cnt);
        }

        { // Perform crypto
            let fifo_in_full = || {
                let cnt: CntReg::Bf = read_reg(Reg::CNT);
                cnt.fifo_in_count.get() == 16
            };
            let fifo_out_empty = || {
                let cnt: CntReg::Bf = read_reg(Reg::CNT);
                cnt.fifo_out_count.get() == 0
            };

            let mut pos = 0;
            while pos < msg.len() {
                while fifo_in_full() { }

                for bytes4 in byte4iter(&msg[pos .. pos + 16]) {
                    write_reg::<u32>(Reg::FIFO_IN, unsafe { mem::transmute(bytes4) });
                }

                while fifo_out_empty() { }

                for c in msg[pos .. pos + 16].chunks_mut(4) {
                    let data = read_reg::<u32>(Reg::FIFO_OUT);
                    let data_bytes: [u8; 4] = unsafe { mem::transmute(data) };
                    c.copy_from_slice(&data_bytes);
                }

                pos += 16;
            }
        }
    }
}

pub mod keywriter {
    use super::*;
    pub fn anykey(ctx: &AesContext, keyslot: u8, key: &[u8], key_y: Option<&[u8]>) {
        let mut key_cnt = KeyCntReg::new(0);
        key_cnt.keyslot.set(keyslot);
        key_cnt.enable_fifo_flush.set(1);
        key_cnt.force_dsi_keygen.set(ctx.force_dsi_keygen as u8);
        write_reg(Reg::KEY_CNT, key_cnt);

        let key_reg = if key_y.is_some() { Reg::KEYX_FIFO }
                      else { Reg::KEY_FIFO };

        assert!(key.len() == 0x10);
        for bytes4 in byte4iter(key) {
            write_reg::<u32>(key_reg, unsafe { mem::transmute(bytes4) });
        }

        if let Some(y) = key_y {
            assert!(y.len() == 0x10);
            for bytes4 in byte4iter(y) {
                write_reg::<u32>(Reg::KEYY_FIFO, unsafe { mem::transmute(bytes4) });
            }
        }
    }

    pub fn twlkey(_ctx: &AesContext, keyslot: u8, key: &[u8], key_y: Option<&[u8]>) {
        assert!(keyslot < 4);
        let mut key_cnt = KeyCntReg::new(0);
        key_cnt.keyslot.set(keyslot);
        write_reg(Reg::KEY_CNT, key_cnt);

        if let Some(_y) = key_y {
            unimplemented!();
        } else {
            let mut buf = [0u8; 16];
            buf.copy_from_slice(key);
            let mut buf: [u32; 4] = unsafe { mem::transmute(buf) };
            buf.reverse();
            write_reg_twlkey::<[u32;4]>(keyslot, TwlKeyReg::NORMAL, buf);
        }
    }
}
