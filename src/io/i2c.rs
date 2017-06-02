use core::ptr;

pub const I2C_BASES: [u32; 3] = [0x10161000, 0x10144000, 0x10148000];

#[derive(Clone, Copy)]
pub enum Device {
    MCU = 0x03,
}

bfdesc!(RegCnt: u8, {
    end: 0 => 0,
    beginning: 1 => 1,
    pause: 2 => 2,
    ack: 4 => 4,
    should_read: 5 => 5,
    enable_irq: 6 => 6,
    running: 7 => 7
});

#[derive(Clone, Copy)]
enum Reg {
    DATA = 0x00,
    CNT = 0x01,
}

#[derive(Clone, Copy)]
struct DevData {
    pub bus_id: u8,
    pub dev_addr: u8,
}

impl DevData {
    fn new(device: Device) -> DevData {
        const DEV_DATA: [DevData; 15] = [
            DevData { bus_id: 0, dev_addr: 0x4A },
            DevData { bus_id: 0, dev_addr: 0x7A },
            DevData { bus_id: 0, dev_addr: 0x78 },
            DevData { bus_id: 1, dev_addr: 0x4A },
            DevData { bus_id: 1, dev_addr: 0x78 },
            DevData { bus_id: 1, dev_addr: 0x2C },
            DevData { bus_id: 1, dev_addr: 0x2E },
            DevData { bus_id: 1, dev_addr: 0x40 },
            DevData { bus_id: 1, dev_addr: 0x44 },
            DevData { bus_id: 2, dev_addr: 0xD6 },
            DevData { bus_id: 2, dev_addr: 0xD0 },
            DevData { bus_id: 2, dev_addr: 0xD2 },
            DevData { bus_id: 2, dev_addr: 0xA4 },
            DevData { bus_id: 2, dev_addr: 0x9A },
            DevData { bus_id: 2, dev_addr: 0xA0 }
        ];
        DEV_DATA[device as usize]
    }

    #[inline(never)]
    fn read_reg(&self, reg: Reg) -> u8 {
        let base = I2C_BASES[self.bus_id as usize];
        unsafe { ptr::read_volatile((base + reg as u32) as *const u8) }
    }

    #[inline(never)]
    fn write_reg(&self, reg: Reg, val: u8) {
        let base = I2C_BASES[self.bus_id as usize];
        unsafe { ptr::write_volatile((base + reg as u32) as *mut u8, val); }
    }

    fn wait_busy(&self) {
        let is_busy = || {
            let cnt = self.read_reg(Reg::CNT);
            bf!(cnt @ RegCnt::running) == 1
        };
        while is_busy() { }
    }

    fn op_result(&self) -> Result<(), ()> {
        self.wait_busy();
        let cnt = self.read_reg(Reg::CNT);
        if bf!(cnt @ RegCnt::ack) == 1 {
            Ok(())
        } else {
            Err(())
        }
    }

    fn halt_xfer(&self) {
        self.write_reg(Reg::CNT, 0xc5);
    }

    fn xfer_last_byte(&self, is_reading: u8) {
        self.write_reg(Reg::CNT, is_reading << 5 | 0xc1);
    }

    fn xfer_byte(&self, is_reading: u8) {
        self.write_reg(Reg::CNT, is_reading << 5 | 0xc0);
    }

    fn select_target(&self, reg: u8, is_reading: bool) -> Result<(), ()> {
        self.wait_busy();
        self.write_reg(Reg::DATA, self.dev_addr);
        self.write_reg(Reg::CNT, 0xc2);
        self.op_result()?;

        self.wait_busy();
        self.write_reg(Reg::DATA, reg);
        self.write_reg(Reg::CNT, 0xc0);
        self.op_result()?;

        if is_reading {
            self.wait_busy();
            self.write_reg(Reg::DATA, self.dev_addr | 1);
            self.write_reg(Reg::CNT, 0xc2);
            self.op_result()?;
        }
        Ok(())
    }
}

pub fn read_byte(dev: Device, reg: u8) -> Result<u8, ()> {
    let dev_data = DevData::new(dev);
    for i in 0..8 {
        if dev_data.select_target(reg, true).is_ok() {
            dev_data.wait_busy();
            dev_data.xfer_byte(1);
            dev_data.wait_busy();
            dev_data.halt_xfer();
            dev_data.wait_busy();
            return Ok(dev_data.read_reg(Reg::DATA))
        }
        dev_data.halt_xfer();
        dev_data.wait_busy();
    }
    Err(())
}

pub fn read_bytes(dev: Device, reg: u8, dest: &mut [u8]) -> Result<(), ()> {
    if dest.len() == 0 {
        return Ok(())
    }

    let dev_data = DevData::new(dev);
    for i in 0..8 {
        if dev_data.select_target(reg, true).is_ok() {
            for n in 0..(dest.len() - 1) {
                dev_data.wait_busy();
                dev_data.write_reg(Reg::CNT, 0xF0);
                dev_data.wait_busy();
                dest[n] = dev_data.read_reg(Reg::DATA);
            }

            dev_data.wait_busy();
            dev_data.xfer_last_byte(1);
            dev_data.wait_busy();

            let dest_end = dest.len() - 1;
            dest[dest_end] = dev_data.read_reg(Reg::DATA);
            return Ok(())
        }
        dev_data.wait_busy();
        dev_data.halt_xfer();
        dev_data.wait_busy();
    }
    Err(())
}

pub fn write_byte(dev: Device, reg: u8, data: u8) -> Result<(), ()> {
    let dev_data = DevData::new(dev);
    for i in 0..8 {
        if dev_data.select_target(reg, false).is_ok() {
            dev_data.wait_busy();
            dev_data.write_reg(Reg::DATA, data);
            dev_data.xfer_last_byte(0);
            dev_data.xfer_byte(0);
            dev_data.wait_busy();
            dev_data.halt_xfer();
            if dev_data.op_result().is_ok() {
                return Ok(())
            }
        }
        dev_data.halt_xfer();
        dev_data.wait_busy();
    }
    Err(())
}
