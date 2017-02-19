#[cfg(feature="hello_world")] mod hello_world;
#[cfg(feature="hello_world")] pub use self::hello_world::main;

#[cfg(feature="sleep")] mod sleep_timer;
#[cfg(feature="sleep")] pub use self::sleep_timer::main;