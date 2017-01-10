mod screen;
mod text;

pub use self::screen::*;
pub use self::text::*;

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

        Bitmap3 {
            bytes: &self.bytes[first_byte..last_byte],
            rect: rect,
            skip_pixels: parent_row_size - rect.0
        }
    }

    pub fn bytes<'b>(&'b self) -> impl Iterator<Item=u8> + 'b {
        let row_size = 3 * (self.rect.0 + self.skip_pixels);
        let rect_size = 3 * self.rect.0;

        self.bytes.iter().enumerate()
            .filter(move |&(n, x)| n % row_size < rect_size)
            .map(|(n, x)| *x)
    }
}