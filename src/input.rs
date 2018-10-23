use io::hid;
use realtime;

fn wait_for_button_mask(active: u16, pressed: u16, prev_ref: &mut u16) -> u16 {
    loop {
        let curr = hid::pressed_mask();
        let prev = *prev_ref;
        *prev_ref = curr;
        if (curr ^ pressed) & active == 0 && (curr ^ prev) & active != 0 {
            return curr
        }
        let _ = realtime::try_msleep(10);
    }
}

fn wait_for_any_mask(active: u16, prev_ref: &mut u16) -> u16 {
    loop {
        let curr = hid::pressed_mask();
        let this_prev = *prev_ref;
        *prev_ref = curr;
        if (curr & !this_prev) & active != 0 {
            return curr
        }
        let _ = realtime::try_msleep(10);
    }
}

#[inline(always)]
pub fn wait_for_any() -> hid::ButtonsPressed {
    let mut prev_state = hid::pressed_mask();
    wait_for_any_mask(!0, &mut prev_state).into()
}

#[inline(always)]
pub fn wait_for_any_of(which: &[hid::Button]) -> hid::ButtonsPressed {
    let mut prev_state = hid::pressed_mask();
    let mut mask = 0;
    for button in which {
        mask |= 1 << (*button as u16);
    }
    wait_for_any_mask(mask, &mut prev_state).into()
}

#[inline(always)]
pub fn wait_for_all_of(which: &[hid::Button]) -> hid::ButtonsPressed {
    let mut prev_state = hid::pressed_mask();
    let mut mask = 0;
    for button in which {
        mask |= 1 << (*button as u16);
    }
    wait_for_button_mask(mask, mask, &mut prev_state).into()
}
