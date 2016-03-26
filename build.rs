extern crate gcc;

fn main() {
    gcc::Config::new()
        .file("src/start.s")
        .flag("-fno-rtti")
        .flag("-fno-exceptions")
        .compile("lib3dsasm.a");
}
