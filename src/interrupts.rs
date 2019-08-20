use io::irq;

extern {
    static ldr_pc_pc_neg4: u32;
    fn wrap_handle_irq();
    fn wrap_handle_fiq();
    fn wrap_handle_swi();
    fn wrap_handle_und();
    fn wrap_handle_pre();
    fn wrap_handle_dta();
    pub fn wait_for_interrupt();
    fn disable_interrupts() -> bool;
    fn enable_interrupts();
}

pub fn without_interrupts<T, F: FnOnce() -> T>(f: F) -> T {
    let was_enabled = unsafe { disable_interrupts() };
    let ret = f();
    if was_enabled { unsafe { enable_interrupts() }; }
    ret
}

#[no_mangle]
pub extern fn init_interrupts() {
    without_interrupts(|| {
        static VECTOR_MAPPINGS: [(u32, unsafe extern fn()); 6] = [
            (0x08000000, wrap_handle_irq),
            (0x08000008, wrap_handle_fiq),
            (0x08000010, wrap_handle_swi),
            (0x08000018, wrap_handle_und),
            (0x08000020, wrap_handle_pre),
            (0x08000028, wrap_handle_dta),
        ];

        for &(addr, handler) in VECTOR_MAPPINGS.iter() {
            unsafe {
                *(addr as *mut u32) = ldr_pc_pc_neg4;
                *((addr + 4) as *mut u32) = handler as u32;
            }
        }

        irq::set_disabled(!0u32);
        irq::clear_all_pending();
    })
}

pub type HandlerFn = fn();

static mut HANDLER_LIST: [(u32, [Option<HandlerFn>; 4]); 30] = [
    (irq::Interrupt::DMAC_1_0,      [None, None, None, None]),
    (irq::Interrupt::DMAC_1_1,      [None, None, None, None]),
    (irq::Interrupt::DMAC_1_2,      [None, None, None, None]),
    (irq::Interrupt::DMAC_1_3,      [None, None, None, None]),
    (irq::Interrupt::DMAC_1_4,      [None, None, None, None]),
    (irq::Interrupt::DMAC_1_5,      [None, None, None, None]),
    (irq::Interrupt::DMAC_1_6,      [None, None, None, None]),
    (irq::Interrupt::DMAC_1_7,      [None, None, None, None]),
    (irq::Interrupt::TIMER_0,       [None, None, None, None]),
    (irq::Interrupt::TIMER_1,       [None, None, None, None]),
    (irq::Interrupt::TIMER_2,       [None, None, None, None]),
    (irq::Interrupt::TIMER_3,       [None, None, None, None]),
    (irq::Interrupt::PXI_SYNC,      [None, None, None, None]),
    (irq::Interrupt::PXI_NOT_FULL,  [None, None, None, None]),
    (irq::Interrupt::PXI_NOT_EMPTY, [None, None, None, None]),
    (irq::Interrupt::AES,           [None, None, None, None]),
    (irq::Interrupt::SDIO_1,        [None, None, None, None]),
    (irq::Interrupt::SDIO_1_ASYNC,  [None, None, None, None]),
    (irq::Interrupt::SDIO_3,        [None, None, None, None]),
    (irq::Interrupt::SDIO_3_ASYNC,  [None, None, None, None]),
    (irq::Interrupt::DEBUG_RECV,    [None, None, None, None]),
    (irq::Interrupt::DEBUG_SEND,    [None, None, None, None]),
    (irq::Interrupt::RSA,           [None, None, None, None]),
    (irq::Interrupt::CTR_CARD_1,    [None, None, None, None]),
    (irq::Interrupt::CTR_CARD_2,    [None, None, None, None]),
    (irq::Interrupt::CGC,           [None, None, None, None]),
    (irq::Interrupt::CGC_DET,       [None, None, None, None]),
    (irq::Interrupt::DS_CARD,       [None, None, None, None]),
    (irq::Interrupt::DMAC_2,        [None, None, None, None]),
    (irq::Interrupt::DMAC_2_ABORT,  [None, None, None, None]),
];

