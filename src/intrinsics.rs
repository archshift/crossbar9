use core;

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() { }

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(_msg: core::fmt::Arguments, _file: &'static str, _line: u32) -> ! {
    ::gfx::clear_screen(0xFF, 0xFF, 0xFF);
    ::gfx::reset_log_cursor();

    ::gfx::log(b"PANIC PANIC PANIC PANIC PANIC\n");
    let mut screen = ::gfx::LogWriter;
    core::fmt::write(&mut screen, _msg);
    core::fmt::write(&mut screen, format_args!(" @ {}, L{}\n", _file, _line));

    ::gfx::log(b"Press SELECT to power off.\n");
    while !::io::hid::buttons_pressed().0[::io::hid::Button::SELECT.trailing_zeros() as usize] {}
    ::power::power_off()
}

#[no_mangle]
pub extern fn abort() -> ! {
    ::gfx::clear_screen(0xFF, 0x00, 0x00);
    ::gfx::reset_log_cursor();

    ::gfx::draw_string((2, 2), b"ABORTED");
    loop {
        unsafe { ::interrupts::wait_for_interrupt() };
    }
}

extern {
    fn memset(dest: *mut u8, val: i32, n: u32);
}

#[no_mangle]
pub unsafe extern fn __aeabi_memset(dest: *mut u8, n: u32, val: i32) {
    memset(dest, val, n);
}
#[no_mangle]
pub unsafe extern fn __aeabi_memset4(dest: *mut u8, n: u32, val: i32) {
    memset(dest, val, n);
}
#[no_mangle]
pub unsafe extern fn __aeabi_memset8(dest: *mut u8, n: u32, val: i32) {
    memset(dest, val, n);
}

#[no_mangle]
pub unsafe extern fn __aeabi_memclr(dest: *mut u8, n: u32) {
    memset(dest, 0, n);
}
#[no_mangle]
pub unsafe extern fn __aeabi_memclr4(dest: *mut u8, n: u32) {
    memset(dest, 0, n);
}
#[no_mangle]
pub unsafe extern fn __aeabi_memclr8(dest: *mut u8, n: u32) {
    memset(dest, 0, n);
}
