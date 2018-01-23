#![feature(lang_items, core_intrinsics, i128_type, conservative_impl_trait)]
#![no_std]
#![no_main]

#[macro_use]
extern crate bitutils;
extern crate rlibc;

#[macro_use]
mod gfx;
#[macro_use]
mod io;

mod caches;
mod ffistr;

pub mod interrupts;
pub mod intrinsics;
mod power;
mod realtime;
pub mod tests;

#[no_mangle]
pub extern fn main() {
    tests::main();

    log!("Press SELECT to power off.");
    while !io::hid::buttons_pressed().0[io::hid::Button::SELECT.trailing_zeros() as usize] {}
    power::power_off()
}