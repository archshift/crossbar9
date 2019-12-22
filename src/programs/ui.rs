use gfx;
use gfx::ui;

use io::timer;
use realtime;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    let timer = timer::Timer::new(lease!(TIMER0), 0, 0, timer::Prescaler::Div1, None);
    let _sleep_timer = realtime::SleepTimer::new(&timer);

    let ui = ui::Ui::new(gfx::top_screen, [0, 0, 0]);
    let center_box = ui.subbox((10, 10));
    ui.draw_box(center_box);
}