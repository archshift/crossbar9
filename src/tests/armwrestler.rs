use core::fmt::Write;

use ffistr;
use gfx;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    unsafe {
        aw_test0();
        aw_test1();
        aw_test2();
        aw_test3();
        // aw_test4();
        aw_test5();
    }
}

extern {
    fn aw_test0();
    fn aw_test1();
    fn aw_test2();
    fn aw_test3();
    fn aw_test4();
    fn aw_test5();

    static szLDRtype: [u8; 12*5];
    static aw_rn_val: u32;
    static aw_mem_val: u32;
}

#[no_mangle]
pub extern fn aw_draw_text(string: *const u8, x: u32, y: u32, color: u32) {

}

#[no_mangle]
pub extern fn aw_draw_result(string: *const u8, status: u32, extra_data: u32) {
    let mut logger = gfx::LogWriter;

    gfx::log(unsafe { ffistr::str_bytes(&string) });

    let mut skip_flags = false;
    if status & 0x80000000 != 0 {
        let txti = extra_data as usize;
        gfx::log(unsafe { &szLDRtype[5*txti .. 5*txti+4] });
    } else if status & 0x40000000 != 0 {
        skip_flags = true;
    }

    gfx::log(b" ");
    if status & 0xFF == 0 {
        gfx::log(b"OK\n");
    } else {
        gfx::log(b"FAIL");
        if !skip_flags {
            if status & 1 != 0 { gfx::log(b" C-flag"); }
            if status & 2 != 0 { gfx::log(b" N-flag"); }
            if status & 4 != 0 { gfx::log(b" V-flag"); }
            if status & 8 != 0 { gfx::log(b" Z-flag"); }
            if status & 0x40 != 0 { gfx::log(b" Q-flag"); }
        }
        if status & 0x10 != 0 { gfx::log(b" Rd-val"); }
        if status & 0x20 != 0 {
            write!(&mut logger, " Rn-val(0x{:08X})", unsafe { aw_rn_val });
        }
        if status & 0x80 != 0 {
            write!(&mut logger, " mem-val(0x{:08X})", unsafe { aw_mem_val });
        }
        gfx::log(b"\n");
    }
}
