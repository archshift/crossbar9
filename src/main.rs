#![allow(unused_parens)]
#![allow(unused_attributes)] // Needed because of rustc bug #60050
#![deny(warnings)]
#![no_std]
#![no_main]

#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate bitutils;
extern crate rlibc;
extern crate u2N;

#[macro_use]
pub mod gfx;
#[macro_use]
pub mod io;

mod ffistr;

pub mod caches;
pub mod interrupts;
pub mod intrinsics;
pub mod fat;
pub mod mem;
pub mod power;
pub mod programs;
pub mod realtime;
pub mod input;


#[no_mangle]
pub extern fn main() {
    programs::main();
    log!("Press SELECT to power off.");
    input::wait_for_all_of(&[io::hid::Button::Select]);
    power::power_off();
}

#[global_allocator]
static ALLOCATOR: mem::Allocator = mem::Allocator::new();

#[alloc_error_handler]
extern fn alloc_error(_: core::alloc::Layout) -> ! {
    panic!("Out of memory!");
}