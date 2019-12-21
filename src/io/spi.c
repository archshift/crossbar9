#include <stdint.h>
#include <stdbool.h>
#include <assert.h>

typedef uint32_t undefined4;
typedef uint32_t dword;
typedef uint32_t uint;
typedef uint8_t byte;

typedef struct {
    byte *xfer1_buf;
    uint xfer1_len;
    byte *xfer2_buf;
    uint xfer2_len;
    dword baudrate;
    byte device_num;
} SpiCtx2;

typedef struct {
    uint cnt;
    uint done;
    uint blk_len;
    uint fifo;
    uint status;
} SpiRegs;

bool device_usenew[3] = {true, true, true};
bool device_initialized[7] = {};
byte device_baud[7] = {};

#define CritEnter(...)
#define CritExit(...)

#define SPI0 (*(volatile SpiRegs*)0x10142000)
#define SPI1 (*(volatile SpiRegs*)0x10143000)
#define SPI2 (*(volatile SpiRegs*)0x10160000)

void SpiCtx2__Write(SpiCtx2 *ctx)
{
  volatile SpiRegs *spi;
  volatile dword *fifo;
  uint cnt;
  uint next_i;
  uint next_i_;
  uint cnt_dev_id;
  byte dev_id;
  uint i;
  
  dev_id = ctx->device_num;
  spi = &SPI2;
  switch(dev_id) {
  case 3:
  case 4:
  case 5:
    spi = &SPI0;
    break;
  case 6:
    spi = &SPI1;
  }
  fifo = &SPI2.fifo;
  switch(dev_id) {
  case 3:
  case 4:
  case 5:
    fifo = &SPI0.fifo;
    break;
  case 6:
    fifo = &SPI1.fifo;
  }
  switch(dev_id) {
  default:
    cnt_dev_id = 0;
    break;
  case 1:
  case 4:
    cnt_dev_id = 0x40;
    break;
  case 2:
  case 5:
    cnt_dev_id = 0x80;
  }
  do {
  } while ((spi->cnt & 0x8000) != 0);
  cnt = ctx->baudrate & 0xff | cnt_dev_id;
  spi->blk_len = ctx->xfer1_len;
  spi->cnt = cnt | 0xa000;
  i = 0;
  if (ctx->xfer1_len != 0) {
    do {
      if ((i & 0x1f) == 0) {
        do {
        } while ((spi->status & 1) != 0);
      }
      next_i = i + 4;
      *fifo = *(dword *)(ctx->xfer1_buf + i);
      i = next_i;
    } while (next_i < ctx->xfer1_len);
  }
  do {
  } while ((spi->cnt & 0x8000) != 0);
  spi->blk_len = ctx->xfer2_len;
  spi->cnt = cnt | 0xa000;
  i = 0;
  if (ctx->xfer2_len != 0) {
    do {
      if ((i & 0x1f) == 0) {
        do {
        } while ((spi->status & 1) != 0);
      }
      next_i_ = i + 4;
      *fifo = *(dword *)(ctx->xfer2_buf + i);
      i = next_i_;
    } while (next_i_ < ctx->xfer2_len);
  }
  do {
  } while ((spi->cnt & 0x8000) != 0);
  spi->done = 0;
  return;
}


#define SpiCtx2__WriteLegacy(...) __builtin_trap()


dword spiDeviceWrite2(void *ctx,byte dev_id,byte *buf1,uint len1,byte *buf2,dword len2)
{
  int bus_id;
  SpiCtx2 spi_ctx;
  
  if (4 < len1) {
    return 0xe0e03ffd;
  }
  switch(dev_id) {
  case 0:
  case 1:
  case 2:
    bus_id = 0;
    break;
  case 3:
  case 4:
  case 5:
    bus_id = 1;
    break;
  default:
    bus_id = 2;
  }
  spi_ctx.baudrate = (dword)device_baud[(uint)dev_id];
  if (device_usenew[bus_id] == false) {
    if (device_initialized[(uint)dev_id] == 0) {
      return 0xc8a03ff8;
    }
    spi_ctx.xfer2_buf = buf2;
    spi_ctx.xfer2_len = len2;
    spi_ctx.xfer1_buf = buf1;
    spi_ctx.xfer1_len = len1;
    spi_ctx.device_num = dev_id;
    CritEnter(&DAT_001050cc + bus_id * 3);
    SpiCtx2__WriteLegacy(&spi_ctx);
    CritExit(&DAT_001050cc + bus_id * 3);
  }
  else {
    if (device_initialized[(uint)dev_id] == 0) {
      return 0xc8a03ff8;
    }
    spi_ctx.xfer2_buf = buf2;
    spi_ctx.xfer2_len = len2;
    spi_ctx.xfer1_buf = buf1;
    spi_ctx.xfer1_len = len1;
    spi_ctx.device_num = dev_id;
    CritEnter(&DAT_001050cc + bus_id * 3);
    SpiCtx2__Write(&spi_ctx);
    CritExit(&DAT_001050cc + bus_id * 3);
  }
  return 0;
}