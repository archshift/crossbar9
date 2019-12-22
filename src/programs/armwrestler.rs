use gfx;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    unsafe {
        aw_test0();
        aw_test1();
        aw_test2();
        aw_test3();
        aw_test4();
        aw_test5();
    }
}

extern {
    fn aw_test0();
    fn aw_test1();
    fn aw_test2();
    fn aw_test3();
    fn aw_test4();
    fn aw_test5();
}
