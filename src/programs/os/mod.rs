#![allow(dead_code)]

extern crate xmas_elf as elf;

use core::ptr::null;
use core::slice::{from_raw_parts};

use io::timer;
use realtime::{SleepTimer};
use caches;
use interrupts;
use gfx;

use fat;
use mem;

// struct Process {

// }

extern {
    fn userspace_jmp(addr: u32, args: *const *const u8, env: *const *const u8);
}

fn patch_got_in_mem(got: &elf::sections::SectionHeader, host_base: *mut u8, guest_base: u32) {
    let host_base = host_base as u32;
    let start = ((got.address() as u32) - guest_base + host_base) as *mut u32;
    let words = (got.size() / 4) as usize;
    for i in 0..words {
        unsafe { *start.add(i) += host_base - guest_base };
    }
}

fn load_program(fs: &mut fat::Fs, path: &str) {
    let mut file = fs.open(path);
    let size = file.size();
    let mut buf = mem::Array::<u8>::new(size);
    let read_amount = file.read(&mut buf[..]);
    assert!(read_amount == size);

    let elf = elf::ElfFile::new(&buf[..]).unwrap();

    let min_addr = elf.program_iter()
        .filter(|x| x.mem_size() != 0)
        .map(|x| x.virtual_addr())
        .min()
        .unwrap();

    let max_align = elf.program_iter()
        .map(|x| x.align())
        .max()
        .unwrap();
    
    let max_addr = elf.program_iter()
        .filter(|x| x.mem_size() != 0)
        .map(|x| x.virtual_addr() + x.mem_size())
        .max()
        .unwrap();

    let proc_size = max_addr - min_addr;
    let mut process = mem::Array::<u8>::aligned_new(proc_size as usize, max_align as usize);

    log!("Found proc size {:X}, base addr {:X}, alignment: {:X}", proc_size, min_addr, max_align);
    
    for program in elf.program_iter() {
        let offs = (program.virtual_addr() - min_addr) as usize;
        let size = program.mem_size() as usize;
        let elf_size = program.file_size() as usize;
        let elf_offs = program.offset() as usize;

        if size == 0 { continue }
        assert!(elf_size <= size);

        log!("loading from {:X} size {:X} to {:X} size {:X}", elf_offs, elf_size, offs, size);
        process[offs..offs + elf_size].copy_from_slice(&buf[elf_offs..elf_offs + elf_size]);
        for b in &mut process[offs + elf_size .. offs + size] {
            *b = 0;
        }
    }

    if let Some(got) = elf.find_section_by_name(".got") {
        patch_got_in_mem(&got, &mut process[0], min_addr as u32);
    }

    log!("Running at entrypoint {:X}", elf.header.pt2.entry_point());

    let entry_offs = (elf.header.pt2.entry_point() - min_addr) as usize;
    log!("at offset {:X}", entry_offs);
    let entry = &process[entry_offs] as *const u8;

    let arg = [b"init.bin\0".as_ptr(), null()];
    let env = [null()];

    unsafe { userspace_jmp(entry as u32, arg.as_ptr(), env.as_ptr()); }
}

fn setup_program_stack(regs: &mut [u32; 15]) {
    let entrypoint = regs[0];
    let argv = regs[1] as *const *const u8;
    let envp = regs[2] as *const *const u8;

    let sp = 0x08004000;
    let full_size = |mut vec: *const *const u8| unsafe {
        use ffistr::str_len;
        let mut bytes = 0;
        let mut count = 1;
        while *vec != null() {
            bytes += str_len(*vec) + 1;
            count += 1;
            vec = vec.offset(1);
        }
        (count, bytes)
    };
    let (argc, arg_size) = full_size(argv);
    let (envc, env_size) = full_size(envp);
    let auxc = 1;

    let link_into = |mut vec: *const *const u8, mut data: *mut u8, mut ptrs: *mut u32| unsafe {
        use ffistr::str_cpy;
        while *vec != null() {
            print!("{:08X} -> {:08X}: ", ptrs as u32, data as u32);
            gfx::log(::ffistr::str_bytes(&*vec));
            log!("");

            let amount = str_cpy(data, *vec) + 1;
            *ptrs = data as u32;
            data = data.add(amount);
            ptrs = ptrs.offset(1);
            vec = vec.offset(1);
        }
        log!("{:08X} -> 0", ptrs as u32);
        *ptrs = 0;
    };

    // STACK DESCRIPTION
    let sp_envdat_end = sp;
    let sp_envdat_start = sp_envdat_end - env_size;

    let sp_argdat_end = sp_envdat_start;
    let sp_argdat_start = sp_argdat_end - arg_size;

    let sp_aux_end = sp_argdat_start & !0xF;
    let sp_aux_start = sp_aux_end - 8*auxc;

    let sp_env_end = sp_aux_end & !0xF;
    let sp_env_start = sp_env_end - 4*envc;

    let sp_arg_start = sp_env_start - 4*argc;
    let sp_argc_start = sp_arg_start - 4;
    // END STACK DESCRIPTION

    link_into(envp, sp_envdat_start as *mut u8, sp_env_start as *mut u32);
    link_into(argv, sp_argdat_start as *mut u8, sp_arg_start as *mut u32);
    // TODO: handle auxvec properly
    unsafe { (sp_aux_start as *mut u8).write_bytes(0u8, auxc*8) };
    unsafe { *(sp_argc_start as *mut u32) = (argc - 1) as u32 };

    regs[13] = sp_argc_start as u32;
    regs[14] = entrypoint;
}

static mut CURR_TLS: u32 = 0;

fn swi_handler(which: u32, _is_thumb: bool, regs: &mut [u32; 15]) {
    if which == 1 {
        setup_program_stack(regs);
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
        0x100 => { // Set thread ID address
            log!("Setting TID address to {:08X}", regs[0]);
        }
        0xf0005 => { // Set TLS
            log!("Setting TLS pointer to {:08X}", regs[0]);
            unsafe { CURR_TLS = regs[0] };
        }
        0xf1000 => { // CUSTOM SYSCALL: Get TLS
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
    load_program(&mut fs, "/init.bin");
}
