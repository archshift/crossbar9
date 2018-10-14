#![allow(dead_code)]

extern crate xmas_elf as elf;

use core::mem::transmute;

use io::timer;
use realtime::{SleepTimer};
use caches;
use interrupts;
use gfx;

use fat;
use mem;

// struct Process {

// }

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
    
    let max_prog = elf.program_iter()
        .filter(|x| x.mem_size() != 0)
        .max_by_key(|x| x.virtual_addr())
        .unwrap();

    let proc_size = max_prog.virtual_addr() + max_prog.mem_size() - min_addr;
    let mut process = mem::Array::<u8>::aligned_new(proc_size as usize, max_align as usize);

    log!("Found proc size {:X}, base addr {:X}, alignment: {:X}", proc_size, min_addr, max_align);
    
    for program in elf.program_iter() {
        let offs = (program.virtual_addr() - min_addr) as usize;
        let size = program.mem_size() as usize;
        let elf_size = program.file_size() as usize;
        let elf_offs = program.offset() as usize;
        if size == 0 { continue }
        log!("loading from {:X} size {:X} to {:X} size {:X}", elf_offs, elf_size, offs, size);
        process[offs..offs + size].copy_from_slice(&buf[elf_offs..elf_offs + elf_size]);
    }

    log!("Running at entrypoint {:X}", elf.header.pt2.entry_point());

    let entry_offs = (elf.header.pt2.entry_point() - min_addr) as usize;
    log!("at offset {:X}", entry_offs);
    let entry = &process[entry_offs] as *const u8;
    let entry_fn: extern fn() = unsafe { transmute(entry) };
    entry_fn();
}

fn swi_handler(which: u32) {
    match which {
        _ => unimplemented!()
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
