#![feature(lang_items, core_intrinsics, conservative_impl_trait)]
#![no_std]
#![no_main]

extern crate rlibc;

mod gfx;
mod io;
pub mod interrupts;
pub mod intrinsics;

#[no_mangle]
pub extern fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    gfx::log(b"Hello, world!");
}