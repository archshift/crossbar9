use core::intrinsics;

pub const HID_BASE: u32 = 0x10146000u32;

#[derive(Clone, Copy)]
enum Reg {
    PAD = 0x00,
}

pub mod Button {
    pub const A: u16      = 1 << 0;
    pub const B: u16      = 1 << 1;
    pub const SELECT: u16 = 1 << 2;
    pub const START: u16  = 1 << 3;
    pub const DPAD_R: u16 = 1 << 4;
    pub const DPAD_L: u16 = 1 << 5;
    pub const DPAD_U: u16 = 1 << 6;
    pub const DPAD_D: u16 = 1 << 7;
    pub const R: u16      = 1 << 8;
    pub const L: u16      = 1 << 9;
    pub const X: u16      = 1 << 10;
    pub const Y: u16      = 1 << 11;
}

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { intrinsics::volatile_load((HID_BASE + reg as u32) as *const T) }
}

#[derive(Clone, Copy)]
pub struct ButtonsPressed(pub [bool; 11]);

pub fn buttons_pressed() -> ButtonsPressed {
    let mut state = [false; 11];
    let mut raw = read_reg::<u16>(Reg::PAD);

    for i in 0..11 {
        let pressed = raw & 1 == 0;
        state[i] = pressed;
        raw >>= 1;
    }

    ButtonsPressed(state)
}