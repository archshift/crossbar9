macro_rules! define_tests {
    ( $($feature:expr => $name:ident),* ) => (
        $(
            pub mod $name;
        )*
        pub fn main() {
            $(
                if (env!("C9_TEST_TYPE") == $feature) {
                    ::tests::$name::main()
                }
            )*
        }
    )
}

define_tests!(
    "aes" => aes,
    "caches" => caches,
    "hello_world" => hello_world,
    "ndma" => ndma,
    "rsa" => rsa,
    "sleep" => sleep_timer,
    "sha" => sha_hasher,
    "armwrestler" => armwrestler
);