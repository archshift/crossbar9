use core::cell::{RefCell, RefMut};

use gfx::Bitmap3;

pub type FbGetter<'a> = fn() -> RefMut<'a, Framebuffer>;

pub fn top_screen<'a>() -> RefMut<'a, Framebuffer> {
    unsafe {
        TOP_SCREEN.borrow_mut()
    }
}

static mut TOP_SCREEN: RefCell<Framebuffer> = RefCell::new(Framebuffer {
    addr: 0,
    width: 400,
    height: 240
});

pub fn init() {
    extern {
        static TOP_FRAMEBUF_START: u32;
    }

    top_screen().addr = unsafe { TOP_FRAMEBUF_START };
}

pub struct Framebuffer {
    addr: u32,
    width: usize,
    height: usize,
}

impl Framebuffer {
    pub fn draw_pixel(&mut self, pos: (usize, usize), [r, g, b]: [u8;3]) {
        let (x, y) = pos;
        let (x, y) = (self.height - y - 1, x);
        let base_addr = self.addr + (3 * (y * self.height + x)) as u32;
        unsafe {
            (base_addr as *mut [u8;3]).write_volatile([r, g, b]);
        }
    }

    pub fn filter_mask_blit<F>(&mut self, pos: (usize, usize), bmp: &Bitmap3, f: F)
        where F: Fn((usize, usize), [u8; 3]) -> (bool, [u8; 3]) {

        bmp.foreach_byte(|(x, y), rgb| {
            let (draw, rgb) = f((x, y), rgb);
            if !draw { return }
            self.draw_pixel((pos.0 + x, pos.1 + y), rgb);
        });
    }

    pub fn clear(&mut self, rgb: [u8; 3]) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.draw_pixel((x, y), rgb);
            }
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

pub fn draw_commit() {
    ::caches::flush_dcache()
}

fn no_filter(_: (usize, usize), rgb: [u8; 3]) -> (bool, [u8; 3]) {
    (true, rgb)
}

pub fn blit(pos: (usize, usize), bmp: &Bitmap3) {
    top_screen().filter_mask_blit(pos, bmp, no_filter);
}

pub fn text_blit(pos: (usize, usize), bmp: &Bitmap3, color: [u8; 3]) {
    let filter = |_, rgb: [u8; 3]| {
        let draw = rgb[0] == 0 && rgb[1] == 0 && rgb[2] == 0;
        (draw, color)
    };
    top_screen().filter_mask_blit(pos, bmp, filter);
}

#[inline(never)]
pub fn clear_screen(r: u8, g: u8, b: u8) {
    top_screen().clear([r, g, b]);
}
