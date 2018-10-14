use gfx;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    log!("Hello, world!");

    for i in 0..10000 {
        print!("{}", i % 10);
    }
}
