use io::timer;
use gfx;
use realtime;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    let timer = timer::Timer::new(lease!(TIMER0), 0, 0, timer::Prescaler::Div1, None);
    let sleep_timer = realtime::SleepTimer::new(&timer);

    gfx::log(b"Starting timer...\n");
    realtime::sleep(1);
    gfx::log(b"1 second later!\n");
}