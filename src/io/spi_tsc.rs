use io::spi;

const TSC_DEVICE_ID: u8 = 3;

pub fn write_reg(reg: TscReg, val: u8) {
    let (mode, index) = get_tsc_addr(reg);
    spi::write2(TSC_DEVICE_ID, &[0u8], &[mode]);
    spi::write2(TSC_DEVICE_ID, &[index << 1], &[val]);
}

pub fn read_reg(reg: TscReg) -> u8 {
    let (mode, index) = get_tsc_addr(reg);
    spi::write2(TSC_DEVICE_ID, &[0u8], &[mode]);
    let out = &mut [0u8];
    spi::write_read(TSC_DEVICE_ID, &[(index << 1) | 1], out);
    out[0]
}

macro_rules! tsc_regpairs {
    ($getter:ident -> $ename:ident, $ety:ty { $( $fname:ident = $fval:expr ),* }) => {
        #[derive(Copy, Clone, Debug)]
        pub enum $ename {
            $( $fname ),*
        }

        fn $getter(enm: $ename) -> $ety {
            match enm {
                $( $ename::$fname => $fval ),*
            }
        }
    };
}

tsc_regpairs! {
    get_tsc_addr -> TscReg, (u8, u8) {
        AdcFlags = (0, 0x24),
        AdcMic = (0, 0x51),
        AdcFineVol = (0, 0x52),
        MicBias = (1, 0x2E)
    }
}