use core::mem;

use gfx;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    use core::fmt::Write;
    gfx::log(b"Starting NDMA memory fill... ");

    let mut data = [0u32; 0x200];
    for (n, word) in data.iter_mut().enumerate() {
        *word = (n as u32) * 0x100;
    }

    ::io::ndma::mem_fill_words(data.as_mut_ptr(), data.len(), 0);
    let ok = data.iter().all(|word| *word == 0);
    gfx::log(if ok { b"SUCCEEDED!\n" } else { b"FAILED!\n" });
}
