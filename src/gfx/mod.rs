mod screen;
mod text;

pub use self::screen::*;
pub use self::text::*;

pub struct RgbIter<'a> {
    index: usize,
    slice: &'a [u8],
    row_size: usize,
    rect_width: usize,
}

impl<'a> Iterator for RgbIter<'a> {
    type Item = (u8, u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let mut index = self.index;
        while index + 3 <= self.slice.len() {
            self.index += 3;
            if index % self.row_size < self.rect_width {
                return Some((self.slice[index],
                             self.slice[index+1],
                             self.slice[index+2]))
            }
            index += 3;
        }
        None
    }
}

pub struct Bitmap3<'a> {
    bytes: &'a [u8],
    rect: (usize, usize),
    skip_pixels: usize
}

impl<'a> Bitmap3<'a> {
    pub fn submap(&self, pos: (usize, usize), rect: (usize, usize)) -> Bitmap3 {
        let ((x, y), (w, h)) = (pos, rect);

        let parent_row_size = self.rect.0 + self.skip_pixels;
        // first byte: rows*row_size + columns
        let first_byte = 3 * (y * parent_row_size + x);
        // last byte: first_byte + extra_rows*row_size + extra_columns
        let last_byte = first_byte + 3 * ((h - 1) * parent_row_size + w);

        assert!((last_byte - first_byte) % 3 == 0);

        Bitmap3 {
            bytes: &self.bytes[first_byte..last_byte],
            rect: rect,
            skip_pixels: parent_row_size - rect.0
        }
    }

    pub fn bytes(&self) -> RgbIter {
        let row_size = 3 * (self.rect.0 + self.skip_pixels);
        let rect_width = 3 * self.rect.0;

        RgbIter {
            index: 0,
            slice: self.bytes,
            row_size: row_size,
            rect_width: rect_width,
        }
    }
}