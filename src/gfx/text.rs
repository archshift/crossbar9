use core::fmt;
use alloc::vec::Vec;

use gfx::{Bitmap3, text_blit, top_screen};

static FONT: [u8; 0x2400] = *include_bytes!("font.data");

macro_rules! print {
    ($($tok:tt)*) => {{
        use ::core::fmt::Write;
        write!(::gfx::LogWriter, $($tok)*).unwrap();
    }};
}

macro_rules! log {
    ($($tok:tt)*) => {{ print!($($tok)*); print!("\n"); }};
}

fn draw_letter(pos: (usize, usize), letter: u8) {
    static FONT_BMP: Bitmap3 = Bitmap3 {
        bytes: &FONT,
        rect: (128, 24),
        stride: 128*3
    };

    if letter >= 128 {
        panic!("Tried to print non-ascii letter! data=0x{:02X}", letter);
    }

    let letter_num_cols = 32;
    let letter_size = (3, 6);
    let letter_padding = (1, 0);

    let letter_col = letter as usize % letter_num_cols;
    let letter_row = letter as usize / letter_num_cols;
    let letter_x = (letter_size.0 + letter_padding.0) * letter_col;
    let letter_y = (letter_size.1 + letter_padding.1) * letter_row;

    let letter_bmp = FONT_BMP.submap((letter_x, letter_y), letter_size);

    text_blit(pos, &letter_bmp, [0x42, 0x09, 0x03]);
}

pub fn draw_string(pos: (usize, usize), string: &[u8]) {
    let mut x = pos.0;
    for c in string {
        draw_letter((x, pos.1), *c);
        x += 4;
    }
}

struct LogPages {
    text: [ Vec<u8>; 8 ],
    head: usize,
    active_page: usize,
}

impl LogPages {
    fn new() -> Self {
        Self {
            text: [ Vec::new(), Vec::new(), Vec::new(), Vec::new(),
                    Vec::new(), Vec::new(), Vec::new(), Vec::new() ],
            head: 0,
            active_page: 0,
        }
    }

    fn flip_to(&mut self, page: usize) {
        if self.active_page == page {
            return;
        }
        self.active_page = page;
        let buf = &self.text[ (self.head + page) % self.text.len() ];
        ::gfx::clear_screen(0xFF, 0xFF, 0xFF);
        log_iter(buf.iter().copied(), (2, 2), None);
    }

    fn flip_by(&mut self, direction: isize) {
        let dst_page = (self.active_page as isize) + direction;
        if dst_page >= (self.text.len() as isize) || dst_page < 0 {
            return;
        }
        self.flip_to(dst_page as usize);
    }

    fn push(&mut self, c: u8) {
        self.text[self.head].push(c);
    }

    fn add_page(&mut self) {
        self.head = (self.head + self.text.len() - 1) % self.text.len();
        self.text[self.head].clear();
        ::gfx::clear_screen(0xFF, 0xFF, 0xFF);
    }
}

static mut LOG_PAGES: Option<LogPages> = None;
static mut CURSOR: (usize, usize) = (2, 2);

pub fn reset_log_cursor() {
    unsafe { CURSOR = (2, 2) };
}

#[inline(never)]
pub fn log(string: &[u8]) {
    unsafe {
        if let Some(l) = LOG_PAGES.as_mut() {
            l.flip_to(0);
        } else {
            LOG_PAGES = Some(LogPages::new())
        }
        let str_it = string.iter().copied();
        CURSOR = log_iter(str_it, CURSOR, LOG_PAGES.as_mut())
    };
}

pub fn log_scroll(direction: isize) {
    unsafe {
        if let Some(l) = LOG_PAGES.as_mut() {
            l.flip_by(direction);
        }
    }
}

pub fn log_clear() {
    unsafe {
        if let Some(l) = LOG_PAGES.as_mut() {
            l.add_page();
        }
    }
    reset_log_cursor();
}

fn log_iter<I>(it: I, cursor: (usize, usize), mut dst_pages: Option<&mut LogPages>)
    -> (usize, usize)
    where I: Iterator<Item = u8>
{
    let (mut x, mut y) = cursor;
    let (screen_w, screen_h) = top_screen().size();

    let newline = |x: &mut usize, y: &mut usize, lp: &mut Option<&mut LogPages>| {
        *y += 10;
        *x = 2;
        if *y >= screen_h {
            *y = 2;

            if let Some(lp) = lp {
                lp.add_page();
            }
        }
    };

    for c in it {
        if c == b'\n' {
            if let Some(lp) = &mut dst_pages {
                lp.push(c);
            }
            newline(&mut x, &mut y, &mut dst_pages);
        } else {
            if x + 4 >= screen_w {
                newline(&mut x, &mut y, &mut dst_pages);
            }

            draw_letter((x, y), c);
            x += 4;
            if let Some(lp) = &mut dst_pages {
                lp.push(c);
            }
        }
    }

    (x, y)
}

pub struct LogWriter;

impl fmt::Write for LogWriter {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        log(s.as_bytes());
        // log_iter(s.chars().flat_map(|c|c.escape_unicode()).map(|c|c as u8));
        Ok(())
    }
}
