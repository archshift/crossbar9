use io::i2c;

pub fn power_off() -> ! {
    loop { i2c::write_byte(i2c::Device::MCU, 0x20, 0x1).unwrap(); }
}

pub fn reboot() -> ! {
    loop { i2c::write_byte(i2c::Device::MCU, 0x20, 0x4).unwrap(); }
}