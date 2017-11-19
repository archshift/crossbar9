extern crate gcc;

fn main() {
    gcc::Config::new()
        .flag("-mthumb-interwork")
        .file("src/start.s")
        .file("src/interrupts.s")
        .file("src/caches.s")
        .compile("libstart.a");

    gcc::Config::new()
        .flag("-w")
        .flag("-mthumb-interwork")
        .file("src/tests/armwrestler.s")
        .file("src/tests/cache_benchers.s")
        .compile("libtestasm.a");
}
