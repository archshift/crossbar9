#![feature(lang_items, core_intrinsics)]
#![no_std]
#![no_main]

#[macro_use]
extern crate bitutils;
extern crate rlibc;

mod gfx;
mod io;
pub mod interrupts;
pub mod intrinsics;
mod realtime;
mod tests;
mod unique;

#[no_mangle]
pub extern fn main() {
    tests::main();
}