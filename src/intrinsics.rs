use core;

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() { }

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(_msg: core::fmt::Arguments, _file: &'static str, _line: u32) -> ! {
    use core::fmt::Write;
    struct Screen {};
    impl Write for Screen {
        fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
            ::gfx::log(s.as_bytes());
            Ok(())
        }
    }

    ::gfx::clear_screen(0xFF, 0xFF, 0xFF);
    ::gfx::log(b"PANIC PANIC PANIC PANIC PANIC\n");
    let mut screen = Screen {};
    core::fmt::write(&mut screen, _msg);
    core::fmt::write(&mut screen, format_args!(" @ {}, L{}\n", _file, _line));

    loop {}
}

#[no_mangle]
pub extern fn panic_3ds() -> ! {
    loop {}
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
