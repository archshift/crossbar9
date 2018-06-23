const KB: u64 = 1024;
const MB: u64 = KB * KB;
const GB: u64 = KB * MB;

extern {
    fn add_mpu_regions(regions: *const u32);
    fn add_mpu_cachedata(cacheability: u32, bufferability: u32, perms: u32);
    fn enable_mpu(to_enable: bool);
    fn enable_icache(to_enable: bool);
    fn enable_dcache(to_enable: bool);
    fn enable_wbuf(to_enable: bool);
    fn invalidate_icache();
    fn invalidate_clean_dcache();
}

fn make_mpu_region_desc(base: u32, size: u64) -> u32 {
    if size.count_ones() > 1 {
        panic!("Improper size {:#X} for region {:08X}", size, base);
    }
    assert!(size <= 4*GB && size >= 4*KB);
    let size = size as u32;
    let size_pot = size.trailing_zeros();
    (base >> 12 << 12) | (size_pot-1) << 1 | 1
}

fn make_mpu_region_list() -> ([u32; 8], u32, u32, u32) {
    ([
        make_mpu_region_desc(0x00000000, 4*GB),
        make_mpu_region_desc(0x00000000, 128*MB),
        make_mpu_region_desc(0x08000000, 2*MB),
        make_mpu_region_desc(0x10000000, 128*MB),
        make_mpu_region_desc(0x18000000, 128*MB),
        make_mpu_region_desc(0x1FF80000, 512*KB),
        make_mpu_region_desc(0x20000000, 256*MB),
        make_mpu_region_desc(0xFFFF0000, 4*KB),
    ], 0b1110110, 0b0110110, 0b11_11_11_11_11_11_11_11)
}

pub fn disable_all() {
    unsafe {
        enable_mpu(false);
        enable_icache(false);
        enable_dcache(false);
    }
}

pub fn enable_all() {
    disable_all();
    let (regions, cacheability, bufferability, perms) = make_mpu_region_list();
    unsafe {
        invalidate_icache();
        invalidate_clean_dcache();

        add_mpu_regions(regions.as_ptr());
        add_mpu_cachedata(cacheability, bufferability, perms);
        enable_mpu(true);
        enable_icache(true);
        enable_dcache(true);
        enable_wbuf(true);
    }
}