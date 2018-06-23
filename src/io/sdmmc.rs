use io::RegEnum;

const SDMMC_BASE: u32 = 0x10006000;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
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

impl RegEnum for Reg {
    fn addr_of(&self) -> u32 {
        SDMMC_BASE + (*self as u32)
    }
}

enum _Status0 {
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

enum _Status1 {
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

pub fn soft_reset() {
    Reg::RESET.write16(Reg::RESET.read16() & 0xFFFE);
	Reg::RESET.write16(Reg::RESET.read16() | 1);
}

pub fn base_init(data32: bool) {
    Reg::DATACTL32.write16(Reg::DATACTL32.read16() & 0xF7FF);
	Reg::DATACTL32.write16(Reg::DATACTL32.read16() & 0xEFFF);
    Reg::DATACTL32.write16(Reg::DATACTL32.read16() | 0x402);
    
    let ctl = Reg::DATACTL.read16();
	Reg::DATACTL.write16((ctl & 0xFFDD) | 2);
    if data32 {
        Reg::DATACTL32.write16(Reg::DATACTL32.read16() & 0xFFFF);
        Reg::DATACTL.write16(Reg::DATACTL.read16() & 0xFFDF);
	    Reg::BLKLEN32.write16(512);
    } else {
        Reg::DATACTL32.write16(Reg::DATACTL32.read16() & 0xFFFD);
        Reg::DATACTL.write16(Reg::DATACTL.read16() & 0xFFDD);
        Reg::BLKLEN32.write16(0);
    }
	Reg::BLKCOUNT32.write16(1);
	
    soft_reset();
	
    Reg::IRMASK0.write16(STAT0_ALL);
	Reg::IRMASK1.write16(STAT1_ALL);
	Reg::UNKNOWN0.write16(Reg::UNKNOWN0.read16() | 0xDB);
	Reg::UNKNOWN1.write16(Reg::UNKNOWN1.read16() | 0xDB);
	Reg::PORTSEL.write16(Reg::PORTSEL.read16() & 0xFFFC);
    if data32 {
        Reg::CLKCTL.write16(0x20);
        Reg::OPT.write16(0x40EE);
    } else {
        Reg::CLKCTL.write16(0x40); //Nintendo sets this to 0x20
        Reg::OPT.write16(0x40EB); //Nintendo sets this to 0x40EE
    }
	Reg::PORTSEL.write16(Reg::PORTSEL.read16() & 0xFFFC);
	Reg::BLKLEN.write16(512);
	Reg::STOP.write16(0);
}
