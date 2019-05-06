use gfx;

use io::xdma;

#[repr(align(16))]
struct AlignedBuf([u8; 0x400]);

fn make_data() -> AlignedBuf {
    let mut data = AlignedBuf([0u8; 0x400]);
    let mut ctr = 0;
    for byte in data.0.iter_mut() {
        *byte = ctr as u8;
        ctr += 5;
    }
    data
}

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    print!("Starting mem->mem XDMA... ");

    let buf0 = make_data();
    let mut buf1 = AlignedBuf([0u8; 0x400]);

    xdma::mem_transfer(
        xdma::XdmaSrc::LinearBuf(buf0.0.as_ptr(), buf0.0.len()),
        xdma::XdmaDst::LinearBuf(buf1.0.as_mut_ptr(), buf1.0.len())
    );

    assert_eq!(&buf0.0[..], &buf1.0[..]);

    log!("SUCCESS!");
}
