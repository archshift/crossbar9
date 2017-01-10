#[lang = "eh_personality"]
extern fn eh_personality() { }

#[lang = "panic_fmt"]
#[no_mangle]
fn panic_fmt() -> ! {
	panic_3ds();
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
