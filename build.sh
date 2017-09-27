#!/bin/bash

TARGET=thumbv5te-none-eabi

FEATURE="$1"
if [ "$1" = "" ]; then
    FEATURE="hello_world"
fi

echo "Compiling test \"$FEATURE\"..."

RELMODE="$2"
if [ "$2" = "debug" ]; then
    RELMODE_FLAG=""
else
    RELMODE="release"
    RELMODE_FLAG="--release"
fi

export CC=arm-none-eabi-gcc
rustup run nightly xargo build --features "$FEATURE" $RELMODE_FLAG || exit -1

TARGET_DIR="./target/${TARGET}/${RELMODE}"
TARGET_ELF="${TARGET_DIR}/crossbar9"
TARGET_BIN="${TARGET_DIR}/crossbar9.bin"
TARGET_FIRM="${TARGET_DIR}/crossbar9.firm"

arm-none-eabi-objcopy -O binary -I elf32-littlearm "$TARGET_ELF" "$TARGET_BIN" || exit -1
./firmtool/firmtool build "$TARGET_FIRM" -i -n 0x23F00000 -e 0 -D "$TARGET_BIN" -A 0x23F00000 -C NDMA || exit -1
