use gfx;
use io::aes;

pub static KEYX: &[u8] = &[0xd2, 0x2f, 0x5e, 0x15, 0xee, 0xfb, 0x12, 0x0d, 0x50, 0xf7, 0x6b, 0xbc, 0x76, 0x1a, 0x8f, 0x41];
pub static KEYY: &[u8] = &[0xe7, 0x1c, 0x6c, 0x13, 0xe8, 0x0e, 0x40, 0x70, 0x1c, 0x1f, 0x03, 0x11, 0x14, 0x8b, 0x73, 0x8b];
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

fn print_ifeq_res<T: PartialEq>(a: T, b: T) {
    if a == b {
        gfx::log(b"SUCCEEDED!\n");
    } else {
        gfx::log(b"FAILED!\n");
    }
}

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    let mut buf = [0u8;32];

    gfx::log(b"Starting AES-CBC encryption (normal, rand-iv)... ");
    buf.copy_from_slice(TEXT);
    aes::crypt128(NORM_KEY, None, Some(IV), &mut buf[..], aes::Mode::CBC, aes::Direction::Encrypt);
    print_ifeq_res(&buf[..], ENCRYPTED_CBC);

    gfx::log(b"Starting AES-CBC decryption (normal, rand-iv)... ");
    buf.copy_from_slice(ENCRYPTED_CBC);
    aes::crypt128(NORM_KEY, None, Some(IV), &mut buf[..], aes::Mode::CBC, aes::Direction::Decrypt);
    print_ifeq_res(&buf[..], TEXT);

    gfx::log(b"Starting AES-CBC encryption (keypair)... ");
    buf.copy_from_slice(TEXT);
    aes::crypt128(KEYX, Some(KEYY), Some(IV), &mut buf[..], aes::Mode::CBC, aes::Direction::Encrypt);
    print_ifeq_res(&buf[..], ENCRYPTED_CBC);

    gfx::log(b"Starting AES-ECB encryption (normal)... ");
    buf.copy_from_slice(TEXT);
    aes::crypt128(NORM_KEY, None, Some(IV), &mut buf[..], aes::Mode::ECB, aes::Direction::Encrypt);
    print_ifeq_res(&buf[..], ENCRYPTED_ECB);

    gfx::log(b"Starting AES-ECB decryption (normal)... ");
    buf.copy_from_slice(ENCRYPTED_ECB);
    aes::crypt128(NORM_KEY, None, None, &mut buf[..], aes::Mode::ECB, aes::Direction::Decrypt);
    print_ifeq_res(&buf[..], TEXT);

    let mut ctr = [0u8;16];

    gfx::log(b"Starting AES-CTR encryption (full)... ");
    buf.copy_from_slice(TEXT);
    aes::crypt128(NORM_KEY, None, Some(IV), &mut buf[..], aes::Mode::CTR, aes::Direction::Encrypt);
    print_ifeq_res(&buf[..], ENCRYPTED_CTR);

    gfx::log(b"Starting AES-CTR encryption (block-wise)... ");
    buf.copy_from_slice(TEXT);
    ctr.copy_from_slice(IV);
    aes::crypt128(NORM_KEY, None, Some(&ctr), &mut buf[0..16], aes::Mode::CTR, aes::Direction::Encrypt);
    ctr = aes::ctr_add(&ctr, aes::buf_num_blocks(&buf[0..16]).unwrap());
    aes::crypt128(NORM_KEY, None, Some(&ctr), &mut buf[16..], aes::Mode::CTR, aes::Direction::Encrypt);
    print_ifeq_res(&buf[..], ENCRYPTED_CTR);

    gfx::log(b"Starting AES-CTR decryption (full)... ");
    buf.copy_from_slice(ENCRYPTED_CTR);
    aes::crypt128(NORM_KEY, None, Some(IV), &mut buf[..], aes::Mode::CTR, aes::Direction::Decrypt);
    print_ifeq_res(&buf[..], TEXT);

    gfx::log(b"Starting AES-CTR decryption (block-wise)... ");
    buf.copy_from_slice(ENCRYPTED_CTR);
    ctr.copy_from_slice(IV);
    aes::crypt128(NORM_KEY, None, Some(&ctr), &mut buf[0..16], aes::Mode::CTR, aes::Direction::Decrypt);
    ctr = aes::ctr_add(&ctr, aes::buf_num_blocks(&buf[0..16]).unwrap());
    aes::crypt128(NORM_KEY, None, Some(&ctr), &mut buf[16..], aes::Mode::CTR, aes::Direction::Decrypt);
    print_ifeq_res(&buf[..], TEXT);
}