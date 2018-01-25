pub mod arbiter {
    use core::cell::RefCell;

    pub struct MicLease;
    pub struct TimerLease;

    pub static mut MIC: RefCell<MicLease> = RefCell::new(MicLease);
    pub static mut TIMER0: RefCell<TimerLease> = RefCell::new(TimerLease);
    pub static mut TIMER1: RefCell<TimerLease> = RefCell::new(TimerLease);
    pub static mut TIMER2: RefCell<TimerLease> = RefCell::new(TimerLease);
    pub static mut TIMER3: RefCell<TimerLease> = RefCell::new(TimerLease);
}

macro_rules! lease {
    ($which:ident) => (unsafe { $crate::io::arbiter::$which.borrow_mut() })
}

macro_rules! lease_ty {
    ($lifetime:tt, $which:ident) => (::core::cell::RefMut<$lifetime, $crate::io::arbiter::$which>);
}

pub mod aes;
pub mod hid;
pub mod i2c;
pub mod irq;
pub mod mic;
pub mod ndma;
pub mod rsa;
pub mod sha;
pub mod timer;