use core;

use gfx::Bitmap3;

pub static SCREEN_WIDTH: usize = 240;
pub static SCREEN_HEIGHT: usize = 400;
fn top_screen_addr() -> u32 {
    extern {
        static TOP_FRAMEBUF_START: u32;
    }
    unsafe { TOP_FRAMEBUF_START }
}

unsafe fn draw_pixel(fb_addr: u32, pos: (usize, usize), r: u8, g: u8, b: u8) {
    let base_addr = fb_addr + (3 * (pos.1 * SCREEN_WIDTH + pos.0)) as u32;
    core::intrinsics::volatile_store((base_addr + 0) as *mut u8, b);
    core::intrinsics::volatile_store((base_addr + 1) as *mut u8, g);
    core::intrinsics::volatile_store((base_addr + 2) as *mut u8, r);
}

unsafe fn blit_(fb_addr: u32, pos: (usize, usize), bmp: &Bitmap3) {
    let width = bmp.rect.0;
    let mut curr_pixel = 0;
    for (r, g, b) in bmp.bytes() {
        draw_pixel(fb_addr, (pos.0 + curr_pixel % width, pos.1 + curr_pixel / width), r, g, b);
        curr_pixel += 1;
    }
}

pub fn blit(pos: (usize, usize), bmp: &Bitmap3) {
    unsafe { blit_(top_screen_addr(), pos, bmp); }
}

unsafe fn clear_screen_(fb_addr: u32, r: u8, g: u8, b: u8) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            draw_pixel(fb_addr, (x, y), r, g, b);
        }
    }
}

pub fn clear_screen(r: u8, g: u8, b: u8) {
    unsafe { clear_screen_(top_screen_addr(), r, g, b); }
}
