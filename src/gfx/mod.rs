mod screen;
#[macro_use]
mod text;
// mod ui;

pub use self::screen::*;
pub use self::text::*;

pub struct Bitmap3<'a> {
    bytes: &'a [u8],
    rect: (usize, usize),
    stride: usize
}

impl<'a> Bitmap3<'a> {
    pub fn submap(&self, pos: (usize, usize), rect: (usize, usize)) -> Bitmap3 {
        let ((x, y), (w, h)) = (pos, rect);
        // first byte: rows*row_size + columns
        let first_byte = y * self.stride + 3*x;
        // last byte: first_byte + extra_rows*row_size + extra_columns
        let last_byte = first_byte + (h - 1) * self.stride + 3*w;

        assert!((last_byte - first_byte) % 3 == 0);

        Bitmap3 {
            bytes: &self.bytes[first_byte..last_byte],
            rect: rect,
            stride: self.stride
        }
    }

    pub fn foreach_byte<F>(&self, f: F)
        where F: Fn((usize, usize), [u8;3]) {

        let (w, _h) = self.rect;

        let chunks = self.bytes.chunks(self.stride);
        for (y, row) in chunks.enumerate() {
            let mut subpix = row;

            for x in 0..w {
                f((x, y), [subpix[0], subpix[1], subpix[2]]);

                subpix = &subpix[3..];
            }
        }

    }
}
