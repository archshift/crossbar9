use core::ptr;

const XDMA_BASE: u32 = 0x1000C000u32;

macro_rules! xdmainst_size {
    (GO) => (6);
    (END) => (1);
    (KILL) => (1);
    (FLUSHP) => (2);
    (WFP) => (2);
    (WFE) => ();
    (LD) => (1);
    (LDPS) => (2);
    (LDPB) => (2);
    (ST) => (1);
    (STP) => (2);
    (STZ) => (1);
    (LP) => (2);
    (LPEND) => (2);
    (MOV) => (6);
}

macro_rules! xdmainst {
    (END) => ([0x00]);
    (KILL) => ([0x01]);
    (FLUSHP $which:expr) => ([0x35, $which << 3]);
    (WFP $which:expr, periph) => ([0x31, $which << 3]);
    (WFE) => ();
    (LD) => ([0x04]);
    (LDPS $which:expr) => ([0x25, $which << 3]);
    (LDPB $which:expr) => ([0x27, $which << 3]);
    (ST) => ([0x08]);
    (STP) => ();
    (STZ) => ([0x0C]);
    (LP $ctr:expr, $iters:expr) => ([0x20 | ($ctr << 1), $iters - 1]);
    (LPEND $ctr:expr) => ([0x38 | ($ctr << 2), 0]);
    (GO $chan:expr, $where:expr) => ({
        let b = ($where as u32).to_le_bytes();
        [0xa2, $chan, b[0], b[1], b[2], b[3]]
    });
    (MOV $where:ident, $what:expr) => {{
        #[allow(dead_code)]
        enum Reg {
            SAR = 0,
            CCR = 1,
            DAR = 2
        }
        let b = ($what as u32).to_le_bytes();
        [0xbc, Reg::$where as u8, b[0], b[1], b[2], b[3]]
    }};
}

macro_rules! handle_lp {
    ($loop_rel:expr; LP $ctr:tt $($rest:tt)* ) => ({
        assert!($loop_rel[$ctr].is_none());
        $loop_rel[$ctr] = Some(0);
    });
    ($loop_rel:expr; $($other:tt)*) => {}
}

macro_rules! handle_lpend {
    ($loop_rel:expr; $inst_buf:expr; LPEND $ctr:tt $($rest:tt)* ) => ({
        assert!($loop_rel[$ctr].is_some());
        let rel = $loop_rel[$ctr].take();
        $inst_buf[1] = rel.unwrap() - xdmainst_size!(LPEND);
    });
    ($loop_rel:expr; $inst_buf:expr; $($other:tt)*) => {}
}

macro_rules! xdma_compile_ {
    ( $( [ $inst_name:ident $($inst_param:tt),* ] )+ ) => {{
        const LEN: usize = 0 $(+ xdmainst_size!($inst_name))+;
        let mut arr = [0u8; LEN];
        let mut loop_rel: [Option<u8>; 2] = [None; 2];
        {
            let arr_sl = &mut arr[..];
        
            $(
                let inst_dat = {
                    const INST_LEN: usize = xdmainst_size!( $inst_name );
                    let inst_dat: [u8; INST_LEN] = xdmainst!( $inst_name $($inst_param),* );
                    inst_dat
                };

                arr_sl[..inst_dat.len()].copy_from_slice(&inst_dat);

                loop_rel[0].as_mut().map(|x| *x += xdmainst_size!( $inst_name ));
                loop_rel[1].as_mut().map(|x| *x += xdmainst_size!( $inst_name ));

                handle_lpend!( &mut loop_rel; arr_sl; $inst_name $($inst_param),* );
                handle_lp!( &mut loop_rel; $inst_name $($inst_param),* );

                let arr_sl = &mut arr_sl[inst_dat.len()..];
            )+
            drop(arr_sl);
        }
        
        arr
    }}
}

macro_rules! xdma_compile {
    ( $( $inst_name:ident $(( $($params:tt),* ))* );+ ) => {
        xdma_compile_!( $( [ $inst_name $($($params),*)* ] )* )
    }
}


pub enum XdmaSrc {
//    FillData(u32),
//    FixedAddr(*const u32),
    LinearBuf(*const u8, usize),
}

