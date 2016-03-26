use core;

const SCREEN_WIDTH: usize = 240;
const SCREEN_HEIGHT: usize = 400;
const TOP_SCREEN0: u32 = 0x20000000;
const TOP_SCREEN1: u32 = 0x20046500;

fn clear_screen_(fb_addr: u32, r: u8, g: u8, b: u8) {
    let mut i: u32 = fb_addr;
    while i < fb_addr + (3 * SCREEN_WIDTH * SCREEN_HEIGHT) as u32 {
        unsafe {
            core::intrinsics::volatile_store((i + 0) as *mut u8, b);
            core::intrinsics::volatile_store((i + 1) as *mut u8, g);
            core::intrinsics::volatile_store((i + 2) as *mut u8, r);
        }
        i += 3;
    }
}

pub fn clear_screen(r: u8, g: u8, b: u8) {
    clear_screen_(TOP_SCREEN0, r, g, b);
    clear_screen_(TOP_SCREEN1, r, g, b);
}

pub fn test() {
    clear_screen(0xFF, 0xFF, 0xFF);
}