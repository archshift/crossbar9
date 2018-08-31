use core;

#[panic_handler]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    ::gfx::clear_screen(0xFF, 0xFF, 0xFF);
    ::gfx::reset_log_cursor();
    
    let msg = info.payload().downcast_ref::<&str>().unwrap();
    let loc = info.location().unwrap();

    log!("PANIC PANIC PANIC PANIC PANIC");
    log!("{} @ {}, L{}:{}", msg, loc.file(), loc.line(), loc.column());

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
