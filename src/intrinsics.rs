use core;

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() { }

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(_msg: core::fmt::Arguments, _file: &'static str, _line: u32) -> ! {
    ::gfx::clear_screen(0xFF, 0xFF, 0xFF);
    ::gfx::reset_log_cursor();

    log!("PANIC PANIC PANIC PANIC PANIC");
    core::fmt::write(&mut ::gfx::LogWriter, _msg).unwrap();
    core::fmt::write(&mut ::gfx::LogWriter, format_args!(" @ {}, L{}\n", _file, _line)).unwrap();

    log!("Press SELECT to power off.");
    while !::io::hid::buttons_pressed().0[::io::hid::button::SELECT.trailing_zeros() as usize] {}
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
