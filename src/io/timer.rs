use core::intrinsics;
use core::u16;

use interrupts::{self, HandlerFn};
use io::irq::{self, Interrupt};

const TIMER_BASE: u32 = 0x10003000u32;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
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

bf!(CntReg[u16] {
    prescaler: 0:1,
    count_up: 2:2,
    irq_enable: 6:6,
    started: 7:7
});

#[inline(never)]
fn read_reg(reg: Reg) -> u16 {
    unsafe { intrinsics::volatile_load((TIMER_BASE + reg as u32) as *const u16) }
}

fn write_reg(reg: Reg, val: u16) {
    unsafe { intrinsics::volatile_store((TIMER_BASE + reg as u32) as *mut u16, val); }
}

fn ticks_to_units(num_ticks: u64, prescaler: Prescaler, units_per_second: u32) -> u64 {
    let max_tps: u64 = 1 << 26;
    let scaled_tps = match prescaler {
        Prescaler::Div1 => max_tps,
        Prescaler::Div64 => max_tps >> 6,
        Prescaler::Div256 => max_tps >> 8,
        Prescaler::Div1024 => max_tps >> 10,
    };
    (num_ticks * units_per_second as u64) / scaled_tps
}

static mut TIMER_OVERFLOWS: [u64; 4] = [0; 4];

fn update_overflows_0() { unsafe { TIMER_OVERFLOWS[0] += 1; } }
fn update_overflows_1() { unsafe { TIMER_OVERFLOWS[1] += 1; } }
fn update_overflows_2() { unsafe { TIMER_OVERFLOWS[2] += 1; } }
fn update_overflows_3() { unsafe { TIMER_OVERFLOWS[3] += 1; } }

pub struct Timer<'a> {
    _lease: lease_ty!('a, TimerLease),
    index: usize,
    val_reg: Reg,
    cnt_reg: Reg,
    interrupt_type: u32,
    start_val: u16,
    overflow_fn: HandlerFn,
    prescaler: Prescaler,
    callback: Option<HandlerFn>
}

impl<'a> Timer<'a> {
    pub fn new(lease: lease_ty!('a, TimerLease), index: usize, start_val: u16,
               prescaler: Prescaler, callback: Option<HandlerFn>) -> Timer {

        let (val_reg, cnt_reg, int_type, overflow_fn) = match index {
            0 => (Reg::VAL_0, Reg::CNT_0, Interrupt::TIMER_0, update_overflows_0 as fn()),
            1 => (Reg::VAL_1, Reg::CNT_1, Interrupt::TIMER_1, update_overflows_1 as fn()),
            2 => (Reg::VAL_2, Reg::CNT_2, Interrupt::TIMER_2, update_overflows_2 as fn()),
            3 => (Reg::VAL_3, Reg::CNT_3, Interrupt::TIMER_3, update_overflows_3 as fn()),
            _ => panic!("Attempted to register timer of invalid index!")
        };

        if let Some(callback) = callback {
            interrupts::register_handler(int_type, callback).unwrap();
        }
        interrupts::register_handler(int_type, overflow_fn).unwrap();
        irq::set_enabled(int_type);

        let timer = Timer {
            _lease: lease,
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
        timer
    }

    #[inline(always)]
    pub fn tick_val(&self) -> u64 {
        let overflows = interrupts::without_interrupts(|| {
            unsafe { TIMER_OVERFLOWS[self.index] }
        });
        (overflows << 16) | (read_reg(self.val_reg) as u64)
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
        let mut cnt = read_reg(self.cnt_reg);
        {
            let cnt = CntReg::alias_mut(&mut cnt);
            cnt.started.set(1);
        }
        write_reg(self.cnt_reg, cnt);
    }

    pub fn stop(&self) {
        let mut cnt = read_reg(self.cnt_reg);
        {
            let cnt = CntReg::alias_mut(&mut cnt);
            cnt.started.set(0);
        }
        write_reg(self.cnt_reg, cnt);
    }

    pub fn reset(&self) {
        let mut cnt = CntReg::new(0u16);
        cnt.prescaler.set(self.prescaler as u16);
        cnt.count_up.set(0);
        cnt.irq_enable.set(1);
        cnt.started.set(0);
        write_reg(self.cnt_reg, cnt.val);
        write_reg(self.val_reg, self.start_val);
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        irq::set_disabled(self.interrupt_type);
        interrupts::unregister_handler(self.interrupt_type, self.overflow_fn).unwrap();
        if let Some(callback) = self.callback {
            interrupts::unregister_handler(self.interrupt_type, callback).unwrap();
        }
    }
}


