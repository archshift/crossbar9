use core::ptr;

pub const SPI_BASE0: u32 = 0x10142000u32;
pub const SPI_BASE1: u32 = 0x10143000u32;
pub const SPI_BASE2: u32 = 0x10160000u32;

#[derive(Clone, Copy)]
enum Reg {
    CNT_LGY = 0x00,
    DATA_LGY = 0x02,
    CNT = 0x800,
    DONE = 0x804,
    DATALEN = 0x808,
    FIFO = 0x80C,
    FIFO_WAIT = 0x810,
}

bfdesc!(RegCnt: u32, {
    baudrate: 0 => 5,
    dev_index: 6 => 7,
    should_write: 13 => 13,
    busy: 15 => 15
});

bfdesc!(RegCntLegacy: u16, {
    baudrate: 0 => 5,
    busy: 7 => 7,
    dev_index: 8 => 9,
    chipsel_hold: 11 => 11,
    irq_enable: 14 => 14,
    bus_enable: 15 => 15
});

#[inline(never)]
fn read_reg<T: Copy>(device_id: u8, reg: Reg) -> T {
    let base = get_base_addr(device_id);
    unsafe { ptr::read_volatile((base + reg as u32) as *const T) }
}

fn write_reg<T: Copy>(device_id: u8, reg: Reg, val: T) {
    let base = get_base_addr(device_id);
    unsafe { ptr::write_volatile((base + reg as u32) as *mut T, val); }
}

fn get_base_addr(device_id: u8) -> u32 {
    match device_id {
        0...2 => SPI_BASE2,
        3...5 => SPI_BASE0,
        6 => SPI_BASE1,
        x => panic!("Tried to match unhandled SPI device ID {}!", x)
    }
}

fn get_baudrate_from_index(baud_index: u8) -> u32 {
    match baud_index {
        0 => 2400,
        1 => 4800,
        2 => 9600,
        3 => 19200,
        4 => 38400,
        5 => 57600,
        x => panic!("Invalid baud index {}", x)
    }
}

fn get_baudrate_index(device_id: u8) -> u8 {
    match device_id {
        0 => 2,
        3 => 5,
        x => panic!("Unknown baudrate for SPI device ID {}!", x)
    }
}

fn get_dev_index(device_id: u8) -> u8 {
    device_id % 3
}

fn uses_legacy(device_id: u8) -> bool {
    // TODO: move to dedicated CFG11 module
    assert!(device_id <= 6);
    let bits = unsafe { ptr::read_volatile(0x101401C0 as *mut u32) };
    bits & (1 << ((device_id as u32) / 3)) != 0
}

fn set_legacy(device_id: u8, on: bool) {
    if on {
        unsafe { ptr::write_volatile(0x101401C0 as *mut u32, 0); }
    } else {
        unsafe { ptr::write_volatile(0x101401C0 as *mut u32, 0b111); }
    }
    return

    // TODO: move to dedicated CFG11 module
    assert!(device_id <= 6);
    let mut bits = unsafe { ptr::read_volatile(0x101401C0 as *mut u32) };
    bits &= !(1 << ((device_id as u32) / 3)); // Clear bit
    bits |= ((!on as u32) << ((device_id as u32) / 3)); // Set bit to !on
    unsafe { ptr::write_volatile(0x101401C0 as *mut u32, bits); }
}

fn wait_busy(device_id: u8) {
    let is_busy = || {
        let cnt = read_reg::<u32>(device_id, Reg::CNT);
        bf!(cnt @ RegCnt::busy) == 1
    };
    while is_busy() { }
}

fn wait_fifo(device_id: u8) {
    let is_busy = || {
        let wait = read_reg::<u32>(device_id, Reg::FIFO_WAIT);
        wait != 0
    };
    while is_busy() { }
}

pub fn write2(device_id: u8, data1: &[u8], data2: &[u8]) {
    set_legacy(device_id, false);

    log!("Waiting busy...");

    wait_busy(device_id);
    let mut cnt = 0u32;
    bf!(cnt @ RegCnt::baudrate = get_baudrate_index(device_id) as u32);
    bf!(cnt @ RegCnt::dev_index = get_dev_index(device_id) as u32);
    bf!(cnt @ RegCnt::should_write = 1);
    bf!(cnt @ RegCnt::busy = 1);

    write_reg(device_id, Reg::DATALEN, data1.len() as u32);
    write_reg(device_id, Reg::CNT, cnt);

    log!("Waiting fifo...");
    wait_fifo(device_id);
    let data32: u32 = unsafe { ::core::mem::transmute([data1[0], data1[0], data1[0], data1[0]]) };
    write_reg(device_id, Reg::FIFO, data32);
    wait_busy(device_id);

    // for bslice in data1.chunks(4) {
    //     log!("Waiting fifo...");
    //     wait_fifo(device_id);

    //     let mut curr_word: u32 = 0;
    //     for (i, byte) in bslice.iter().enumerate() {
    //         curr_word |= (*byte as u32) << (i * 8);
    //     }
    //     for (i, byte) in bslice.iter().enumerate() {
    //         curr_word |= (*byte as u32) << (i * 8);
    //     }
    //     write_reg(device_id, Reg::FIFO, curr_word);
    // }

    write_reg(device_id, Reg::DATALEN, data2.len() as u32);
    write_reg(device_id, Reg::CNT, cnt);

    log!("Waiting fifo...");
    wait_fifo(device_id);
    let data32: u32 = unsafe { ::core::mem::transmute([data2[0], data2[0], data2[0], data2[0]]) };
    write_reg(device_id, Reg::FIFO, data32);

    // for bslice in data2.chunks(4) {
    //     wait_fifo(device_id);

    //     let mut curr_word: u32 = 0;
    //     for (i, byte) in bslice.iter().enumerate() {
    //         curr_word |= (*byte as u32) << (i * 8);
    //     }
    //     write_reg(device_id, Reg::FIFO, curr_word);
    // }

    wait_busy(device_id);
    write_reg(device_id, Reg::DONE, 1u32);
}