pub enum XdmaDst {
//    FixedAddr(*mut u32),
    LinearBuf(*mut u8, usize)
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum Reg {
    MANAGER_FTYPE = 0x038,
    CHANNEL_FTYPE0 = 0x040,
    CHANNEL_STAT0 = 0x100,
    CHANNEL_PC0 = 0x104,
    DEBUG_STAT = 0xD00,
    DEBUG_CMD = 0xD04,
    DEBUG_INST0 = 0xD08,
    DEBUG_INST1 = 0xD0C,
}

#[inline(never)]
fn read_reg<T: Copy>(reg: Reg) -> T {
    unsafe { ptr::read_volatile((XDMA_BASE + reg as u32) as *const T) }
}

fn write_reg<T: Copy>(reg: Reg, val: T) {
    unsafe { ptr::write_volatile((XDMA_BASE + reg as u32) as *mut T, val); }
}

bf!(ChannelCtrl[u32] {
    src_inc: 0:0,
    src_burst_size: 1:3,
    src_burst_len: 4:7,
    src_prot: 8:10,
    src_cache: 11:13,
    dst_inc: 14:14,
    dst_burst_size: 15:17,
    dst_burst_len: 18:21,
    dst_prot: 22:24,
    dst_cache: 25:27
});

bf!(DmaInst[u64] {
    use_channel: 0:0,
    channel: 8:10,
    inst_b0: 16:23,
    inst_b1: 24:31,
    inst_b2: 32:39,
    inst_b3: 40:47,
    inst_b4: 48:55,
    inst_b5: 56:63
});

pub fn mem_transfer(src: XdmaSrc, dst: XdmaDst) {
    let XdmaSrc::LinearBuf(src, len) = src;
    let XdmaDst::LinearBuf(dst, dst_len) = dst;

    assert_eq!(len, dst_len);

    const LINE_SIZE: usize = 8;
    const BURST_LINES: usize = 16;

    let lines = len / LINE_SIZE;
    let chunks = lines / BURST_LINES;

    assert_eq!(src as usize % LINE_SIZE, 0, "XDMA source unaligned!");
    assert_eq!(dst as usize % LINE_SIZE, 0, "XDMA dest unaligned!");
    
    assert_eq!(len % (LINE_SIZE * BURST_LINES), 0, "XDMA xfer len is not a multiple of the transfer width!");

    let mut ctrl_big = ChannelCtrl::new(0);
    ctrl_big.src_inc.set(1);
    ctrl_big.src_burst_size.set((LINE_SIZE.trailing_zeros()) as u32);
    ctrl_big.src_burst_len.set((BURST_LINES - 1) as u32);
    ctrl_big.src_prot.set(0b011);
    ctrl_big.src_cache.set(0b010);
    ctrl_big.dst_inc.set(1);
    ctrl_big.dst_burst_size.set((LINE_SIZE.trailing_zeros()) as u32);
    ctrl_big.dst_burst_len.set((BURST_LINES - 1) as u32);
    ctrl_big.dst_prot.set(0b011);
    ctrl_big.dst_cache.set(0b010);

    let program = xdma_compile! {
        MOV(SAR, (src as u32));
        MOV(CCR, (ctrl_big.val));
        MOV(DAR, (dst as u32));
        LP(0, (chunks as u8));
            LD;
            ST;
        LPEND(0);
        END
    };


    let mut dmainst = DmaInst::new(0);
    let go = xdma_compile! {
        GO(0, (program.as_ptr() as u32))
    };
    dmainst.inst_b0.set(go[0] as u64);
    dmainst.inst_b1.set(go[1] as u64);
    dmainst.inst_b2.set(go[2] as u64);
    dmainst.inst_b3.set(go[3] as u64);
    dmainst.inst_b4.set(go[4] as u64);
    dmainst.inst_b5.set(go[5] as u64);

    write_reg(Reg::DEBUG_INST0, dmainst.val as u32);
    write_reg(Reg::DEBUG_INST1, (dmainst.val >> 32) as u32);
    write_reg(Reg::DEBUG_CMD, 0u32);

    let mut counter1 = 0x10000;
    while read_reg::<u32>(Reg::DEBUG_STAT) & 1 != 0 {}
    while counter1 != 0 && read_reg::<u32>(Reg::CHANNEL_STAT0) & 0xF != 0 { counter1 -= 1 }


    let ftype = read_reg::<u32>(Reg::CHANNEL_FTYPE0);
    if ftype != 0 {
        let pc = read_reg::<u32>(Reg::CHANNEL_PC0);
        let failed_inst = {
            let offset = (pc - (program.as_ptr() as u32)) as usize;
            let buf_size = (program.len() - offset).min(6);
            &program[offset..offset + buf_size];
        };

        panic!(
            "XDMA channel faulted!\n\
                Final channel PC: {:08X}\n\
                Data at channel PC: {:X?}\n\
                Final manager fault type: {:08X}\n\
                Final channel fault type: {:08X}\n\
                Final channel state: {:08X}",
            pc, failed_inst, ftype,
            read_reg::<u32>(Reg::CHANNEL_FTYPE0),
            read_reg::<u32>(Reg::CHANNEL_STAT0)
        );
    }
}
