use core::ptr::null;
use core::slice::from_raw_parts_mut;

use xmas_elf as elf;
use xmas_elf::dynamic::Tag;

use alloc::vec::Vec;

use fat;
use gfx;
use mem;

extern {
    fn userspace_jmp(addr: u32, args: *const *const u8, env: *const *const u8) -> !;
}

pub enum LoadDst {
    Static(u32),
    Dynamic,
}

fn read_bin(fs: &mut fat::Fs, path: &str, load_dst: LoadDst) -> &'static mut [u8] {
    let mut file = fs.open(path);
    let size = file.size();
    let buf = match load_dst {
        LoadDst::Static(addr) => unsafe {
            from_raw_parts_mut(addr as *mut u8, size)
        },
        LoadDst::Dynamic => {
            mem::Array::aligned_new(size, 0x1000).leak()
        }
    };
    let read_amount = file.read(buf);
    assert!(read_amount == size);
    buf
}

fn copy_program_sections<'a>(program: &elf::program::ProgramHeader<'a>, buf: &'a [u8], reloc_offs: usize) {
    let mem_base = (program.virtual_addr() + reloc_offs as u64) as *mut u8;
    let size = program.mem_size() as usize;
    let elf_offs = program.offset() as usize;
    let elf_size = program.file_size() as usize;

    if size == 0 {
        return
    }

    log!("loading from {:X} size {:X} to {:X} size {:X}", elf_offs, elf_size, mem_base as u32, size);
    unsafe {
        mem_base.copy_from_nonoverlapping(&buf[elf_offs], elf_size);

        let mem_program_end = mem_base.add(elf_size);
        assert!(size >= elf_size);
        mem_program_end.write_bytes(0, size - elf_size);
    }
}

const DT_REL: u32 = 17;
const DT_RELA: u32 = 7;
fn do_relocation(elf: &elf::ElfFile, rel_type: u32, rel_table: u32, rel_table_size: u32, symtab: u32, reloc_offs: usize) {
    let reloc_size = match rel_type {
        DT_REL => 8,
        // Some(DT_RELA) => 12,
        _ => panic!("Relocation table present with invalid PLTREL!"),
    };
    let num_relocs = rel_table_size / reloc_size;
    assert_eq!(num_relocs * reloc_size, rel_table_size, "Relocation table size mismatch!");
    
    let mut reloc_entry = elf_mkptr::<u32>(rel_table, reloc_offs);
    for _i in 0..num_relocs {
        unsafe {
            let offs = *reloc_entry;
            let addr = elf_mkptr::<u32>(offs, reloc_offs);
            let info = *reloc_entry.add(1);
            let ty = info as u8;
            let sym = info >> 8;

            let sym_size = 16;
            let sym_name_idx = elf_mkptr::<u32>(symtab + sym_size * sym, reloc_offs);
            let sym_name = elf.get_dyn_string(*sym_name_idx).unwrap();

            log!("Relocating {} {} at {:08X}, with type {}",
                if sym != 0 {"symbol"} else {""},
                if sym != 0 {sym_name} else {""},
                addr as u32,
                ty);
            
            reloc_entry = reloc_entry.add((reloc_size / 4) as usize);
        }
    }
}

fn elf_mkptr<T>(pos: u32, reloc_offs: usize) -> *mut T {
    (pos + reloc_offs as u32) as *mut T
}

static mut LOADED_BINS: Option<Vec<&'static [u8]>> = None;

#[allow(unconditional_recursion)] // False positive
pub fn load_elf(fs: &mut fat::Fs, path: &str, load_dst: LoadDst) {
    let loaded_bins = unsafe {
        if LOADED_BINS.is_some() {
            LOADED_BINS.as_mut().unwrap()
        } else {
            LOADED_BINS = Some(Vec::new());
            LOADED_BINS.as_mut().unwrap()
        }
    };

    let buf = read_bin(fs, path, load_dst);
    loaded_bins.push(buf);
    let elf = elf::ElfFile::new(&buf).unwrap();

    for program in elf.program_iter() {
        let reloc_offs = match elf.header.pt2.type_().as_type() {
            elf::header::Type::Executable => 0,
            elf::header::Type::SharedObject => buf.as_ptr() as usize,
            _ => unimplemented!(),
        };
        copy_program_sections(&program, &buf, reloc_offs);

        let mut jmprel = None;
        let mut pltrel = None;
        let mut pltrel_size = None;
        let mut rel = None;
        let mut rel_size = None;
        let mut symtab = None;
        let mut needed = [0u32; 16];
        let mut num_needed = 0;

        if let Ok(elf::program::SegmentData::Dynamic32(seg_data)) = program.get_data(&elf) {
            for dynamic in seg_data {
                let tag = dynamic.get_tag();

                match tag {
                    Ok(Tag::Needed) => {
                        needed[num_needed] = dynamic.get_val().unwrap();
                        num_needed += 1;
                    }

                    Ok(Tag::JmpRel) => jmprel = dynamic.get_ptr().ok(),
                    Ok(Tag::PltRel) => pltrel = dynamic.get_val().ok(),
                    Ok(Tag::PltRelSize) => pltrel_size = dynamic.get_val().ok(),
                    Ok(Tag::Rel) => rel = dynamic.get_ptr().ok(),
                    Ok(Tag::RelSize) => rel_size = dynamic.get_val().ok(),
                    Ok(Tag::SymTab) => symtab = dynamic.get_ptr().ok(),

                    Ok(Tag::Null) => break,
                    
                    Err(s) => panic!("ERROR loading elf file: `{}`!", s),
                    _ => {}
                }
            }
        }

        for i in 0..num_needed {
            let needed_str = elf.get_dyn_string(needed[i]).unwrap();
            log!("Loading library `{}`!", needed_str);
            load_elf(fs, needed_str, LoadDst::Dynamic); // Todo: breadth-first dep resolution
        }

        if let Some(jmprel) = jmprel {
            do_relocation(&elf, pltrel.unwrap(), jmprel, pltrel_size.unwrap(), symtab.unwrap(), reloc_offs);
        }

        if let Some(rel) = rel {
            do_relocation(&elf, DT_REL, rel, rel_size.unwrap(), symtab.unwrap(), reloc_offs);
        }

    }

    // if let Some(got) = elf.find_section_by_name(".got") {
    //     patch_got_in_mem(&got, &mut process[0], min_addr as u32);
    // }

    let entry = elf.header.pt2.entry_point() as u32;
    let arg = [b"init.bin\0".as_ptr(), null()];
    let env = [null()];

    log!("Entering _start at {:08X}", entry);

    unsafe { userspace_jmp(entry, arg.as_ptr(), env.as_ptr()); }
}

pub fn setup_program_stack(regs: &mut [u32; 15], pc: &mut u32) {
    let entrypoint = regs[0];
    let argv = regs[1] as *const *const u8;
    let envp = regs[2] as *const *const u8;

    let sp = 0x0800C000;
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
    regs[14] = 0xFFFF0000;
    *pc = entrypoint;
}