#!/bin/bash

TARGET=thumbv5te-none-eabi

export C9_PROG_TYPE="$1"
if [ "$1" = "" ]; then
    export C9_PROG_TYPE="os"
fi

echo "Compiling program \"$C9_PROG_TYPE\"..."

RELMODE="$2"
if [ "$2" = "release" ]; then
    RELMODE="release"
    RELMODE_FLAG="--release"
else
    RELMODE="debug"
    RELMODE_FLAG=""
fi

export CROSS_COMPILE=arm-none-eabi-
export RUST_TARGET_PATH="$(pwd)"
rustup run nightly xargo build $RELMODE_FLAG || exit -1

TARGET_DIR="./target/${TARGET}/${RELMODE}"
TARGET_ELF="${TARGET_DIR}/crossbar9"
TARGET_BIN="${TARGET_DIR}/crossbar9.bin"
TARGET_FIRM="${TARGET_DIR}/crossbar9.firm"

arm-none-eabi-objcopy -O binary -I elf32-littlearm "$TARGET_ELF" "$TARGET_BIN" || exit -1
./firmtool/firmtool build "$TARGET_FIRM" -i -n 0x23F00000 -e 0 -D "$TARGET_BIN" -A 0x23F00000 -C NDMA || exit -1
