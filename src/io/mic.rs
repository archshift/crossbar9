use core::ptr;

pub const MIC_BASE: u32 = 0x10162000u32;

#[derive(Clone, Copy)]
enum Reg {
    CNT = 0x00,
    DATA = 0x04,
}

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { ptr::read_volatile((MIC_BASE + reg as u32) as *const T) }
}

fn write_reg<T: Copy>(reg: Reg, val: T) {
    unsafe { ptr::write_volatile((MIC_BASE + reg as u32) as *mut T, val); }
}

#[derive(Clone, Copy)]
pub enum SampleRate {
    Hz32k = 0,
    Hz16k = 1,
    Hz11k = 2,
    Hz8k = 3
}

pub struct Mic<'a> {
    lease: lease_ty!('a, MicLease)
}

impl<'a> Mic<'a> {
    pub fn enable(lease: lease_ty!('a, MicLease), sample_rate: SampleRate) -> Mic {
        write_reg(Reg::CNT, 0xF002u16 | (sample_rate as u16) << 2);
        Mic {
            lease: lease
        }
    }

    pub fn curr_data(&self) -> u32 {
        read_reg(Reg::DATA)
    }
}
