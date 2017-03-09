use io::irq;

use core::intrinsics::ctlz;
use core::iter;

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

fn without_interrupts<T, F: FnOnce() -> T>(f: F) -> T {
    let was_enabled = unsafe { disable_interrupts() };
    let ret = f();
    if was_enabled { unsafe { enable_interrupts() }; }
    ret
}

#[no_mangle]
pub extern fn init_interrupts() {
    without_interrupts(|| {
        static vector_mappings: [(u32, unsafe extern fn()); 6] = [
            (0x08000000, wrap_handle_irq),
            (0x08000008, wrap_handle_fiq),
            (0x08000010, wrap_handle_swi),
            (0x08000018, wrap_handle_und),
            (0x08000020, wrap_handle_pre),
            (0x08000028, wrap_handle_dta),
        ];

        for &(addr, handler) in vector_mappings.iter() {
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

static mut handler_list: [(u32, [Option<HandlerFn>; 4]); 30] = [
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

pub enum Error {
    InvalidInterrupt,
    HandlersFull,
    NotFound,
}

pub fn find_handler<'a>(int_type: u32, val: Option<HandlerFn>) -> Result<&'a mut Option<HandlerFn>, Error> {
    without_interrupts(|| {
        let pos = match unsafe { handler_list.iter() }.position(|&x| x.0 == int_type) {
            Some(x) => x,
            None => return Err(Error::InvalidInterrupt)
        };
        let found_pos = match unsafe { handler_list[pos].1.iter() }.position(|&x| x == val) {
            Some(x) => x,
            None => return Err(Error::NotFound)
        };
        Ok(unsafe { &mut handler_list[pos].1[found_pos] })
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

    let found = unsafe { handler_list.iter() }.find(|&x| pending_interrupts & x.0 != 0);
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

#[no_mangle]
pub extern fn handle_swi(swi_index: u32) {
    panic!("Software interrupts not yet handled!");
}

#[no_mangle]
pub extern fn handle_und(addr: u32) {
    panic!("Undefined instruction @ 0x{:X}!", addr);
}

#[no_mangle]
pub extern fn handle_pre() {
    panic!("Prefetch abort!");
}

#[no_mangle]
pub extern fn handle_dta(addr: u32) {
    panic!("Data abort @ 0x{:X}!", addr);
}