use core::slice;

pub unsafe fn str_bytes<'a>(raw: &'a *const u8) -> &'a [u8] {
    slice::from_raw_parts(*raw, str_len(*raw))
}

pub unsafe fn str_len(mut s: *const u8) -> usize {
    let mut count = 0;
    while *s != 0 {
        count += 1;
        s = s.add(1);
    }
    count
}

pub unsafe fn str_cpy(mut dst: *mut u8, mut s: *const u8) -> usize {
    let mut count = 0;
    while *s != 0 {
        *dst = *s;
        count += 1;
        s = s.add(1);
        dst = dst.add(1);
    }
    count
}
