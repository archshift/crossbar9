macro_rules! define_test {
    ($feature:expr, $name:ident) => (
        #[cfg(feature=$feature)] pub mod $name;
        #[cfg(feature=$feature)] pub use self::$name::main;
    )
}

define_test!("aes", aes);
define_test!("hello_world", hello_world);
define_test!("ndma", ndma);
define_test!("rsa", rsa);
define_test!("sleep", sleep_timer);
define_test!("sha", sha_hasher);
define_test!("armwrestler", armwrestler);