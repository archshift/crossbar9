use gfx;
use input;
use io::hid;
use io::timer;
use realtime;

pub fn main() {
    gfx::clear_screen(255, 255, 255);

    let timer = timer::Timer::new(lease!(TIMER0), 0, 0, timer::Prescaler::Div1, None);
    let _sleep_timer = realtime::SleepTimer::new(&timer);

    log!("Start pressing buttons!");
    loop {
        let hid::ButtonsPressed(pressed) = input::wait_for_any();

        let only_pressed = pressed.iter().enumerate().filter(|&(_, x)| *x);
        for (i, _) in only_pressed {
            let string = match i.into() {
                hid::Button::A => "A",
                hid::Button::B => "B",
                hid::Button::X => "X",
                hid::Button::Y => "Y",
                hid::Button::L => "L",
                hid::Button::R => "R",
                hid::Button::Start => "Start",
                hid::Button::Select => "Select",
                hid::Button::DPadU => "D-Pad Up",
                hid::Button::DPadD => "D-Pad Down",
                hid::Button::DPadL => "D-Pad Left",
                hid::Button::DPadR => "D-Pad Right",
            };
            log!("{}", string);
        }
    }
}
