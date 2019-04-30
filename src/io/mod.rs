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

pub trait RegEnum {
    fn addr_of(&self) -> u32;
    fn ptr<T: Copy>(&self) -> *mut T {
        assert!((self.addr_of() as usize) & (::core::mem::align_of::<T>() - 1) == 0);
        self.addr_of() as *mut T
    }

    fn read8(&self) -> u8 {
        unsafe { self.ptr::<u8>().read_volatile() }
    }
    fn read16(&self) -> u16 {
        unsafe { self.ptr::<u16>().read_volatile() }
    }
    fn read32(&self) -> u32 {
        unsafe { self.ptr::<u32>().read_volatile() }
    }

    fn write8(&self, val: u8) {
        unsafe { self.ptr::<u8>().write_volatile(val) };
    }
    fn write16(&self, val: u16) {
        unsafe { self.ptr::<u16>().write_volatile(val) };
    }
    fn write32(&self, val: u32) {
        unsafe { self.ptr::<u32>().write_volatile(val) };
    }
}

pub mod aes;
pub mod hid;
pub mod i2c;
pub mod irq;
pub mod mic;
pub mod ndma;
pub mod rsa;
pub mod sdmmc;
pub mod sha;
pub mod timer;
pub mod xdma;
