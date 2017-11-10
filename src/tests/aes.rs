use gfx;
use io::aes;

pub static KEYX: &[u8] = &[0xd2, 0x2f, 0x5e, 0x15, 0xee, 0xfb, 0x12, 0x0d, 0x50, 0xf7, 0x6b, 0xbc, 0x76, 0x1a, 0x8f, 0x41];
pub static KEYY: &[u8] = &[0xe7, 0x1c, 0x6c, 0x13, 0xe8, 0x0e, 0x40, 0x70, 0x1c, 0x1f, 0x03, 0x11, 0x14, 0x8b, 0x73, 0x8b];
pub static KEYY_TWL: &[u8] = &[0xd3, 0xa5, 0xee, 0xe7, 0x30, 0x65, 0xb0, 0xe3, 0x1e, 0xcd, 0xa1, 0x28, 0x33, 0x74, 0x2b, 0x82];
pub static NORM_KEY: &[u8] = &[0xde, 0x95, 0x19, 0xe2, 0x8b, 0x67, 0xcd, 0x7e, 0xf7, 0x8c, 0xf0, 0x06, 0x26, 0xb1, 0x04, 0x1f];
pub static IV: &[u8] = &[0x4a, 0x25, 0x3b, 0xd1, 0x0a, 0xf1, 0x4a, 0xc4, 0x7c, 0xfd, 0xae, 0xf8, 0x20, 0xbe, 0x56, 0x58];

pub static TEXT: &[u8] = b"I'm just going to input 32 chars";
pub static ENCRYPTED_CBC: &[u8] = &[0xf2, 0xa2, 0x4e, 0x2b, 0xba, 0x56, 0x67, 0xa0, 0x56, 0x3c, 0x4d, 0xf8,
                                    0xca, 0xa6, 0x84, 0x63, 0xc1, 0xcf, 0x2f, 0x8f, 0xcf, 0x1e, 0x86, 0x3d,
                                    0x10, 0x9e, 0x51, 0x94, 0x7a, 0xf3, 0x5a, 0xe7];
pub static ENCRYPTED_ECB: &[u8] = &[0x6a, 0xcc, 0xde, 0xba, 0x78, 0x83, 0x2c, 0x32, 0x37, 0x44, 0xfd, 0x7f,
                                    0x3e, 0xf4, 0x12, 0x28, 0x14, 0x3c, 0xd4, 0x0b, 0xff, 0xa5, 0x7d, 0xab,
                                    0xf1, 0x8a, 0x28, 0xe9, 0x24, 0xc7, 0x80, 0x14];
pub static ENCRYPTED_CTR: &[u8] = &[0x43, 0xde, 0x1b, 0x35, 0x20, 0x9b, 0xc6, 0xba, 0x5f, 0xe8, 0xfd, 0xdb,
                                    0x33, 0xee, 0x1a, 0x04, 0x96, 0x9d, 0x12, 0x82, 0x74, 0xdb, 0x7d, 0x21,
                                    0xc5, 0x1e, 0xb8, 0x19, 0xa5, 0xa7, 0x40, 0x14];

fn print_ifeq_res<T: PartialEq, I1, I2>(a: I1, b: I2)
    where I1: Iterator<Item = T>, I2: Iterator<Item = T> {
    if a.eq(b) {
        gfx::log(b"SUCCEEDED!\n");
    } else {
        gfx::log(b"FAILED!\n");
    }
}

fn reverse_word_bytes<'a>(buf: &'a mut [u8]) {
    for c in buf.chunks_mut(4) {
        c.reverse();
    }
}

fn with_reverse_words<'a>(buf: &'a [u8]) -> impl Iterator<Item = &'a u8> {
    buf.chunks(4).rev().flat_map(|c| c.iter())
}

fn with_reverse_words_in_block<'a>(buf: &'a [u8]) -> impl Iterator<Item = &'a u8> {
    buf.chunks(16).flat_map(|c| with_reverse_words(c))
}

fn with_reverse_word_bytes<'a>(buf: &'a [u8]) -> impl Iterator<Item = &'a u8> {
    buf.chunks(4).flat_map(|c| c.iter().rev())
}


#[inline(never)]
fn test_keypair() {
    let mut buf = [0u8;32];
    let mut ctx = aes::AesContext::new().unwrap()
        .with_keypair(KEYX, KEYY);

    gfx::log(b"Starting AES-CBC encryption (keypair)... ");
    buf.copy_from_slice(TEXT);
    ctx.crypt128(aes::Mode::CBC, aes::Direction::Encrypt, &mut buf[..], Some(IV));
    print_ifeq_res(buf.iter(), ENCRYPTED_CBC.iter());

    ctx = ctx.force_dsi_keygen(true)
        .with_keypair(KEYX, KEYY_TWL);

    gfx::log(b"Starting AES-CBC decryption (keypair, DSigen)... ");
    buf.copy_from_slice(ENCRYPTED_CBC);
    ctx.crypt128(aes::Mode::CBC, aes::Direction::Decrypt, &mut buf[..], Some(IV));
    print_ifeq_res(buf.iter(), TEXT.iter());
}

