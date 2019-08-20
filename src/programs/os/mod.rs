#![allow(dead_code)]

use core::slice::from_raw_parts;

use io::timer;
use realtime::{SleepTimer};
use caches;
use interrupts;
use gfx;

use fat;
use mem;

mod loader;

static mut CURR_TLS: u32 = 0;

fn swi_handler(which: u32, _is_thumb: bool, regs: &mut [u32; 15], pc: &mut u32) {
    if which == 1 {
        loader::setup_program_stack(regs, pc);
        return
    }
    if which == 2 {
        log!("ping! at pc={:08X}, lr={:08X}", pc, regs[14]);
        return
    }

    log!("syscall#: {:X}, r0: {:08X}, r1: {:08X}, r2: {:08X}, r3: {:08X}, r4: {:08X}, r5: {:08X}", regs[7], regs[0], regs[1], regs[2], regs[3], regs[4], regs[5]);
    assert!(which == 0);

    let syscall = regs[7];
    match syscall {
        1 => loop {}
        4 => { // Write
            let fd = regs[0];
            let buf = regs[1];
            let count = regs[2];
            assert!(fd == 1);

            gfx::log(unsafe { from_raw_parts(buf as *const u8, count as usize) });
        }
        0xa8 => { // Poll
            #[repr(C)]
            struct pollfd {
                fd: i32,
                events: i16,
                revents: i16,
            }

            const POLLOUT: i16 = 4;

            let fds = regs[0] as *mut pollfd;
            let num_fds = regs[1] as usize;
            let _timeout = regs[2];

            for i in 0..num_fds {
                let polldat = unsafe { &mut *fds.add(i) };
                match polldat.fd {
                    0 => polldat.revents |= polldat.events & POLLOUT,
                    _ => {}
                }
            }
            gfx::draw_commit();
            ::input::wait_for_all_of(&[::io::hid::Button::Select]);
        }
        0xc0 => { // MMAP2
            const MAP_ANONYMOUS: u32 = 0x20;

            let addr = regs[0];
            let len = regs[1];
            let _prot = regs[2];
            let flags = regs[3];
            let fd = regs[4];
            let pgoff = regs[5];
            assert!(addr == 0);
            assert!(fd == !0);
            assert!(pgoff == 0);
            assert!(flags & MAP_ANONYMOUS != 0);

            log!("Attempting to map memory of size 0x{:X}", len);
            if let Ok(ptr) = unsafe { mem::Global.alloc_array(len as usize, 4096) } {
                regs[0] = ptr.as_ptr() as *mut u8 as u32;
            } else {
                regs[0] = 0;
            }
        }
        0x100 => { // Set thread ID address
            log!("Setting TID address to {:08X}", regs[0]);
        }
        0xf0005 => { // Set TLS
            log!("Setting TLS pointer to {:08X}", regs[0]);
            unsafe { CURR_TLS = regs[0] };
        }
        0xf0006 => { // CUSTOM SYSCALL: Get TLS
            unsafe { regs[0] = CURR_TLS };
            log!("Requesting TLS pointer {:08X}", regs[0]);
        }
        0xf1001 => { // CUSTOM SYSCALL: Compare Exchange
            let old = regs[0];
            let new = regs[1];
            let ptr = regs[2] as *mut u32;
            unsafe {
                if *ptr != old {
                    regs[0] = !0;
                } else {
                    *ptr = new;
                    regs[0] = 0;
                }
            }
        }
        0xf1002 => { // CUSTOM SYSCALL: Data/Memory Barrier
        }
        _ => panic!("Unimplemented syscall {:X}", syscall)
    }

}

pub fn main() {
    let timer0 = timer::Timer::new(lease!(TIMER0), 0, 0, timer::Prescaler::Div1, None);
    let _sleep_timer = SleepTimer::new(&timer0);
    caches::enable_all();

    gfx::clear_screen(255, 255, 255);

    interrupts::register_swi_handler(swi_handler).unwrap();

    let mut fs = fat::Fs::init();
    loader::load_elf(&mut fs, "/init.elf", loader::LoadDst::Static(0x08010000));
}
