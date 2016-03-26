# crossbar9 

### Cross-compiled rust template for the 3DS ARM9

## Before use

(You should probably use multirust to make your life easier.)

First, you need to install a cross-compiled version of libcore to your machine.
- Clone https://github.com/phil-opp/nightly-libcore, and copy thumbv5te-none-eabi.json to that folder.
- Run nightly-libcore's `install-libcore.sh` script: `multirust run nightly sh install-libcore.sh thumbv5te-none-eabi`