pub fn write_read(device_id: u8, data1: &[u8], data2: &mut [u8]) {
    log!("Reading...");
    set_legacy(device_id, false);

    wait_busy(device_id);
    let mut cnt = 0;
    bf!(cnt @ RegCnt::baudrate = get_baudrate_index(device_id) as u32);
    bf!(cnt @ RegCnt::dev_index = get_dev_index(device_id) as u32);
    bf!(cnt @ RegCnt::should_write = 1);
    bf!(cnt @ RegCnt::busy = 1);

    write_reg(device_id, Reg::DATALEN, data1.len() as u32);
    write_reg(device_id, Reg::CNT, cnt);

    // Past 0x40 bytes we have to wait for one whole baud cycle every
    // time the fifo fills up. Let's not worry about that now.
    assert!(data2.len() < 0x40);

    log!("Waiting fifo...");
    wait_fifo(device_id);
    let data32: u32 = unsafe { ::core::mem::transmute([data1[0], data1[0], data1[0], data1[0]]) };
    write_reg(device_id, Reg::FIFO, data32);
    wait_busy(device_id);

    // for bslice in data1.chunks(4) {
    //     wait_fifo(device_id);

    //     let mut curr_word: u32 = 0;
    //     for (i, byte) in bslice.iter().enumerate() {
    //         curr_word |= (*byte as u32) << (i * 8);
    //     }
    //     write_reg(device_id, Reg::FIFO, curr_word);
    // }

    bf!(cnt @ RegCnt::should_write = 0);
    write_reg(device_id, Reg::DATALEN, data2.len() as u32);
    write_reg(device_id, Reg::CNT, cnt);

    wait_fifo(device_id);
    let mut curr_word: u32 = read_reg(device_id, Reg::FIFO);
    log!("Reading word from FIFO: {:08X}", curr_word);
    data2[0] = curr_word as u8;


    // for bslice in data2.chunks_mut(4) {
    //     wait_fifo(device_id);

    //     let mut curr_word: u32 = read_reg(device_id, Reg::FIFO);
    //     for byte in bslice {
    //         *byte = curr_word as u8;
    //         curr_word >>= 8;
    //     }
    // }

    wait_busy(device_id);
    write_reg(device_id, Reg::DONE, 1u32);
}


// fn wait_busy_legacy(device_id: u8) {
//     let is_busy = || {
//         let cnt = read_reg::<u16>(device_id, Reg::CNT);
//         bf!(cnt @ RegCntLegacy::busy) == 1
//     };
//     while is_busy() { }
// }

// #[inline(never)]
// pub fn write2_legacy(device_id: u8, data1: &[u8], data2: &[u8]) {
//     log!("Writing...");
//     if data2.len() == 0 { return }
//     set_legacy(device_id, true);

//     log!("Busy wait");
//     wait_busy_legacy(device_id);
//     let mut cnt = 0u16;
//     bf!(cnt @ RegCntLegacy::baudrate = get_baudrate_index(device_id) as u16);
//     bf!(cnt @ RegCntLegacy::dev_index = get_dev_index(device_id) as u16);
//     bf!(cnt @ RegCntLegacy::chipsel_hold = 1);
//     bf!(cnt @ RegCntLegacy::bus_enable = 1);
//     write_reg(device_id, Reg::CNT, cnt);

//     for b in data1.iter().cloned() {
//         write_reg(device_id, Reg::DATA_LGY, b);
//         log!("Xfer1 busy wait");
//         wait_busy_legacy(device_id);
//     }

//     let mut iter = data2.iter().cloned();
//     for i in 0..(data2.len() - 2) {
//         write_reg(device_id, Reg::DATA_LGY, iter.next().unwrap());
//         log!("Xfer2 busy wait");
//         wait_busy_legacy(device_id);
//     }

//     let mut cnt: u16 = read_reg(device_id, Reg::CNT);
//     bf!(cnt @ RegCntLegacy::chipsel_hold = 0);
//     write_reg(device_id, Reg::CNT, cnt);

//     write_reg(device_id, Reg::DATA_LGY, iter.next().unwrap());
//     log!("XferF busy wait");
//     wait_busy_legacy(device_id);
// }