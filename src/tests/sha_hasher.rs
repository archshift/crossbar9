use io::sha;
use gfx;

use core::fmt::Write;

pub fn main() {
    let mut logger = gfx::LogWriter;
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    let input = &[1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4,
                  1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4,
                  1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4,
                  1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4,
                  1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];

    gfx::log(b"Testing SHA256 hash...");
    let hash_256 = sha::hash_256(input);
    let expected_256 = &[0xd1, 0x82, 0xef, 0x8e, 0x01, 0x93, 0xa6, 0xd2, 0xb8, 0x7e, 0xfb, 0x5c,
                         0xc8, 0xfc, 0xa2, 0xaf, 0x3c, 0xe6, 0xd4, 0xe0, 0x0d, 0x78, 0x47, 0xea,
                         0x31, 0x67, 0x11, 0x04, 0x94, 0xd2, 0x86, 0xed];

    if &hash_256 == expected_256 { gfx::log(b"SUCCESS!\n"); }
    else { gfx::log(b"FAILURE!\n"); }


    gfx::log(b"Testing SHA224 hash...");
    let hash_224 = sha::hash_224(input);
    let expected_224 = &[0xbb, 0x70, 0xc3, 0xd3, 0x51, 0x0f, 0x35, 0x09, 0xd5, 0x85, 0x9a, 0x55,
                         0x49, 0x25, 0x84, 0x4a, 0x0d, 0x52, 0xa1, 0xc6, 0x2f, 0x7e, 0x23, 0x4c,
                         0x56, 0xdb, 0xac, 0x4d];

    if &hash_224 == expected_224 { gfx::log(b"SUCCESS!\n"); }
    else { gfx::log(b"FAILURE!\n"); }


    gfx::log(b"Testing SHA1 hash...");
    let hash_160 = sha::hash_160(input);
    let expected_160 = &[0xee, 0xd0, 0xe5, 0x9c, 0xd1, 0xba, 0x52, 0x1e, 0xfb, 0xe3, 0xe5, 0xf2,
                         0x8d, 0x6c, 0xad, 0xef, 0x93, 0x0c, 0xe4, 0xf7];

    if &hash_160 == expected_160 { gfx::log(b"SUCCESS!\n"); }
    else { gfx::log(b"FAILURE!\n"); }
}