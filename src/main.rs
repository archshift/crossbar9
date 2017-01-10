#![feature(lang_items, core_intrinsics)]
#![no_std]
#![no_main]

extern crate rlibc;
mod io;
pub mod interrupts;
pub mod intrinsics;
mod test;

#[no_mangle]
pub extern fn main() {
    test::test();
}
