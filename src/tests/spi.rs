use gfx;
use io::timer;
use realtime;

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    log!("Hello, world!");

    let timer = timer::Timer::new(lease!(TIMER0), 0, 0, timer::Prescaler::Div1, None);
    let sleep_timer = realtime::SleepTimer::new(&timer);

    use io::{mic, spi, spi_tsc};

    // spi::write2(3, &[0], &[0]);
    // spi::write2(3, &[0x43], &[0x80]);
    // for i in 0..10000 {
    //     spi::write2(3, &[0], &[0]);
    //     let mut out = [0u8];
    //     spi::write_read(3, &[0x43], &mut out);
    //     log!("{:02X}", out[0]);
    // }

    unsafe { ::core::ptr::write_volatile(0x10141220 as *mut u8, 2u8); }
    realtime::msleep(10);

    spi_tsc::write_reg(spi_tsc::TscReg::MicBias, 3); // AVDD bias
    let adc_mic = spi_tsc::read_reg(spi_tsc::TscReg::AdcMic);
    spi_tsc::write_reg(spi_tsc::TscReg::AdcMic, adc_mic | 0x80); // Power on
    while spi_tsc::read_reg(spi_tsc::TscReg::AdcFlags) & 0x40 == 0 {
    } // Wait till powered on
    log!("Powered on...");
    let adc_finevol = spi_tsc::read_reg(spi_tsc::TscReg::AdcFineVol);
    spi_tsc::write_reg(spi_tsc::TscReg::AdcMic, adc_mic & !0x80); // Unmute
    log!("Unmuted...");

    let mic = mic::Mic::enable(lease!(MIC), mic::SampleRate::Hz32k);
    let mic2 = mic::Mic::enable(lease!(MIC), mic::SampleRate::Hz32k);
    for i in 0..10000 {
        log!("{:X}", mic.curr_data());
    }
}
