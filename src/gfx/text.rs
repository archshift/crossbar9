use gfx::{Bitmap3, blit, SCREEN_WIDTH, SCREEN_HEIGHT};

static FONT: &'static [u8; 0x2400] = include_bytes!("font.data");

fn draw_letter(pos: (usize, usize), letter: u8) {
    let font_bmp = Bitmap3 {
        bytes: &FONT[..],
        rect: (128, 24),
        skip_pixels: 0
    };

    let letter_num_cols = 32;
    let letter_size = (3, 5);
    let letter_padding = 1;

    let letter_col = letter as usize % letter_num_cols;
    let letter_row = letter as usize / letter_num_cols;
    let letter_x = (letter_size.0 + letter_padding) * letter_col;
    let letter_y = (letter_size.1 + letter_padding) * letter_row;

    let letter_bmp = font_bmp.submap((letter_x, letter_y), letter_size);

    blit(pos, &letter_bmp);
}

pub fn draw_string(pos: (usize, usize), string: &[u8]) {
    let mut x = pos.0;
    for c in string {
        draw_letter((x, pos.1), *c);
        x += 4;
    }
}

static mut CURSOR: (usize, usize) = (2, 2);

pub fn log(string: &[u8]) {
    let (mut x, mut y) = unsafe { (CURSOR.0, CURSOR.1) };

    for c in string {
        if *c == b'\n' {
            y += 10; x = 2; // newline
        } else {
            draw_letter((x, y), *c);
            x += 4;

            if x > SCREEN_WIDTH {
                y += 10; x = 2; // newline
            }
        }
        if y > SCREEN_HEIGHT {
            y = 2;
        }
    }

    unsafe { CURSOR = (x, y) };
}