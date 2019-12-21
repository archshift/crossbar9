use ffistr;
use gfx;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    unsafe {
        tw_test0();
        // tw_test1();
        // tw_test2();
    }
}

extern {
    fn tw_test0();
    fn tw_test1();
    fn tw_test2();
}