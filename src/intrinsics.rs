use core;

#[panic_handler]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    ::gfx::clear_screen(0xFF, 0xFF, 0xFF);
    ::gfx::reset_log_cursor();
    
    log!("PANIC PANIC PANIC PANIC PANIC");
    log!("{}", info);

    log!("Press SELECT to power off.");
    ::gfx::draw_commit();
    ::input::wait_for_all_of(&[::io::hid::Button::Select]);
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

#[no_mangle]
pub extern fn __clzsi2(mut val: i32) -> i32 {
    let mut i = 32;
    let mut j = 16;
    let mut temp;

    while j != 0 {
        temp = val >> j;
        if temp != 0 {
            if j == 1 {
                return i - 2;
            } else {
                i -= j;
                val = temp;
            }
        }
        j >>= 1;
    }

    i - val
}
