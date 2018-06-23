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
    // fn aw_test4();
    fn aw_test5();

    static szLDRtype: [u8; 12*5];
    static aw_rn_val: u32;
    static aw_mem_val: u32;
}

#[no_mangle]
pub extern fn aw_draw_text(string: *const u8, _x: u32, _y: u32, _color: u32) {
    gfx::log(unsafe { ffistr::str_bytes(&string) });
    gfx::log(b"\n");
}

#[no_mangle]
pub extern fn aw_draw_result(string: *const u8, status: u32, extra_data: u32) {
    gfx::log(unsafe { ffistr::str_bytes(&string) });

    let mut skip_flags = false;
    if status & 0x80000000 != 0 {
        let txti = extra_data as usize;
        gfx::log(unsafe { &szLDRtype[5*txti .. 5*txti+4] });
    } else if status & 0x40000000 != 0 {
        skip_flags = true;
    }

    print!(" ");
    if status & 0xFF == 0 {
        log!("OK");
    } else {
        print!("FAIL");
        if !skip_flags {
            if status & 1 != 0 { print!(" C-flag"); }
            if status & 2 != 0 { print!(" N-flag"); }
            if status & 4 != 0 { print!(" V-flag"); }
            if status & 8 != 0 { print!(" Z-flag"); }
            if status & 0x40 != 0 { print!(" Q-flag"); }
        }
        if status & 0x10 != 0 { print!(" Rd-val"); }
        if status & 0x20 != 0 {
            print!(" Rn-val(0x{:08X})", unsafe { aw_rn_val });
        }
        if status & 0x80 != 0 {
            print!(" mem-val(0x{:08X})", unsafe { aw_mem_val });
        }
        log!("")
    }
}
