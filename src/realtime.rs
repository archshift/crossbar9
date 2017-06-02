use interrupts;
use io::timer;

static mut rt_timer: Option<*const timer::Timer> = None;

// Borrowing shim for the global timer
pub struct SleepTimer<'a> {
    #[allow(dead_code)]
    timer_ref: &'a timer::Timer
}

impl<'a> SleepTimer<'a> {
    pub fn new(timer_ref: &'a timer::Timer) -> SleepTimer {
        timer_ref.start();
        unsafe { rt_timer = Some(timer_ref as *const timer::Timer); }
        SleepTimer {
            timer_ref: timer_ref,
        }
    }
}

impl<'a> Drop for SleepTimer<'a> {
    fn drop(&mut self) {
        unsafe { rt_timer = None; }
    }
}


pub fn usleep(us: u64) {
    let timer = unsafe { &*rt_timer.expect("Attempted to sleep without initializing timer!") };

    let interval = timer.interrupt_interval_us();
    let initial = timer.us_val();

    while (timer.us_val() - initial + interval) < us {
        unsafe { interrupts::wait_for_interrupt(); }
    }

    while (timer.us_val() - initial) < us { }
}

pub fn msleep(ms: u32) {
    usleep((ms * 1000) as u64);
}

pub fn sleep(sec: u32) {
    let timer = unsafe { &*rt_timer.expect("Attempted to sleep without initializing timer!") };

    let ms = sec * 1000;
    let interval = timer.interrupt_interval_ms();
    let initial = timer.ms_val();

    while (timer.ms_val() - initial + interval) < ms as u64 {
        unsafe { interrupts::wait_for_interrupt(); }
    }

    while (timer.ms_val() - initial) < ms as u64 { }
}
