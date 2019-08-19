use gfx::Bitmap3;

pub static SCREEN_WIDTH: usize = 400;
pub static SCREEN_HEIGHT: usize = 240;
fn top_screen_addr() -> u32 {
    extern {
        static TOP_FRAMEBUF_START: u32;
    }
    unsafe { TOP_FRAMEBUF_START }
}

pub unsafe fn draw_pixel(fb_addr: u32, pos: (usize, usize), [r, g, b]: [u8;3]) {
    let (x, y) = pos;
    let (x, y) = (SCREEN_HEIGHT - y - 1, x);
    let base_addr = fb_addr + (3 * (y * SCREEN_HEIGHT + x)) as u32;
    (base_addr as *mut [u8;3]).write_volatile([r, g, b]);
}

pub fn draw_commit() {
    ::caches::flush_dcache()
}

fn no_filter(_: (usize, usize), rgb: [u8; 3]) -> (bool, [u8; 3]) {
    (true, rgb)
}

unsafe fn filter_mask_blit_<F>(fb_addr: u32, pos: (usize, usize), bmp: &Bitmap3, f: F)
    where F: Fn((usize, usize), [u8; 3]) -> (bool, [u8; 3]) {

    bmp.foreach_byte(|(x, y), rgb| {
        let (draw, rgb) = f((x, y), rgb);
        if !draw { return }
        draw_pixel(fb_addr, (pos.0 + x, pos.1 + y), rgb);
    });
}

pub fn blit(pos: (usize, usize), bmp: &Bitmap3) {
    unsafe { filter_mask_blit_(top_screen_addr(), pos, bmp, no_filter); }
}

pub fn text_blit(pos: (usize, usize), bmp: &Bitmap3, color: [u8; 3]) {
    let filter = |_, rgb: [u8; 3]| {
        let draw = rgb[0] == 0 && rgb[1] == 0 && rgb[2] == 0;
        (draw, color)
    };
    unsafe { filter_mask_blit_(top_screen_addr(), pos, bmp, filter); }
}

unsafe fn clear_screen_(fb_addr: u32, r: u8, g: u8, b: u8) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            draw_pixel(fb_addr, (x, y), [r, g, b]);
        }
    }
}

#[inline(never)]
pub fn clear_screen(r: u8, g: u8, b: u8) {
    unsafe { clear_screen_(top_screen_addr(), r, g, b); }
}
