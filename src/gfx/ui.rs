use gfx::screen::FbGetter;

pub struct UiBox {
    pos: (usize, usize),
    size: (usize, usize),
    color: [u8; 3],
}

impl UiBox {
    pub fn subbox(&self, margin: (usize, usize)) -> UiBox {
        UiBox {
            pos: (self.pos.0 + margin.0, self.pos.1 + margin.1),
            size: (self.size.0 - 2*margin.0, self.size.1 - 2*margin.1),
            color: self.color
        }
    }

    pub fn color(self, rgb: [u8; 3]) -> UiBox {
        UiBox { color: rgb, ..self }
    }
}

pub struct Ui<'a> {
    fb_getter: FbGetter<'a>,
    bg_color: [u8;3]
}

impl<'a> Ui<'a> {
    pub fn new(fb_getter: FbGetter<'a>, bg_color: [u8;3]) -> Self {
        Self {
            fb_getter, bg_color
        }
    }

    pub fn draw_box(&self, b: UiBox) {
        let mut fb = (self.fb_getter)();

        for x in b.pos.0 .. b.pos.1 + b.size.0 {
            for y in b.pos.1 .. b.pos.1 + b.size.1 {
                fb.draw_pixel((x, y), b.color);
            }
        }
    }

    pub fn subbox(&self, margin: (usize, usize)) -> UiBox {
        let size = (self.fb_getter)().size();
        UiBox {
            pos: (margin.0, margin.1),
            size: (size.0 - 2*margin.0, size.1 - 2*margin.1),
            color: self.bg_color
        }
    }
}
