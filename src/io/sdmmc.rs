use core::ptr;

const SDMMC_BASE: u32 = 0x10006000;

#[derive(Clone, Copy)]
enum Reg {
    CMD = 0x00,
    PORTSEL = 0x02,
    CMDARG0 = 0x04,
    CMDARG1 = 0x06,
    STOP = 0x08,
    BLKCOUNT = 0x0A,

    RESP0 = 0x0C,
    RESP1 = 0x0E,
    RESP2 = 0x10,
    RESP3 = 0x12,
    RESP4 = 0x14,
    RESP5 = 0x16,
    RESP6 = 0x18,
    RESP7 = 0x1A,

    STATUS0 = 0x1C,
    STATUS1 = 0x1E,

    IRMASK0 = 0x20,
    IRMASK1 = 0x22,
    CLKCTL = 0x24,

    BLKLEN = 0x26,
    OPT = 0x28,
    FIFO = 0x30,

    DATACTL = 0xD8,
    RESET = 0xE0,
    PROTECTED = 0xF6, //bit 0 determines if sd is protected or not?

    UNKNOWN0 = 0xFC,
    UNKNOWN1 = 0xFE,

    DATACTL32 = 0x100,
    BLKLEN32 = 0x104,
    BLKCOUNT32 = 0x108,
    FIFO32 = 0x10C,

    CLK_AND_WAIT_CTL = 0x138,
    RESET_SDIO = 0x1E0
}

enum Status0 {
    CmdResponseEnd = (1 << 0),
    DataEnd     = (1 << 2),
    CardRemove  = (1 << 3),
    CardInsert  = (1 << 4),
    SigState    = (1 << 5),
    WRProtect   = (1 << 7),
    CardRemoveA = (1 << 8),
    CardInsertA = (1 << 9),
    SigStateA   = (1 << 10)
}

enum Status1 {
    CmdIndexErr = (1 << 0),
    CrcFail     = (1 << 1),
    StopBitErr  = (1 << 2),
    DataTimeout = (1 << 3),
    RxOverflow  = (1 << 4),
    TxUnderrun  = (1 << 5),
    CmdTimeout  = (1 << 6),
    RxReady     = (1 << 8),
    TxRq        = (1 << 9),
    IllFunc     = (1 << 13),
    CmdBusy     = (1 << 14),
    IllegalCmd  = (1 << 15),
}

const STAT0_ALL: u16 = 0xf031;
const STAT1_ALL: u16 = 0x0837;

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { ptr::read_volatile((SDMMC_BASE + reg as u32) as *const T) }
}

fn write_reg<T: Copy>(reg: Reg, val: T) {
    unsafe { ptr::write_volatile((SDMMC_BASE + reg as u32) as *mut T, val); }
}

fn soft_reset() {
    write_reg::<u16>(Reg::RESET, read_reg::<u16>(Reg::RESET) & 0xFFFE);
	write_reg::<u16>(Reg::RESET, read_reg::<u16>(Reg::RESET) | 1);
}

fn base_init(data32: bool) {
    write_reg::<u16>(Reg::DATACTL32, read_reg::<u16>(Reg::DATACTL32) & 0xF7FF);
	write_reg::<u16>(Reg::DATACTL32, read_reg::<u16>(Reg::DATACTL32) & 0xEFFF);
    write_reg::<u16>(Reg::DATACTL32, read_reg::<u16>(Reg::DATACTL32) | 0x402);
    
    let ctl: u16 = read_reg(Reg::DATACTL);
	write_reg::<u16>(Reg::DATACTL, (ctl & 0xFFDD) | 2);
    if data32 {
        write_reg::<u16>(Reg::DATACTL32, read_reg::<u16>(Reg::DATACTL32) & 0xFFFF);
        write_reg::<u16>(Reg::DATACTL, read_reg::<u16>(Reg::DATACTL) & 0xFFDF);
	    write_reg::<u16>(Reg::BLKLEN32, 512);
    } else {
        write_reg::<u16>(Reg::DATACTL32, read_reg::<u16>(Reg::DATACTL32) & 0xFFFD);
        write_reg::<u16>(Reg::DATACTL, read_reg::<u16>(Reg::DATACTL) & 0xFFDD);
        write_reg::<u16>(Reg::BLKLEN32, 0);
    }
	write_reg::<u16>(Reg::BLKCOUNT32, 1);
	
    soft_reset();
	
    write_reg::<u16>(Reg::IRMASK0, STAT0_ALL);
	write_reg::<u16>(Reg::IRMASK1, STAT1_ALL);
	write_reg::<u16>(Reg::UNKNOWN0, read_reg::<u16>(Reg::UNKNOWN0) | 0xDB);
	write_reg::<u16>(Reg::UNKNOWN1, read_reg::<u16>(Reg::UNKNOWN1) | 0xDB);
	write_reg::<u16>(Reg::PORTSEL, read_reg::<u16>(Reg::PORTSEL) & 0xFFFC);
    if data32 {
        write_reg::<u16>(Reg::CLKCTL, 0x20);
        write_reg::<u16>(Reg::OPT, 0x40EE);
    } else {
        write_reg::<u16>(Reg::CLKCTL, 0x40); //Nintendo sets this to 0x20
        write_reg::<u16>(Reg::OPT, 0x40EB); //Nintendo sets this to 0x40EE
    }
	write_reg::<u16>(Reg::PORTSEL, read_reg::<u16>(Reg::PORTSEL) & 0xFFFC);
	write_reg::<u16>(Reg::BLKLEN, 512);
	write_reg::<u16>(Reg::STOP, 0);
}
