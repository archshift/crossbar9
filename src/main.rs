#![feature(lang_items, core_intrinsics, conservative_impl_trait)]
#![no_std]
#![no_main]

#[macro_use]
extern crate bitutils;
extern crate rlibc;

mod gfx;
mod io;
pub mod interrupts;
pub mod intrinsics;
mod unique;

#[no_mangle]
pub extern fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    gfx::log(b"Hello, world! ");

    fn timer_test() { gfx::log(b"T"); }
    let mut timer = io::timer::Timer::new(0, 0, io::timer::Prescaler::Div1024, Some(timer_test)).unwrap();
    timer.start();
    loop {}
}