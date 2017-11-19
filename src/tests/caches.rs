use caches;
use gfx;
use io::timer::{Timer, Prescaler};

fn bench_busyloop(timer: &Timer) {
    extern {
        fn cbench_busywait(start: u32, end: u32);
    }

    let num = 0x100000-1;
    let now = timer.us_val();
    unsafe { cbench_busywait(0, num); }
    let end = timer.us_val();

    log!("{:#X} instructions took {}us", 3 * ((num as u64) + 1), end-now);
}

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    let timer = Timer::new(0, 0, Prescaler::Div1, None).unwrap();
    timer.start();

    log!("Caches disabled...");
    caches::disable_all();
    bench_busyloop(&timer);

    log!("Caches enabled...");
    caches::enable_all();
    bench_busyloop(&timer);
}
