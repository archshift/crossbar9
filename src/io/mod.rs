pub mod aes;
pub mod hid;
pub mod i2c;
pub mod irq;
pub mod ndma;
pub mod rsa;
pub mod sha;
pub mod timer;

pub mod arbiter {
    // pub struct AesLease;
    // pub struct HidLease;
    // pub struct I2cLease;
    // pub struct IrqLease;
    // pub struct NdmaLease;
    // pub struct RsaLease;
    // pub struct ShaLease;
    pub struct TimerLease;

    // pub static mut AES: AesLease = AesLease;
    // pub static mut HID: HidLease = HidLease;
    // pub static mut I2C: I2cLease = I2cLease;
    // pub static mut IRQ: IrqLease = IrqLease;
    // pub static mut NDMA: NdmaLease = NdmaLease;
    // pub static mut RSA: RsaLease = RsaLease;
    // pub static mut SHA: ShaLease = ShaLease;
    pub static mut TIMER0: TimerLease = TimerLease;
    pub static mut TIMER1: TimerLease = TimerLease;
    pub static mut TIMER2: TimerLease = TimerLease;
    pub static mut TIMER3: TimerLease = TimerLease;
}

macro_rules! lease {
    ($which:ident) => (unsafe { &mut $crate::io::arbiter::$which })
}