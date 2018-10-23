use core::ptr;

pub const HID_BASE: u32 = 0x10146000u32;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
enum Reg {
    PAD = 0x00,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Button {
    A      = 0,
    B      = 1,
    Select = 2,
    Start  = 3,
    DPadR  = 4,
    DPadL  = 5,
    DPadU  = 6,
    DPadD  = 7,
    R      = 8,
    L      = 9,
    X      = 10,
    Y      = 11,
}

impl From<usize> for Button {
    fn from(idx: usize) -> Self {
        match idx {
            0 => Button::A,
            1 => Button::B,
            2 => Button::Select,
            3 => Button::Start,
            4 => Button::DPadR,
            5 => Button::DPadL,
            6 => Button::DPadU,
            7 => Button::DPadD,
            8 => Button::R,
            9 => Button::L,
            10 => Button::X,
            11 => Button::Y,
            _ => unreachable!()
        }
    }
}

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { ptr::read_volatile((HID_BASE + reg as u32) as *const T) }
}

#[derive(Clone, Copy)]
pub struct ButtonsPressed(pub [bool; 11]);

impl From<u16> for ButtonsPressed {
    fn from(mut mask: u16) -> Self {
        let mut state = [false; 11];
        for i in 0..11 {
            let pressed = mask & 1 == 1;
            state[i] = pressed;
            mask >>= 1;
        }
        ButtonsPressed(state)
    }
}

pub fn pressed_mask() -> u16 {
    let raw = read_reg::<u16>(Reg::PAD);
    !raw & 0b1111_1111_1111
}

pub fn buttons_pressed() -> ButtonsPressed {
    pressed_mask().into()
}

pub fn is_pressed(button: Button) -> bool {
    pressed_mask() & (1 << (button as u16)) != 0
}
