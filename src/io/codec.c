/*---------------------------------------------------------------------------------

	DSi "codec" Touchscreen/Sound Controller control for ARM7

	Copyright (C) 2017
		fincs

	This software is provided 'as-is', without any express or implied
	warranty.  In no event will the authors be held liable for any
	damages arising from the use of this software.

	Permission is granted to anyone to use this software for any
	purpose, including commercial applications, and to alter it and
	redistribute it freely, subject to the following restrictions:

	1.	The origin of this software must not be misrepresented; you
		must not claim that you wrote the original software. If you use
		this software in a product, an acknowledgment in the product
		documentation would be appreciated but is not required.
	2.	Altered source versions must be plainly marked as such, and
		must not be misrepresented as being the original software.
	3.	This notice may not be removed or altered from any source
		distribution.

---------------------------------------------------------------------------------*/

#include <stdint.h>
#include <stddef.h>

#include "spi.c"

#define u8 uint8_t
#define u16 uint16_t
#define u32 uint32_t

#define BIT(n) (1 << n)

#define REG_SPICNT (*(volatile u16*)0x10142000)
#define REG_SPIDATA (*(volatile u16*)0x10142002)

#define SPI_ENABLE  BIT(15)
#define SPI_IRQ     BIT(14)
#define SPI_BUSY    BIT(7)

// meh
#define SPI_BAUD_4MHz    0
#define SPI_BAUD_2MHz    1
#define SPI_BAUD_1MHz    2
#define SPI_BAUD_512KHz  3

// Pick the SPI transfer length
#define SPI_BYTE_MODE   (0<<10)
#define SPI_HWORD_MODE  (1<<10)

// Pick the SPI device
#define SPI_DEVICE_POWER      (0 << 8)
#define SPI_DEVICE_FIRMWARE   (1 << 8)
#define SPI_DEVICE_NVRAM      (1 << 8)
#define SPI_DEVICE_TOUCH      (2 << 8)
#define SPI_DEVICE_MICROPHONE (2 << 8)

// When used, the /CS line will stay low after the transfer ends
// i.e. when we're part of a continuous transfer
#define SPI_CONTINUOUS       BIT(11)

enum cdcBanks {
	CDC_CONTROL     = 0x00, // Chip control
	CDC_SOUND       = 0x01, // ADC/DAC control
	CDC_TOUCHCNT	= 0x03, // TSC control
	CDC_TOUCHDATA	= 0xFC, // TSC data buffer
};

// Direct register functions
u8   cdcReadReg(u8 bank, u8 reg);
void cdcReadRegArray(u8 bank, u8 reg, void* data, u8 size);
void cdcWriteReg(u8 bank, u8 reg, u8 value);
void cdcWriteRegMask(u8 bank, u8 reg, u8 mask, u8 value);
void cdcWriteRegArray(u8 bank, u8 reg, const void* data, u8 size);

//---------------------------------------------------------------------------------
static u8 readTSC(u8 reg) {
//---------------------------------------------------------------------------------

	while (REG_SPICNT & 0x80);

	REG_SPICNT = SPI_ENABLE | SPI_BAUD_4MHz | SPI_DEVICE_TOUCH | SPI_CONTINUOUS;
	REG_SPIDATA = 1 | (reg << 1);

	while (REG_SPICNT & 0x80);

	REG_SPICNT = SPI_ENABLE | SPI_BAUD_4MHz | SPI_DEVICE_TOUCH;
	REG_SPIDATA = 0;

	while (REG_SPICNT & 0x80);
	return REG_SPIDATA;
}

//---------------------------------------------------------------------------------
static void writeTSC(u8 reg, u8 value) {
//---------------------------------------------------------------------------------
	spiDeviceWrite2(NULL, 3, &reg, 1, &value, 1);
}

//---------------------------------------------------------------------------------
static void bankSwitchTSC(u8 bank) {
//---------------------------------------------------------------------------------

	static u8 curBank = 0x63;
	if (bank != curBank) {
		writeTSC(curBank == 0xFF ? 0x7F : 0x00, bank);
		curBank = bank;
	}
}

//---------------------------------------------------------------------------------
u8 cdcReadReg(u8 bank, u8 reg) {
//---------------------------------------------------------------------------------

	bankSwitchTSC(bank);
	return readTSC(reg);
}

//---------------------------------------------------------------------------------
void cdcReadRegArray(u8 bank, u8 reg, void* data, u8 size) {
//---------------------------------------------------------------------------------

	u8* out = (u8*)data;
	bankSwitchTSC(bank);

	while (REG_SPICNT & 0x80);

	REG_SPICNT = SPI_ENABLE | SPI_BAUD_4MHz | SPI_DEVICE_TOUCH | SPI_CONTINUOUS;
	REG_SPIDATA = 1 | (reg << 1);

	while (REG_SPICNT & 0x80);

	for (; size > 1; size--) {
		REG_SPIDATA = 0;
		while (REG_SPICNT & 0x80);
		*out++ = REG_SPIDATA;
	}

	REG_SPICNT = SPI_ENABLE | SPI_BAUD_4MHz | SPI_DEVICE_TOUCH;
	REG_SPIDATA = 0;

	while (REG_SPICNT & 0x80);

	*out++ = REG_SPIDATA;
}

//---------------------------------------------------------------------------------
void cdcWriteReg(u8 bank, u8 reg, u8 value) {
//---------------------------------------------------------------------------------

	bankSwitchTSC(bank);
	writeTSC(reg, value);
}

//---------------------------------------------------------------------------------
void cdcWriteRegMask(u8 bank, u8 reg, u8 mask, u8 value) {
//---------------------------------------------------------------------------------

	bankSwitchTSC(bank);
	writeTSC(reg, (readTSC(reg) &~ mask) | (value & mask));
}

//---------------------------------------------------------------------------------
void cdcWriteRegArray(u8 bank, u8 reg, const void* data, u8 size) {
//---------------------------------------------------------------------------------

	const u8* in = (u8*)data;
	bankSwitchTSC(bank);

	while (REG_SPICNT & 0x80);

	REG_SPICNT = SPI_ENABLE | SPI_BAUD_4MHz | SPI_DEVICE_TOUCH | SPI_CONTINUOUS;
	REG_SPIDATA = reg << 1;

	while (REG_SPICNT & 0x80);

	for (; size > 1; size--) {
		REG_SPIDATA = *in++;
		while (REG_SPICNT & 0x80);
	}

	REG_SPICNT = SPI_ENABLE | SPI_BAUD_4MHz | SPI_DEVICE_TOUCH;
	REG_SPIDATA = *in++;
}