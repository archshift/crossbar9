extern crate gcc;

fn main() {
    gcc::Config::new()
        .file("src/start.s")
        .file("src/interrupts.s")
        .compile("libstart.a");

    gcc::Config::new()
        .flag("-w")
        .file("src/tests/armwrestler.s")
        .compile("libarmwrestler.a");
}
