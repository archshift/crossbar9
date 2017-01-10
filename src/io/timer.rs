use core::intrinsics;
use core::u16;

use interrupts::{self, HandlerFn};
use io::irq::{self, Interrupt};
use unique;

const TIMER_BASE: u32 = 0x10003000u32;

#[derive(Clone, Copy)]
enum Reg {
    VAL_0 = 0x00,
    VAL_1 = 0x04,
    VAL_2 = 0x08,
    VAL_3 = 0x0C,

    CNT_0 = 0x02,
    CNT_1 = 0x06,
    CNT_2 = 0x0A,
    CNT_3 = 0x0E,
}

#[derive(Clone, Copy)]
pub enum Prescaler {
    Div1 = 0,
    Div64 = 1,
    Div256 = 2,
    Div1024 = 3,
}

#[inline(never)]
fn read_reg(reg: Reg) -> u16 {
    unsafe { intrinsics::volatile_load((TIMER_BASE + reg as u32) as *const u16) }
}

fn write_reg(reg: Reg, val: u16) {
    unsafe { intrinsics::volatile_store((TIMER_BASE + reg as u32) as *mut u16, val); }
}

fn make_cnt(prescaler: Prescaler, count_up: bool, irq_enable: bool, started: bool) -> u16 {
    bitfield!(CntReg: u16, {
        prescaler: 0 => 1,
        count_up: 2 => 2,
        irq_enable: 6 => 6,
        started: 7 => 7
    });

    let mut cnt = CntReg::new(0);
    bf!(cnt.prescaler = (prescaler as u16));
    bf!(cnt.count_up = (count_up as u16));
    bf!(cnt.irq_enable = (irq_enable as u16));
    bf!(cnt.started = (started as u16));
    cnt.raw()
}

fn ticks_to_units(num_ticks: u64, prescaler: Prescaler, units_per_second: u32) -> u64 {
    let scaled_tps: u64 = (1 << 26) * (units_per_second as u64);
    let scaled_ticks = match prescaler {
        Prescaler::Div1 => num_ticks,
        Prescaler::Div64 => num_ticks << 6,
        Prescaler::Div256 => num_ticks << 8,
        Prescaler::Div1024 => num_ticks << 10,
    };
    scaled_ticks / scaled_tps
}


static mut timers_used: [bool; 4] = [false; 4];
static mut timer_overflows: [u64; 4] = [0; 4];

fn update_overflows_0() { unsafe { timer_overflows[0] += 1; } }
fn update_overflows_1() { unsafe { timer_overflows[1] += 1; } }
fn update_overflows_2() { unsafe { timer_overflows[2] += 1; } }
fn update_overflows_3() { unsafe { timer_overflows[3] += 1; } }

#[derive(Debug)]
pub enum Error {
    ImproperIndex,
    Unique(unique::Error),
}

impl From<unique::Error> for Error {
    fn from(o: unique::Error) -> Error {
        Error::Unique(o)
    }
}

pub struct Timer {
    index: usize,
    val_reg: Reg,
    cnt_reg: Reg,
    interrupt_type: u32,
    start_val: u16,
    overflow_fn: HandlerFn,
    prescaler: Prescaler,
    callback: Option<HandlerFn>
}

impl Timer {
    pub fn new(index: usize, start_val: u16, prescaler: Prescaler,
            callback: Option<HandlerFn>) -> Result<Timer, Error> {

        unique::lock(unsafe { &mut timers_used[index] })?;

        let (val_reg, cnt_reg, int_type, overflow_fn) = match index {
            0 => (Reg::VAL_0, Reg::CNT_0, Interrupt::TIMER_0, update_overflows_0 as fn()),
            1 => (Reg::VAL_1, Reg::CNT_1, Interrupt::TIMER_1, update_overflows_1 as fn()),
            2 => (Reg::VAL_2, Reg::CNT_2, Interrupt::TIMER_2, update_overflows_2 as fn()),
            3 => (Reg::VAL_3, Reg::CNT_3, Interrupt::TIMER_3, update_overflows_3 as fn()),
            _ => return Err(Error::ImproperIndex)
        };

        if let Some(callback) = callback {
            interrupts::register_handler(int_type, callback);
        }
        interrupts::register_handler(int_type, overflow_fn);
        irq::set_enabled(int_type);

        let timer = Timer {
            index: index,
            val_reg: val_reg,
            cnt_reg: cnt_reg,
            interrupt_type: int_type,
            start_val: start_val,
            overflow_fn: overflow_fn,
            prescaler: prescaler,
            callback: callback,
        };

        timer.reset();
        Ok(timer)
    }

    #[inline(always)]
    pub fn tick_val(&self) -> u64 {
        let overflows = unsafe { timer_overflows[self.index] };
        (overflows * (u16::MAX as u64)) + (read_reg(self.val_reg) as u64)
    }

    #[inline(always)]
    pub fn us_val(&self) -> u64 {
        ticks_to_units(self.tick_val(), self.prescaler, 1000000)
    }

    #[inline(always)]
    pub fn ms_val(&self) -> u64 {
        ticks_to_units(self.tick_val(), self.prescaler, 1000)
    }

    #[inline(always)]
    pub fn sec_val(&self) -> u64 {
        ticks_to_units(self.tick_val(), self.prescaler, 1)
    }

    #[inline(always)]
    pub fn interrupt_interval_us(&self) -> u64 {
        ticks_to_units(u16::MAX as u64, self.prescaler, 1000000)
    }

    #[inline(always)]
    pub fn interrupt_interval_ms(&self) -> u64 {
        ticks_to_units(u16::MAX as u64, self.prescaler, 1000)
    }

    #[inline(always)]
    pub fn interrupt_interval_sec(&self) -> u64 {
        ticks_to_units(u16::MAX as u64, self.prescaler, 1)
    }

    pub fn start(&self) {
        write_reg(self.cnt_reg, read_reg(self.cnt_reg) | (1 << 7));
    }

    pub fn stop(&self) {
        write_reg(self.cnt_reg, read_reg(self.cnt_reg) & !(1 << 7));
    }

    pub fn reset(&self) {
        write_reg(self.val_reg, self.start_val);
        write_reg(self.cnt_reg, make_cnt(self.prescaler, false, true, false));
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        irq::set_disabled(self.interrupt_type);
        interrupts::unregister_handler(self.interrupt_type, self.overflow_fn);
        if let Some(callback) = self.callback {
            interrupts::unregister_handler(self.interrupt_type, callback);
        }
        unique::unlock(unsafe { &mut timers_used[self.index] });
    }
}


