mod screen;
#[macro_use]
mod text;

pub use self::screen::*;
pub use self::text::*;

pub struct RgbIter<'a> {
    slice: &'a [u8],
    x: usize,
    y: usize,
    w: usize,
    stride: usize,
}

impl<'a> Iterator for RgbIter<'a> {
    type Item = ((usize, usize), [u8;3]);

    fn next(&mut self) -> Option<Self::Item> {
        let xy = (self.x, self.y);
        let pos = self.y*self.stride + self.x*3;

        if pos >= self.slice.len() {
            return None
        }

        self.x += 1;
        if self.x == self.w {
            self.x = 0;
            self.y += 1;
        }

        Some((xy, [self.slice[pos], self.slice[pos+1], self.slice[pos+2]]))
    }
}

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

    pub fn bytes(&self) -> RgbIter {
        RgbIter {
            slice: self.bytes,
            x: 0,
            y: 0,
            w: self.rect.0,
            stride: self.stride,
        }
    }
}
