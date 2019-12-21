use gfx;

use io::{timer, mic};
use realtime::{self, msleep};

pub fn main() {
    gfx::clear_screen(0xFF, 0xFF, 0xFF);
    log!("Hello, world!");

    let timer = timer::Timer::new(lease!(TIMER0), 0, 0, timer::Prescaler::Div1, None);
    let _sleep_timer = realtime::SleepTimer::new(&timer);

    extern {
        fn cdcReadReg(bank: u8, reg: u8) -> u8;
        // fn cdcReadRegArray(bank: u8, reg: u8, buf: *mut u8, buf_size: u8);
        fn cdcWriteReg(bank: u8, reg: u8, val: u8);
    }
    unsafe { (0x10145000 as *mut u16).write_volatile(0x9820); }
    unsafe { (0x10145002 as *mut u16).write_volatile(0xE000); }

    unsafe { (0x10141230 as *mut u32).write_volatile(0b11); } // DSP_CNT
    msleep(20);
    unsafe { (0x10141114 as *mut u16).write_volatile(0b11); } // CODEC0
    unsafe { (0x10141116 as *mut u16).write_volatile(0b11); } // CODEC1
    unsafe { (0x10141220 as *mut u32).write_volatile(0b11); } // CODEC_CNT
    msleep(100);

    //let mut spi_buf_out = [0u8; 0x80];
    //unsafe { cdcReadRegArray(1, 1, spi_buf_out.as_mut_ptr(), 0x7F); }

    fn mic_enable_amp(gain: u8) -> Result<(), ()> {
        const CDC_CONTROL: u8 = 0;
        const CDC_SOUND: u8 = 1;
        let gaintbl = [ 0x1F, 0x2B, 0x37, 0x43 ];

        unsafe {
            cdcWriteReg(CDC_SOUND, 0x2E, 0x03); // set adc bias
            cdcWriteReg(CDC_CONTROL, 0x51, 0x80); // turn on adc

            for _ in 0..100 {
                let adc_on = cdcReadReg(CDC_CONTROL, 0x24) & 0x40 != 0;
                if adc_on {
                    cdcWriteReg(CDC_CONTROL, 0x52, 0x00); // unmute adc
                    cdcWriteReg(CDC_SOUND, 0x2F, gaintbl[(gain&3) as usize]); // set gain
                    return Ok(())
                }
                msleep(1);
            }
        }
        Err(())
    }

    // let mut spi = spi::Spi::new(lease!(SPI0));
    // spi.chipselect(3);
    // spi.write(&[0x00, 0x00, 0x01]);
    // let mut spi_buf_out = [0u8; 0x80];
    // spi.read(&mut spi_buf_out);
    // spi.deselect();

    // for b in spi_buf_out.iter() {
    //     print!("{:02X} ", b);
    // }

    log!("Enabling amp... {}", if mic_enable_amp(1).is_ok() { "ostensibly ok!" } else {"timed out!" });

    let mic = mic::Mic::enable(lease!(MIC), mic::SampleRate::Hz32k);
    for _i in 0..100 {
        print!("{:X?} ", mic.curr_data());
        msleep(10);
    }
}
