use ffistr;
use gfx;

extern {
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

#[no_mangle]
pub extern fn tw_draw_text(string: *const u8, _x: u32, _y: u32, _color: u32) {
    gfx::log(unsafe { ffistr::str_bytes(&string) });
    log!("");
}

#[no_mangle]
pub extern fn tw_draw_result_(string: *const u8, status: u32) {
    gfx::log(unsafe { ffistr::str_bytes(&string) });

    print!(" ");
    if status == 0 {
        log!("OK");
    } else {
        log!("FAIL");
    }
}

// _drawresult:
// @ r0: lpszText
// @ r6: bitmask
// 	push 	{r4-r5,lr}

// 	mov 	r1,#16
// 	mov 	r2,r7
// 	mov 	r3,#3
// 	bl 	_drawtext

// 	cmp 	r6,#0
// 	beq 	_dr_ok

// 	ldr 	r0,=_szBad
// 	mov 	r1,#72
// 	mov 	r2,r7
// 	mov 	r3,#2
// 	bl 	_drawtext

// 	mov 	r5,#C_MASK
// 	ldr 	r0,=_szC
// 	mov 	r1,#104
// 	mov 	r2,r7
// 	mov 	r3,#2
// 	_dr_test_flags:
// 		tst 	r6,r5
// 		beq 	_dr_flag_ok
// 			bl 	_drawtext
// 		_dr_flag_ok:
// 		add 	r0,#2
// 		add 	r1,#8
// 		lsl 	r5,r5,#1
// 		cmp 	r5,#16
// 		bne 	_dr_test_flags

// 	mov 	r5,#V_MASK
// 	ldr 	r0,=_szV
// 	mov 	r1,#112
// 	mov 	r2,r7
// 	mov 	r3,#2
// 	_dv_test_flags:
// 		tst 	r6,r5
// 		beq 	_dv_flag_ok
// 			bl 	_drawtext
// 		_dv_flag_ok:
// 		add 	r0,#2
// 		add 	r1,#8
// 		lsl 	r5,r5,#1
// 		cmp 	r5,#16
// 		bne 	_dv_test_flags

// 	ldr 	r5,=Rd_MASK
// 	tst 	r6,r5
// 	beq 	_dr_rd_ok
// 		ldr 	r0,=_szRd
// 		mov 	r1,#144
// 		mov 	r2,r7
// 		mov 	r3,#2
// 		bl 	_drawtext
// 	_dr_rd_ok:

// 	b 	_dr_done

// 	_dr_ok:
// 	ldr 	r0,=_szOK
// 	mov 	r1,#72
// 	mov 	r2,r7
// 	mov 	r3,#1
// 	bl 	_drawtext

// 	_dr_done:
// 	pop 	{r4-r5,pc}
// .align
// .pool