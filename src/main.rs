#![feature(lang_items, core_intrinsics)]
#![no_std]
#![no_main]

pub mod intrinsics;
mod test;

#[no_mangle]
pub extern fn main() {
    test::test();
}
