extern crate cc;

use std::env;

static ARCH_FLAGS: &[&str] = &["-mthumb-interwork", "-mcpu=arm946e-s", "-msoft-float"];

fn gcc_config() -> cc::Build {
    let mut config = cc::Build::new();
    for flag in ARCH_FLAGS {
        config.flag(flag);
    }
    config
        .flag("-fno-strict-aliasing")
        .flag("-std=c11");
    config
}

fn main() {
    println!("cargo:rerun-if-env-changed=C9_PROG_TYPE");

    // Make sure the requested program actually exists
    let prog = env::var("C9_PROG_TYPE").unwrap();
    let modfile = include_str!("src/programs/mod.rs");
    let start = modfile.find("define_programs!(").unwrap();
    let end = modfile[start..].find(");").unwrap() + start;
    modfile[start..end].find(&format!("\"{}\"", prog))
        .expect(&format!("Could not find program `{}`!", prog));
    
    println!("cargo:rerun-if-changed=src/start.s");
    println!("cargo:rerun-if-changed=src/interrupts.s");
    println!("cargo:rerun-if-changed=src/caches.s");

    gcc_config()
        .file("src/start.s")
        .file("src/interrupts.s")
        .file("src/caches.s")
        .compile("libstart.a");

    println!("cargo:rerun-if-changed=src/armwrestler.s");
    println!("cargo:rerun-if-changed=src/cache_benchers.s");

    gcc_config()
        .flag("-w")
        .file("src/programs/armwrestler.s")
        .file("src/programs/cache_benchers.s")
        .compile("libtestasm.a");

    println!("cargo:rerun-if-changed=src/programs/os/entry.s");

    gcc_config()
        .flag("-w")
        .file("src/programs/os/entry.s")
        .compile("libosasm.a");

    gcc_config()
        .flag("-w")
        .include("Decrypt9WIP/source/fatfs")
        .include("Decrypt9WIP/source")
        .file("Decrypt9WIP/source/fatfs/ff.c")
        .file("Decrypt9WIP/source/fatfs/delay.s")
        .file("Decrypt9WIP/source/fatfs/sdmmc.c")
        .file("Decrypt9WIP/source/fatfs/diskio.c")
        .file("Decrypt9WIP/source/fs.c")
        .compile("libd9fs.a");
}
