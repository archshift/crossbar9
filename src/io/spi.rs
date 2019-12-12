use core::ptr;

// TODO: This SPI driver needs more testing...

pub struct Spi<'a> {
    _lease: lease_ty!('a, SpiLease),
    base_reg: u32,
}

bf!(SpiFifoCntReg[u32] {
    baudrate: 0:5,
    dev_select: 6:7,
    is_outgoing: 13:13,
    busy: 15:15
});

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
enum Reg {
    FIFO_CNT = 0x800,
    FIFO_DONE = 0x804,
    FIFO_BLKLEN = 0x808,
    FIFO_DATA = 0x80C,
    FIFO_STATUS = 0x810,
}

impl<'a> Spi<'a> {
    pub fn new(lease: lease_ty!('a, SpiLease)) -> Spi {
        let idx = lease.idx;
        Spi {
            _lease: lease,
            base_reg: match idx {
                0 => 0x10160000,
                _ => unimplemented!()
            },
        }
    }

    #[inline(never)]
    fn read_reg(&mut self, reg: Reg) -> u32 {
        unsafe { ptr::read_volatile((self.base_reg + reg as u32) as *const u32) }
    }

    fn write_reg(&mut self, reg: Reg, val: u32) {
        unsafe { ptr::write_volatile((self.base_reg + reg as u32) as *mut u32, val); }
    }

    pub fn chipselect(&mut self, device_id: u32) {
        let dev_select = device_id % 3;
        let baudrate = match device_id {
            0 => 2,
            3 => 5,
            _ => unimplemented!()
        };
        
        let mut fifo_cnt = SpiFifoCntReg::new(0);
        fifo_cnt.baudrate.set(baudrate);
        fifo_cnt.dev_select.set(dev_select);

        self.write_reg(Reg::FIFO_CNT, fifo_cnt.val);
    }

    pub fn write(&mut self, bytes: &[u8]) {
        let mut fifo_cnt_raw = self.read_reg(Reg::FIFO_CNT);
        {
            let fifo_cnt = SpiFifoCntReg::alias_mut(&mut fifo_cnt_raw);
            fifo_cnt.is_outgoing.set(1);
            fifo_cnt.busy.set(1);
        }

        self.write_reg(Reg::FIFO_BLKLEN, bytes.len() as u32);
        self.write_reg(Reg::FIFO_CNT, fifo_cnt_raw);

        for chunk in bytes.chunks(4) {
            let mut word: u32 = 0;
            for byte in chunk.iter().rev() {
                word <<= 8;
                word |= *byte as u32;
            }

            self.write_reg(Reg::FIFO_DATA, word);

            while self.read_reg(Reg::FIFO_STATUS) & 0 == 1 {}
        }

        self.write_reg(Reg::FIFO_DONE, 0);
    }

    pub fn read(&mut self, bytes: &mut [u8]) {
        let mut fifo_cnt_raw = self.read_reg(Reg::FIFO_CNT);
        {
            let fifo_cnt = SpiFifoCntReg::alias_mut(&mut fifo_cnt_raw);
            fifo_cnt.is_outgoing.set(0);
            fifo_cnt.busy.set(1);
        }

        self.write_reg(Reg::FIFO_BLKLEN, bytes.len() as u32);
        self.write_reg(Reg::FIFO_CNT, fifo_cnt_raw);

        for chunk in bytes.chunks_mut(4) {
            let mut word: u32 = self.read_reg(Reg::FIFO_DATA);

            for byte in chunk.iter_mut() {
                *byte = word as u8;
                word >>= 8;
            }

            while self.read_reg(Reg::FIFO_STATUS) & 0 == 1 {}
        }
    }

    pub fn deselect(&mut self) {
        self.write_reg(Reg::FIFO_DONE, 0);
    }
}