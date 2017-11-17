use core::mem;

use gfx;
use io::ndma;

fn make_data() -> [u32; 0x200] {
    let mut data = [0u32; 0x200];
    for (n, word) in data.iter_mut().enumerate() {
        *word = (n as u32) * 0x100;
    }
    data
}

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    gfx::log(b"Starting NDMA memory fill... ");
    let mut data = make_data();
    let src = ndma::NdmaSrc::FillData(0);
    let dst = ndma::NdmaDst::LinearBuf(data.as_mut_ptr(), data.len());
    ndma::mem_transfer(src, dst);

    let ok = data.iter().all(|word| *word == 0);
    gfx::log(if ok { b"SUCCEEDED!\n" } else { b"FAILED!\n" });


    gfx::log(b"Starting NDMA memory copy... ");
    let src_data = make_data();
    let mut dst_data = [0u32; 0x200];
    let src = ndma::NdmaSrc::LinearBuf(src_data.as_ptr(), src_data.len());
    let dst = ndma::NdmaDst::LinearBuf(dst_data.as_mut_ptr(), dst_data.len());
    ndma::mem_transfer(src, dst);

    let ok = src_data[..] == dst_data[..];
    gfx::log(if ok { b"SUCCEEDED!\n" } else { b"FAILED!\n" });
}
