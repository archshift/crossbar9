extern crate cc;

static ARCH_FLAGS: &[&str] = &["-mthumb-interwork", "-mcpu=arm946e-s", "-msoft-float"];

fn gcc_config() -> cc::Build {
    let mut config = cc::Build::new();
    for flag in ARCH_FLAGS {
        config.flag(flag);
    }
    config
}

fn main() {
    gcc_config()
        .file("src/start.s")
        .file("src/interrupts.s")
        .file("src/caches.s")
        .compile("libstart.a");

    gcc_config()
        .flag("-w")
        .file("src/tests/armwrestler.s")
        .file("src/tests/cache_benchers.s")
        .compile("libtestasm.a");
}
