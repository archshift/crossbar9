use core::slice;

pub unsafe fn str_bytes<'a>(raw: &'a *const u8) -> &'a [u8] {
    let mut offs_raw = *raw;
    let mut len = 0usize;
    while *offs_raw != 0 {
        offs_raw = offs_raw.offset(1);
        len += 1;
    }

    slice::from_raw_parts(*raw, len)
}