#[inline(never)]
fn test_normkey() {
    let mut ctr = [0u8;16];
    let mut buf = [0u8;32];
    let mut ctx = aes::AesContext::new().unwrap()
        .with_normalkey(NORM_KEY);

    gfx::log(b"Starting AES-CBC encryption (normal)... ");
    buf.copy_from_slice(TEXT);
    ctx.crypt128(aes::Mode::CBC, aes::Direction::Encrypt, &mut buf[..], Some(IV));
    print_ifeq_res(buf.iter(), ENCRYPTED_CBC.iter());

    gfx::log(b"Starting AES-CBC decryption (normal)... ");
    buf.copy_from_slice(ENCRYPTED_CBC);
    ctx.crypt128(aes::Mode::CBC, aes::Direction::Decrypt, &mut buf[..], Some(IV));
    print_ifeq_res(buf.iter(), TEXT.iter());

    gfx::log(b"Starting AES-ECB encryption (normal)... ");
    buf.copy_from_slice(TEXT);
    ctx.crypt128(aes::Mode::ECB, aes::Direction::Encrypt, &mut buf[..], None);
    print_ifeq_res(buf.iter(), ENCRYPTED_ECB.iter());

    gfx::log(b"Starting AES-ECB decryption (normal)... ");
    buf.copy_from_slice(ENCRYPTED_ECB);
    ctx.crypt128(aes::Mode::ECB, aes::Direction::Decrypt, &mut buf[..], None);
    print_ifeq_res(buf.iter(), TEXT.iter());

    gfx::log(b"Starting AES-CTR encryption (full)... ");
    buf.copy_from_slice(TEXT);
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Encrypt, &mut buf[..], Some(IV));
    print_ifeq_res(buf.iter(), ENCRYPTED_CTR.iter());

    gfx::log(b"Starting AES-CTR decryption (full)... ");
    buf.copy_from_slice(ENCRYPTED_CTR);
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Decrypt, &mut buf[..], Some(IV));
    print_ifeq_res(buf.iter(), TEXT.iter());

    gfx::log(b"Starting AES-CTR encryption (block-wise)... ");
    buf.copy_from_slice(TEXT);
    ctr.copy_from_slice(IV);
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Encrypt, &mut buf[0..16], Some(&ctr));
    ctr = aes::ctr_add(&ctr, aes::buf_num_blocks(&buf[0..16]).unwrap());
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Encrypt, &mut buf[16..], Some(&ctr));
    print_ifeq_res(buf.iter(), ENCRYPTED_CTR.iter());

    gfx::log(b"Starting AES-CTR decryption (block-wise)... ");
    buf.copy_from_slice(ENCRYPTED_CTR);
    ctr.copy_from_slice(IV);
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Decrypt, &mut buf[0..16], Some(&ctr));
    ctr = aes::ctr_add(&ctr, aes::buf_num_blocks(&buf[0..16]).unwrap());
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Decrypt, &mut buf[16..], Some(&ctr));
    print_ifeq_res(buf.iter(), TEXT.iter());

    gfx::log(b"Starting AES-ECB decryption (output-le)... ");
    buf.copy_from_slice(ENCRYPTED_ECB);
    ctx = ctx.with_output_le(true);
    ctx.crypt128(aes::Mode::ECB, aes::Direction::Decrypt, &mut buf[..], Some(IV));
    ctx = ctx.with_output_le(false);
    print_ifeq_res(buf.iter(), with_reverse_word_bytes(TEXT));

    gfx::log(b"Starting AES-ECB decryption (output-rev)... ");
    buf.copy_from_slice(ENCRYPTED_ECB);
    ctx = ctx.with_output_rev_words(true);
    ctx.crypt128(aes::Mode::ECB, aes::Direction::Decrypt, &mut buf[..], Some(IV));
    ctx = ctx.with_output_rev_words(false);
    print_ifeq_res(buf.iter(), with_reverse_words_in_block(TEXT));
}

#[inline(never)]
fn test_rev_normkey() {
    let mut rev_key = [0u8; 16];
    rev_key.copy_from_slice(NORM_KEY);
    reverse_word_bytes(&mut rev_key);

    let mut buf = [0u8;32];
    let mut ctx = aes::AesContext::new().unwrap()
        .with_normalkey(&rev_key);

    gfx::log(b"Starting AES-ECB decryption (input-le, normal)... ");
    buf.copy_from_slice(ENCRYPTED_CTR);
    reverse_word_bytes(&mut buf);

    let mut rev_ctr = [0u8; 16];
    rev_ctr.copy_from_slice(IV);
    reverse_word_bytes(&mut rev_ctr);

    ctx = ctx.with_input_le(true);
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Decrypt, &mut buf[..], Some(&rev_ctr));
    ctx = ctx.with_input_le(false);
    print_ifeq_res(buf.iter(), TEXT.iter());
}

#[inline(never)]
fn test_rev_keypair() {
    let mut rev_key0 = [0u8; 16];
    rev_key0.copy_from_slice(KEYX);
    reverse_word_bytes(&mut rev_key0);

    let mut rev_key1 = [0u8; 16];
    rev_key1.copy_from_slice(KEYY);
    reverse_word_bytes(&mut rev_key1);

    let mut buf = [0u8;32];
    let mut ctx = aes::AesContext::new().unwrap()
        .with_keypair(&rev_key0, &rev_key1);

    gfx::log(b"Starting AES-ECB decryption (input-le, keypair)... ");
    buf.copy_from_slice(ENCRYPTED_CTR);
    reverse_word_bytes(&mut buf);

    let mut rev_ctr = [0u8; 16];
    rev_ctr.copy_from_slice(IV);
    reverse_word_bytes(&mut rev_ctr);

    ctx = ctx.with_input_le(true);
    ctx.crypt128(aes::Mode::CTR, aes::Direction::Decrypt, &mut buf[..], Some(&rev_ctr));
    ctx = ctx.with_input_le(false);
    print_ifeq_res(buf.iter(), TEXT.iter());
}

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    test_keypair();
    test_normkey();
    test_rev_normkey();
    test_rev_keypair();
}