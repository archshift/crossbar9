#!/bin/bash

TARGET=thumbv5te-none-eabi

export CC=arm-none-eabi-gcc
rustup run nightly xargo build --release || exit

TARGET_DIR="./target/${TARGET}/release"
TARGET_ELF="${TARGET_DIR}/3dstest"
TARGET_BIN="${TARGET_DIR}/3dstest.bin"
TARGET_3DSX="${TARGET_DIR}/3dstest.3dsx"

arm-none-eabi-objcopy -O binary -I elf32-littlearm "$TARGET_ELF" "$TARGET_BIN" || exit

LDR_APPINFO_FILE="./BrahmaLoader/resources/AppInfo"
LDR_PAYLOAD_FILE="./BrahmaLoader/data/payload.bin"
LDR_OUTPUT_DIR="./BrahmaLoader/output"

cp "$TARGET_BIN" "$LDR_PAYLOAD_FILE"
cp ./AppInfo "$LDR_APPINFO_FILE"
( cd ./BrahmaLoader && make )
cp -R "${LDR_OUTPUT_DIR}/" "$TARGET_DIR"
rm -r "${LDR_OUTPUT_DIR}"
