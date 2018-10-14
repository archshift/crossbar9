macro_rules! define_programs {
    ( $($feature:expr => $name:ident),* $(,)* ) => (
        $(
            pub mod $name;
        )*
        pub fn main() {
            $(
                if (env!("C9_PROG_TYPE") == $feature) {
                    ::programs::$name::main()
                }
            )*
        }
    )
}

define_programs!(
    "aes" => aes,
    "caches" => caches,
    "hello_world" => hello_world,
    "ndma" => ndma,
    "rsa" => rsa,
    "sleep" => sleep_timer,
    "sha" => sha_hasher,
    "armwrestler" => armwrestler,

    "os" => os
);
