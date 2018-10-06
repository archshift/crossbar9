#![feature(core_intrinsics)]
#![allow(unused_parens)]
#![deny(warnings)]
#![no_std]
#![no_main]

#[macro_use]
extern crate bitutils;
extern crate rlibc;

#[macro_use]
mod gfx;
#[macro_use]
pub mod io;

mod caches;
mod ffistr;

pub mod interrupts;
pub mod intrinsics;
pub mod power;
pub mod realtime;
pub mod tests;

#[no_mangle]
pub extern fn main() {
    tests::main();

    log!("Press SELECT to power off.");
    while !io::hid::buttons_pressed().0[io::hid::button::SELECT.trailing_zeros() as usize] {}
    power::power_off()
}
