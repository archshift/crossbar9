use gfx::screen::draw_pixel;

pub struct UiBox {
    pos: (usize, usize),
    size: (usize, usize),
    color: [u8; 3],
}

impl UiBox {
    fn subbox(&self, margin: (usize, usize)) -> UiBox {
        UiBox {
            pos: (self.pos.0 + margin.0, self.pos.1 + margin.1),
            size: (self.size.0 - 2*margin.0, self.pos.1 - 2*margin.1),
            color: self.color
        }
    }

    fn color(self, rgb: [u8; 3]) -> UiBox {
        UiBox { color: rgb, ..self }
    }
}

pub struct Ui {
    fb_addr: u32,
}

impl Ui {
    pub fn draw_box(&self, b: UiBox) {
        for x in 0 .. b.size.0 {
            for y in 0 .. b.size.1 {
                unsafe {
                    draw_pixel(self.fb_addr, (x, y), b.color);
                }
            }
        }
    }
}
