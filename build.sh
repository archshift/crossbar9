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
TARGET_3DSX="${TARGET_DIR}/crossbar9.3dsx"

arm-none-eabi-objcopy -O binary -I elf32-littlearm "$TARGET_ELF" "$TARGET_BIN" || exit -1

LDR_APPINFO_FILE="./BrahmaLoader/resources/AppInfo"
LDR_PAYLOAD_FILE="./BrahmaLoader/data/payload.bin"
LDR_OUTPUT_DIR="./BrahmaLoader/output"

cp "$TARGET_BIN" "$LDR_PAYLOAD_FILE" || exit -1
cp ./AppInfo "$LDR_APPINFO_FILE" || exit -1
( cd ./BrahmaLoader && make ) || exit -1
cp -R "${LDR_OUTPUT_DIR}/" "$TARGET_DIR" || exit -1
rm -r "${LDR_OUTPUT_DIR}" || exit -1
