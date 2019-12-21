use core::ptr;

pub const MIC_BASE: u32 = 0x10162000u32;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
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
    sample_rate: SampleRate,
    _lease: lease_ty!('a, MicLease)
}

impl<'a> Mic<'a> {
    pub fn enable(lease: lease_ty!('a, MicLease), sample_rate: SampleRate) -> Mic {
        let mut out = Mic {
            sample_rate,
            _lease: lease
        };
        out.clear_overrun();
        out
    }

    pub fn clear_overrun(&mut self) {
        let mut old_cnt: u16 = read_reg(Reg::CNT);
        old_cnt &= 0x7000;
        write_reg(Reg::CNT, old_cnt);
        write_reg(Reg::CNT, 0xF002u16 | (self.sample_rate as u16) << 2);
    }

    pub fn curr_data(&self) -> [u32; 8] {
        let mut out = [0; 8];
        for i in 0..8 {
            out[i] = read_reg(Reg::DATA)
        }
        out
    }
}
