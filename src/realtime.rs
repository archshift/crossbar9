use interrupts;
use io::timer;

use core::mem::transmute;

static mut RT_TIMER: Option<*const timer::Timer> = None;

// Borrowing shim for the global timer
pub struct SleepTimer<'a> {
    #[allow(dead_code)]
    timer_ref: &'a timer::Timer<'a>
}

impl<'a> SleepTimer<'a> {
    pub fn new(timer_ref: &'a timer::Timer<'a>) -> SleepTimer {
        timer_ref.start();

        let timer_ptr = timer_ref as *const timer::Timer<'a>;
        unsafe { RT_TIMER = Some(transmute(timer_ptr)); }
        SleepTimer {
            timer_ref: timer_ref,
        }
    }
}

impl<'a> Drop for SleepTimer<'a> {
    fn drop(&mut self) {
        unsafe { RT_TIMER = None; }
    }
}


pub fn try_usleep(us: u64) -> Result<(), &'static str> {
    let timer = unsafe {
        &*RT_TIMER.ok_or("Attempted to sleep without initializing timer!")?
    };

    let interval = timer.interrupt_interval_us();
    let initial = timer.us_val();

    while (timer.us_val() - initial + interval) < us {
        unsafe { interrupts::wait_for_interrupt(); }
    }

    while (timer.us_val() - initial) < us { }
    Ok(())
}

pub fn try_msleep(ms: u32) -> Result<(), &'static str> {
    try_usleep((ms * 1000) as u64)
}

pub fn try_sleep(sec: u32) -> Result<(), &'static str> {
    let timer = unsafe {
        &*RT_TIMER.ok_or("Attempted to sleep without initializing timer!")?
    };

    let ms = sec * 1000;
    let interval = timer.interrupt_interval_ms();
    let initial = timer.ms_val();

    while (timer.ms_val() - initial + interval) < ms as u64 {
        unsafe { interrupts::wait_for_interrupt(); }
    }

    while (timer.ms_val() - initial) < ms as u64 { }
    Ok(())
}

pub fn usleep(us: u64) {
    try_usleep(us).unwrap()
}

pub fn msleep(ms: u32) {
    try_msleep(ms).unwrap()
}

pub fn sleep(sec: u32) {
    try_sleep(sec).unwrap()
}
