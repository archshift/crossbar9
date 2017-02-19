# crossbar9

### Cross-compiled tests for the 3DS ARM9

## Before use

To run `./build.sh`, you need to install `xargo` and be using `rustup` for managing rust installations.

## Usage

```
./build.sh [test name] [debug|release]
```

Copy the generated file `./target/thumbv5te-none-eabi/<relmode>/crossbar9.3dsx` to your 3DS's SD card, and run using the Homebrew launcher.