#[derive(Debug)]
pub enum Error {
    InvalidInterrupt,
    HandlersFull,
    NotFound,
}

pub fn find_handler<'a>(int_type: u32, val: Option<HandlerFn>) -> Result<&'a mut Option<HandlerFn>, Error> {
    without_interrupts(|| {
        let pos = match unsafe { HANDLER_LIST.iter() }.position(|&x| x.0 == int_type) {
            Some(x) => x,
            None => return Err(Error::InvalidInterrupt)
        };
        let found_pos = match unsafe { HANDLER_LIST[pos].1.iter() }.position(|&x| x == val) {
            Some(x) => x,
            None => return Err(Error::NotFound)
        };
        Ok(unsafe { &mut HANDLER_LIST[pos].1[found_pos] })
    })
}

pub fn register_handler(int_type: u32, handler: HandlerFn) -> Result<(), Error> {
    *find_handler(int_type, None)? = Some(handler);
    Ok(())
}

pub fn unregister_handler(int_type: u32, handler: HandlerFn) -> Result<(), Error> {
    *find_handler(int_type, Some(handler))? = None;
    Ok(())
}

#[no_mangle]
pub extern fn handle_irq() {
    let pending_interrupts = irq::get_pending() & irq::get_enabled();

    let found = unsafe { HANDLER_LIST.iter() }.find(|&x| pending_interrupts & x.0 != 0);
    let &(interrupt, handlers) = match found {
        Some(x) => x,
        None => {
            // Should never occur
            irq::clear_pending(pending_interrupts);
            return
        }
    };

    irq::clear_pending(interrupt);

    for handler in handlers.iter().filter_map(|&x| x) {
        handler();
    }
}

pub type SwiHandlerFn = fn(u32, bool, &mut [u32; 15], &mut u32);
static mut SWI_HANDLER: Option<SwiHandlerFn> = None;

pub fn register_swi_handler(handler: SwiHandlerFn) -> Result<(), &'static str> {
    unsafe { 
        if SWI_HANDLER.is_none() {
            SWI_HANDLER = Some(handler);
            Ok(())
        } else {
            Err("Attempted to override existing SWI handler")
        }
    }
}

pub fn unregister_swi_handler() {
    unsafe { SWI_HANDLER = None };
}

#[no_mangle]
pub extern fn handle_swi(swi_index: u32, is_thumb: u32, regs: *mut [u32; 15], pc: *mut u32) {
    if let Some(h) = unsafe { SWI_HANDLER } {
        h(swi_index, is_thumb != 0, unsafe { &mut *regs }, unsafe { &mut *pc });
    } else {
        panic!("Handling software interrupt {:02X} failed!", swi_index);
    }
}

#[no_mangle]
pub extern fn handle_und(addr: u32) {
    panic!("Undefined instruction @ 0x{:X}!", addr);
}

#[no_mangle]
pub extern fn handle_pre(addr: u32, lr: u32, sp: u32) {
    use alloc::string::String;
    use core::slice;
    use core::fmt::Write;

    let stack = unsafe { slice::from_raw_parts(sp as *mut u32, 6*4) }; 
    let mut stack_str = String::new();
    for i in 0..6 {
        let _ = write!(stack_str, "    {:08X}, {:08X}, {:08X}, {:08X}\n",
            stack[i*4+0], stack[i*4+1], stack[i*4+2], stack[i*4+3]);
    }
    let stack_str = "";
    let _ = sp;

    panic!("Prefetch abort @ 0x{:X}! lr=0x{:X}, stack=\n{}", addr, lr, stack_str);
}

#[no_mangle]
pub extern fn handle_dta(addr: u32) {
    panic!("Data abort @ 0x{:X}!", addr);
}
