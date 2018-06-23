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

    let mut dst_data = make_data();

    print!("Starting NDMA memory fill... ");
    let src = ndma::NdmaSrc::FillData(0);
    let dst = ndma::NdmaDst::LinearBuf(dst_data.as_mut_ptr(), dst_data.len());
    ndma::mem_transfer(src, dst);

    let ok = dst_data.iter().all(|word| *word == 0);
    gfx::log(if ok { b"SUCCEEDED!\n" } else { b"FAILED!\n" });


    print!("Starting NDMA memory copy... ");
    let src_data = make_data();
    dst_data = [0u32; 0x200];
    let src = ndma::NdmaSrc::LinearBuf(src_data.as_ptr(), src_data.len());
    let dst = ndma::NdmaDst::LinearBuf(dst_data.as_mut_ptr(), dst_data.len());
    ndma::mem_transfer(src, dst);

    let ok = src_data[..] == dst_data[..];
    gfx::log(if ok { b"SUCCEEDED!\n" } else { b"FAILED!\n" });


    print!("Starting NDMA memory copy (fixed-src)... ");
    let src_data = 0xF000BAAA;
    dst_data = [0u32; 0x200];
    let src = ndma::NdmaSrc::FixedAddr(&src_data);
    let dst = ndma::NdmaDst::LinearBuf(dst_data.as_mut_ptr(), dst_data.len());
    ndma::mem_transfer(src, dst);

    let ok = dst_data.iter().all(|word| *word == src_data);
    gfx::log(if ok { b"SUCCEEDED!\n" } else { b"FAILED!\n" });


    // Shows that NDMA actually loads from memory with each store
    print!("Starting NDMA memory copy (rand-src)... ");
    let mut dst_data = [0u32; 0x500];
    let src = ndma::NdmaSrc::FixedAddr(0x10011000 as *const u32);
    let dst = ndma::NdmaDst::LinearBuf(dst_data.as_mut_ptr(), dst_data.len());
    ndma::mem_transfer(src, dst);

    let avg: u32 = dst_data.iter().map(|x| x / (dst_data.len() as u32)).sum();
    // Ideally, distribution is even and has range 0 to FFFFFFFF
    let within_mean_range = |x| x > 0x78000000 && x < 0x88000000;
    let ok = within_mean_range(avg) && !dst_data.iter().cloned().all(within_mean_range);
    gfx::log(if ok { b"SUCCEEDED!\n" } else { b"FAILED!\n" });
}
